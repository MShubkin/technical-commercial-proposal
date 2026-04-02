use asez2_shared_db::DbItem;
use shared_essential::domain::tables::tcp::*;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

mod subject_purchased;

pub(crate) const TCP_TRANSIENT_TABLES: &[&str] = &[
    OrganizationQuestion::TABLE,
    ProposalHeader::TABLE,
    ProposalItem::TABLE,
    RequestHeader::TABLE,
    RequestItem::TABLE,
    RequestPartner::TABLE,
    StatusHistory::TABLE,
    PartnerSubjectPurchased::TABLE,
    RequestSubjectPurchased::TABLE,
];

pub(crate) async fn run_db_test<F, FutFn>(
    extra_migrations: &'static [&'static str],
    run: FutFn,
) where
    F: futures::Future<Output = ()>,
    FutFn: FnOnce(Arc<PgPool>) -> F + 'static,
{
    testing::BaseMigPath::New
        .run_test_with_migrations(
            "src/tests/extra_migrations/", // Extra migs dir
            extra_migrations,              // Extra migs
            TCP_TRANSIENT_TABLES,
            run,
        )
        .await
}

#[tokio::test]
async fn test_status_history_cruds() {
    run_db_test(&[], |pool| async move {
        let uuid = Uuid::new_v4();
        let entity_item = StatusHistory {
            uuid,
            object_uuid: Default::default(),
            tcp_status_type_id: Some(1),
            status_id: 2,
            created_by: 0,
            created_at: Default::default(),
        };

        let mut transaction = pool.begin().await.unwrap();
        let created_item = entity_item.insert_ret(&mut transaction).await.unwrap();
        transaction.commit().await.unwrap();
        assert_eq!(uuid, created_item.uuid);

        let mut transaction = pool.begin().await.unwrap();
        let _del =
            StatusHistory::delete_by_uuids(&[created_item.uuid], &mut transaction)
                .await
                .unwrap();
        transaction.commit().await.unwrap();

        let deleted_item =
            StatusHistory::get_by_uuid(created_item.uuid, &pool).await;

        assert!(deleted_item.is_err());
    })
    .await
}

#[tokio::test]
async fn test_request_supplier_cruds() {
    run_db_test(&[], |pool| async move {
        let uuid = Uuid::new_v4();
        let entity_item = RequestPartner {
            uuid,
            request_uuid: Uuid::new_v4(),
            supplier_id: 2,
            number: 0,
            is_public: false,
            is_phone_check: false,
            is_email_check: false,
            resume: Some("resume".to_string()),
            comment: Some("comment".to_string()),
            is_removed: false,
        };

        let mut transaction = pool.begin().await.unwrap();
        let mut created_item =
            entity_item.insert_ret(&mut transaction).await.unwrap();
        transaction.commit().await.unwrap();
        assert_eq!(uuid, created_item.uuid);

        let created_uuid = created_item.uuid;

        created_item.is_public = true;
        created_item.is_phone_check = true;
        created_item.is_email_check = true;

        let mut transaction = pool.begin().await.unwrap();
        let updated_item = created_item.update_ret(&mut transaction).await.unwrap();
        transaction.commit().await.unwrap();

        assert!(updated_item.is_public);
        assert!(updated_item.is_email_check);
        assert!(updated_item.is_phone_check);

        let mut transaction = pool.begin().await.unwrap();
        let _del =
            RequestPartner::delete_by_uuids(&[created_uuid], &mut transaction)
                .await;
        transaction.commit().await.unwrap();

        let deleted_item = RequestPartner::get_by_uuid(created_uuid, &pool).await;

        assert!(deleted_item.is_err());
    })
    .await
}

#[tokio::test]
async fn test_request_item_multi_cruds() {
    run_db_test(&[], |pool| async move {
        let uuid = Uuid::new_v4();
        let entity_item = RequestItem {
            uuid,
            request_uuid: Uuid::new_v4(),
            plan_item_uuid: Uuid::new_v4(),
            number: 1,
            ..Default::default()
        };
        let mut transaction = pool.begin().await.unwrap();
        entity_item.insert_ret(&mut transaction).await.unwrap();
        transaction.commit().await.unwrap();

        let mut transaction = pool.begin().await.unwrap();
        let count =
            RequestItem::delete_by_uuids(&[uuid], &mut transaction).await.unwrap();
        transaction.commit().await.unwrap();
        assert_eq!(count, 1);
    })
    .await
}

