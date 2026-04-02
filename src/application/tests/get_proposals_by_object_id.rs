use super::*;

use asez2_shared_db::uuid;
use shared_essential::presentation::dto::general::ObjectIdentifier;

use crate::application::calls::get_proposals_by_object_id::*;
use crate::presentation::dto::ProposalPricingItem;

const MIGS: &[&str] = &["get_proposals_by_object_id.sql"];

#[tokio::test]
async fn success() {
    let req = ObjectIdentifier {
        uuid: uuid!("90000000-0000-0000-0000-000000000002"),
        ..ObjectIdentifier::default()
    };
    let exp = vec![
        ProposalPricingItem {
            uuid: uuid!("00000000-0000-0000-0000-000000000001"),
            id: 6500000000,
            supplier_id: 1,
            supplier_vat_id: 11,
            result_id: TCPReviewResult::Consider,
            sum_excluded_vat: 0.55.into(),
            sum_included_vat: 13.into(),
        },
        ProposalPricingItem {
            uuid: uuid!("00000000-0000-0000-0000-000000000002"),
            id: 6500000001,
            supplier_id: 2,
            supplier_vat_id: 11,
            result_id: TCPReviewResult::Consider,
            sum_excluded_vat: 11.into(),
            sum_included_vat: 18.into(),
        },
    ];
    run_db_test(MIGS, |pool| async move {
        let (data, messages) =
            get_proposals_by_object_id(1, req, &pool).await.unwrap();

        assert!(messages.is_empty());

        assert_eq!(exp, data.item_list);
    })
    .await
}

#[tokio::test]
async fn not_found_request_header() {
    let req = ObjectIdentifier {
        // Не существует request_head с таким uuid
        uuid: uuid!("90000000-0000-0000-0000-000000000666"),
        ..ObjectIdentifier::default()
    };

    run_db_test(MIGS, |pool| async move {
        let (data, messages) =
            get_proposals_by_object_id(1, req, &pool).await.unwrap();

        assert!(messages.is_empty());

        assert!(data.item_list.is_empty());
    })
    .await
}
