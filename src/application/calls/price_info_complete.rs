use crate::application::calls::messages_from_success_and_errors;
use crate::presentation::dto::{PriceInfoCompleteItem, PriceInfoCompleteResponse};

use asez2_shared_db::db_item::{AsezTimestamp, Select};
use asez2_shared_db::DbItem;
use shared_essential::domain::tcp::{PriceInformationRequestStatus, RequestHeader};
use shared_essential::presentation::dto::{
    general::ObjectIdentifier, response_request::*,
    technical_commercial_proposal::TcpResult,
};

use itertools::{Either, Itertools};
use sqlx::PgPool;

/// Обработчик по ручке - "/action/request_price_info_complete/"
pub(crate) async fn process_price_info_complete(
    user_id: i32,
    item_list: Vec<ObjectIdentifier>,
    pool: &PgPool,
) -> TcpResult<ApiResponse<PriceInfoCompleteResponse, ()>> {
    let uuids = item_list.iter().map(|x| x.uuid);

    let select = Select::with_fields(FIELDS).in_any(RequestHeader::uuid, uuids);
    let headers = RequestHeader::select(&select, pool).await?;

    // Проверка на полномочия
    let mut messages = Messages::default();
    let not_permitted_headers = headers
        .iter()
        .filter(|item| user_id != item.created_by)
        .collect::<Vec<_>>();
    if !not_permitted_headers.is_empty() {
        let message = Message::error("Нет полномочий")
            .with_param_description(
                "Невозможно закрыть ЗЦИ, созданный другим пользователем",
            )
            .with_param_items(not_permitted_headers);
        messages.add_prepared_message(message);
    }

    if messages.is_error() {
        return Ok(messages.into());
    }

    let now = AsezTimestamp::now();
    let mut to_update = Vec::with_capacity(headers.len());
    let (err_params, ok_params): (Vec<_>, Vec<_>) =
        headers.into_iter().partition_map(|mut h| {
            use PriceInformationRequestStatus::*;
            let id = h.id;
            // Найти запись в request_head, где status_id = 110 "Приём закрыт"
            // ИЛИ  120 "Приём закрыт досрочно".
            // Если условие выполнено выполняем обновление данных.
            if !matches!(h.status_id, EntryClosed | EntryClosedEarly) {
                Either::Left(ParamItem::from_id(id))
            } else {
                h.status_id = Reviewed;
                h.changed_by = user_id;
                h.changed_at = now;
                to_update.push(h);
                Either::Right(ParamItem::from_id(id))
            }
        });
    // После обработки всего массива item_list:
    // Для успешно обработанных сформировать сообщение типа Success
    // Для id для которых проверка не была выполнена сформировать сообщение типа Error:
    messages = messages_from_success_and_errors(
        SUCCESS_TEXT,
        FAIL_TEXT,
        FAIL_DESCRIPTION,
        ok_params,
        err_params,
    );
    // Сохраняем данные в request_head для текущего UID ЗЦИ.
    let mut tx = pool.begin().await?;
    let response = RequestHeader::update_vec_returning(
        &to_update,
        UPDATE_FIELDS,
        Some(FIELDS),
        &mut tx,
    )
    .await?
    .into_iter()
    .map(|x| PriceInfoCompleteItem {
        identifier: ObjectIdentifier::new(x.id, x.uuid),
        status_id: x.status_id,
    })
    .collect::<Vec<_>>();
    tx.commit().await?;

    let paginated = PaginatedData::from(response);
    Ok((paginated, messages).into())
}

const FAIL_DESCRIPTION: &str = "Завершение рассмотрения возможно только для ЗЦИ со статусом \"Приём закрыт\" или \"Приём закрыт досрочно\". Скорректируйте выбор";

const FAIL_TEXT: &str = "Завершение рассмотрения выбранных ЗЦИ невозможно";

const SUCCESS_TEXT: &str =
    "Статус выбранных ЗЦИ изменен на \"Рассмотрение завершено\"";

const FIELDS: &[&str] = &[
    RequestHeader::uuid,
    RequestHeader::id,
    RequestHeader::status_id,
    RequestHeader::created_by,
];

const UPDATE_FIELDS: Option<&[&str]> = Some(&[
    RequestHeader::changed_by,
    RequestHeader::changed_at,
    RequestHeader::status_id,
]);
