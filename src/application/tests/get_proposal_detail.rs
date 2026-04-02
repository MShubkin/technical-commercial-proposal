use super::*;

use asez2_shared_db::db_item::AsezDate;
use asez2_shared_db::{uuid, DbAdaptor};
use shared_essential::presentation::dto::general::UserId;
use shared_essential::presentation::dto::response_request::Status;

use crate::application::calls::get_proposal_detail::*;
use crate::presentation::dto::{GetProposalDataResponse, GetProposalItem};

const MIGS: &[&str] = &["get_proposal_detail.sql"];

#[tokio::test]
async fn test_get_proposal_detail() {
    let header = ProposalHeader {
        id: 6500000001,
        uuid: uuid!("00000000-0000-0000-0000-000000000002"),
        supplier_uuid: uuid!("00000000-0000-0000-0000-100000000003"),
        request_uuid: uuid!("00000000-0000-0000-0000-000000000001"),
        hierarchy_uuid: Some(uuid!("90000000-0000-0000-0000-000000000002")),
        sum_excluded_vat_total: Some(11.into()),
        contact_phone: Some("c411m3".to_string()),
        currency_id: 3,
        start_date: Some(AsezDate::try_from("1999-01-01").unwrap()),
        end_date: Some(AsezDate::try_from("1999-01-10").unwrap()),
        status_id: 20.into(),
        status_check_id: 30.into(),
        result_id: Some(50.into()),
        created_by: 999,
        ..Default::default()
    };
    let header =
        ProposalHeaderRep::from_item::<&str>(header, Some(HEADER_RET_FIELDS));
    let item_list = vec![GetProposalItem {
        proposal_item: ProposalItemRep {
            uuid: Some(uuid!("00000000-0000-0000-0000-000000000004")),
            number: Some(1),
            description_internal: Some("very good".to_string()),
            quantity: Some(5.into()),
            unit_id: Some(55),
            supplier_price: Some(Some(12.into())),
            supplier_vat_id: Some(Some(11.into())),
            supplier_sum_excluded_vat: Some(Some(11.into())),
            supplier_sum_included_vat: Some(Some(13.into())),
            manufacturer: Some(Some("Ichigo Station Design".to_string())),
            mark: Some(Some("3".to_string())),
            execution_percent: Some(Some(99.into())),
            pay_condition_id: Some(1.into()),
            prepayment_percent: Some(Some(1.into())),
            delivery_condition: Some(Some("On a blue moon".to_string())),
            is_possibility: Some(true),
            possibility_note: Some(Some("Really?".to_string())),
            analog_description: Some(Some("010101".to_string())),
            delivery_period: Some(Some("2001-02-20 00:00:00".to_string())),
            request_item_uuid: Some(uuid!("00000000-0000-0000-0000-000000000001")),
            ..Default::default()
        },
        price: 5.into(),
        vat_id: 11.into(),
        _meta: None,
    }];
    let exp = GetProposalDataResponse {
        header,
        request_id: 2000000000,
        created_by: 666,
        supplier_id: 5,
        item_list,
    };
    run_db_test(MIGS, |pool| async move {
        let req = 6500000001;
        let res =
            get_proposal_detail(UserId { user_id: 999 }, req, &pool).await.unwrap();

        assert_eq!(res.status, Status::Ok);
        assert!(res.messages.is_empty());

        assert_eq!(exp, res.data, "{:#?}\n{:#?}", exp, res.data);
    })
    .await
}

#[tokio::test]
async fn test_user_mismatch() {
    run_db_test(MIGS, |pool| async move {
        let res = get_proposal_detail(UserId { user_id: 666 }, 6500000001, &pool)
            .await
            .unwrap();

        assert_eq!(res.status, Status::Ok);
        assert!(res.data.item_list[0]._meta.is_some());
    })
    .await
}

#[tokio::test]
async fn test_status_deleted() {
    run_db_test(MIGS, |pool| async move {
        let res = get_proposal_detail(UserId { user_id: 999 }, 6500000002, &pool)
            .await
            .unwrap();

        assert_eq!(res.status, Status::Ok);
        assert!(res.data.item_list[0]._meta.is_some());
    })
    .await
}

#[tokio::test]
async fn test_created_by_from_request_head() {
    run_db_test(MIGS, |pool| async move {
        let res = get_proposal_detail(UserId { user_id: 999 }, 6500000002, &pool)
            .await
            .unwrap();

        assert_eq!(res.status, Status::Ok);
        assert_eq!(res.data.header.created_by, None);
        assert_eq!(res.data.created_by, 666);
    })
    .await
}
