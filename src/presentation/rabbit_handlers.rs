use crate::application::action::commercial_offer::{
    commercial_offer_add_doc_response, commercial_offer_request_confirmation,
    commercial_offer_response,
};
use shared_essential::presentation::dto::integration::commercial_offer::{
    request_confirmation::CommercialOfferRequestConfirmationData,
    response::CommercialOfferResponseData,
};
use shared_essential::presentation::dto::technical_commercial_proposal::TcpResult;
use tracing::info;

use sqlx::PgPool;

pub(crate) async fn handle_commercial_offer_request_confirmation(
    request: CommercialOfferRequestConfirmationData,
    pool: &PgPool,
) -> TcpResult<()> {
    info!(
        kind = "tcp",
        "process_commercial_offer_request_confirmation. message: {:?}", &request
    );
    commercial_offer_request_confirmation(request, pool).await?;
    Ok(())
}

pub(crate) async fn handle_commercial_offer_response(
    request: CommercialOfferResponseData,
    pool: &PgPool,
) -> TcpResult<()> {
    info!(
        kind = "tcp",
        "process_commercial_offer_response. message: {:?}", &request
    );
    commercial_offer_response(request, pool).await?;
    Ok(())
}

pub(crate) async fn handle_commercial_offer_add_doc_response(
    request: i32,
    pool: &PgPool,
) -> TcpResult<uuid::Uuid> {
    info!(
        kind = "tcp",
        "process_commercial_offer_add_doc_response. message: {:?}", &request
    );
    let hierarchy_uuid = commercial_offer_add_doc_response(request, pool).await?;
    Ok(hierarchy_uuid)
}
