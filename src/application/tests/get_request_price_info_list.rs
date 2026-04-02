//! We test only `database_request` here, since we do not want to mess
//! with view-storage-service in unit tests.
use super::*;

use asez2_shared_db::db_item::{AsezDate, Select, SelectionKind};
use asez2_shared_db::{uuid, DbAdaptor};

use shared_essential::presentation::dto::general::FeWrapper;
use uuid::Uuid;

use crate::application::calls::get_request_price_info_list::*;

const MIGS: &[&str] = &["get_request_price_info_list.sql"];

struct ExpectedItem {
    header: RequestHeader,
    supplier_list: Vec<i32>,
    tkp_in: Vec<i32>,
    tkp_done: [usize; 2],
    hierarchy_uuid_list: Vec<Uuid>,
}

#[tokio::test]
async fn get_full_data() {
    let selection = Select::with_fields(RequestHeader::FIELDS)
        .add_expand_filter("plan_id", SelectionKind::LessEqual, [12])
        .add_replace_order_asc("plan_id");
    let expected = vec![
        ExpectedItem {
            header: RequestHeader {
                uuid: uuid!("00000000-0000-0000-0000-000000000001"),
                plan_uuid: Some(uuid!("90000000-0000-0000-0000-000000000001")),
                plan_id: Some(10),
                id: 2000000000,
                type_request_id: Some(PriceInformationRequestType::Public),
                request_subject: Some("Something 1".to_string()),
                start_date: Some(
                    AsezDate::try_from("2030-02-12").unwrap().to_timestamp(),
                ),
                end_date: Some(
                    AsezDate::try_from("2030-02-12").unwrap().to_timestamp(),
                ),
                status_id: 10.into(),
                created_by: 999,
                changed_by: 998,
                created_at: AsezDate::try_from("1999-01-01")
                    .unwrap()
                    .to_timestamp(),
                changed_at: AsezDate::try_from("2024-01-01")
                    .unwrap()
                    .to_timestamp(),
                ..Default::default()
            },
            supplier_list: vec![345],
            tkp_in: vec![345],
            tkp_done: [0, 1],
            hierarchy_uuid_list: vec![uuid!(
                "00000000-0000-0000-0001-000000000001"
            )],
        },
        ExpectedItem {
            header: RequestHeader {
                uuid: uuid!("00000000-0000-0000-0000-000000000002"),
                plan_uuid: Some(uuid!("90000000-0000-0000-0000-000000000002")),
                plan_id: Some(11),
                id: 2000000001,
                type_request_id: Some(PriceInformationRequestType::Public),
                request_subject: Some("Something 2".to_string()),
                start_date: Some(
                    AsezDate::try_from("2030-02-12").unwrap().to_timestamp(),
                ),
                end_date: Some(
                    AsezDate::try_from("2030-02-12").unwrap().to_timestamp(),
                ),
                status_id: 80.into(),
                created_by: 999,
                changed_by: 998,
                created_at: AsezDate::try_from("1999-01-01")
                    .unwrap()
                    .to_timestamp(),
                changed_at: AsezDate::try_from("2024-01-01")
                    .unwrap()
                    .to_timestamp(),
                ..Default::default()
            },
            supplier_list: vec![456],
            tkp_in: vec![456],
            tkp_done: [1, 1],
            hierarchy_uuid_list: vec![uuid!(
                "00000000-0000-0000-0001-000000000002"
            )],
        },
        ExpectedItem {
            header: RequestHeader {
                uuid: uuid!("00000000-0000-0000-0000-000000000003"),
                plan_uuid: Some(uuid!("90000000-0000-0000-0000-000000000003")),
                plan_id: Some(12),
                id: 2000000002,
                type_request_id: Some(PriceInformationRequestType::Public),
                request_subject: Some("Something 3".to_string()),
                start_date: Some(
                    AsezDate::try_from("2030-02-12").unwrap().to_timestamp(),
                ),
                end_date: Some(
                    AsezDate::try_from("2030-02-12").unwrap().to_timestamp(),
                ),
                status_id: 20.into(),
                created_by: 999,
                changed_by: 998,
                created_at: AsezDate::try_from("1999-01-01")
                    .unwrap()
                    .to_timestamp(),
                changed_at: AsezDate::try_from("2024-01-01")
                    .unwrap()
                    .to_timestamp(),
                ..Default::default()
            },
            supplier_list: vec![567],
            tkp_in: vec![567],
            tkp_done: [1, 1],
            hierarchy_uuid_list: vec![uuid!(
                "00000000-0000-0000-0001-000000000003"
            )],
        },
    ];
    run_db_test(MIGS, |pool| async move {
        let res = database_request(selection, &pool).await.unwrap();

        assert_eq!(3, res.len());

        verify_price_info(res, expected);
    })
    .await
}

