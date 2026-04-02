use crate::application::calls::check_request_price_info::check_request_price_info;
use crate::presentation::dto::{
    PriceInfoPublicationReq, PriceInfoPublicationResponse,
    UpdatePriceInformationRequest,
};
use asez2_shared_db::db_item::AdaptorableIter;
use asez2_shared_db::{
    db_item::{AsezTimestamp, Select},
    DbAdaptor, DbItem,
};
use monolith_service::{
    dto::{
        attachment::{
            Attachment as MonolithAttachment, GetHierarchyReq,
            GetHierarchyResponseData,
        },
        organization::Organization,
        unit::Unit,
    },
    http::MonolithHttpService,
};
use rabbit_services::integration::IntegrationService;
use shared_essential::{
    domain::{
        tables::tcp::PriceInformationRequestType,
        tcp::{
            BasicRequestDetails, BasicRequestDetailsSelector,
            PriceInformationRequestStatus, RequestHeader, RequestHeaderRep,
            RequestItem, RequestItemRep, RequestPartner, RequestPartnerRep,
        },
    },
    presentation::dto::{
        integration::commercial_offer::request::*,
        response_request::{Message, MessageKind, Messages},
        technical_commercial_proposal::{TcpError, TcpResult},
    },
};

use ahash::{AHashMap, AHashSet};
use sqlx::PgPool;
use tokio::join;
use uuid::Uuid;

const UPDATE_FIELDS: Option<&[&str]> = Some(&[
    RequestHeader::changed_by,
    RequestHeader::changed_at,
    RequestHeader::status_id,
]);

const FIELDS: &[&str] =
    &[RequestHeader::uuid, RequestHeader::id, RequestHeader::status_id];

const PRODUCT_TYPE_ITEM: i16 = 1;
const PRODUCT_TYPE_LABOR: i16 = 2;
const PRODUCT_TYPE_SERVICE: i16 = 3;

/// Данные, полученные из монолита, необходимые
/// для формирования запроса в сервис интеграции
pub(crate) struct MonolithData {
    /// Справочник ОКПД2
    pub okpd2: Vec<Okpd2>,
    /// Справочник ОКВЭД2
    pub okved2: Vec<Okved2>,
    /// Заказчик
    pub customer: Customer,
    /// Организатор ЗЦИ
    pub placer: Placer,
    /// Валюта
    pub currency: String,
    /// Мапа для получения данных пользователя
    pub units_map: AHashMap<i32, Unit>,
    /// Мапа для получения данных организации
    pub organizations_map: AHashMap<i32, Organization>,
    /// Вид предмета закупки
    pub subject_type: Vec<String>,
    /// Санкционный признак
    pub is_under_sanctions: bool,
}

/// Основная функция публикации ЗЦИ.
///
/// Этапы:
/// 1. Получение данных из БД по UUID.
/// 2. Валидация запроса.
/// 3. Получение дополнительных данных из монолита.
/// 4. Формирование запроса и отправка в сервис интеграции.
/// 5. Обновление статуса ЗЦИ и возврат результата.
///
/// # Параметры
/// - `user_id`: ID пользователя
/// - `monolith_token`: токен доступа к монолиту
/// - `req`: структура с uuid и id ЗЦИ
/// - `monolith_service`: клиент для обращения в монолит
/// - `pool`: пул соединений к БД
/// - `integration_service`: сервис интеграции для отправки в SAP PI
///
/// # Возвращает
/// - (PriceInfoPublicationResponse, Messages): ответ с обновлённым статусом ЗЦИ и сообщения валидации
pub(crate) async fn process_publication_price_info(
    user_id: i32,
    monolith_token: String,
    req: PriceInfoPublicationReq,
    monolith_service: &MonolithHttpService,
    pool: &PgPool,
    integration_service: IntegrationService,
) -> TcpResult<(PriceInfoPublicationResponse, Messages)> {
    // Получение данных из БД
    let data = fetch_data(req.uuid, pool).await?;

    // Валидация и получение файлов
    let (mut messages, monolith_attachments) = prepare_publish_request(
        &data,
        monolith_token.clone(),
        monolith_service,
        user_id,
    )
    .await?;

    if messages.is_error() {
        return Err(TcpError::Business(messages));
    }

    // Получение дополнительных данных из монолита
    let monolith_data = get_data_from_monolith(
        &data,
        monolith_token.clone(),
        monolith_service,
        user_id,
    )
    .await?;

    // Формирование запроса
    let commercial_offer_data = build_commercial_offer_request(
        &data,
        monolith_data,
        monolith_attachments,
        monolith_token,
        user_id,
    )?;

    // Отправка в сервис интеграции
    let integration_response = integration_service
        .send_commercial_offer_request(commercial_offer_data)
        .await
        .map_err(|e| TcpError::InternalError(e.error().to_string()))?;

    if !integration_response.success {
        return Err(TcpError::InternalError(integration_response.message));
    }

    // Обновление статуса
    let header_id = data.header.id;
    let mut to_update = data.header;
    to_update.status_id = PriceInformationRequestStatus::TransferredToEtp;
    to_update.changed_by = user_id;
    to_update.changed_at = AsezTimestamp::now();

    let mut tx = pool.begin().await?;
    let item_list = RequestHeaderRep::update_vec_returning::<Vec<_>>(
        &[to_update],
        UPDATE_FIELDS,
        Some(FIELDS),
        &mut tx,
    )
    .await?;
    tx.commit().await?;

    messages.add_prepared_message(Message::success(format!(
        " ЗЦИ {} направлено на ЭТП ГПБ",
        header_id
    )));

    Ok((PriceInfoPublicationResponse { item_list }, messages))
}

