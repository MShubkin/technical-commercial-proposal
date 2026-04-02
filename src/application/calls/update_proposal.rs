use asez2_shared_db::db_item::{
    from_item_with_fields, AsezTimestamp, DbItem, DbUpsert, Select,
};
use asez2_shared_db::DbAdaptor;

use itertools::{Either, Itertools};
use monolith_service::dto::attachment::{
    Attachment, FoldersCategory, UpdateHierarchyReq, UpdateHierarchyReqItem,
};
use monolith_service::http::MonolithHttpService;
use shared_essential::domain::tcp::{RequestItem, RequestPartner};
use shared_essential::{
    domain::maths::CurrencyValue,
    domain::tables::tcp::{
        ProposalHeader, ProposalHeaderRep, ProposalItem, ProposalItemRep,
        TCPCheckStatus, TcpGeneralStatus,
    },
    presentation::dto::{
        response_request::{Message, Messages, ParamItem},
        technical_commercial_proposal::{TcpError, TcpResult},
    },
};

use crate::presentation::dto::{
    UpdateProposalReq, UpdateProposalResponseData, UpdateProposalResponseItem,
};
use ahash::AHashMap;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

const PROPOSAL_HEADER_UPDATE_FIELDS: &[&str] = &[
    ProposalHeader::hierarchy_uuid,
    ProposalHeader::start_date,
    ProposalHeader::end_date,
    ProposalHeader::currency_id,
    ProposalHeader::changed_by,
    ProposalHeader::changed_at,
    ProposalHeader::status_id,
    ProposalHeader::status_check_id,
    ProposalHeader::result_id,
    ProposalHeader::proposal_source,
    ProposalHeader::sum_excluded_vat_total,
];

const PROPOSAL_HEADER_RETURN_FIELDS: &[&str] = &[
    ProposalHeader::hierarchy_uuid,
    ProposalHeader::id,
    ProposalHeader::uuid,
    ProposalHeader::supplier_uuid,
    ProposalHeader::request_uuid,
    ProposalHeader::sum_excluded_vat_total,
    ProposalHeader::currency_id,
    ProposalHeader::start_date,
    ProposalHeader::end_date,
    ProposalHeader::status_id,
    ProposalHeader::result_id,
];
const PROPOSAL_ITEM_RETURN_FIELDS: &[&str] = &[
    ProposalItem::uuid,
    ProposalItem::request_item_uuid,
    ProposalItem::number,
    ProposalItem::quantity,
    ProposalItem::unit_id,
    ProposalItem::price,
    ProposalItem::vat_id,
    ProposalItem::description_internal,
    ProposalItem::mark,
    ProposalItem::sum_included_vat,
    ProposalItem::sum_excluded_vat,
    ProposalItem::manufacturer,
    ProposalItem::execution_percent,
    ProposalItem::pay_condition_id,
    ProposalItem::prepayment_percent,
    ProposalItem::delivery_condition,
    ProposalItem::is_possibility,
    ProposalItem::possibility_note,
    ProposalItem::analog_description,
    ProposalItem::delivery_period,
];
const REQUEST_ITEM_FIELDS: &[&str] =
    &[RequestItem::uuid, RequestItem::price, RequestItem::vat_id];

