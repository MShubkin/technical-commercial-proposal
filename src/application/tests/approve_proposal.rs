use super::*;
use asez2_shared_db::{
    db_item::{AsezTimestamp, Select},
    uuid,
};
use monolith_service::dto::attachment::{
    Attachment, FoldersCategory, GetHierarchyResponseItem,
};
use shared_essential::presentation::dto::{
    general::ObjectIdentifier,
    response_request::{Message, MessageKind, Messages},
    technical_commercial_proposal::TcpError,
};

use asez2_shared_db::db_item::DbItem;
use testing::monolith::MockMonolithService;

use crate::application::calls::approve_proposal::{
    process_approve_proposal, ApproveProposalMessage,
};
use crate::presentation::dto::ApproveProposalReq;

const MIGS: &[&str] = &["approve_proposal.sql"];
const USER_ID: i32 = 123;

#[tokio::test]
async fn success() {
    let req = ApproveProposalReq {
        item_list: vec![
            ObjectIdentifier::new(1, uuid!("00000000-0000-0000-0000-000000000001")),
            ObjectIdentifier::new(2, uuid!("00000000-0000-0000-0000-000000000002")),
            ObjectIdentifier::new(3, uuid!("00000000-0000-0000-0000-000000000003")),
            ObjectIdentifier::new(4, uuid!("00000000-0000-0000-0000-000000000004")),
        ],
    };
    run_db_test(MIGS, |pool| async move {
        let now = AsezTimestamp::now();

        let (_handler, monolith) = MockMonolithService::new()
            .get_hierarchy(vec![GetHierarchyResponseItem {
                uuid: uuid!("00000000-0000-0000-0000-000000000001"),
                item_list: vec![
                    Attachment {
                        id: 1,
                        kind_id: 2,
                        category_id: Some(FoldersCategory::TenderDocumentation),
                        ..Default::default()
                    },
                    Attachment {
                        id: 2,
                        parent_id: Some(1),
                        kind_id: 1,
                        is_removed: false,
                        is_classified: false,
                        ..Default::default()
                    },
                ],
            }])
            .run()
            .unwrap();

        let (res, messages) = process_approve_proposal(
            USER_ID,
            String::new(),
            req.clone(),
            &monolith,
            &pool,
        )
        .await
        .unwrap();

        assert_eq!(res.item_list.len(), 4);
        assert_eq!(messages.messages.len(), 4);
        assert_eq!(messages.kind, MessageKind::Success);

        let expected_ids: Vec<i64> =
            vec![6500000000, 6500000001, 6500000002, 6500000003];

        let expected_uuids: Vec<_> = req.item_list.iter().map(|x| x.uuid).collect();

        for id in expected_ids {
            let expected_message =
                Message::success(format!("ТКП {} подтверждено", id));
            assert!(
                messages.messages.contains(&expected_message),
                "Ожидалось сообщение: '{}', но оно отсутствует",
                expected_message.text
            );
        }

        res.item_list.into_iter().for_each(|proposal| {
            assert_eq!(
                proposal.status_id.expect("Должно вернуть статус"),
                TcpGeneralStatus::Received,
                "У записи {} неправильный статус",
                proposal.uuid.unwrap(),
            );
        });

        let headers_check = ProposalHeader::select(
            &Select::full::<ProposalHeader>()
                .in_any(ProposalHeader::uuid, expected_uuids),
            &*pool,
        )
        .await
        .unwrap();

        [
            (uuid!("00000000-0000-0000-0000-000000000001"), false),
            (uuid!("00000000-0000-0000-0000-000000000002"), false),
            (uuid!("00000000-0000-0000-0000-000000000003"), true),
            (uuid!("00000000-0000-0000-0000-000000000004"), true),
        ]
        .into_iter()
        .for_each(|(proposal_uuid, should_update_receive_date)| {
            let proposal = headers_check
                .iter()
                .find(|p| p.uuid == proposal_uuid)
                .expect("Обязано найти, иначе we are so doooomed");

            assert_eq!(
                proposal.status_id,
                TcpGeneralStatus::Received,
                "У записи {} неправильный статус",
                proposal.uuid,
            );
            assert!(
                proposal.changed_at > now,
                "У записи {} неправильный changed_at",
                proposal.uuid
            );
            assert!(
                proposal.changed_by == USER_ID,
                "У записи {} неправильный changed_at",
                proposal.uuid
            );
            if should_update_receive_date {
                assert!(
                    proposal.receive_date.unwrap() > now,
                    "У записи {} должен был обновиться receive_date",
                    proposal.uuid
                );
            } else {
                assert!(
                    proposal.receive_date.unwrap() < now,
                    "У записи {} не должен был обновиться receive_date",
                    proposal.uuid
                );
            }
        })
    })
    .await
}