#[tokio::test]
async fn get_full_data2() {
    let selection = Select::with_fields(RequestHeader::FIELDS);

    let expected = vec![
        ExpectedItem {
            header: RequestHeader {
                uuid: uuid!("00000000-0000-0000-0000-000000000001"),
                plan_uuid: Some(uuid!("90000000-0000-0000-0000-000000000001")),
                plan_id: Some(10),
                id: 2000000000,
                type_request_id: Some(PriceInformationRequestType::Public),
                request_subject: Some("Something 1".to_string()),
                start_date: Some(
                    AsezDate::try_from("2030-02-12").unwrap().to_timestamp(),
                ),
                end_date: Some(
                    AsezDate::try_from("2030-02-12").unwrap().to_timestamp(),
                ),
                status_id: 10.into(),
                created_by: 999,
                changed_by: 998,
                created_at: AsezDate::try_from("1999-01-01")
                    .unwrap()
                    .to_timestamp(),
                changed_at: AsezDate::try_from("2024-01-01")
                    .unwrap()
                    .to_timestamp(),
                ..Default::default()
            },
            supplier_list: vec![345],
            tkp_in: vec![345],
            tkp_done: [0, 1],
            hierarchy_uuid_list: vec![uuid!(
                "00000000-0000-0000-0001-000000000001"
            )],
        },
        ExpectedItem {
            header: RequestHeader {
                uuid: uuid!("00000000-0000-0000-0000-000000000002"),
                plan_uuid: Some(uuid!("90000000-0000-0000-0000-000000000002")),
                plan_id: Some(11),
                id: 2000000001,
                type_request_id: Some(PriceInformationRequestType::Public),
                request_subject: Some("Something 2".to_string()),
                start_date: Some(
                    AsezDate::try_from("2030-02-12").unwrap().to_timestamp(),
                ),
                end_date: Some(
                    AsezDate::try_from("2030-02-12").unwrap().to_timestamp(),
                ),
                status_id: 80.into(),
                created_by: 999,
                changed_by: 998,
                created_at: AsezDate::try_from("1999-01-01")
                    .unwrap()
                    .to_timestamp(),
                changed_at: AsezDate::try_from("2024-01-01")
                    .unwrap()
                    .to_timestamp(),
                ..Default::default()
            },
            supplier_list: vec![456],
            tkp_in: vec![456],
            tkp_done: [1, 1],
            hierarchy_uuid_list: vec![uuid!(
                "00000000-0000-0000-0001-000000000002"
            )],
        },
        ExpectedItem {
            header: RequestHeader {
                uuid: uuid!("00000000-0000-0000-0000-000000000003"),
                plan_uuid: Some(uuid!("90000000-0000-0000-0000-000000000003")),
                plan_id: Some(12),
                id: 2000000002,
                type_request_id: Some(PriceInformationRequestType::Public),
                request_subject: Some("Something 3".to_string()),
                start_date: Some(
                    AsezDate::try_from("2030-02-12").unwrap().to_timestamp(),
                ),
                end_date: Some(
                    AsezDate::try_from("2030-02-12").unwrap().to_timestamp(),
                ),
                status_id: 20.into(),
                created_by: 999,
                changed_by: 998,
                created_at: AsezDate::try_from("1999-01-01")
                    .unwrap()
                    .to_timestamp(),
                changed_at: AsezDate::try_from("2024-01-01")
                    .unwrap()
                    .to_timestamp(),
                ..Default::default()
            },
            supplier_list: vec![567],
            tkp_in: vec![567],
            tkp_done: [1, 1],
            hierarchy_uuid_list: vec![uuid!(
                "00000000-0000-0000-0001-000000000003"
            )],
        },
        ExpectedItem {
            header: RequestHeader {
                uuid: uuid!("00000000-0000-0000-0000-000000000004"),
                plan_uuid: Some(uuid!("90000000-0000-0000-0000-000000000004")),
                plan_id: Some(13),
                id: 2000000003,
                type_request_id: Some(PriceInformationRequestType::Public),
                request_subject: None,
                start_date: Some(
                    AsezDate::try_from("2030-02-12").unwrap().to_timestamp(),
                ),
                end_date: Some(
                    AsezDate::try_from("2030-02-12").unwrap().to_timestamp(),
                ),
                status_id: 90.into(),
                created_by: 999,
                changed_by: 998,
                created_at: AsezDate::try_from("1999-01-01")
                    .unwrap()
                    .to_timestamp(),
                changed_at: AsezDate::try_from("2024-01-01")
                    .unwrap()
                    .to_timestamp(),
                ..Default::default()
            },
            supplier_list: vec![678],
            tkp_in: vec![678],
            tkp_done: [0, 2],
            hierarchy_uuid_list: vec![
                uuid!("00000000-0000-0000-0001-000000000004"),
                uuid!("00000000-0000-0000-0001-000000000005"),
            ],
        },
    ];

    run_db_test(MIGS, |pool| async move {
        let res = database_request(selection, &pool).await.unwrap();

        assert_eq!(4, res.len());

        verify_price_info(res, expected);
    })
    .await
}