/// Сохранение и добавление ТКП
pub async fn process_update_proposal(
    dto: UpdateProposalReq,
    token: String,
    user_id: i32,
    db_pool: &PgPool,
    monolith_service: &MonolithHttpService,
) -> TcpResult<(UpdateProposalResponseData, Messages)> {
    let UpdateProposalReq {
        supplier_id,
        mut header,
        item_list,
        attachment_list,
    } = dto;
    let mut messages = Messages::default();

    check_supplier_field(supplier_id, &mut messages);
    check_header_fields(&header, &mut messages);
    check_item_fields(&item_list, &mut messages);
    check_attachment_fields(&attachment_list, &mut messages);

    // Если "supplier_uuid" пусто, то необходимо выполнить проверки
    let request_partner_check =
        match (header.supplier_uuid, header.request_uuid, supplier_id) {
            // supplier_id должен быть, так как выше была проверка
            (None, Some(request_uuid), Some(supplier_id)) => check_request_partner(
                request_uuid,
                supplier_id,
                db_pool,
                &mut messages,
            )
            .await?
            .into(),
            _ => None,
        };

    // Заранее запрашиваем данные по ЗЦИ, так как они требуются в ответе
    // Все request_item_uuid тут есть, так как была проверка в check_item_fields
    let mut request_uuids = item_list.iter().filter_map(|x| x.request_item_uuid);
    let select = Select::with_fields(REQUEST_ITEM_FIELDS)
        .in_any(RequestItem::uuid, request_uuids.clone());
    let request_items = RequestItem::select(&select, db_pool)
        .await?
        .into_iter()
        .map(|x| (x.uuid, x))
        .collect::<AHashMap<_, _>>();
    if let Some(missing_request_item) =
        request_uuids.find(|request_uuid| request_items.get(request_uuid).is_none())
    {
        return Err(TcpError::RecordNotFound(
            missing_request_item.to_string(),
            RequestItem::TABLE.to_string(),
        ));
    }

    if !messages.is_empty() {
        return Err(TcpError::Business(messages));
    }

    // Является ли заголовок ЗЦИ новым
    let is_new = if header.id.is_none() {
        // Устанвливаем uuid, если запись является новой
        header.uuid = Some(Uuid::new_v4());
        true
    } else {
        false
    };
    let mut header = header.into_item()?;

    let hierarchy_uuid = update_hierarchy(
        attachment_list,
        token,
        user_id,
        monolith_service,
        &mut messages,
    )
    .await?;

    let mut tx = db_pool.begin().await?;
    let now = AsezTimestamp::now();

    // Сначала позиции, так как нужна сумма беь НДС для обновления заголовка.
    let (updated_items, total_sum_excluded_vat) =
        update_items(is_new, header.uuid, item_list, &mut tx).await?;

    header.hierarchy_uuid = Some(hierarchy_uuid);
    header.sum_excluded_vat_total = Some(total_sum_excluded_vat);
    header.status_id = TcpGeneralStatus::Created;
    header.status_check_id = TCPCheckStatus::Review;
    header.created_by = user_id;
    header.created_at = now;
    header.changed_by = user_id;
    header.changed_at = now;

    match request_partner_check {
        // Если запись в request_partner не найдена - необходимо создать запись в request_partner с supplier_id
        Some(Either::Left(new_supplier_id)) => {
            let partner_uuid = Uuid::new_v4();
            // Номер записи берется в рамках инкрементального счетчика по request_uuid
            let number: i16 = sqlx::query_scalar(
                "
                SELECT (COALESCE(MAX(number), 0) + 1)::SMALLINT
                FROM request_partner
                WHERE request_uuid = $1
            ",
            )
            .bind(header.request_uuid)
            .fetch_one(&mut tx)
            .await?;

            let mut request_partner = RequestPartner {
                uuid: partner_uuid,
                request_uuid: header.request_uuid,
                supplier_id: new_supplier_id,
                is_public: false,
                number,
                // остальные поля не заполняются
                ..Default::default()
            };

            request_partner.insert(&mut tx).await?;
            // При этом надо указать у заголовка supplier_uuid
            header.supplier_uuid = partner_uuid;
        }
        // Если запись в request_partner найдена и проверка пройдена,
        // то request_partner.uuid записать в proposal_head-supplier_uuid
        Some(Either::Right(old_partner)) => {
            header.supplier_uuid = old_partner.uuid;
        }
        None => {}
    };

    let updated_header = ProposalHeader::upsert_returning(
        &mut [header],
        Some(PROPOSAL_HEADER_UPDATE_FIELDS),
        &mut tx,
    )
    .await?
    .pop()
    .expect("Одна запись явно была передана на апсерт");

    tx.commit().await?;

    messages.add_prepared_message(UpdateProposalMessage::success());

    let res_data = finalise_response(updated_header, updated_items, request_items)?;
    Ok((res_data, messages))
}

