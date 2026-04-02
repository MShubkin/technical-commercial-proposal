use super::*;
use asez2_shared_db::db_item::AsezDate;
use asez2_shared_db::db_item::AsezTimestamp;

use asez2_shared_db::uuid;
use shared_essential::presentation::dto::general::ObjectIdentifier;
use shared_essential::presentation::dto::response_request::{Messages, Status};

use crate::application::calls::get_technical_commercial_proposal::*;
use crate::presentation::dto::{
    GetTechnicalCommercialProposalPosition, GetTechnicalCommercialProposalResponse,
};

const MIGS: &[&str] = &["get_technical_commercial_proposal.sql"];

#[tokio::test]
async fn test_get_technical_commercial_proposal() {
    run_db_test(MIGS, |pool| async move {
        let exp_proposal_header = ProposalHeaderRep {
            uuid: Some(uuid!("00000000-0000-0000-0000-000000000001")),
            currency_id: Some(3),
            supplier_sum_excluded_vat_total: Some(Some(11.into())),
            created_at: Some(
                AsezTimestamp::try_from("1999-01-01 00:00:00").unwrap(),
            ),
            ..Default::default()
        };

        let exp_request_header = RequestHeaderRep {
            uuid: Some(uuid!("00000000-0000-0000-0000-000000000001")),
            id: Some(2000000000.into()),
            plan_id: Some(Some(10.into())),
            request_subject: Some(Some("help".into())),
            customer_id: Some(Some(1)),
            purchasing_trend_id: Some(Some(2_i16)),
            ..Default::default()
        };
        let exp_positions = vec![GetTechnicalCommercialProposalPosition {
            proposal_item: ProposalItemRep {
                uuid: Some(uuid!("00000000-0000-0000-0000-000000000004")),
                number: Some(1_i16),
                description_internal: Some("very good".into()),
                quantity: Some(5.into()),
                unit_id: Some(55),
                supplier_price: Some(Some(12.into())),
                supplier_sum_excluded_vat: Some(Some(11.into())),
                manufacturer: Some(Some("Ichigo Station Design".into())),
                pay_condition_id: Some(Some(1_i16)),
                prepayment_percent: Some(Some(1.into())),
                delivery_condition: Some(Some("On a blue moon".into())),
                execution_percent: Some(Some(99.into())),
                is_possibility: Some(true),
                possibility_note: Some(Some("Really?".into())),
                analog_description: Some(Some("010101".into())),
                delivery_period: Some(Some("2001-02-20 00:00:00".into())),
                ..Default::default()
            },
            request_item: RequestItemRep {
                uuid: Some(uuid!("00000000-0000-0000-0000-000000000001")),
                delivery_start_date: Some(
                    AsezDate::try_from("1999-01-01").unwrap(),
                ),
                delivery_end_date: Some(AsezDate::try_from("1999-10-10").unwrap()),
                ..Default::default()
            },
        }];

        let requests: Vec<ObjectIdentifier> = vec![ObjectIdentifier {
            id: 1,
            uuid: uuid!("00000000-0000-0000-0000-000000000001"),
            object_type: Default::default(),
        }];

        let response =
            process_get_technical_commercial_proposal(999, requests, &pool)
                .await
                .unwrap();

        assert_eq!(response.status, Status::Ok);
        assert!(response.messages.messages.is_empty());
        let mut item_list = response.data.item_list;
        assert_eq!(item_list.len(), 1);

        let item = item_list.pop().unwrap();

        assert_eq!(
            exp_proposal_header, item.proposal_header,
            "{:#?}\n{:#?}",
            exp_proposal_header, item.proposal_header
        );
        assert_eq!(
            exp_request_header, item.request_header,
            "{:#?}\n{:#?}",
            exp_request_header, item.request_header
        );
        assert_eq!(
            exp_positions, item.position_list,
            "{:#?}\n{:#?}",
            exp_positions, item.position_list
        );
    })
    .await
}

#[tokio::test]
async fn test_get_technical_commercial_proposal_tcp_not_found() {
    run_db_test(MIGS, |pool| async move {
        let exp_data = GetTechnicalCommercialProposalResponse { item_list: vec![] };

        let mut exp_messages = Messages::default();
        exp_messages.add_prepared_message(
            GetTechnicalCommercialProposalMessage::is_tcp_not_found(&[999]),
        );

        let requests: Vec<ObjectIdentifier> = vec![ObjectIdentifier {
            id: 999,
            uuid: uuid!("00000000-0000-0000-0000-000000000999"),
            object_type: Default::default(),
        }];
        let api_response =
            process_get_technical_commercial_proposal(999, requests, &pool)
                .await
                .unwrap();

        assert_eq!(api_response.status, Status::Ok);
        assert_eq!(
            exp_data, api_response.data,
            "{:#?}\n{:#?}",
            exp_data, api_response.data
        );
        assert_eq!(
            exp_messages, api_response.messages,
            "{:#?}\n{:#?}",
            exp_messages, api_response.messages
        );
    })
    .await
}
