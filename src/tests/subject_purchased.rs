use asez2_shared_db::{db_item::Select, uuid};
use shared_essential::presentation::dto::{
    general::UserId,
    response_request::{Message, MessageKind, Messages},
};

use crate::{
    application::calls::{
        get_organizations, get_purchasing_subject_by_group_uuid,
        get_purchasing_subject_group, organizations_remove, organizations_update,
        purchasing_subject_group_remove, purchasing_subject_group_update,
        purchasing_subject_remove, purchasing_subject_update,
    },
    presentation::dto::{
        ActionOrganizationsRequest, ActionPurchasingSubjectGroupRequest,
        ActionPurchasingSubjectRequest, ActionSubjectItem,
        UpdatePurchasingSubjectGroupReq, UpdatePurchasingSubjectReq,
    },
};

use super::*;

const SUBJECT_PURCHASED_MIGS: &[&str] = &["subject_purchased.sql"];

/// Поиск предметов закупок ЗЦИ по parent_uuid
#[tokio::test]
async fn test_valid_get_request_subjects_by_group_uuid() {
    run_db_test(SUBJECT_PURCHASED_MIGS, |pool| async move {
        let res = get_purchasing_subject_by_group_uuid(
            &pool,
            uuid!("00000000-0000-0000-0001-000000000001"),
        )
        .await
        .unwrap()
        .data;

        assert_eq!(3, res.item_list.len());
        assert!(res.item_list.iter().all(|item| item.uuid
            == Some(uuid!("00000000-0000-0000-0001-000000000002"))
            || item.uuid == Some(uuid!("00000000-0000-0000-0001-000000000003"))
            || item.uuid == Some(uuid!("00000000-0000-0000-0001-000000000004"))));
    })
    .await;
}

/// Поиск предметов закупок ЗЦИ, среди которых имеются удалённые элементы (is_removed=true)
#[tokio::test]
async fn test_get_some_removed_request_subjects_by_group_uuid() {
    run_db_test(SUBJECT_PURCHASED_MIGS, |pool| async move {
        let res = get_purchasing_subject_by_group_uuid(
            &pool,
            uuid!("00000000-0000-0000-0000-000000000001"),
        )
        .await
        .unwrap()
        .data;

        assert_eq!(2, res.item_list.len());
        assert!(res.item_list.iter().all(|item| item.uuid
            == Some(uuid!("00000000-0000-0000-0000-000000000002"))
            || item.uuid == Some(uuid!("00000000-0000-0000-0000-000000000003"))));
    })
    .await;
}

/// Успешное получение списка организаций по идентификатору "Предмета закупки"
#[tokio::test]
async fn test_valid_get_organizations_by_uuid_subject() {
    run_db_test(SUBJECT_PURCHASED_MIGS, |pool| async move {
        let res =
            get_organizations(&pool, uuid!("00000000-0000-0000-0000-000000000001"))
                .await
                .unwrap()
                .data;

        assert_eq!(1, res.item_list.len());
        assert!(res
            .item_list
            .iter()
            .all(|item| item.uuid
                == Some(uuid!("00000000-0000-0000-0000-000000000001"))));
    })
    .await;
}

/// Получение списка организаций по идентификатору "Предмета закупки", но организация удалена
/// (is_removed=true)
#[tokio::test]
async fn test_get_removed_organizations_by_uuid_subject() {
    run_db_test(SUBJECT_PURCHASED_MIGS, |pool| async move {
        let res =
            get_organizations(&pool, uuid!("00000000-0000-0000-0000-000000000002"))
                .await
                .unwrap()
                .data;

        assert!(res.item_list.is_empty());
    })
    .await;
}

#[tokio::test]
async fn test_valid_get_purchasing_subject_group() {
    run_db_test(SUBJECT_PURCHASED_MIGS, |pool| async move {
        let res = get_purchasing_subject_group(&pool, 666).await.unwrap().data;

        assert_eq!(2, res.item_list.len());
    })
    .await;
}

