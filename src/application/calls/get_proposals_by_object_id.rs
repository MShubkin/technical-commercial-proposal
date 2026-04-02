use crate::presentation::dto::{
    GetProposalPriceDataByIdResponse, ProposalPricingItem,
};

use asez2_shared_db::db_item::{AsezDate, DbItem, Select, SelectionKind};
use shared_essential::domain::{
    tables::{
        maths::VatId,
        tcp::{
            GetProposalDetailData, GetProposalDetailDataSelector, ProposalHeader,
            RequestHeader, TCPReviewResult,
        },
    },
    tcp::{PriceInformationRequestStatus, TcpGeneralStatus},
};
use shared_essential::presentation::dto::{
    general::ObjectIdentifier, response_request::Messages,
    technical_commercial_proposal::TcpResult,
};

use sqlx::PgPool;

/// Действие публикации ЗЦИ
/// route - "/get/proposal_list_by_object_id/"
pub(crate) async fn get_proposals_by_object_id(
    _user_id: i32,
    req: ObjectIdentifier,
    db_pool: &PgPool,
) -> TcpResult<(GetProposalPriceDataByIdResponse, Messages)> {
    let proposals = get_proposals(req, db_pool).await?;

    build_response(proposals)
}

/// Сначала идёт выборка RequestHeader (ЗЦИ) по uuid ППЗ/ДС.
/// Потом по uuid RequestHeader (ЗЦИ) идёт запрос на соответствующие ТКП
/// (ProposalHead).
///
/// При этом выбираем ТКП у которых срок действие включает сегодняшней день
/// (не берём просроченные, не берём недозрелые). Также статус result_id должен
/// быть рассмотренный (50).
async fn get_proposals(
    req: ObjectIdentifier,
    pool: &PgPool,
) -> TcpResult<Vec<GetProposalDetailData>> {
    let today = AsezDate::today();
    let req_sel = Select::with_fields([RequestHeader::uuid])
        .eq(RequestHeader::plan_uuid, req.uuid)
        .ne(RequestHeader::status_id, PriceInformationRequestStatus::Deleted);

    let request_uuids = RequestHeader::select(&req_sel, pool).await?;
    // Если же не были найдены ЗЦИ, то и выборка по ТКП не будет иметь смысла
    if request_uuids.is_empty() {
        return Ok(Vec::new());
    };

    let proposal_sel = Select::full::<ProposalHeader>()
        .eq(ProposalHeader::result_id, TCPReviewResult::Consider)
        .ne(ProposalHeader::status_id, TcpGeneralStatus::Deleted)
        .add_expand_filter(
            ProposalHeader::start_date,
            SelectionKind::LessEqual,
            [today],
        )
        .add_expand_filter(
            ProposalHeader::end_date,
            SelectionKind::GreaterEqual,
            [today],
        )
        .in_any(
            ProposalHeader::request_uuid,
            request_uuids.into_iter().map(|x| x.uuid),
        );

    let proposals =
        GetProposalDetailDataSelector::new(proposal_sel).get(pool).await?;
    Ok(proposals)
}

/// Сумма без НДС берётся из заголовка ТКП. Сумма с НДС берётся из позиций
/// (не знаю зачем). ИД НДС берётся из позиций, при этом если он разный в разных
/// позициях, то ставится "compound".
fn build_response(
    proposals: Vec<GetProposalDetailData>,
) -> TcpResult<(GetProposalPriceDataByIdResponse, Messages)> {
    let item_list = proposals
        .iter()
        .map(|x| {
            let supplier_vat_id =
                x.items
                    .iter()
                    .map(|x| x.vat_id.unwrap_or(VatId::Compound))
                    .reduce(|acc, x| match acc == x {
                        true => acc,
                        false => VatId::Compound,
                    })
                    .unwrap_or(VatId::Compound) as i32;
            let sum_exc =
                x.proposal_header.sum_excluded_vat_total.unwrap_or_default();
            let sum_inc = x.items.iter().fold(0.into(), |acc, x| {
                acc + x.sum_included_vat.unwrap_or_default()
            });
            ProposalPricingItem {
                uuid: x.proposal_header.uuid,
                id: x.proposal_header.id,
                supplier_id: x.partner.supplier_id,
                supplier_vat_id,
                // `result_id` должно быть и так и так 50
                result_id: x.proposal_header.result_id.unwrap_or_default(),
                sum_excluded_vat: sum_exc,
                sum_included_vat: sum_inc,
            }
        })
        .collect::<Vec<_>>();

    let data = GetProposalPriceDataByIdResponse { item_list };
    let messages = Messages::default();
    Ok((data, messages))
}