/// Since we need to update numbers, we need a more robust method for updating,
/// to do this we do a full sync of items with the DB if necessary.
async fn update_items(
    new: bool,
    proposal_uuid: Uuid,
    item_list: Vec<ProposalItemRep>,
    tx: &mut Transaction<'_, Postgres>,
) -> TcpResult<(Vec<ProposalItem>, CurrencyValue)> {
    let mut sum = CurrencyValue::from(0);

    let mut item_list = if new {
        item_list
            .into_iter()
            .sorted_by_key(|item| item.number)
            .enumerate()
            .map(|(i, x)| {
                let mut item = x.into_item()?;
                item.number = i as i16 + 1;
                item.proposal_uuid = proposal_uuid;
                item.uuid = Uuid::new_v4();

                sum += item.sum_excluded_vat.unwrap_or_default();

                Ok(item)
            })
            .collect::<TcpResult<Vec<_>>>()?
    } else {
        let mut number = 1;

        let select = Select::full::<ProposalItem>()
            .eq(ProposalItem::proposal_uuid, proposal_uuid);

        let mut old_items = ProposalItem::select(&select, &mut *tx)
            .await?
            .into_iter()
            .map(|x| (x.uuid, x))
            .collect::<AHashMap<_, _>>();

        item_list
            .into_iter()
            .map(|new| {
                let item = if let Some(old) =
                    new.uuid.and_then(|x| old_items.remove(&x))
                {
                    new.into_item_merged(old)?
                } else {
                    let mut item = new.into_item()?;
                    item.uuid = Uuid::new_v4();
                    item.proposal_uuid = proposal_uuid;
                    item
                };
                sum += item.sum_excluded_vat.unwrap_or_default();
                Ok(item.numerate(&mut number))
            })
            .collect::<TcpResult<Vec<_>>>()?
            .into_iter()
            // We then update the remaining items, placing them at the bottom of
            // the item list to avoid potential conflict in numeration.
            .chain(
                old_items.into_iter().map(|(_, item)| item.numerate(&mut number)),
            )
            .collect()
    };

    let returned = ProposalItem::upsert_returning(&mut item_list, None, tx).await?;
    Ok((returned, sum))
}

/// Обновление иерархии в АСЭЗ 1.0. Возвращает uuid в иерархии файлов
async fn update_hierarchy(
    attachment_list: Vec<Attachment>,
    token: String,
    user_id: i32,
    monolith_service: &MonolithHttpService,
    messages: &mut Messages,
) -> TcpResult<Uuid> {
    let hierarchy_list = vec![UpdateHierarchyReqItem {
        uuid: None,
        item_list: attachment_list,
    }];
    let monolith_request = UpdateHierarchyReq { hierarchy_list };
    let mut monolith_res = monolith_service
        .update_hierarchy(monolith_request, token, user_id)
        .await?;
    let Some(hierarchy_uuid) =
        monolith_res.data.hierarchy_list.pop().map(|i| i.uuid)
    else {
        return Err(TcpError::MonolithError(String::from(
            "При создании иерархии АСЭ3 1.0 не вернул uuid",
        )));
    };
    messages.add_messages(monolith_res.messages.into());

    Ok(hierarchy_uuid)
}

/// Если "supplier_uuid" пусто необходимо найти запись в request_partner, где request_partner.request_uuid  = "request_uuid"
/// и request_partner.supplier_id = "supplier_id" и is_removed = false.
///
/// Если запись в request_partner не найдена - необходимо создать запись в request_partner с supplier_id, поэтому
/// [`check_request_partner`] вернет [`Either::Left`] c supplier_id
///
/// Если запись в request_partner найдена - необходимо проверить признак is_public - если false - продолжить обработку запроса,
/// request_partner.uuid записать в proposal_head-supplier_uuid
/// Если is_public=true - отменить выполнение запроса и вернуть сообщение: "Нет полномочий". Возвращается [`Either::Right`] c уже
/// существующим [`RequestPartner`]
async fn check_request_partner(
    request_uuid: Uuid,
    supplier_id: i32,
    db_pool: &PgPool,
    messages: &mut Messages,
) -> TcpResult<Either<i32, RequestPartner>> {
    let select = Select::full::<RequestPartner>()
        .eq(RequestPartner::request_uuid, request_uuid)
        .eq(RequestPartner::supplier_id, supplier_id)
        .eq(RequestPartner::is_removed, false);
    let partner = RequestPartner::select_option(&select, db_pool).await?;

    match partner {
        Some(partner) => {
            if partner.is_public {
                messages
                    .add_prepared_message(UpdateProposalMessage::no_authority());
            }

            Ok(Either::Right(partner))
        }
        None => Ok(Either::Left(supplier_id)),
    }
}

