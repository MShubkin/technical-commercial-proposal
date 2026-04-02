use crate::presentation::dto::{
    ApplyPricingProposal, ApplyPricingProposalResponse,
};

use asez2_shared_db::db_item::{AdaptorableIter, AsezTimestamp, Select};
use asez2_shared_db::DbItem;

use monolith_service::http::MonolithHttpService;

use shared_essential::domain::tcp::{
    ProposalHeader, ProposalWithPartnersSelector, RequestPartner, TCPCheckStatus,
    TCPReviewResult,
};
use shared_essential::presentation::dto::{
    response_request::{ApiResponse, Message, Messages, PaginatedData, ParamItem},
    technical_commercial_proposal::{TcpError, TcpResult},
};

use ahash::AHashMap;
use itertools::Itertools;
use sqlx::PgPool;
use uuid::Uuid;

#[tracing::instrument(skip_all)]
pub(crate) async fn process_apply_proposal_pricing(
    user_id: i32,
    monolith_token: String,
    req: ApplyPricingProposal,
    monolith: &MonolithHttpService,
    pool: &PgPool,
) -> TcpResult<ApiResponse<ApplyPricingProposalResponse, ()>> {
    // Если "is_apply_pricing_consider": true записать 50 (Учитывать при АЦ)
    // Если "is_apply_pricing_consider": false записать 60 (Не учитывать при АЦ)
    let allowed = req.is_apply_pricing_consider.unwrap_or(false);
    let result_id = match allowed {
        false => TCPReviewResult::Ignore,
        true => TCPReviewResult::Consider,
    };
    let changed_at = AsezTimestamp::now();

    let uuids = req.item_list.iter().map(|x| x.uuid);
    let proposals =
        Select::full::<ProposalHeader>().in_any(ProposalHeader::uuid, uuids);

    let proposals = ProposalWithPartnersSelector::new(proposals).get(pool).await?;

    // This is for querying the monolith.
    let mut partner_ids = Vec::new();
    // We need this because the monolith will only return an id, and we
    // need uuids for matching to header.
    let mut partner_map_a = AHashMap::new();
    let proposals = proposals
        .into_iter()
        .map(|x| {
            let s = x.supplier;
            partner_map_a.insert(s.supplier_id, s.uuid);
            partner_ids.push(s.supplier_id);
            x.header
        })
        .collect::<Vec<_>>();

    let mut partner_map_b = AHashMap::new();
    // We do the monolith request before starting the transaction and not during,
    // in order to avoid long transactions.
    // We do the request before starting the transaction and not after, to avoid
    // updating in the DB and then falling out with a different error without
    // possibility of rollback.
    let suppliers = monolith
        .search_organization_by_id(&partner_ids, monolith_token, user_id)
        .await?;
    // We need to find supplier.text by supplier uuid.
    for supplier in suppliers {
        let id = supplier.id;
        if let Some(uuid) = partner_map_a.remove(&id) {
            partner_map_b.insert(uuid, supplier.text);
        };
    }
    // Аn extra set of checks, so that if appropriate records are not found
    // in the monolith, we should catch it, as all the supplier ids requested
    // should be removed from the map once they are found. If they are not found
    // and therefore not removed, this means that the monolith
    if !partner_map_a.is_empty() {
        let ids = partner_map_a.iter().map(|(k, _)| k.to_string()).join(", ");
        let msg = format!("В монолите не найдены организации ИД = {ids}.");
        return Err(TcpError::MonolithError(msg));
    }

    // Do update.
    let updatable = proposals
        .into_iter()
        .map(|mut x| {
            x.changed_at = changed_at;
            x.changed_by = user_id;
            x.result_id = Some(result_id);
            // Установить 40 (Рассмотрено)
            x.status_check_id = TCPCheckStatus::Reviewed;
            x
        })
        .collect::<Vec<_>>();

    let mut tx = pool.begin().await?;
    let updated = ProposalHeader::update_vec_returning(
        &updatable,
        Some(UPDATE_FIELDS),
        Some(UPDATE_RET_FIELDS),
        &mut tx,
    )
    .await?;

    // Create messages.
    let messages = append_messages(&updated, partner_map_b, allowed)?;

    let updated =
        updated.into_iter().adaptors_with_fields(RET_FIELDS).collect::<Vec<_>>();

    // Only commit when there are no more errors for us to deal with.
    tx.commit().await?;

    Ok((PaginatedData::from(updated), messages).into())
}

fn append_messages(
    updated: &[ProposalHeader],
    partners: AHashMap<Uuid, String>,
    is_applied: bool,
) -> TcpResult<Messages> {
    let mut messages = Messages::default();
    for updated in updated.iter() {
        let id = updated.id;
        let org = partners.get(&updated.supplier_uuid).ok_or_else(|| {
            TcpError::RecordNotFound(
                "organization (request_partner.uuid) из монолита".to_string(),
                RequestPartner::TABLE.to_string(),
            )
        })?;
        let allowed = match is_applied {
            true => "можно применить",
            false => "нельзя учесть",
        };
        let text = format!("ТКП {id} от {org} {allowed} при АЦ");
        let id = ParamItem::from_id(updated.id);
        let message = Message::success(text).with_param_item(id);
        messages.add_prepared_message(message);
    }
    Ok(messages)
}

const UPDATE_FIELDS: &[&str] = &[
    ProposalHeader::changed_at,
    ProposalHeader::changed_by,
    ProposalHeader::result_id,
    ProposalHeader::status_check_id,
];

const UPDATE_RET_FIELDS: &[&str] = &[
    ProposalHeader::changed_at,
    ProposalHeader::changed_by,
    ProposalHeader::id,
    ProposalHeader::uuid,
    ProposalHeader::result_id,
    ProposalHeader::status_check_id,
    ProposalHeader::supplier_uuid,
];

const RET_FIELDS: &[&str] = &[
    ProposalHeader::id,
    ProposalHeader::uuid,
    ProposalHeader::result_id,
    ProposalHeader::status_check_id,
];
