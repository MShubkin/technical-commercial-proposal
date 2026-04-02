use ahash::AHashMap;

use asez2_shared_db::db_item::AdaptorableIter;
use itertools::Itertools;
use shared_essential::domain::tcp::ProposalHeader;
use sqlx::PgPool;

use asez2_shared_db::DbAdaptor;
use shared_essential::domain::tables::tcp::{
    JoinedPriceInformationInfoDetail, RequestItemRep, RequestPartnerRep,
};
use shared_essential::presentation::dto::{
    general::UserId, response_request::*, technical_commercial_proposal::TcpResult,
    value::UiValue,
};

use crate::application::calls::get_price_information_request_info_details_by_id;
use crate::presentation::dto::{GetRequestPriceInfoDetail, PriceInformationDetail};
use shared_essential::presentation::dto::general::FeWrapper;

const PROPOSAL_ID: &str = "proposal_id";
const PROPOSAL_UUID: &str = "proposal_uuid";
const PROPOSAL_SOURCE: &str = "proposal_source";
const QUESTION_ANSWER: &str = "question_answer";

/// Операции для ручки "/get/request_price_info_detail/"
pub(crate) async fn get_request_price_info_detail(
    pool: &PgPool,
    detail: GetRequestPriceInfoDetail,
    _user_id: UserId,
) -> TcpResult<ApiResponse<PriceInformationDetail, ()>> {
    let selected_detail =
        get_price_information_request_info_details_by_id(detail.id, pool).await?;
    Ok((PriceInformationDetail::from(selected_detail), Messages::default()).into())
}

impl From<JoinedPriceInformationInfoDetail> for PriceInformationDetail {
    fn from(
        JoinedPriceInformationInfoDetail {
            header,
            items,
            suppliers,
            proposal_headers,
            organization_questions,
        }: JoinedPriceInformationInfoDetail,
    ) -> Self {
        let proposal_map = proposal_headers
            .iter()
            .map(|x| (x.supplier_uuid, x))
            .collect::<AHashMap<_, _>>();

        let mut questions_map = AHashMap::new();
        for s in organization_questions.iter().unique_by(|x| x.uuid) {
            questions_map.entry(s.supplier_uuid).or_insert(vec![]).push(s);
        }

        let item_list = items
            .into_iter()
            .sorted_by_key(|x| x.number)
            .adaptors::<RequestItemRep>()
            .update(|x| x.request_uuid = None)
            .map(FeWrapper::new)
            .collect::<Vec<_>>();
        let partner_list = suppliers
            .into_iter()
            .unique_by(|x| x.uuid)
            .map(|x| {
                let uuid = x.uuid;
                let mut fe_item = RequestPartnerRep::from_item::<&str>(x, None);
                fe_item.request_uuid = None;
                let mut p = FeWrapper::new(fe_item);

                if let Some(pr) = proposal_map.get(&uuid) {
                    p = p
                        .add_field(PROPOSAL_ID, pr.id)
                        .add_field(PROPOSAL_SOURCE, pr.proposal_source.clone())
                        .add_field(PROPOSAL_UUID, pr.uuid)
                        .add_field(ProposalHeader::receive_date, pr.receive_date)
                        .add_field(
                            ProposalHeader::result_id,
                            pr.result_id.map(|v| v as i16),
                        )
                        .add_field(ProposalHeader::start_date, pr.start_date)
                        .add_field(ProposalHeader::end_date, pr.end_date)
                        .add_field(
                            ProposalHeader::status_check_id,
                            pr.status_check_id as i16,
                        )
                        .add_field(ProposalHeader::status_id, pr.status_id as i16)
                        .add_field(
                            ProposalHeader::hierarchy_uuid,
                            pr.hierarchy_uuid,
                        );
                } else {
                    p = p.add_field(ProposalHeader::status_id, UiValue::Null)
                }

                let qa = questions_map
                    .get(&uuid)
                    .map(|question| {
                        let q = question.len() as i64;
                        let a = question
                            .iter()
                            .filter(|x| x.answer_created_at.is_some());
                        vec![q, a.count() as i64]
                    })
                    .unwrap_or(vec![0, 0]);
                p.add_field(QUESTION_ANSWER, qa)
            })
            .collect::<Vec<_>>();

        PriceInformationDetail {
            request_header: DbAdaptor::from_item::<&str>(header, None),
            item_list,
            partner_list,
        }
    }
}
