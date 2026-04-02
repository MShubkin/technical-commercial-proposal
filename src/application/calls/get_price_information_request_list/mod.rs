use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use shared_essential::presentation::dto::{
    response_request::*, technical_commercial_proposal::TcpResult,
};

use crate::application::calls::{
    get_price_information_requests_by_plan_uuid,
    get_price_information_requests_by_plan_uuid_vec, get_tkp_by_uuid_request,
    get_tkp_by_uuid_request_vec,
};
use crate::presentation::dto::{
    PriceInformationRequest, TechnicalCommercialProposal,
};

/// Получение списка ЗЦИ по uuid ППЗ
/// Route - /rest/technical_commercial_proposal/v1/get_price_information_request_by_plan_uuid/{uuid}/
pub(crate) async fn get_price_information_request_by_plan_uuid(
    path: String,
    pool: &PgPool,
) -> TcpResult<ApiResponse<PaginatedData<PriceInformationRequest>, ()>> {
    let ppz_uuid_str = &path;
    let uuid = Uuid::parse_str(ppz_uuid_str)?;

    let data = get_price_information_requests_by_plan_uuid(pool, uuid).await?;

    Ok((data, Messages::default()).into())
}

/// Route - /rest/technical_commercial_proposal/v1/get_price_information_request_by_plan_uuid_vec/
pub(crate) async fn get_price_information_request_by_plan_uuid_vec(
    plan_uuids: Vec<Uuid>,
    pool: &PgPool,
) -> TcpResult<ApiResponse<HashMap<Uuid, Vec<PriceInformationRequest>>, ()>> {
    let data =
        get_price_information_requests_by_plan_uuid_vec(pool, plan_uuids).await?;

    Ok((data, Messages::default()).into())
}

/// Получение списка ТКП
/// Route - /rest/technical_commercial_proposal/v1/get_tkp_by_request_uuid/{uuid}/
pub(crate) async fn get_tkp_by_request_uuid(
    path: String,
    pool: &PgPool,
) -> TcpResult<ApiResponse<PaginatedData<TechnicalCommercialProposal>, ()>> {
    let uuid = Uuid::parse_str(&path)?;
    let data = get_tkp_by_uuid_request(pool, uuid).await?;

    Ok((data, Messages::default()).into())
}

/// Получение списка ТКП
/// Route - /rest/technical_commercial_proposal/v1/get_tkp_by_request_uuid_vec/
pub(crate) async fn get_tkp_by_request_uuid_vec(
    request_uuids: Vec<Uuid>,
    pool: &PgPool,
) -> TcpResult<ApiResponse<HashMap<Uuid, Vec<TechnicalCommercialProposal>>, ()>> {
    tracing::info!(
        kind = "tcp",
        "get_tkp_by_request_uuid_vec. request_uuids:: `{:?}`",
        &request_uuids
    );
    let data = get_tkp_by_uuid_request_vec(pool, request_uuids).await?;

    Ok((data, Messages::default()).into())
}