#[tokio::test]
async fn test_organizations_remove_success() {
    run_db_test(SUBJECT_PURCHASED_MIGS, |pool| async move {
        let removed_uuid = uuid!("00000000-0000-0000-0000-000000000001");

        let res = organizations_remove(
            &pool,
            ActionOrganizationsRequest {
                item: ActionSubjectItem {
                    id: 0,
                    uuid: removed_uuid,
                },
            },
        )
        .await
        .unwrap();
        assert_eq!(MessageKind::Success, res.messages.kind);

        let removed_list = PartnerSubjectPurchased::select(
            &Select::full::<PartnerSubjectPurchased>()
                .eq(PartnerSubjectPurchased::uuid, removed_uuid)
                .eq(PartnerSubjectPurchased::is_removed, true),
            &*pool,
        )
        .await
        .unwrap();

        assert_eq!(1, removed_list.len());
    })
    .await;
}

/// Попытка удалить организацию, который нет в БД, ведёт к Message::Error
#[tokio::test]
async fn test_organizations_remove_not_found_rec() {
    run_db_test(SUBJECT_PURCHASED_MIGS, |pool| async move {
        let res = organizations_remove(
            &pool,
            ActionOrganizationsRequest {
                item: ActionSubjectItem {
                    id: 0,
                    uuid: uuid!("00000000-0000-0000-0000-000000000000"),
                },
            },
        )
        .await
        .unwrap();
        assert_eq!(MessageKind::Error, res.messages.kind);
    })
    .await;
}

// Успешное удаление группы
#[tokio::test]
async fn test_purchasing_subject_group_remove_success() {
    run_db_test(SUBJECT_PURCHASED_MIGS, move |pool| async move {
        let res = purchasing_subject_group_remove(
            &pool,
            ActionPurchasingSubjectGroupRequest {
                item: ActionSubjectItem {
                    id: 0,
                    uuid: uuid!("00000000-0000-0000-0001-000000000001"),
                },
            },
        )
        .await
        .unwrap();
        assert_eq!(MessageKind::Success, res.messages.kind);

        let requests = RequestSubjectPurchased::select(
            &Select::full::<RequestSubjectPurchased>().in_any(
                RequestSubjectPurchased::uuid,
                [
                    uuid!("00000000-0000-0000-0001-000000000001"),
                    uuid!("00000000-0000-0000-0001-000000000002"),
                    uuid!("00000000-0000-0000-0001-000000000003"),
                    uuid!("00000000-0000-0000-0001-000000000004"),
                ],
            ),
            &*pool,
        )
        .await
        .unwrap();
        let partners = PartnerSubjectPurchased::select(
            &Select::full::<PartnerSubjectPurchased>().in_any(
                PartnerSubjectPurchased::uuid_subject,
                requests.iter().map(|x| x.uuid),
            ),
            &*pool,
        )
        .await
        .unwrap();

        assert!(requests.iter().all(|x| x.is_removed));
        assert!(partners.iter().all(|x| x.is_removed));
    })
    .await;
}

// Удаление предмета, а не группы
#[tokio::test]
async fn test_purchasing_subject_group_remove_subject() {
    run_db_test(SUBJECT_PURCHASED_MIGS, |pool| async move {
        let res = purchasing_subject_group_remove(
            &pool,
            ActionPurchasingSubjectGroupRequest {
                item: ActionSubjectItem {
                    id: 0,
                    uuid: uuid!("00000000-0000-0000-0001-000000000002"),
                },
            },
        )
        .await
        .unwrap();
        assert_eq!(MessageKind::Error, res.messages.kind);
    })
    .await;
}

// Удаление несуществуюшей группы
#[tokio::test]
async fn test_purchasing_subject_group_remove_non_existent() {
    run_db_test(SUBJECT_PURCHASED_MIGS, |pool| async move {
        let res = purchasing_subject_group_remove(
            &pool,
            ActionPurchasingSubjectGroupRequest {
                item: ActionSubjectItem {
                    id: 0,
                    uuid: uuid!("00000000-0000-0000-0000-000000000000"),
                },
            },
        )
        .await
        .unwrap();
        assert_eq!(MessageKind::Error, res.messages.kind);
    })
    .await;
}