// Монолит не имеет записи в иерархии с parent_id = 1
#[tokio::test]
async fn not_found_tcp() {
    let req = ApproveProposalReq {
        item_list: vec![ObjectIdentifier::new(
            1,
            uuid!("00000000-0000-0000-0000-000000000001"),
        )],
    };
    run_db_test(MIGS, |pool| async move {
        let (_handler, monolith) = MockMonolithService::new()
            .get_hierarchy(vec![GetHierarchyResponseItem {
                uuid: uuid!("00000000-0000-0000-0000-000000000001"),
                item_list: vec![Attachment {
                    id: 1,
                    kind_id: 2,
                    category_id: Some(FoldersCategory::TenderDocumentation),
                    ..Default::default()
                }],
            }])
            .run()
            .unwrap();

        let messages = match process_approve_proposal(
            USER_ID,
            String::new(),
            req.clone(),
            &monolith,
            &pool,
        )
        .await
        .unwrap_err()
        {
            TcpError::Business(messages) => messages,
            err => panic!("Была возвращена не та ошибка: {:?}", err),
        };

        let expected_messages = Messages {
            kind: MessageKind::Error,
            messages: vec![ApproveProposalMessage::not_found_tcp_document()],
        };

        assert_eq!(messages, expected_messages);
    })
    .await
}

#[tokio::test]
async fn missing_fields() {
    let req = ApproveProposalReq {
        item_list: vec![ObjectIdentifier::new(
            6500000004,
            uuid!("00000000-0000-0000-0000-000000000005"),
        )],
    };
    run_db_test(MIGS, |pool| async move {
        let (_handler, monolith) = MockMonolithService::new()
            .get_hierarchy(vec![GetHierarchyResponseItem {
                uuid: uuid!("00000000-0000-0000-0000-000000000005"),
                item_list: vec![
                    Attachment {
                        id: 1,
                        kind_id: 2,
                        category_id: Some(FoldersCategory::TenderDocumentation),
                        ..Default::default()
                    },
                    Attachment {
                        id: 2,
                        parent_id: Some(1),
                        kind_id: 1,
                        is_removed: false,
                        is_classified: false,
                        ..Default::default()
                    },
                ],
            }])
            .run()
            .unwrap();

        let messages = match process_approve_proposal(
            USER_ID,
            String::new(),
            req.clone(),
            &monolith,
            &pool,
        )
        .await
        .unwrap_err()
        {
            TcpError::Business(msgs) => msgs,
            err => panic!("Была возвращена не та ошибка: {:?}", err),
        };

        let header = ProposalHeader {
            uuid: uuid!("00000000-0000-0000-0000-000000000005"),
            id: 6500000004,
            ..Default::default()
        };
        let item1 = ProposalItem {
            uuid: uuid!("00000000-0000-0000-0000-000000000007"),
            ..Default::default()
        };
        let item2 = ProposalItem {
            uuid: uuid!("00000000-0000-0000-0000-000000000008"),
            ..Default::default()
        };
        let expected_messages = Messages {
            kind: MessageKind::Error,
            messages: vec![
                ApproveProposalMessage::missing_header_field(
                    "Организация",
                    &header,
                ),
                ApproveProposalMessage::missing_header_field(
                    "Начало срока действия",
                    &header,
                ),
                ApproveProposalMessage::missing_header_field(
                    "Окончание срока действия",
                    &header,
                ),
                ApproveProposalMessage::missing_header_field(
                    "UUID иерархии",
                    &header,
                ),
                ApproveProposalMessage::missing_item_field(
                    "Цена Организации (без НДС)",
                    &item1,
                ),
                ApproveProposalMessage::missing_item_field(
                    "Ставка НДС Организации",
                    &item1,
                ),
                ApproveProposalMessage::missing_item_field(
                    "Размер аванса, %",
                    &item1,
                ),
                ApproveProposalMessage::missing_item_field(
                    "Причина невозможности поставки",
                    &item2,
                ),
            ],
        };

        assert_eq!(expected_messages, messages);
    })
    .await
}
