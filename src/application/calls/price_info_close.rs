use crate::presentation::dto::{
    PrePriceInfoCloseItem, PrePriceInfoCloseResponse, PriceInfoCloseResponse,
    RequestCloseItem,
};

use asez2_shared_db::db_item::joined::JoinTo;
use asez2_shared_db::db_item::{from_item_with_fields, AsezTimestamp, Select};
use asez2_shared_db::DbItem;
use shared_essential::domain::tcp::{
    PriceInformationRequestStatus, RequestHeader, RequestPartner,
    RequestWithPartners, RequestWithPartnersSelector,
};
use shared_essential::presentation::dto::{
    general::ObjectIdentifier,
    response_request::{ApiResponse, Message, Messages, PaginatedData, ParamItem},
    technical_commercial_proposal::{TcpError, TcpResult},
};

use ahash::AHashMap;
use sqlx::PgPool;
use uuid::Uuid;

async fn get_requests<I: Iterator<Item = Uuid>>(
    uuids: I,
    pool: &PgPool,
) -> TcpResult<Vec<RequestWithPartners>> {
    let request_select =
        Select::full::<RequestHeader>().in_any(RequestHeader::uuid, uuids);
    let partners =
        Select::full::<RequestPartner>().eq(RequestPartner::is_removed, false);

    RequestWithPartnersSelector::new(request_select)
        .set_suppliers(RequestPartner::join_default().selecting(partners))
        .get(pool)
        .await
        .map_err(Into::into)
}

// Returns a list of valid requests and an error message if appropriate
fn check_requests(
    r: Vec<RequestWithPartners>,
    user_id: i32,
) -> (Vec<RequestWithPartners>, Option<Messages>) {
    let mut error_item_list_status = Vec::new();
    let mut error_item_list_creator = Vec::new();
    let valid_requests = r
        .into_iter()
        .filter(|x| {
            !check_and_fill_params_status(&x.header, &mut error_item_list_status)
        })
        .filter(|x| {
            check_and_fill_params_creator(
                &x.header,
                user_id,
                &mut error_item_list_creator,
            )
        })
        .collect::<Vec<_>>();

    let messages = if error_item_list_status.is_empty()
        && error_item_list_creator.is_empty()
    {
        None
    } else {
        Some(fill_messages(error_item_list_status, error_item_list_creator))
    };
    (valid_requests, messages)
}

pub(crate) async fn process_pre_price_info_close(
    user_id: i32,
    req: Vec<ObjectIdentifier>,
    pool: &PgPool,
) -> TcpResult<ApiResponse<PrePriceInfoCloseResponse, ()>> {
    let uuids = req.iter().map(|x| x.uuid);
    let requests = get_requests(uuids, pool).await?;
    let (valid, err_messages) = check_requests(requests, user_id);

    if let Some(m) = err_messages {
        return Ok(m.into());
    }
    let from_header = from_item_with_fields(REQUEST_PRE_RET_FIELDS);
    let item_list = valid
        .into_iter()
        .map(|item| {
            let header = from_header(item.header);
            let supplier_list = item
                .suppliers
                .into_iter()
                .map(|x| x.supplier_id)
                .collect::<Vec<_>>();
            PrePriceInfoCloseItem {
                header,
                supplier_list,
            }
        })
        .collect();
    let response = PrePriceInfoCloseResponse { item_list };
    Ok((response, Messages::default()).into())
}

pub(crate) async fn process_price_info_close(
    user_id: i32,
    req: Vec<RequestCloseItem>,
    pool: &PgPool,
) -> TcpResult<ApiResponse<PriceInfoCloseResponse, ()>> {
    let uuids = req.iter().map(|x| x.identifier.uuid);
    let requests = get_requests(uuids, pool).await?;
    let (valid, err_messages) = check_requests(requests, user_id);

    if let Some(m) = err_messages {
        return Ok(m.into());
    }

    let closing_reasons = req
        .iter()
        .map(|x| (x.identifier.uuid, &x.reason_closing))
        .collect::<AHashMap<_, _>>();

    let changed_at = AsezTimestamp::now();
    let mut ret_list = Vec::with_capacity(req.len());
    let mut param_list = Vec::with_capacity(req.len());

    let from_header = from_item_with_fields(REQUEST_RET_FIELDS);
    let update_items = valid
        .into_iter()
        .map(|item| {
            let mut header = item.header;

            header.changed_at = changed_at;
            header.changed_by = user_id;
            header.status_id = PriceInformationRequestStatus::EntryClosedEarly;
            header.reason_closing = Some(
                closing_reasons
                    .get(&header.uuid)
                    .map(|x| x.to_string())
                    .ok_or_else(|| {
                        TcpError::RecordNotFound(
                            header.id.to_string(),
                            RequestHeader::TABLE.to_string(),
                        )
                    })?,
            );
            let ret = from_header(header.clone());

            param_list.push(ParamItem::from_id(header.id));
            ret_list.push(ret);
            Ok(header)
        })
        .collect::<TcpResult<Vec<_>>>()?;

    let mut tx = pool.begin().await?;
    RequestHeader::update_vec(&update_items, UPDATE_FIELDS, &mut tx).await?;
    tx.commit().await?;

    let m = Message::success("Статус ЗЦИ изменен на \"Прием закрыт досрочно\"")
        .with_parameters(param_list);
    let mut messages = Messages::default();
    messages.add_prepared_message(m);

    let response = PaginatedData::from(ret_list);
    Ok((response, messages).into())
}

