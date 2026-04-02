use asez2_shared_db::uuid;

use crate::{
    application::calls::get_proposal_items_for_pricing,
    presentation::dto::ProposalItemsForPricingRequest,
};

use super::run_db_test;

const MIGS: &[&str] = &["get_proposal_items_for_pricing.sql"];

#[tokio::test]
async fn test_successful_get_proposals_for_pricing() {
    run_db_test(MIGS, |pool| async move {
        let (data, messages) = get_proposal_items_for_pricing(
            ProposalItemsForPricingRequest {
                uuid: uuid!("10000000-0000-0000-0000-000000000001"),
                id: 0,
            },
            &pool,
        )
        .await
        .unwrap();

        assert_eq!(2, data.item_list.len());
        assert!(messages.is_empty());
    })
    .await;
}

#[tokio::test]
async fn test_not_found_proposal() {
    run_db_test(MIGS, |pool| async move {
        let (data, _messages) = get_proposal_items_for_pricing(
            ProposalItemsForPricingRequest {
                uuid: uuid!("10000000-0000-0000-0000-000000000000"),
                id: 0,
            },
            &pool,
        )
        .await
        .unwrap();

        assert!(data.item_list.is_empty());
    })
    .await;
}

#[tokio::test]
async fn test_empty_price_warn() {
    run_db_test(MIGS, |pool| async move {
        let (data, messages) = get_proposal_items_for_pricing(
            ProposalItemsForPricingRequest {
                uuid: uuid!("10000000-0000-0000-0000-000000000002"),
                id: 0,
            },
            &pool,
        )
        .await
        .unwrap();

        assert!(data.item_list.is_empty());
        assert!(messages.is_warn());
    })
    .await;
}
