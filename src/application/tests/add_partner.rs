use asez2_shared_db::uuid;

use shared_essential::presentation::dto::response_request::{
    Message, MessageKind, Status,
};

use crate::application::calls::add_partner::process_check_add_partner;
use crate::presentation::dto::{CheckPartnerItem, CheckPartnerReq, SupplierId};

use super::run_db_test;

const MIGS: &[&str] = &["add_partner.sql"];

#[tokio::test]
async fn test_check_add_partners_ok1() {
    let item_list = [345, 456]
        .into_iter()
        .map(|supplier_id| SupplierId { supplier_id })
        .collect::<Vec<_>>();
    let request = CheckPartnerReq {
        uuid: uuid!("99999999-7777-6666-5555-100000000001"),
        id: 999,
        item_list,
    };
    let exp = vec![
        CheckPartnerItem {
            supplier_id: 345,
            is_allowed: true,
        },
        CheckPartnerItem {
            supplier_id: 456,
            is_allowed: true,
        },
    ];
    run_db_test(MIGS, |pool| async move {
        let res = process_check_add_partner(0, request, &pool).await.unwrap();
        let data = res.data.item_list;

        assert_eq!(res.status, Status::Ok);
        assert!(res.messages.messages.is_empty());
        assert_eq!(exp, data, "{:#?}, {:#?}", exp, data);
    })
    .await
}

// По партнеру нет ТКП
#[tokio::test]
async fn test_check_add_partners_ok2() {
    let item_list = [345, 456]
        .into_iter()
        .map(|supplier_id| SupplierId { supplier_id })
        .collect::<Vec<_>>();
    let request = CheckPartnerReq {
        uuid: uuid!("00000000-0000-0000-0000-000000000001"),
        id: 999,
        item_list,
    };
    let exp = vec![
        CheckPartnerItem {
            supplier_id: 345,
            is_allowed: true,
        },
        CheckPartnerItem {
            supplier_id: 456,
            is_allowed: true,
        },
    ];
    run_db_test(MIGS, |pool| async move {
        let res = process_check_add_partner(0, request, &pool).await.unwrap();
        let data = res.data.item_list;

        assert_eq!(res.status, Status::Ok);
        assert!(res.messages.messages.is_empty());
        assert_eq!(exp, data, "{:#?}, {:#?}", exp, data);
    })
    .await
}

// По партнеру есть ТКП, но с 25 статусом
#[tokio::test]
async fn test_check_add_partners_ok3() {
    let item_list = [345, 456]
        .into_iter()
        .map(|supplier_id| SupplierId { supplier_id })
        .collect::<Vec<_>>();
    let request = CheckPartnerReq {
        uuid: uuid!("00000000-0000-0000-0000-000000000002"),
        id: 999,
        item_list,
    };
    let exp = vec![
        CheckPartnerItem {
            supplier_id: 345,
            is_allowed: true,
        },
        CheckPartnerItem {
            supplier_id: 456,
            is_allowed: true,
        },
    ];
    run_db_test(MIGS, |pool| async move {
        let res = process_check_add_partner(0, request, &pool).await.unwrap();
        let data = res.data.item_list;

        assert_eq!(res.status, Status::Ok);
        assert!(res.messages.messages.is_empty());
        assert_eq!(exp, data, "{:#?}, {:#?}", exp, data);
    })
    .await
}

// По партнеру есть ТКП
#[tokio::test]
async fn check_add_partners_err() {
    let item_list = [456, 567]
        .into_iter()
        .map(|supplier_id| SupplierId { supplier_id })
        .collect::<Vec<_>>();
    let request = CheckPartnerReq {
        uuid: uuid!("00000000-0000-0000-0000-000000000003"),
        id: 999,
        item_list,
    };
    let exp = vec![
        CheckPartnerItem {
            supplier_id: 456,
            is_allowed: true,
        },
        CheckPartnerItem {
            supplier_id: 567,
            is_allowed: false,
        },
    ];
    let msg = Message::error("Для выбранной организации уже существует ТКП");

    run_db_test(MIGS, |pool| async move {
        let res = process_check_add_partner(0, request, &pool).await.unwrap();
        let data = res.data.item_list;

        assert_eq!(res.status, Status::Ok);
        assert_eq!(res.messages.messages.len(), 1);
        assert_eq!(res.messages.kind, MessageKind::Error);
        assert_eq!(res.messages.messages[0], msg);
        assert_eq!(exp, data, "{:#?}, {:#?}", exp, data);
    })
    .await
}
