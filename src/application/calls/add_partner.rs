use std::collections::hash_map::Entry;

use crate::presentation::dto::{
    CheckAddPartnerResponse, CheckPartnerItem, CheckPartnerReq, SupplierId,
};

use asez2_shared_db::db_item::joined::JoinTo;
use asez2_shared_db::db_item::Select;
use shared_essential::domain::tcp::{
    PartnerWithProposalsSelector, ProposalHeader, RequestPartner, TcpGeneralStatus,
};
use shared_essential::presentation::dto::{
    response_request::{ApiResponse, Message, Messages, PaginatedData},
    technical_commercial_proposal::TcpResult,
};

use ahash::AHashMap;
use sqlx::PgPool;

// Обработчик по ручке
pub(crate) async fn process_check_add_partner(
    _user_id: i32,
    req: CheckPartnerReq,
    pool: &PgPool,
) -> TcpResult<ApiResponse<CheckAddPartnerResponse, ()>> {
    let partners = get_partners(&req, pool).await?;
    let (result, messages) = check_partners(req, partners);

    let paginated = PaginatedData::from(result);
    Ok((paginated, messages).into())
}

async fn get_partners(
    req: &CheckPartnerReq,
    pool: &PgPool,
) -> TcpResult<AHashMap<i32, Vec<ProposalHeader>>> {
    let partner_select = Select::full::<RequestPartner>()
        .eq(RequestPartner::request_uuid, req.uuid)
        .in_any(
            RequestPartner::supplier_id,
            req.item_list.iter().map(|x| x.supplier_id),
        );
    let proposal_select = Select::full::<ProposalHeader>()
        .ne(ProposalHeader::status_id, TcpGeneralStatus::Deleted);

    let partners = PartnerWithProposalsSelector::new(partner_select)
        .set_proposals(ProposalHeader::join_default().selecting(proposal_select))
        .get(pool)
        .await?;

    let mut partner_map: AHashMap<i32, Vec<ProposalHeader>> =
        AHashMap::with_capacity(req.item_list.len());

    partners.into_iter().for_each(|p| {
        match partner_map.entry(p.partner.supplier_id) {
            Entry::Occupied(mut o) => {
                o.get_mut().extend(p.proposals);
            }
            Entry::Vacant(v) => {
                v.insert(p.proposals);
            }
        };
    });

    Ok(partner_map)
}

/// При поступлении запроса найти в request_partner запись, где request_uuid = "uuid"
/// И supplier_id = "supplier_id". Если запись НЕ НАЙДЕНА установить "is_allowed": true
/// и вернуть сообщение типа Success. Никаких текстов не передаётся.
/// Если запись НАЙДЕНА переходим к шагу 2
///
/// 2. Найти в proposal_head запись, где supplier_uuid = request_partner-supplier_uuid
/// из записи найденной на предыдущем шаге. Если запись НЕ НАЙДЕНА установить
/// "is_allowed": true и вернуть сообщение типа Success. Никаких текстов не передаётся.
/// Если запись НАЙДЕНА установить "is_allowed": false и вернуть сообщение типа Error:
///
/// "kind" = Error
/// "text": "Для выбранной организаций уже существует ТКП"
fn check_partners(
    req: CheckPartnerReq,
    partners: AHashMap<i32, Vec<ProposalHeader>>,
) -> (Vec<CheckPartnerItem>, Messages) {
    let mut messages = Messages::default();

    let res = req
        .item_list
        .into_iter()
        .map(|SupplierId { supplier_id }| {
            let is_without_partners = partners
                .get(&supplier_id)
                .map(|proposals| proposals.is_empty())
                .unwrap_or(true);

            if !is_without_partners {
                messages.add_prepared_message(Message::error(ERROR_TEXT));
            }

            CheckPartnerItem {
                supplier_id,
                is_allowed: is_without_partners,
            }
        })
        .collect::<Vec<_>>();

    (res, messages)
}

const ERROR_TEXT: &str = "Для выбранной организации уже существует ТКП";
