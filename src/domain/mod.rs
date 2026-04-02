use std::collections::HashMap;

use uuid::Uuid;

use shared_essential::domain::{CompletePlanRep, PlanItemFullRep};

use crate::presentation::dto::PriceInformationRequest;

/// Заполнение струткруры ЗЦИ данными ППЗ/ДС
pub(crate) fn enrich_request_with_plan_data(
    requests: &mut [PriceInformationRequest],
    plans: &[CompletePlanRep],
) {
    let complete_plan_map = plans
        .iter()
        .map(|data| (data.plan.uuid.unwrap_or_default(), data))
        .collect::<HashMap<Uuid, &CompletePlanRep>>();

    for request in requests {
        if let Some(complete_plan) =
            complete_plan_map.get(&request.header.plan_uuid.unwrap_or_default())
        {
            enrich_request_with_plan_header(request, complete_plan);
            enrich_request_with_plan_positions(request, complete_plan);
        }
    }
}

fn enrich_request_with_plan_header(
    request: &mut PriceInformationRequest,
    complete_plan: &CompletePlanRep,
) {
    request.header.plan_id = Some(complete_plan.plan.id.unwrap_or_default());
    request.header.customer_id =
        Some(complete_plan.plan.customer_id.unwrap_or_default());
    request.header.currency_id =
        Some(complete_plan.plan.currency_id.unwrap_or_default());
}

fn enrich_request_with_plan_positions(
    request: &mut PriceInformationRequest,
    complete_plan: &CompletePlanRep,
) {
    let plan_items_map = complete_plan
        .items
        .iter()
        .map(|item| (item.uuid.unwrap_or_default(), item))
        .collect::<HashMap<Uuid, &PlanItemFullRep>>();
    for request_item in request.items.iter_mut() {
        if let Some(plan_item) = plan_items_map.get(&request_item.plan_item_uuid) {
            //TODO дополнить
            request_item.description_internal = plan_item
                .description_internal
                .clone()
                .flatten()
                .unwrap_or_default();
            request_item.category_id = plan_item.category_id.unwrap_or_default();
            request_item.okved2_id = plan_item.okved2_id.unwrap_or_default() as i32;
            request_item.quantity = plan_item.quantity.unwrap_or_default();
            request_item.unit_id = plan_item.unit_id.unwrap_or_default();
        }
    }
}