fn check_supplier_field(supplier_id: Option<i32>, messages: &mut Messages) {
    if supplier_id.is_none() {
        messages.add_prepared_message(
            UpdateProposalMessage::missing_supplier_id_field(),
        );
    }
}

fn check_header_fields(header: &ProposalHeaderRep, messages: &mut Messages) {
    [
        (header.start_date.is_none(), "Начало срока действия"),
        (header.end_date.is_none(), "Окончание срока действия"),
    ]
    .iter()
    .filter(|(is_empty, _)| *is_empty)
    .for_each(|(_, name)| {
        messages.add_prepared_message(UpdateProposalMessage::missing_header_field(
            name, header,
        ))
    });
}

fn check_item_fields(items: &[ProposalItemRep], messages: &mut Messages) {
    items.iter().for_each(|i| {
        [
            (i.request_item_uuid.is_none(), "UUID позиции ЗЦИ"),
            (i.supplier_price.is_none(), "Цена Организации (без НДС)"),
            (i.supplier_vat_id.is_none(), "Ставка НДС Организации"),
            (
                i.is_possibility == Some(true) && i.possibility_note.is_none(),
                "Причина невозможности поставки",
            ),
            (
                i.pay_condition_id == Some(Some(10))
                    && i.prepayment_percent.is_none(),
                "Размер аванса, %",
            ),
        ]
        .iter()
        .filter(|(is_empty, _)| *is_empty)
        .for_each(|(_, name)| {
            messages.add_prepared_message(
                UpdateProposalMessage::missing_item_field(name, i),
            )
        });
    });
}

fn check_attachment_fields(items: &[Attachment], messages: &mut Messages) {
    if !items
        .iter()
        .any(|i| i.category_id == Some(FoldersCategory::TenderDocumentation))
    {
        messages
            .add_prepared_message(UpdateProposalMessage::not_found_tcp_document())
    }
}

fn finalise_response(
    header: ProposalHeader,
    item_list: Vec<ProposalItem>,
    request_items: AHashMap<Uuid, RequestItem>,
) -> TcpResult<UpdateProposalResponseData> {
    let header =
        ProposalHeaderRep::from_item(header, Some(PROPOSAL_HEADER_RETURN_FIELDS));
    let from_proposal_item = from_item_with_fields(PROPOSAL_ITEM_RETURN_FIELDS);
    let item_list = item_list
        .into_iter()
        .map(|i| {
            // По факту тут не должно произойти ошибки, так как выше были проверки на то что по всем позициям ТКП
            // будет позиция ЗЦИ,
            let request_item =
                request_items.get(&i.request_item_uuid).ok_or_else(|| {
                    TcpError::RecordNotFound(
                        i.request_item_uuid.to_string(),
                        RequestItem::TABLE.to_string(),
                    )
                })?;

            let proposal_item = from_proposal_item(i);

            Ok(UpdateProposalResponseItem {
                proposal_item,
                price: request_item.price,
                vat_id: request_item.vat_id,
            })
        })
        .collect::<TcpResult<_>>()?;

    Ok(UpdateProposalResponseData { header, item_list })
}

pub struct UpdateProposalMessage;

impl UpdateProposalMessage {
    pub fn success() -> Message {
        Message::success(String::from("Данные успешно сохранены"))
    }

    pub fn missing_header_field(
        field_name: &str,
        header: &ProposalHeaderRep,
    ) -> Message {
        Message::info(format!("Заполните поле {}", field_name)).with_param_item(
            ParamItem {
                // Запись может создаваться, поэтому значения в айди могут быть пустыми
                uuid: header.uuid,
                id: header.id.map(|i| i.to_string()).unwrap_or_default(),
                ..Default::default()
            },
        )
    }

    pub fn missing_item_field(field_name: &str, item: &ProposalItemRep) -> Message {
        Message::info(format!("Заполните поле {}", field_name)).with_param_item(
            ParamItem {
                uuid: item.uuid,
                ..Default::default()
            },
        )
    }
    pub fn missing_supplier_id_field() -> Message {
        Message::error(String::from("Выберите организацию"))
    }

    pub fn not_found_tcp_document() -> Message {
        Message::info(String::from("Прикрепите Документ ТКП"))
    }

    pub fn no_authority() -> Message {
        Message::error(String::from("Нет полномочий")).with_param_description(
            "ТКП запрошено на ЭТП ГПБ. Ручное создание невозможно",
        )
    }
}
