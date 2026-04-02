use sqlx::PgPool;
use uuid::Uuid;

use asez2_shared_db::{db_item::Select, DbItem};
use shared_essential::domain::tcp::ProposalHeader;
use shared_essential::presentation::dto::technical_commercial_proposal::{
    TcpError, TcpResult,
};

/// Возвращает hierarchy_uuid ТКП
///
/// # Аргументы
///
/// * `tcp_id` — id ЭТП ГПБ.
/// * `pool` — Пул соединений с БД.
///
/// # Возвращает
///
/// `Ok(())` при успешной обработке, иначе `Err(TcpError)`.
pub async fn commercial_offer_add_doc_response(
    tcp_id: i32,
    pool: &PgPool,
) -> TcpResult<Uuid> {
    let header = ProposalHeader::select_single(
        &Select::full::<ProposalHeader>().eq(ProposalHeader::etp_id, tcp_id),
        pool,
    )
    .await?;

    header.hierarchy_uuid.ok_or_else(|| {
        tracing::error!(
            "ProposalHeader(etp_id={}) найден, но hierarchy_uuid = NULL",
            tcp_id
        );
        TcpError::InternalError(format!(
            "У ТКП (TCPID={}) отсутствует hierarchy_uuid",
            tcp_id
        ))
    })
}