// Успешное удаление Предмета закупки
#[tokio::test]
async fn test_purchasing_subject_successful_remove() {
    run_db_test(SUBJECT_PURCHASED_MIGS, |pool| async move {
        let res = purchasing_subject_remove(
            &pool,
            ActionPurchasingSubjectRequest {
                item: ActionSubjectItem {
                    id: 0,
                    uuid: uuid!("00000000-0000-0000-0000-000000000002"),
                },
            },
        )
        .await
        .unwrap();
        assert_eq!(MessageKind::Success, res.messages.kind);

        let request_subject = RequestSubjectPurchased::select_single(
            &Select::full::<RequestSubjectPurchased>().eq(
                RequestSubjectPurchased::uuid,
                uuid!("00000000-0000-0000-0000-000000000002"),
            ),
            &*pool,
        )
        .await
        .unwrap();
        let partner_subject = PartnerSubjectPurchased::select_single(
            &Select::full::<PartnerSubjectPurchased>()
                .eq(PartnerSubjectPurchased::uuid_subject, request_subject.uuid),
            &*pool,
        )
        .await
        .unwrap();

        assert!(request_subject.is_removed);
        assert!(partner_subject.is_removed);
    })
    .await;
}

// Удаление группы Предметов вместо Предмета закупки
#[tokio::test]
async fn test_purchasing_subject_remove_group_instead() {
    run_db_test(SUBJECT_PURCHASED_MIGS, |pool| async move {
        let res = purchasing_subject_remove(
            &pool,
            ActionPurchasingSubjectRequest {
                item: ActionSubjectItem {
                    id: 0,
                    uuid: uuid!("00000000-0000-0000-0000-000000000001"),
                },
            },
        )
        .await
        .unwrap();

        assert_eq!(
            Messages::from(Message::error(
                "Выбрана группа Предметов закупки вместо Предмета закупки"
            )),
            res.messages
        );
    })
    .await;
}