async fn get_hierarchy_from_monolith(
    uuid: uuid::Uuid,
    monolith_token: String,
    monolith_service: &MonolithHttpService,
    user_id: i32,
) -> TcpResult<GetHierarchyResponseData> {
    let monolith_req = GetHierarchyReq {
        hierarchy_list: vec![uuid],
    };

    let monolith_res = monolith_service
        .get_hierarchy(monolith_req, monolith_token, user_id)
        .await?;

    if monolith_res.data.hierarchy_list.is_empty() {
        return Err(TcpError::MonolithError(
            "Монолит вернул пустой список иерархий".to_string(),
        ));
    }

    Ok(monolith_res.data)
}

async fn fetch_data(uuid: Uuid, pool: &PgPool) -> TcpResult<BasicRequestDetails> {
    let req_select = Select::full::<RequestHeader>().eq(RequestHeader::uuid, uuid);
    let mut data = BasicRequestDetailsSelector::new(req_select)
        .get(pool)
        .await?
        .pop()
        .ok_or_else(|| TcpError::not_found(uuid, RequestHeader::TABLE))?;

    let mut seen_suppliers = AHashSet::new();
    data.suppliers.retain(|u| seen_suppliers.insert(u.uuid));

    let mut seen_items = AHashSet::new();
    data.items.retain(|c| seen_items.insert(c.uuid));

    Ok(data)
}

/// Получение дополнительных данных из монолита
///
/// # Параметры
/// - `data`: данные из БД
/// - `monolith_token`: токен монолита
/// - `monolith_service`: клиент для обращения в монолит
///
/// # Возвращаемое значение
/// - `MonolithData` - данные из монолита
async fn get_data_from_monolith(
    data: &BasicRequestDetails,
    monolith_token: String,
    monolith_service: &MonolithHttpService,
    user_id: i32,
) -> TcpResult<MonolithData> {
    let okpd2_ids: Vec<i32> = data
        .items
        .iter()
        .map(|item| item.okpd2_id)
        .collect::<AHashSet<_>>()
        .into_iter()
        .collect();

    let okved2_ids: Vec<i32> = data
        .items
        .iter()
        .map(|item| item.okved2_id)
        .collect::<AHashSet<_>>()
        .into_iter()
        .collect();

    let organization_ids: Vec<i32> = data
        .suppliers
        .iter()
        .map(|s| s.supplier_id)
        .collect::<AHashSet<_>>()
        .into_iter()
        .collect();

    let (okved2_list, okpd2_list, organizations, common_dictionaries) = join!(
        monolith_service.search_okved_by_id(
            okved2_ids.iter().copied(),
            monolith_token.clone(),
            user_id,
        ),
        monolith_service.search_okpd_by_id(
            okpd2_ids.iter().copied(),
            monolith_token.clone(),
            user_id,
        ),
        monolith_service.search_organization_by_id(
            &organization_ids,
            monolith_token.clone(),
            user_id,
        ),
        monolith_service.get_common_dictionaries(monolith_token.clone()),
    );

    let okved2: Vec<Okved2> = okved2_list
        .map_err(TcpError::from)?
        .into_iter()
        .filter(|item| okved2_ids.contains(&item.id))
        .map(|item| Okved2 {
            code: item.code,
            name: item.text,
        })
        .collect();

    let okpd2: Vec<Okpd2> = okpd2_list
        .map_err(TcpError::from)?
        .into_iter()
        .filter(|item| okpd2_ids.contains(&item.id))
        .map(|item| Okpd2 {
            code: item.code,
            name: item.text,
        })
        .collect();

    let common_dictionaries = common_dictionaries.map_err(TcpError::from)?;

    let (customer, is_under_sanctions) = match data.header.customer_id {
        Some(id) => common_dictionaries
            .customers
            .iter()
            .find(|c| c.id == id)
            .map(|c| (Customer::new(&c.inn, &c.kpp, c.id), c.is_under_sanctions))
            .unwrap_or_else(|| (Customer::default(), false)),
        None => (Customer::default(), false),
    };

    let placer = match data.header.organizer_id {
        Some(id) => common_dictionaries
            .customers
            .iter()
            .find(|c| c.id == id)
            .map(|c| Placer::new(&c.inn, &c.kpp, c.id))
            .unwrap_or_default(),
        None => Placer::default(),
    };

    let subject_type: Vec<String> = common_dictionaries
        .categories
        .iter()
        .filter(|c| !c.is_removed)
        .filter(|c| data.items.iter().any(|item| item.category_id == c.id as i16))
        .map(|c| c.code.clone())
        .collect();

    let currency = data
        .header
        .currency_id
        .and_then(|cid| {
            common_dictionaries
                .currencies
                .iter()
                .find(|c| c.id == cid as i32)
                .map(|c| c.text.clone())
        })
        .unwrap_or_default();

    let organizations_map: AHashMap<i32, Organization> = organizations
        .map_err(TcpError::from)?
        .into_iter()
        .map(|x: Organization| (x.id, x))
        .collect();

    let units_map: AHashMap<i32, Unit> =
        common_dictionaries.units.into_iter().map(|x| (x.id, x)).collect();

    Ok(MonolithData {
        okpd2,
        okved2,
        customer,
        placer,
        currency,
        units_map,
        organizations_map,
        subject_type,
        is_under_sanctions,
    })
}

