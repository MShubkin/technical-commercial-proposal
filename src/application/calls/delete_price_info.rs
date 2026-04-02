use crate::application::calls::messages_from_success_and_errors;
use crate::presentation::dto::{DeletePriceInfoResponse, PriceInfoCompleteItem};

use asez2_shared_db::db_item::{AsezTimestamp, Select};
use asez2_shared_db::DbItem;
use shared_essential::domain::tcp::{PriceInformationRequestStatus, RequestHeader};
use shared_essential::presentation::dto::{
    general::ObjectIdentifier,
    response_request::{ApiResponse, PaginatedData, ParamItem},
    technical_commercial_proposal::TcpResult,
};

use itertools::{Either, Itertools};
use shared_essential::presentation::dto::response_request::{Message, Messages};
use shared_essential::presentation::dto::technical_commercial_proposal::TcpError;
use sqlx::PgPool;

pub(crate) async fn process_delete_price_info(
    user_id: i32,
    req: Vec<ObjectIdentifier>,
    pool: &PgPool,
) -> TcpResult<ApiResponse<DeletePriceInfoResponse, ()>> {
    let uuids = req.into_iter().map(|x| x.uuid);
    let select =
        Select::with_fields(SELECT_FIELDS).in_any(RequestHeader::uuid, uuids);

    let requests = RequestHeader::select(&select, pool).await?;

    let not_permitted_headers = requests
        .iter()
        .filter(|header| header.created_by != user_id)
        .collect::<Vec<_>>();
    if !not_permitted_headers.is_empty() {
        let message = Message::error(FAILED_CHECK_CREATED_BY)
            .with_param_items(not_permitted_headers)
            .with_param_description(
                "Невозможно удалить ЗЦИ, созданный другим пользователем",
            );
        return Err(TcpError::Business(Messages::from(message)));
    }

    let mut requests_to_delete = Vec::with_capacity(requests.len());
    let now = AsezTimestamp::now();

    // Найти запись в request_head, где status_id = 70 "Проект ТКП"
    // ИЛИ  100 "Ошибка передачи на ЭТП"
    // Если условие выполнено выполняем обновление данных.
    let (ok_params, err_params) = requests.into_iter().partition_map(|mut h| {
        use PriceInformationRequestStatus::*;
        let param = ParamItem::from(&h);
        if matches!(h.status_id, TcpProject | TransferToEtpError) {
            h.changed_at = now;
            h.changed_by = user_id;
            h.status_id = Deleted;
            requests_to_delete.push(h);
            Either::Left(param)
        } else {
            Either::Right(param)
        }
    });
    // Обновление.
    let mut tx = pool.begin().await?;
    RequestHeader::update_vec(&requests_to_delete, UPDATE_FIELDS, &mut tx).await?;
    tx.commit().await?;

    let data = requests_to_delete
        .into_iter()
        .map(|x| PriceInfoCompleteItem {
            identifier: ObjectIdentifier::new(x.id, x.uuid),
            status_id: x.status_id,
        })
        .collect::<Vec<_>>();
    // После обработки всего массива item_list:
    // Для успешно обработанных сформировать сообщение типа Success
    // Для id для которых проверка не была выполнена сформировать сообщение типа Error:
    let messages = messages_from_success_and_errors(
        SUCCESS_TEXT,
        FAIL_TEXT,
        "",
        ok_params,
        err_params,
    );
    let data = PaginatedData::from(data);

    Ok((data, messages).into())
}

const UPDATE_FIELDS: Option<&[&str]> = Some(&[
    RequestHeader::changed_at,
    RequestHeader::changed_by,
    RequestHeader::status_id,
]);

const SELECT_FIELDS: &[&str] = &[
    RequestHeader::id,
    RequestHeader::status_id,
    RequestHeader::uuid,
    RequestHeader::created_by,
];

const SUCCESS_TEXT: &str = "Выбранные ЗЦИ удалены";
const FAIL_TEXT: &str =
    "Выбранные ЗЦИ опубликованы на ЭТП ГПБ. Удаление невозможно";
const FAILED_CHECK_CREATED_BY: &str = "Нет полномочий";