/// Создание записи Шаблонов заключений Экспертов АЦ
#[testing::test]
async fn test_organizations_create() {
    run_db_test(SUBJECT_PURCHASED_MIGS, |pool| async move {
        let update_uuid: Uuid = uuid!("90000000-0000-0000-0000-000000000001");

        // Before update there's nothing
        let request_subject = PartnerSubjectPurchased::select_option(
            &Select::full::<PartnerSubjectPurchased>()
                .eq(PartnerSubjectPurchased::uuid_subject, update_uuid),
            &*pool,
        )
        .await
        .unwrap();
        assert!(request_subject.is_none());

        let res = organizations_update(
            &pool,
            UserId { user_id: 999 },
            PartnerSubjectPurchasedRep {
                uuid_subject: Some(update_uuid),
                supplier_id: Some(25),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        // No warns
        assert!(!res.messages.is_error());

        // After update
        let request_subject = PartnerSubjectPurchased::select_single(
            &Select::full::<PartnerSubjectPurchased>()
                .eq(PartnerSubjectPurchased::uuid_subject, update_uuid),
            &*pool,
        )
        .await
        .unwrap();
        assert_eq!(request_subject.supplier_id, 25);
        assert_eq!(request_subject.created_by, 999);
        assert_eq!(request_subject.changed_by, 999);
    })
    .await;
}

/// Создание группы Предметов закупки
#[testing::test]
async fn test_purchasing_subject_group_create() {
    run_db_test(SUBJECT_PURCHASED_MIGS, |pool| async move {
        let res = purchasing_subject_group_update(
            &pool,
            UserId { user_id: 999 },
            UpdatePurchasingSubjectGroupReq {
                uuid: None,
                text: "test".to_string(),
            },
        )
        .await
        .unwrap();

        // No warns
        assert!(!res.messages.is_error());

        // After creation
        let request_subject = RequestSubjectPurchased::select_single(
            &Select::full::<RequestSubjectPurchased>()
                .eq(RequestSubjectPurchased::created_by, 999),
            &*pool,
        )
        .await
        .unwrap();

        assert_eq!(request_subject.contract_subject_purchase_text, "test");
    })
    .await;
}

/// Обновление группы Предметов закупки
#[testing::test]
async fn test_purchasing_subject_group_update() {
    run_db_test(SUBJECT_PURCHASED_MIGS, |pool| async move {
        let uuid = uuid!("00000000-0000-0000-0002-000000000001");

        // Before update
        let request_subject = RequestSubjectPurchased::select_single(
            &Select::full::<RequestSubjectPurchased>()
                .eq(RequestSubjectPurchased::uuid, uuid),
            &*pool,
        )
        .await
        .unwrap();

        assert!(request_subject.is_removed);
        assert_eq!(request_subject.created_by, 666);
        assert_eq!(request_subject.changed_by, 666);

        let res = purchasing_subject_group_update(
            &pool,
            UserId { user_id: 999 },
            UpdatePurchasingSubjectGroupReq {
                uuid: Some(uuid),
                text: "test text".to_string(),
            },
        )
        .await
        .unwrap();

        // No warns
        assert!(!res.messages.is_error());

        // After update
        let request_subject = RequestSubjectPurchased::select_single(
            &Select::full::<RequestSubjectPurchased>()
                .eq(RequestSubjectPurchased::uuid, uuid),
            &*pool,
        )
        .await
        .unwrap();

        assert!(!request_subject.is_removed);
        assert_eq!(request_subject.contract_subject_purchase_text, "test text");
        assert_eq!(request_subject.hierarchy_id, 1);
        assert_eq!(request_subject.created_by, 666);
        assert_eq!(request_subject.changed_by, 999);
    })
    .await;
}

/// Создание Предметов закупки
#[testing::test]
async fn test_purchasing_subject_create() {
    run_db_test(SUBJECT_PURCHASED_MIGS, |pool| async move {
        let parent_uuid = uuid!("00000000-0000-1000-1000-000000000001");
        let res = purchasing_subject_update(
            &pool,
            UserId { user_id: 999 },
            UpdatePurchasingSubjectReq {
                uuid: None,
                parent_uuid,
                text: "test".to_string(),
            },
        )
        .await
        .unwrap();

        // No warns
        assert!(!res.messages.is_error());

        // After creation
        let request_subject = RequestSubjectPurchased::select_single(
            &Select::full::<RequestSubjectPurchased>()
                .eq(RequestSubjectPurchased::created_by, 999),
            &*pool,
        )
        .await
        .unwrap();

        assert_eq!(request_subject.hierarchy_uuid, parent_uuid);
        assert_eq!(request_subject.contract_subject_purchase_text, "test");
    })
    .await;
}

/// Обновление Предметов закупки
#[testing::test]
async fn test_purchasing_subject_update() {
    run_db_test(SUBJECT_PURCHASED_MIGS, |pool| async move {
        let uuid = uuid!("00000000-0000-0000-0000-000000000004");
        let parent_uuid = uuid!("00000000-0000-1000-1000-000000000001");

        // Before update
        let request_subject = RequestSubjectPurchased::select_single(
            &Select::full::<RequestSubjectPurchased>()
                .eq(RequestSubjectPurchased::uuid, uuid),
            &*pool,
        )
        .await
        .unwrap();

        assert_eq!(request_subject.created_by, 666);
        assert_eq!(request_subject.changed_by, 666);

        let res = purchasing_subject_update(
            &pool,
            UserId { user_id: 999 },
            UpdatePurchasingSubjectReq {
                uuid: Some(uuid),
                parent_uuid,
                text: "new text".to_string(),
            },
        )
        .await
        .unwrap();

        // No warns
        assert!(!res.messages.is_error());

        // After update
        let request_subject = RequestSubjectPurchased::select_single(
            &Select::full::<RequestSubjectPurchased>()
                .eq(RequestSubjectPurchased::uuid, uuid),
            &*pool,
        )
        .await
        .unwrap();

        assert_eq!(request_subject.contract_subject_purchase_text, "new text");
        assert_eq!(request_subject.hierarchy_id, 2);
        assert_eq!(request_subject.created_by, 666);
        assert_eq!(request_subject.changed_by, 999);
    })
    .await;
}
