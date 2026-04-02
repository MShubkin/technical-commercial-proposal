use uuid::Uuid;

use super::*;

use asez2_shared_db::db_item::AsezDate;
use asez2_shared_db::{uuid, DbAdaptor};

use shared_essential::presentation::dto::general::{FeWrapper, UserId};
use shared_essential::presentation::dto::response_request::Status;

use crate::application::calls::get_request_price_info_detail::*;
use crate::presentation::dto::{GetRequestPriceInfoDetail, PriceInformationDetail};

const MIGS: &[&str] = &["get_price_details.sql"];

#[tokio::test]
async fn test_get_price_detail() {
    let request_header = RequestHeader {
        uuid: uuid!("00000000-0000-0000-0000-000000000002"),
        plan_uuid: Some(uuid!("90000000-0000-0000-0000-000000000002")),
        plan_id: Some(11),
        id: 2000000001,
        type_request_id: Some(PriceInformationRequestType::Public),
        request_subject: Some("help".to_string()),
        start_date: Some(AsezDate::try_from("2030-02-12").unwrap().to_timestamp()),
        end_date: Some(AsezDate::try_from("2030-02-12").unwrap().to_timestamp()),
        status_id: 10.into(),
        created_by: 999,
        changed_by: 998,
        created_at: AsezDate::try_from("1999-01-01").unwrap().to_timestamp(),
        changed_at: AsezDate::try_from("2024-01-01").unwrap().to_timestamp(),
        ..Default::default()
    };
    let request_header = RequestHeaderRep::from_item::<&str>(request_header, None);
    let item_list = [
        RequestItem {
            uuid: uuid!("00000000-0000-0000-0000-000000000003"),
            number: 3,
            request_uuid: uuid!("00000000-0000-0000-0000-000000000002"),
            plan_item_uuid: uuid!("82000000-0000-0000-0000-000000000003"),
            delivery_start_date: AsezDate::try_from("1999-01-01").unwrap(),
            delivery_end_date: AsezDate::try_from("1999-10-10").unwrap(),
            ..Default::default()
        },
        RequestItem {
            uuid: uuid!("00000000-0000-0000-0000-000000000004"),
            number: 4,
            request_uuid: uuid!("00000000-0000-0000-0000-000000000002"),
            plan_item_uuid: uuid!("82000000-0000-0000-0000-000000000004"),
            delivery_start_date: AsezDate::try_from("1999-01-01").unwrap(),
            delivery_end_date: AsezDate::try_from("1999-10-10").unwrap(),
            ..Default::default()
        },
    ]
    .into_iter()
    .map(|x| {
        let mut entity = RequestItemRep::from_item::<&str>(x, None);
        entity.request_uuid = None;
        FeWrapper::new(entity)
    })
    .collect();
    let partner_list = [RequestPartner {
        uuid: uuid!("00000000-0000-0000-0000-100000000002"),
        request_uuid: uuid!("00000000-0000-0000-0000-000000000002"),
        ..Default::default()
    }]
    .into_iter()
    .map(|x| {
        let mut entity = RequestPartnerRep::from_item::<&str>(x, None);
        entity.request_uuid = None;
        FeWrapper::new(entity)
            .add_field(
                "proposal_uuid",
                uuid!("00000000-0000-0000-0000-000000000002"),
            )
            .add_field("proposal_id", 6500000001i64)
            .add_field("question_answer", [1, 0].to_vec())
            .add_field(
                "receive_date",
                AsezDate::try_from_yo(2024, 1).unwrap().to_timestamp(),
            )
            .add_field("start_date", AsezDate::try_from_yo(2025, 10).unwrap())
            .add_field("end_date", AsezDate::try_from_yo(2025, 20).unwrap())
            .add_field("status_id", 0)
            .add_field("proposal_source", Option::<String>::None)
            .add_field("status_check_id", 0i64)
            .add_field("result_id", Option::<i64>::None)
            .add_field("hierarchy_uuid", Option::<Uuid>::None)
    })
    .collect::<Vec<_>>();
    let exp = PriceInformationDetail {
        request_header,
        item_list,
        partner_list,
    };
    run_db_test(MIGS, |pool| async move {
        let req = GetRequestPriceInfoDetail { id: 2000000001 };
        let user_id = UserId { user_id: 997 };
        let res = get_request_price_info_detail(&pool, req, user_id).await.unwrap();

        assert_eq!(res.status, Status::Ok);
        assert!(res.messages.is_empty());

        assert_eq!(exp, res.data, "{:#?}\n{:#?}", exp, res.data);
    })
    .await
}