/// Формирование запроса для сервиса интеграции
///
/// # Параметры
/// - `data`: данные запроса
/// - `monolith_data`: данные монолита
///
/// # Возвращает
/// - CommercialOfferRequest - структура для отправки
pub(crate) fn build_commercial_offer_request(
    data: &BasicRequestDetails,
    monolith_data: MonolithData,
    monolith_attachments: Vec<MonolithAttachment>,
    monolith_token: String,
    user_id: i32,
) -> TcpResult<CommercialOfferData> {
    Ok(CommercialOfferData {
        data: CommercialOfferRequest {
            request_id: Uuid::new_v4(),
            request_price_info: build_request_price_info(data, monolith_data)?,
            // Заполняем в integration-service
            attachment: Default::default(),
        },
        monolith_attachments,
        monolith_token,
        user_id,
    })
}

fn build_request_price_info(
    data: &BasicRequestDetails,
    monolith_data: MonolithData,
) -> TcpResult<RequestPriceInfo> {
    let MonolithData {
        okpd2,
        okved2,
        customer,
        placer,
        currency,
        units_map,
        organizations_map,
        subject_type,
        is_under_sanctions,
    } = monolith_data;

    let (first_name, patronymic, last_name) =
        parse_full_name(data.header.organizer_name.as_deref().unwrap_or(""));

    Ok(RequestPriceInfo {
        req_info: ReqInfo {
            req_number: data.header.id.to_string(),
            req_uuid: data.header.uuid,
        },
        private: matches!(
            data.header.type_request_id,
            Some(PriceInformationRequestType::Private)
        ),
        submission_close_date_time: data.header.end_date,
        procedure_completion_date: data.header.end_date,
        publication_planned_date: AsezTimestamp::now(),
        request_subject: data.header.request_subject.clone().unwrap_or_default(),
        subject_type,
        okved2,
        okpd2,
        sanctions: Some(is_under_sanctions),
        customer,
        placer,
        contract_info: build_contract_info(data, first_name, patronymic, last_name),
        currency,
        specifications: build_specifications(&data.items, &units_map)?,
        supplier: build_suppliers(&data.suppliers, &organizations_map),
    })
}

fn build_specifications(
    items: &[RequestItem],
    units_map: &AHashMap<i32, Unit>,
) -> TcpResult<Vec<Specification>> {
    items
        .iter()
        .map(|item|
            Ok(Specification {
            pos_nr: item.number.to_string(),
            pos_name: item.description_internal.clone(),
            quantity: item.quantity.to_string(),
            unit_of_measure: units_map
                .get(&(item.unit_id as i32))
                .map(|u| u.okei)
                .unwrap_or_default(),
            delivery_basis: Some(item.delivery_basis.clone()),
            technical_requirements: item.technical_requirements.clone(),
            product_mark: item.mark.clone(),
            delivery_date: format!(
                "Предварительный срок поставки/выполнения работ/оказания услуг {} - {}",
                item.delivery_start_date,
                item.delivery_end_date,
            ),
            pos_vid: map_pos_vid(item.product_type_id)?,
             })
    ).collect()
}

