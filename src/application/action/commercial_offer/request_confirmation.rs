use shared_essential::presentation::dto::integration::commercial_offer::request_confirmation::*;
use shared_essential::{
    domain::tcp::{PriceInformationRequestStatus, RequestHeader},
    presentation::dto::technical_commercial_proposal::{TcpError, TcpResult},
};

use asez2_shared_db::{
    db_item::{AsezTimestamp, Select},
    DbItem,
};

const UPDATE_FIELDS_SUCCESS: Option<&[&str]> = Some(&[
    RequestHeader::status_id,
    RequestHeader::start_date,
    RequestHeader::changed_at,
    RequestHeader::changed_by,
]);

const UPDATE_FIELDS_ERROR: Option<&[&str]> = Some(&[
    RequestHeader::status_id,
    RequestHeader::changed_at,
    RequestHeader::changed_by,
]);

/// Обработка ответа подтверждения по CommercialOfferRequest,
///
/// # Аргументы
/// * `data` — Структура, десериализованная из XML.
/// * `pool` — Соединение с БД.
///
/// # Логика
/// - Если `Status = "success"`:
///     - Обновляются поля `status_id = 90`, `start_date = now()`, `changed_at`, `changed_by`.
///
/// - Если `Status = "error"`:
///     - Обновляются поля `status_id = 100`, `changed_at`, `changed_by`.
///     - В лог выводятся ошибки из поля `errors`, если они есть, и содержимое json.
pub async fn commercial_offer_request_confirmation(
    req: CommercialOfferRequestConfirmationData,
    pool: &sqlx::PgPool,
) -> TcpResult<()> {
    let CommercialOfferRequestConfirmationData { data, user_id, id } = req;

    let req_select = Select::full::<RequestHeader>().eq(RequestHeader::id, id);
    let mut header = RequestHeader::select(&req_select, pool)
        .await?
        .pop()
        .ok_or_else(|| TcpError::not_found(id, RequestHeader::TABLE))?;

    if header.status_id == PriceInformationRequestStatus::TransferToEtpError {
        return Ok(());
    }

    let now = AsezTimestamp::now();

    let (status_id, start_date, fields_to_update) =
        match data.status.to_lowercase().as_str() {
            "success" => (
                PriceInformationRequestStatus::AcceptingIncomingTCPs,
                Some(now),
                UPDATE_FIELDS_SUCCESS,
            ),
            "error" => (
                PriceInformationRequestStatus::TransferToEtpError,
                None,
                UPDATE_FIELDS_ERROR,
            ),
            _ => {
                let msg = format!(
                    "CommercialOfferRequestConfirmation: неизвестный статус '{}'",
                    data.status
                );
                tracing::error!(kind = "tcp", "{msg}");
                return Err(TcpError::InternalError(msg));
            }
        };

    header.status_id = status_id;
    header.start_date = start_date;
    header.changed_at = now;
    header.changed_by = user_id;

    if data.status.eq_ignore_ascii_case("error") {
        if let Some(errors) = &data.errors {
            let flat_errors = errors
                .error
                .iter()
                .map(|e| {
                    format!(
                        "code={}, message={}, field={}",
                        e.code, e.message, e.details.field
                    )
                })
                .collect::<Vec<_>>()
                .join("; ");

            tracing::error!(
                kind = "tcp",
                "CommercialOfferRequestConfirmation: Ошибка подтверждения ЗЦИ: {flat_errors}. Содержимое сообщения: {:#?}",
                data
            );
        }
    }

    let mut tx = pool.begin().await?;
    RequestHeader::update_vec(&[header], fields_to_update, &mut tx).await?;
    tx.commit().await?;

    Ok(())
}
