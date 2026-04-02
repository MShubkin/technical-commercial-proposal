use tracing::info;
use uuid::Uuid;

use asez2_shared_db::db_item::selection::SelectionKind::In;
use asez2_shared_db::db_item::Select;
use rabbit_services::processing::ProcessingService;
use shared_essential::{
    domain::Section,
    presentation::dto::{
        processing::{CompletePlansRequest, GetPlanResponse},
        AsezResult,
    },
};

const FIELDS: &[&str] = &[
    "id",
    "customer_id",
    "uuid",
    "purchasing_method_id",
    "purchasing_type_id",
    "purchasing_kind_id",
    "currency_id",
    "delivery_start_date",
    "delivery_end_date",
];

pub async fn get_processing_plans(
    plan_uuids: Vec<Uuid>,
    user_id: i32,
    processing: ProcessingService,
) -> AsezResult<GetPlanResponse> {
    info!(kind = "tcp", "get_processing_plans. json:\n{:?}", &plan_uuids);

    //Fill Select Struct
    let select =
        Select::with_fields(FIELDS).add_expand_filter("uuid", In, plan_uuids);

    let item_fields = FIELDS.iter().map(|x| x.to_string()).collect::<Vec<_>>();

    let req = CompletePlansRequest {
        select,
        item_fields,
        section: Section::None,
        user_id,
    };
    processing.get_complete_plans(req).await
}