#[tokio::test]
async fn test_request_item_cruds() {
    run_db_test(&[], |pool| async move {
        let uuid = Uuid::new_v4();
        let entity_item = RequestItem {
            uuid,
            request_uuid: Uuid::new_v4(),
            plan_item_uuid: Uuid::new_v4(),
            number: 1,
            ..Default::default()
        };

        let mut transaction = pool.begin().await.unwrap();
        let created_item = entity_item.insert_ret(&mut transaction).await.unwrap();
        transaction.commit().await.unwrap();
        assert_eq!(uuid, created_item.uuid);

        let created_uuid = created_item.uuid;

        let mut transaction = pool.begin().await.unwrap();
        let _updated_price_information_header =
            created_item.update_ret(&mut transaction).await.unwrap();
        transaction.commit().await.unwrap();

        let mut transaction = pool.begin().await.unwrap();
        let _del =
            RequestItem::delete_by_uuids(&[created_uuid], &mut transaction).await;
        transaction.commit().await.unwrap();

        let deleted_item = RequestItem::get_by_uuid(created_uuid, &pool).await;

        assert!(deleted_item.is_err());
    })
    .await
}

#[tokio::test]
async fn test_request_header_multi_cruds() {
    run_db_test(&[], |pool| async move {
        let uuid = Uuid::new_v4();
        let entity_header = RequestHeader {
            uuid,
            plan_uuid: Some(Uuid::new_v4()),
            plan_id: Some(1),
            type_request_id: Some(PriceInformationRequestType::Public),
            status_id: PriceInformationRequestStatus::Created,
            ..Default::default()
        };
        let mut transaction = pool.begin().await.unwrap();
        entity_header.insert_ret(&mut transaction).await.unwrap();
        transaction.commit().await.unwrap();

        let mut transaction = pool.begin().await.unwrap();
        let count = RequestHeader::delete_by_uuids(&[uuid], &mut transaction)
            .await
            .unwrap();
        transaction.commit().await.unwrap();
        assert_eq!(count, 1);
    })
    .await
}

#[tokio::test]
async fn test_request_header_cruds() {
    run_db_test(&[], |pool| async move {
    let uuid = Uuid::new_v4();
    let entity_header = RequestHeader {
        uuid,
        plan_uuid: Some(Uuid::new_v4()),
        plan_id: Some(1),
        type_request_id: Some(PriceInformationRequestType::Public),
        status_id: PriceInformationRequestStatus::Created,
        ..Default::default()
    };

    let mut transaction = pool.begin().await.unwrap();

    let mut created_price_information_header =
        entity_header.insert_ret(&mut transaction).await.unwrap();
    transaction.commit().await.unwrap();

    assert_eq!(uuid, created_price_information_header.uuid);

    let created_uuid = created_price_information_header.uuid;

    created_price_information_header.type_request_id =
        Some(shared_essential::domain::tables::tcp::PriceInformationRequestType::Private);
    created_price_information_header.status_id =
        shared_essential::domain::tables::tcp::PriceInformationRequestStatus::Created;

    let mut transaction = pool.begin().await.unwrap();
    let updated_price_information_header =
        created_price_information_header.update_ret(&mut transaction).await.unwrap();
    transaction.commit().await.unwrap();

    assert_eq!(
        updated_price_information_header.type_request_id,
        Some(PriceInformationRequestType::Private)
    );
    assert_eq!(
        updated_price_information_header.status_id,
        PriceInformationRequestStatus::Created
    );

    let mut transaction = pool.begin().await.unwrap();
    let _del = RequestHeader::delete_by_uuids(&[created_uuid], &mut transaction).await;
    transaction.commit().await.unwrap();

    let deleted_header = RequestHeader::get_by_uuid(created_uuid, &pool).await;

    assert!(deleted_header.is_err());
}).await
}