fn build_contract_info(
    data: &BasicRequestDetails,
    first_name: String,
    patronymic: String,
    last_name: String,
) -> ContactInfo {
    ContactInfo {
        phone: data.header.organizer_phone.clone().unwrap_or_default(),
        email: data.header.organizer_mail.clone().unwrap_or_default(),
        first_name,
        patronymic,
        last_name,
        legal_address: data.header.organizer_location.clone().unwrap_or_default(),
    }
}

fn build_suppliers(
    suppliers: &[RequestPartner],
    organizations_map: &AHashMap<i32, Organization>,
) -> Option<Vec<Supplier>> {
    (!suppliers.is_empty()).then(|| {
        suppliers
            .iter()
            .map(|item| {
                let org = organizations_map.get(&item.supplier_id);
                Supplier {
                    inn: org.map(|o| o.inn.clone()).unwrap_or_default(),
                    kpp: org.map(|o| o.kpp.clone()).unwrap_or_default(),
                    //TODO: требуется доработка монолита
                    email: "".into(),
                    asez_id: item.supplier_id.to_string(),
                }
            })
            .collect()
    })
}

/// Выполняет валидацию ЗЦИ перед публикацией и получает список файлов
///
/// # Параметры
/// - `request`: данные ЗЦИ
/// - `uuid`: UUID ЗЦИ
/// - `monolith_token`: токен монолита
/// - `monolith_service`: клиент для обращения в монолит
/// - `user_id`: ID пользователя
///
/// # Возвращает
/// - `Messages` - список сообщений
/// - `Vec<MonolithAttachment>` - список файлов
pub(crate) async fn prepare_publish_request(
    request: &BasicRequestDetails,
    monolith_token: String,
    monolith_service: &MonolithHttpService,
    user_id: i32,
) -> TcpResult<(Messages, Vec<MonolithAttachment>)> {
    let hierarchy = get_hierarchy_from_monolith(
        request.header.hierarchy_uuid.unwrap_or_default(),
        monolith_token.clone(),
        monolith_service,
        user_id,
    )
    .await?;

    let header = RequestHeaderRep::from_item::<&str>(request.header.clone(), None);

    let item_list: Vec<RequestItemRep> =
        request.items.iter().cloned().adaptors().collect();

    let partner_list: Vec<RequestPartnerRep> =
        request.suppliers.iter().cloned().adaptors().collect();

    let attachment_list: Vec<MonolithAttachment> = hierarchy
        .hierarchy_list
        .into_iter()
        .flat_map(|item| item.item_list)
        .collect();

    // Формируем список файлов (kind_id = 1) для заполнения Attachments в integration-service
    let filtered_attachments: Vec<MonolithAttachment> = attachment_list
        .iter()
        .filter(|a| a.kind_id == 1 && !a.is_classified)
        .cloned()
        .collect();

    let request = UpdatePriceInformationRequest {
        header,
        item_list,
        partner_list,
        attachment_list,
    };

    let messages_raw = check_request_price_info(&request).await;

    let mut messages = match messages_raw.kind {
        MessageKind::Success | MessageKind::Error => messages_raw,
        MessageKind::Information
        | MessageKind::Warning
        | MessageKind::Stop
        | MessageKind::None => Messages {
            kind: MessageKind::Error,
            messages: messages_raw
                .messages
                .into_iter()
                .map(|mut msg| {
                    msg.kind = MessageKind::Error;
                    msg
                })
                .collect(),
        },
    };

    if request.header.end_date.flatten().is_none() {
        messages.add_prepared_message(
            Message::error("Заполните поле \"Дата окончания\"")
                .with_fields(vec!["end_date".to_string()]),
        );
    }

    Ok((messages, filtered_attachments))
}

fn parse_full_name(full_name: &str) -> (String, String, String) {
    let parts: Vec<&str> = full_name.split_whitespace().collect();

    match parts.as_slice() {
        [first, patronymic, last] => {
            (first.to_string(), patronymic.to_string(), last.to_string())
        }
        [first, last] => (first.to_string(), "".to_string(), last.to_string()),
        [only] => (only.to_string(), "".to_string(), "".to_string()),
        _ => ("".to_string(), "".to_string(), "".to_string()),
    }
}

fn map_pos_vid(t: i16) -> TcpResult<String> {
    match t {
        PRODUCT_TYPE_ITEM => Ok("0".to_string()),
        PRODUCT_TYPE_LABOR => Ok("2".to_string()),
        PRODUCT_TYPE_SERVICE => Ok("1".to_string()),
        other => Err(TcpError::InternalError(format!(
            "Неизвестный product_type_id: {other}"
        ))),
    }
}