/// Возвращает false если всё в порядки и мы в Messages ничего не добавляем.
///
/// Найти запись в request_head, где status_id НЕ РАВНО (90 "Приём ТКП" ИЛИ  150 "Ошибка публикации изменений")
/// Если такие записи найдены дальнейшую обработку не проводить и сформировать сообщение
/// "kind" = Error
/// "text": "Досрочное закрытие выбранных ЗЦИ невозможно"
/// parameters.description: "Досрочное закрытие возможно только для ЗЦИ со статусом
/// "Приём ТКП" или "Ошибка публикации изменений". Скорректируйте выбор"
/// "parameters.item_list": указать id найденных записей.
///
fn check_and_fill_params_status(
    header: &RequestHeader,
    ids: &mut Vec<ParamItem>,
) -> bool {
    use PriceInformationRequestStatus::*;

    if matches!(header.status_id, AcceptingIncomingTCPs | ErrorPublishingChanges) {
        return false;
    }
    ids.push(ParamItem::from_id(header.id));
    true
}

/// При поступлении запроса проверить user_id и request_head - created_by ЗЦИ. Если значения совпадают - продолжать обработку запроса.
/// Если значения разные - отменять выполнение запроса и отобразить сообщение:
/// "kind" = Error
/// "text": "Нет полномочий"
/// "parameters.description": "Невозможно закрыть ЗЦИ, созданное другим пользователем"
fn check_and_fill_params_creator(
    header: &RequestHeader,
    user_id: i32,
    ids: &mut Vec<ParamItem>,
) -> bool {
    if header.created_by == user_id {
        return true;
    };
    ids.push(ParamItem::from_id(header.id));
    false
}

fn fill_messages(
    item_list_status: Vec<ParamItem>,
    item_list_creator: Vec<ParamItem>,
) -> Messages {
    let mut messages = Messages::default();

    const MSG_TEXT_STATUS: &str = "Досрочное закрытие выбранных ЗЦИ невозможно";
    const DESCRIPTION_STATUS: &str =
        "Досрочное закрытие возможно только для ЗЦИ со статусом \
        \"Приём ТКП\" или \"Ошибка публикации изменений\". Скорректируйте выбор";
    if !item_list_status.is_empty() {
        let msg = Message::error(MSG_TEXT_STATUS)
            .with_parameters(item_list_status)
            .with_param_description(DESCRIPTION_STATUS);
        messages.add_prepared_message(msg);
    }

    const MSG_TEXT_CREATOR: &str = "Нет полномочий";
    const DESCRIPTION_CREATOR: &str =
        "Невозможно закрыть ЗЦИ, созданное другим пользователем";
    if !item_list_creator.is_empty() {
        let msg = Message::error(MSG_TEXT_CREATOR)
            .with_parameters(item_list_creator)
            .with_param_description(DESCRIPTION_CREATOR);
        messages.add_prepared_message(msg);
    }

    messages
}

const REQUEST_PRE_RET_FIELDS: &[&str] =
    &[RequestHeader::id, RequestHeader::uuid, RequestHeader::request_subject];

const REQUEST_RET_FIELDS: &[&str] =
    &[RequestHeader::id, RequestHeader::uuid, RequestHeader::status_id];

pub(crate) const UPDATE_FIELDS: Option<&[&str]> = Some(&[
    RequestHeader::status_id,
    RequestHeader::changed_at,
    RequestHeader::changed_by,
    RequestHeader::reason_closing,
]);
