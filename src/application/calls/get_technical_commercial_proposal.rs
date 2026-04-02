use crate::presentation::dto::{
    GetTechnicalCommercialProposalItemResponse,
    GetTechnicalCommercialProposalPosition, GetTechnicalCommercialProposalResponse,
};
use ahash::AHashMap;
use asez2_shared_db::db_item::Select;
use asez2_shared_db::{DbAdaptor, DbItem};
use itertools::Itertools;
use shared_essential::domain::tcp::{
    GetProposalDetailDataSelector, ProposalHeader, ProposalHeaderRep, ProposalItem,
    ProposalItemRep, RequestHeader, RequestHeaderRep, RequestItem, RequestItemRep,
    RequestPartner, RequestPartnerRep, TcpGeneralStatus,
};
use shared_essential::presentation::dto::general::ObjectIdentifier;
use shared_essential::presentation::dto::{
    response_request::{ApiResponse, Messages},
    technical_commercial_proposal::TcpResult,
};

use asez2_shared_db::db_item::joined::JoinTo;
use shared_essential::presentation::dto::response_request::{Message, ParamItem};
use sqlx::PgPool;

pub(crate) async fn process_get_technical_commercial_proposal(
    _user_id: i32,
    req: Vec<ObjectIdentifier>,
    pool: &PgPool,
) -> TcpResult<ApiResponse<GetTechnicalCommercialProposalResponse, ()>> {
    let mut messages = Messages::default();

    let request_uuids = req.iter().map(|x| x.uuid);

    let select = Select::with_fields(PROPOSAL_HEADER_FIELDS)
        .in_any(ProposalHeader::request_uuid, request_uuids)
        .eq(ProposalHeader::status_id, TcpGeneralStatus::Received);

    let item_select = Select::full::<ProposalItem>();
    let item_select =
        ProposalItem::join_default().selecting(item_select).distinct_aggr(true);

    let details = GetProposalDetailDataSelector::new(select)
        .set_items(item_select)
        .get(pool)
        .await?;

    if details.is_empty() {
        messages.add_prepared_message(
            GetTechnicalCommercialProposalMessage::is_tcp_not_found(
                &req.iter().map(|x| x.id).collect_vec(),
            ),
        );
        return Ok(
            (GetTechnicalCommercialProposalResponse::default(), messages).into()
        );
    }

    let request_item_uuids = details
        .iter()
        .flat_map(|item| item.items.iter().map(|x| x.request_item_uuid))
        .collect_vec();

    let select =
        Select::full::<RequestItem>().in_any(RequestItem::uuid, request_item_uuids);
    let mut request_items = RequestItem::select(&select, pool)
        .await?
        .into_iter()
        .map(|x| (x.uuid, x))
        .collect::<AHashMap<_, _>>();

    let mut item_list = Vec::with_capacity(details.len());

    details.into_iter().for_each(|item| {
        let proposal_header = ProposalHeaderRep::from_item(
            item.proposal_header,
            Some(PROPOSAL_HEADER_FIELDS),
        );
        let partner =
            RequestPartnerRep::from_item(item.partner, Some(PARTNER_FIELDS));
        let request_header = RequestHeaderRep::from_item(
            item.request_header,
            Some(REQUEST_HEADER_FIELDS),
        );

        let mut position_list = Vec::with_capacity(item.items.len());
        item.items.into_iter().for_each(|item| {
            let request_item = if let Some(request_item) =
                request_items.remove(&item.request_item_uuid)
            {
                RequestItemRep::from_item(request_item, Some(REQUEST_ITEM_FIELDS))
            } else {
                Default::default()
            };

            let proposal_item =
                ProposalItemRep::from_item(item, Some(PROPOSAL_ITEM_FIELDS));

            let item = GetTechnicalCommercialProposalPosition {
                proposal_item,
                request_item,
            };
            position_list.push(item);
        });

        let response_item = GetTechnicalCommercialProposalItemResponse {
            proposal_header,
            partner,
            request_header,
            position_list,
        };
        item_list.push(response_item);
    });

    let response = GetTechnicalCommercialProposalResponse { item_list };

    Ok((response, messages).into())
}

pub(crate) const PROPOSAL_HEADER_FIELDS: &[&str] = &[
    ProposalHeader::sum_excluded_vat_total,
    ProposalHeader::currency_id,
    ProposalHeader::created_at,
];

pub(crate) const REQUEST_HEADER_FIELDS: &[&str] = &[
    RequestHeader::id,
    RequestHeader::purchasing_trend_id,
    RequestHeader::request_subject,
    RequestHeader::customer_id,
    RequestHeader::plan_id,
];

pub(crate) const PROPOSAL_ITEM_FIELDS: &[&str] = &[
    ProposalItem::number,
    ProposalItem::description_internal,
    ProposalItem::quantity,
    ProposalItem::price,
    ProposalItem::sum_excluded_vat,
    ProposalItem::unit_id,
    ProposalItem::execution_percent,
    ProposalItem::pay_condition_id,
    ProposalItem::prepayment_percent,
    ProposalItem::delivery_condition,
    ProposalItem::is_possibility,
    ProposalItem::possibility_note,
    ProposalItem::manufacturer,
    ProposalItem::analog_description,
    ProposalItem::delivery_period,
];

pub(crate) const REQUEST_ITEM_FIELDS: &[&str] =
    &[RequestItem::delivery_start_date, RequestItem::delivery_end_date];

pub(crate) const PARTNER_FIELDS: &[&str] = &[RequestPartner::supplier_id];

pub(crate) struct GetTechnicalCommercialProposalMessage;

impl GetTechnicalCommercialProposalMessage {
    pub fn is_tcp_not_found(request_ids: &[i64]) -> Message {
        let join_ids =
            request_ids.iter().map(i64::to_string).collect::<Vec<_>>().join(", ");
        Message::info(format!("Для ЗЦИ {} ТКП не найдено", join_ids))
            .with_param_items(
                request_ids.iter().map(ParamItem::from_id).collect_vec(),
            )
    }
}
