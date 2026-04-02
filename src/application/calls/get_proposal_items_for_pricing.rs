use asez2_shared_db::db_item::{joined::JoinTo, Select};
use either::Either;
use itertools::Itertools;
use shared_essential::{
    domain::tcp::{
        JoinedProposalItemRequestItemSelector, ProposalItem, RequestItem,
    },
    presentation::dto::{
        response_request::{Message, Messages},
        technical_commercial_proposal::TcpResult,
    },
};
use sqlx::PgPool;

use crate::presentation::dto::{
    ProposalItemForPricing, ProposalItemsForPricingRequest,
    ProposalItemsForPricingResponse,
};

// Получение данных ППЗ/ДС позиций ТКП
// route - "/get/proposal_items_for_pricing/"
pub(crate) async fn get_proposal_items_for_pricing(
    ProposalItemsForPricingRequest { uuid, .. }: ProposalItemsForPricingRequest,
    db_pool: &PgPool,
) -> TcpResult<(ProposalItemsForPricingResponse, Messages)> {
    let joined_partner_request_items = JoinedProposalItemRequestItemSelector::new(
        Select::with_fields([ProposalItem::price])
            .eq(ProposalItem::proposal_uuid, uuid),
    )
    .set_request_item(
        RequestItem::join_default()
            .selecting(Select::with_fields([RequestItem::plan_item_uuid])),
    )
    .get(db_pool)
    .await?;

    let (item_list, messages): (_, Vec<Message>) = joined_partner_request_items
        .into_iter()
        .partition_map(|item| match item.proposal_item.price {
            Some(price) => Either::Left(ProposalItemForPricing {
                plan_item_uuid: item.request_item.plan_item_uuid,
                proposal_price: price,
            }),
            // price не должен быть None, т.к. проверяется в предыдущем запросе
            // Сообщение на случай, если данные в базе некорректны, либо некорреткный
            // переданный uuid
            None => Either::Right(Message::warn(format!(
                "В выбранном ТКП ({}) не указана Цена Организации",
                item.proposal_item.uuid
            ))),
        });

    Ok((ProposalItemsForPricingResponse { item_list }, Messages::from(messages)))
}
