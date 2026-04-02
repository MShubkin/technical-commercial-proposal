use super::run_db_test;
use crate::application::calls::delete_partner::process_check_delete_partner;
use crate::presentation::dto::{CheckPartnerReq, SupplierId};
use asez2_shared_db::db_item::Select;
use asez2_shared_db::{uuid, DbItem};
use itertools::Itertools;
use shared_essential::domain::tcp::{
    ProposalHeader, RequestPartner, TcpGeneralStatus,
};
use shared_essential::presentation::dto::response_request::{MessageKind, Status};

const MIGS: &[&str] = &["delete_partner.sql"];

/// удаление организации на стутусах зци 70, 100
#[tokio::test]
async fn test_check_delete_partners_70_100_statuses() {
    run_db_test(MIGS, |pool| async move {
        let check = CheckPartnerReq {
            id: 0,
            uuid: uuid!("00000000-0000-0000-0000-000000000003"),
            item_list: vec![
                SupplierId { supplier_id: 6 },
                SupplierId { supplier_id: 7 },
            ],
        };
        let ids = check.item_list.iter().map(|item| item.supplier_id).collect_vec();
        let ids_str = ids.iter().map(|item| item.to_string()).collect_vec();
        let response = process_check_delete_partner(0, check, &pool).await.unwrap();
        assert_eq!(response.status, Status::Ok);
        assert_eq!(response.messages.messages.len(), 1);
        let message = response.messages.messages.get(0).unwrap();
        assert_eq!(message.kind, MessageKind::Success);
        assert!(message.text.contains("Выбранные записи удалены"));
        assert_eq!(message.parameters.item_list.len(), ids.len());
        message.parameters.item_list.iter().for_each(|item| {
            assert!(ids_str.contains(&item.id));
        });
        let rh_select = Select::full::<RequestPartner>()
            .in_any(RequestPartner::supplier_id, ids.clone());
        let partners = RequestPartner::select(&rh_select, &*pool).await.unwrap();
        assert_eq!(partners.len(), ids.len());
        partners.iter().for_each(|item| {
            assert!(item.is_removed);
        });
    })
    .await
}
/// Удаление организации, добавленной только на фронте
#[tokio::test]
async fn test_check_delete_partners_front() {
    run_db_test(MIGS, |pool| async move {
        let check = CheckPartnerReq {
            id: 0,
            uuid: uuid!("00000000-0000-0000-0000-000000000001"),
            item_list: vec![
                SupplierId { supplier_id: 111 },
                SupplierId { supplier_id: 222 },
            ],
        };
        let ids = check.item_list.iter().map(|item| item.supplier_id).collect_vec();
        let ids_str = ids.iter().map(|item| item.to_string()).collect_vec();
        let response = process_check_delete_partner(0, check, &pool).await.unwrap();
        assert_eq!(response.status, Status::Ok);
        assert_eq!(response.messages.messages.len(), 1);
        let message = response.messages.messages.get(0).unwrap();
        assert_eq!(message.kind, MessageKind::Success);
        assert!(message.text.contains("Выбранные записи удалены"));
        assert_eq!(message.parameters.item_list.len(), ids.len());
        message.parameters.item_list.iter().for_each(|item| {
            assert!(ids_str.contains(&item.id));
        });
    })
    .await
}

/// Удаление организации из ЗЦИ, не находящимся в статусах 90, 150, 70, 100
#[tokio::test]
async fn test_check_delete_partners_request_not_found() {
    run_db_test(MIGS, |pool| async move {
        let check = CheckPartnerReq {
            id: 0,
            uuid: uuid!("00000000-0000-0000-0000-000000000004"), // not in statuses: 90, 150, 70, 100.
            item_list: vec![
                SupplierId { supplier_id: 1 },
                SupplierId { supplier_id: 2 },
            ],
        };
        let ids = check.item_list.iter().map(|item| item.supplier_id).collect_vec();
        let response = process_check_delete_partner(0, check, &pool).await.unwrap();
        assert_eq!(response.status, Status::Ok);
        assert_eq!(response.messages.messages.len(), 1);
        let response_data = response.data.item_list;
        assert_eq!(response_data.len(), ids.len());
        response_data.iter().for_each(|item| {
            assert!(!item.is_allowed);
            assert!(ids.contains(&item.supplier_id));
        });
    })
    .await
}
/// Информация опубликована на ЭТП ГПБ. Удаление невозможно
#[tokio::test]
async fn test_check_delete_partners_error_etp() {
    run_db_test(MIGS, |pool| async move {
        let check = CheckPartnerReq {
            id: 0,
            uuid: uuid!("00000000-0000-0000-0000-000000000001"),
            item_list: vec![
                SupplierId { supplier_id: 1 },
                SupplierId { supplier_id: 2 },
            ],
        };
        let ids = check.item_list.iter().map(|item| item.supplier_id).collect_vec();
        let ids_str = ids.iter().map(|item| item.to_string()).collect_vec();

        let response = process_check_delete_partner(0, check, &pool).await.unwrap();
        assert_eq!(response.status, Status::Ok);
        assert_eq!(response.messages.messages.len(), 1);
        let message = response.messages.messages.get(0).unwrap();
        assert_eq!(message.kind, MessageKind::Error);
        assert!(message
            .text
            .contains("Информация опубликована на ЭТП ГПБ. Удаление невозможно"));
        assert_eq!(message.parameters.item_list.len(), ids.len());

        message.parameters.item_list.iter().for_each(|item| {
            assert!(ids_str.contains(&item.id));
        });
        let response_data = response.data.item_list;
        assert_eq!(response_data.len(), ids.len());
        response_data.iter().for_each(|item| {
            assert!(!item.is_allowed);
            assert!(ids.contains(&item.supplier_id));
        });
    })
    .await
}
/// Удаление организации невозможно. Имеется подтвержденное ТКП
#[tokio::test]
async fn test_check_delete_partners_error_tcp_exist() {
    run_db_test(MIGS, |pool| async move {
        let check = CheckPartnerReq {
            id: 0,
            uuid: uuid!("00000000-0000-0000-0000-000000000001"),
            item_list: vec![
                SupplierId { supplier_id: 3 },
                SupplierId { supplier_id: 4 },
            ],
        };

        let response = process_check_delete_partner(0, check, &pool).await.unwrap();
        assert_eq!(response.status, Status::Ok);
        assert_eq!(response.messages.messages.len(), 1);
        let message = response.messages.messages.get(0).unwrap();
        assert_eq!(message.kind, MessageKind::Error);
        assert!(message.text.contains(
            "Удаление организации невозможно. Имеется подтвержденное ТКП"
        ));
        assert_eq!(message.parameters.item_list.len(), 1);
        assert_eq!(message.parameters.item_list.get(0).unwrap().id, 3.to_string());

        let response_data = response.data.item_list;
        assert_eq!(response_data.len(), 1);
        assert_eq!(response_data.get(0).unwrap().supplier_id, 3);
        assert!(!response_data.get(0).unwrap().is_allowed);
    })
    .await
}

