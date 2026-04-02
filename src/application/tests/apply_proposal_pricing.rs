use asez2_shared_db::uuid;

use shared_essential::domain::tcp::{
    ProposalHeaderRep, TCPCheckStatus, TCPReviewResult,
};
use shared_essential::presentation::dto::{
    general::ObjectIdentifier,
    response_request::{Message, ParamItem},
};

use crate::application::calls::apply_proposal_pricing::process_apply_proposal_pricing;
use crate::presentation::dto::ApplyPricingProposal;

use super::run_db_test_with_monolith;

const MIGS: &[&str] = &["apply_proposal_pricing.sql"];
const USER_ID: i32 = 123;

#[tokio::test]
async fn test_get_price_detail() {
    let id = 6500000000;
    let req = ApplyPricingProposal {
        is_apply_pricing_consider: Some(true),
        item_list: vec![ObjectIdentifier {
            id,
            uuid: uuid!("00000000-0000-0000-0000-000000000001"),
            ..Default::default()
        }],
    };
    let exp = vec![ProposalHeaderRep {
        id: Some(id),
        uuid: Some(uuid!("00000000-0000-0000-0000-000000000001")),
        status_check_id: Some(TCPCheckStatus::Reviewed),
        result_id: Some(Some(TCPReviewResult::Consider)),
        ..Default::default()
    }];
    let text = "ТКП 6500000000 от Поставщик 455 можно применить при АЦ".to_string();
    let exp_messages =
        vec![Message::success(text).with_param_item(ParamItem::from_id(id))];
    run_db_test_with_monolith(MIGS, |pool, mono| async move {
        let res = process_apply_proposal_pricing(
            USER_ID,
            String::new(),
            req,
            &mono,
            &pool,
        )
        .await
        .unwrap();
        let messages = res.messages.messages;
        let data = res.data.item_list;

        assert_eq!(exp, data, "{:#?}, {:#?}", exp, data);
        assert_eq!(exp_messages, messages);
    })
    .await
}

#[tokio::test]
async fn test_get_price_detail2() {
    let id = 6500000000;
    let req = ApplyPricingProposal {
        is_apply_pricing_consider: Some(false),
        item_list: vec![ObjectIdentifier {
            id,
            uuid: uuid!("00000000-0000-0000-0000-000000000001"),
            ..Default::default()
        }],
    };
    let exp = vec![ProposalHeaderRep {
        id: Some(id),
        uuid: Some(uuid!("00000000-0000-0000-0000-000000000001")),
        status_check_id: Some(TCPCheckStatus::Reviewed),
        result_id: Some(Some(TCPReviewResult::Ignore)),
        ..Default::default()
    }];
    let text = "ТКП 6500000000 от Поставщик 455 нельзя учесть при АЦ".to_string();
    let exp_messages =
        vec![Message::success(text).with_param_item(ParamItem::from_id(id))];
    run_db_test_with_monolith(MIGS, |pool, mono| async move {
        let res = process_apply_proposal_pricing(
            USER_ID,
            String::new(),
            req,
            &mono,
            &pool,
        )
        .await
        .unwrap();
        let messages = res.messages.messages;
        let data = res.data.item_list;

        assert_eq!(exp, data, "{:#?}, {:#?}", exp, data);
        assert_eq!(exp_messages, messages);
    })
    .await
}
