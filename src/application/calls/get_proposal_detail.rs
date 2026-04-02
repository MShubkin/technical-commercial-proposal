use crate::presentation::dto::{GetProposalDataResponse, GetProposalItem};

use asez2_shared_db::db_item::{from_item_with_fields, Select};
use asez2_shared_db::{DbAdaptor, DbItem};
use shared_essential::domain::tcp::{
    GetProposalDetailDataSelector, ProposalHeader, ProposalHeaderRep, ProposalItem,
    RequestItem, TcpGeneralStatus,
};
use shared_essential::presentation::dto::general::{Metadata, UserId};
use shared_essential::presentation::dto::{
    response_request::{ApiResponse, Messages},
    technical_commercial_proposal::{TcpError, TcpResult},
};

use ahash::AHashMap;
use asez2_shared_db::db_item::joined::JoinTo;
use itertools::Itertools;
use sqlx::PgPool;

pub(crate) async fn get_proposal_detail(
    UserId { user_id }: UserId,
    id: i64,
    pool: &PgPool,
) -> TcpResult<ApiResponse<GetProposalDataResponse, ()>> {
    let select = Select::full::<ProposalHeader>().eq(ProposalHeader::id, id);

    let item_select = Select::full::<ProposalItem>();
    let item_select =
        ProposalItem::join_default().selecting(item_select).distinct_aggr(true);

    let details = GetProposalDetailDataSelector::new(select)
        .set_items(item_select)
        .get(pool)
        .await?
        .pop()
        .ok_or_else(|| {
            TcpError::RecordNotFound(
                id.to_string(),
                ProposalHeader::TABLE.to_string(),
            )
        })?;

    let uuids = details.items.iter().map(|x| x.request_item_uuid);

    let select =
        Select::with_fields(REQUEST_ITEM_FIELDS).in_any(RequestItem::uuid, uuids);
    let request_items = RequestItem::select(&select, pool)
        .await?
        .into_iter()
        .map(|x| (x.uuid, x))
        .collect::<AHashMap<_, _>>();

    let from_item = from_item_with_fields(P_ITEM_RET_FIELDS);
    let item_list = details
        .items
        .into_iter()
        .sorted_by_key(|x| x.number)
        .map(|x| {
            let uuid = &x.request_item_uuid;
            let request_item = request_items.get(uuid).ok_or_else(|| {
                TcpError::RecordNotFound(
                    uuid.to_string(),
                    RequestItem::TABLE.to_string(),
                )
            })?;
            let proposal_item = from_item(x);

            let meta = if details.proposal_header.created_by != user_id
                || details.proposal_header.status_id == TcpGeneralStatus::Deleted
            {
                Some(Metadata {
                    disabled_field_list: DISABLED_FIELDS
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                })
            } else {
                None
            };

            Ok(GetProposalItem {
                proposal_item,
                _meta: meta,
                price: request_item.price,
                vat_id: request_item.vat_id,
            })
        })
        .collect::<TcpResult<Vec<_>>>()?;

    let header = ProposalHeaderRep::from_item(
        details.proposal_header,
        Some(HEADER_RET_FIELDS),
    );

    Ok((
        GetProposalDataResponse {
            supplier_id: details.partner.supplier_id,
            created_by: details.request_header.created_by,
            request_id: details.request_header.id,
            header,
            item_list,
        },
        Messages::default(),
    )
        .into())
}

const DISABLED_FIELDS: &[&str] = &[
    "supplier_price",
    "supplier_vat_id",
    "manufacturer",
    "mark",
    "pay_condition_id",
    "prepayment_percent",
    "delivery_condition",
    "execution_percent",
    "is_possibility",
    "possibility_note",
    "analog_description",
    "delivery_period",
];

pub(crate) const HEADER_RET_FIELDS: &[&str] = &[
    ProposalHeader::id,
    ProposalHeader::uuid,
    ProposalHeader::supplier_uuid,
    ProposalHeader::request_uuid,
    ProposalHeader::hierarchy_uuid,
    ProposalHeader::sum_excluded_vat_total,
    ProposalHeader::contact_phone,
    ProposalHeader::currency_id,
    ProposalHeader::start_date,
    ProposalHeader::end_date,
    ProposalHeader::status_id,
    ProposalHeader::status_check_id,
    ProposalHeader::result_id,
    // It goes from request_head
    // ProposalHeader::created_by,
    ProposalHeader::etp_id,
];

/// ТODO: Some of these fields do not exist at the moment, pending
/// consultant decision.
pub(crate) const P_ITEM_RET_FIELDS: &[&str] = &[
    ProposalItem::uuid,
    ProposalItem::number,
    ProposalItem::description_internal,
    ProposalItem::request_item_uuid,
    ProposalItem::quantity,
    ProposalItem::unit_id,
    ProposalItem::price,
    ProposalItem::vat_id,
    ProposalItem::sum_excluded_vat,
    ProposalItem::sum_included_vat,
    ProposalItem::manufacturer,
    ProposalItem::mark,
    ProposalItem::execution_percent,
    ProposalItem::pay_condition_id,
    ProposalItem::prepayment_percent,
    ProposalItem::delivery_condition,
    ProposalItem::is_possibility,
    ProposalItem::possibility_note,
    ProposalItem::analog_description,
    ProposalItem::delivery_period,
];

/// ТODO: Some of these fields do not exist at the moment, pending
/// consultant decision.
const REQUEST_ITEM_FIELDS: &[&str] =
    &[RequestItem::uuid, RequestItem::price, RequestItem::vat_id];