/// Множественные сообщения
#[tokio::test]
async fn test_check_delete_partners_error_multi() {
    run_db_test(MIGS, |pool| async move {
        let check = CheckPartnerReq {
            id: 0,
            uuid: uuid!("00000000-0000-0000-0000-000000000001"),
            item_list: vec![
                SupplierId { supplier_id: 1 },
                SupplierId { supplier_id: 2 },
                SupplierId { supplier_id: 3 },
                SupplierId { supplier_id: 4 },
                SupplierId { supplier_id: 555 },
            ],
        };
        let response = process_check_delete_partner(0, check, &pool).await.unwrap();
        assert_eq!(response.status, Status::Ok);
        assert_eq!(response.messages.messages.len(), 3);

        let message = response.messages.messages.get(0).unwrap();
        assert_eq!(message.kind, MessageKind::Success);
        assert!(message.text.contains("Выбранные записи удалены"));
        assert_eq!(message.parameters.item_list.len(), 1);
        assert_eq!(
            message.parameters.item_list.get(0).unwrap().id,
            "555".to_string()
        );
        let message = response.messages.messages.get(1).unwrap();
        assert_eq!(message.kind, MessageKind::Error);
        assert!(message
            .text
            .contains("Информация опубликована на ЭТП ГПБ. Удаление невозможно"));
        assert_eq!(message.parameters.item_list.len(), 2);
        let message = response.messages.messages.get(2).unwrap();
        assert_eq!(message.kind, MessageKind::Error);
        assert!(message.text.contains(
            "Удаление организации невозможно. Имеется подтвержденное ТКП"
        ));
        assert_eq!(message.parameters.item_list.len(), 1);
    })
    .await
}
/// Усмешное удаление на статусах 90, 150
#[tokio::test]
async fn test_check_delete_partners_success() {
    run_db_test(MIGS, |pool| async move {
        let check = CheckPartnerReq {
            id: 0,
            uuid: uuid!("00000000-0000-0000-0000-000000000001"),
            item_list: vec![
                SupplierId { supplier_id: 4 },
                SupplierId { supplier_id: 5 },
            ],
        };

        let ids = check.item_list.iter().map(|item| item.supplier_id).collect_vec();
        let ids_str = ids.iter().map(|item| item.to_string()).collect_vec();

        let response =
            process_check_delete_partner(0, check.clone(), &pool).await.unwrap();
        assert_eq!(response.status, Status::Ok);
        assert_eq!(response.messages.messages.len(), 1);

        assert_eq!(response.data.item_list.len(), 2);

        response.data.item_list.iter().for_each(|item| {
            assert!(item.is_allowed);
            assert!(ids.contains(&item.supplier_id));
        });

        let message = response.messages.messages.get(0).unwrap();
        assert_eq!(message.kind, MessageKind::Success);
        assert!(message.text.contains("Выбранные записи удалены"));
        assert_eq!(message.parameters.item_list.len(), ids.len());
        message.parameters.item_list.iter().for_each(|item| {
            assert!(ids_str.contains(&item.id));
        });

        let rh_select = Select::full::<RequestPartner>()
            .in_any(RequestPartner::supplier_id, ids.clone());
        let partners = RequestPartner::select(&rh_select, &*pool).await.unwrap();
        assert_eq!(partners.len(), ids.len());
        partners.iter().for_each(|item| {
            assert!(item.is_removed);
        });

        let proposal_select = Select::full::<ProposalHeader>()
            .eq(ProposalHeader::supplier_uuid, partners.get(0).unwrap().uuid);
        let proposals =
            ProposalHeader::select(&proposal_select, &*pool).await.unwrap();
        proposals.iter().for_each(|item| {
            assert_eq!(item.status_id, TcpGeneralStatus::Deleted);
        });
    })
    .await
}
