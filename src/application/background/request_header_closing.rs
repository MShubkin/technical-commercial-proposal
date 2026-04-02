use std::sync::Arc;

use asez2_shared_db::DbItem;
use sqlx::PgPool;

use asez2_shared_db::db_item::{AsezTimestamp, Select};
use shared_essential::{
    domain::tcp::{PriceInformationRequestStatus, RequestHeader},
    presentation::dto::technical_commercial_proposal::TcpResult,
};

/// Фоновая задача для закрытия ЗЦИ
///
/// 1. Выбираются все ЗЦИ с end_date <= Текущая дата И status_id = [`PriceInformationRequestStatus::AcceptingIncomingTCPs`]
/// 2. Им устанавливается [`PriceInformationRequestStatus::EntryClosed`] статус и
/// в changed_by устанавливается технический пользователь
pub(crate) async fn request_header_closing(
    pool: Arc<PgPool>,
    monolith_tech_user_id: i32,
) -> TcpResult<()> {
    let now = AsezTimestamp::now();
    let select = Select::default()
        .eq(
            RequestHeader::status_id,
            PriceInformationRequestStatus::AcceptingIncomingTCPs,
        )
        .less_eq(RequestHeader::end_date, now);

    let mut tx = pool.begin().await?;
    let mut headers = RequestHeader::select(&select, &mut tx).await?;

    headers.iter_mut().for_each(|h| {
        h.changed_at = now;
        h.changed_by = monolith_tech_user_id;
        h.status_id = PriceInformationRequestStatus::EntryClosed
    });
    RequestHeader::update_vec(
        &headers,
        Some(&[
            RequestHeader::changed_at,
            RequestHeader::changed_by,
            RequestHeader::status_id,
        ]),
        &mut tx,
    )
    .await?;

    tx.commit().await?;

    Ok(())
}