#[tokio::test]
async fn basic_orderings() {
    let selection = Select::with_fields(RequestHeader::FIELDS)
        .add_replace_order_asc(RequestHeader::request_subject);

    run_db_test(MIGS, |pool| async move {
        let res = database_request(selection, &pool).await.unwrap();

        assert_eq!(res.len(), 4);

        [13, 10, 11, 12].into_iter().zip(res).enumerate().for_each(
            |(idx, (plan_id, item))| {
                assert_eq!(
                    item.entity.plan_id.unwrap().unwrap(),
                    plan_id,
                    "plan_id не соответствует в {} элементе",
                    idx
                );
            },
        )
    })
    .await
}

/// Случай, когда нам надо получить только те записи, которые имеют поле с массивом
/// значений, в котором хотя бы одно значение есть в фильтре
#[tokio::test]
async fn calculated_filters_in_any() {
    let selection = Select::with_fields(RequestHeader::FIELDS)
        .in_any(SUPPLIER_LIST, [345, 456])
        .add_replace_order_asc("plan_id");

    run_db_test(MIGS, |pool| async move {
        let res = database_request(selection, &pool).await.unwrap();

        assert_eq!(2, res.len());

        [10, 11].into_iter().zip(res).enumerate().for_each(
            |(idx, (plan_id, item))| {
                assert_eq!(
                    item.entity.plan_id.unwrap().unwrap(),
                    plan_id,
                    "Не тот plan_id для {} элемента",
                    idx,
                );
            },
        );
    })
    .await
}

/// Случай, когда нам надо получить только те записи, которые имеют поле с массивом
/// значений, которое польностью соответствует переданному массиву
#[tokio::test]
async fn calculated_filters_eq() {
    let selection = Select::with_fields(RequestHeader::FIELDS)
        .in_any(TKP_DONE, [1, 1])
        .add_replace_order_asc("plan_id");

    run_db_test(MIGS, |pool| async move {
        let res = database_request(selection, &pool).await.unwrap();

        assert_eq!(2, res.len());

        [11, 12].into_iter().zip(res).enumerate().for_each(
            |(idx, (plan_id, item))| {
                assert_eq!(
                    item.entity.plan_id.unwrap().unwrap(),
                    plan_id,
                    "Не тот plan_id для {} элемента",
                    idx,
                );
            },
        );
    })
    .await
}

fn verify_price_info(
    res: Vec<FeWrapper<RequestHeaderRep>>,
    expected: Vec<ExpectedItem>,
) {
    res.into_iter().zip(expected).for_each(|(res, expected)| {
        let expected = FeWrapper::new(RequestHeaderRep::from_item::<&str>(
            expected.header,
            None,
        ))
        .add_field(SUPPLIER_LIST, expected.supplier_list)
        .add_field(TKP_IN, expected.tkp_in)
        .add_field(TKP_DONE, expected.tkp_done.to_vec())
        .add_field(HIERARCHY_UUID_LIST, expected.hierarchy_uuid_list);

        assert_eq!(res, expected);
    })
}