#[tokio::test]
async fn test_tkp_item_vec_cruds() {
    run_db_test(&[], |pool| async move {
        let uuid = Uuid::new_v4();
        let entity_header = ProposalItem {
            uuid,
            proposal_uuid: Uuid::new_v4(),
            request_item_uuid: Uuid::new_v4(),
            ..Default::default()
        };
        let mut transaction = pool.begin().await.unwrap();
        entity_header.insert_ret(&mut transaction).await.unwrap();
        transaction.commit().await.unwrap();

        let mut transaction = pool.begin().await.unwrap();
        let count =
            ProposalItem::delete_by_uuids(&[uuid], &mut transaction).await.unwrap();
        transaction.commit().await.unwrap();
        assert_eq!(count, 1);
    })
    .await
}

#[tokio::test]
async fn test_tkp_item_single_cruds() {
    run_db_test(&[], |pool| async move {
        let uuid = Uuid::new_v4();

        let entity_header = ProposalItem {
            uuid,
            proposal_uuid: Uuid::new_v4(),
            request_item_uuid: Uuid::new_v4(),
            ..Default::default()
        };

        let mut transaction = pool.begin().await.unwrap();
        let mut created_tkp_header =
            entity_header.insert_ret(&mut transaction).await.unwrap();
        transaction.commit().await.unwrap();
        assert_eq!(uuid, created_tkp_header.uuid);

        let created_uuid = created_tkp_header.uuid;

        created_tkp_header.unit_id = 11;

        let mut transaction = pool.begin().await.unwrap();
        let updated_tkp_header =
            created_tkp_header.update_ret(&mut transaction).await.unwrap();
        transaction.commit().await.unwrap();

        assert_eq!(updated_tkp_header.unit_id, 11);

        let mut transaction = pool.begin().await.unwrap();
        let _del =
            ProposalItem::delete_by_uuids(&[created_uuid], &mut transaction).await;
        transaction.commit().await.unwrap();

        let deleted_header = ProposalItem::get_by_uuid(created_uuid, &pool).await;

        assert!(deleted_header.is_err());
    })
    .await
}

#[tokio::test]
async fn test_tkp_header_vec_cruds() {
    run_db_test(&[], |pool| async move {
        let uuid = Uuid::new_v4();
        let entity_header = ProposalHeader {
            uuid,
            id: 1,
            request_uuid: Uuid::new_v4(),
            ..Default::default()
        };
        let mut transaction = pool.begin().await.unwrap();
        entity_header.insert_ret(&mut transaction).await.unwrap();
        transaction.commit().await.unwrap();

        let mut transaction = pool.begin().await.unwrap();
        let count = ProposalHeader::delete_by_uuids(&[uuid], &mut transaction)
            .await
            .unwrap();
        transaction.commit().await.unwrap();
        assert_eq!(count, 1);
    })
    .await
}

#[tokio::test]
async fn test_tkp_header_single_cruds() {
    run_db_test(&[], |pool| async move {
        let uuid = Uuid::new_v4();

        let entity_header = ProposalHeader {
            uuid,
            id: 1,
            ..Default::default()
        };

        let mut transaction = pool.begin().await.unwrap();
        let mut created_tkp_header =
            entity_header.insert_ret(&mut transaction).await.unwrap();
        transaction.commit().await.unwrap();
        assert_eq!(uuid, created_tkp_header.uuid);

        let created_uuid = created_tkp_header.uuid;

        created_tkp_header.status_id = TcpGeneralStatus::Received;

        let mut transaction = pool.begin().await.unwrap();
        let updated_tkp_header =
            created_tkp_header.update_ret(&mut transaction).await.unwrap();
        transaction.commit().await.unwrap();

        assert_eq!(updated_tkp_header.status_id, TcpGeneralStatus::Received);

        let mut transaction = pool.begin().await.unwrap();
        let _del =
            ProposalHeader::delete_by_uuids(&[created_uuid], &mut transaction)
                .await;
        transaction.commit().await.unwrap();

        let get_deleted_header_result =
            ProposalHeader::get_by_uuid(created_uuid, &pool).await;

        assert!(get_deleted_header_result.is_err());
    })
    .await
}
