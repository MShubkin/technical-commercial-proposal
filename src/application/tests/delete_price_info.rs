use super::*;
use crate::application::calls::delete_price_info::process_delete_price_info;
use crate::presentation::dto::PriceInfoCompleteItem;

use asez2_shared_db::uuid;
use shared_essential::presentation::dto::{
    general::ObjectIdentifier,
    response_request::{Message, MessageKind, Messages, Status},
    technical_commercial_proposal::TcpError,
};

const MIGS: &[&str] = &["delete_price_info.sql"];

#[tokio::test]
async fn test_get_price_detail() {
    let req = [
        uuid!("00000000-0000-0000-0000-000000000001"),
        uuid!("00000000-0000-0000-0000-000000000002"),
        uuid!("00000000-0000-0000-0000-000000000004"),
        uuid!("00000000-0000-0000-0000-000000000005"),
    ]
    .into_iter()
    .map(|uuid| ObjectIdentifier {
        uuid,
        id: 0,
        ..Default::default()
    })
    .collect::<Vec<_>>();

    let exp = vec![
        PriceInfoCompleteItem {
            identifier: ObjectIdentifier {
                uuid: uuid!("00000000-0000-0000-0000-000000000004"),
                id: 2000000003,
                ..Default::default()
            },
            status_id: PriceInformationRequestStatus::Deleted,
        },
        PriceInfoCompleteItem {
            identifier: ObjectIdentifier {
                uuid: uuid!("00000000-0000-0000-0000-000000000005"),
                id: 2000000004,
                ..Default::default()
            },
            status_id: PriceInformationRequestStatus::Deleted,
        },
    ];
    run_db_test(MIGS, |pool| async move {
        let res = process_delete_price_info(999, req, &pool).await.unwrap();

        assert_eq!(res.status, Status::Ok);
        assert_eq!(res.messages.messages.len(), 2);
        assert_eq!(res.messages.kind, MessageKind::Error);

        let msg = &res.messages.messages[0];
        assert_eq!(&msg.text, "Выбранные ЗЦИ удалены");

        let msg = &res.messages.messages[1];
        assert_eq!(
            &msg.text,
            "Выбранные ЗЦИ опубликованы на ЭТП ГПБ. Удаление невозможно"
        );

        assert_eq!(
            exp, res.data.item_list,
            "{:#?}\n{:#?}",
            exp, res.data.item_list
        );
    })
    .await
}

#[tokio::test]
async fn permission_error() {
    let req = vec![
        ObjectIdentifier {
            uuid: uuid!("00000000-0000-0000-0000-000000000001"),
            id: 2000000000,
            ..Default::default()
        },
        ObjectIdentifier {
            uuid: uuid!("00000000-0000-0000-0000-000000000002"),
            id: 2000000001,
            ..Default::default()
        },
    ];

    run_db_test(MIGS, |pool| async move {
        let res = process_delete_price_info(0, req, &pool).await.unwrap_err();
        let TcpError::Business(messages) = res else {
            panic!("Была возвращена не та ошибка: {:?}", res)
        };

        let expected_messages = Messages {
            kind: MessageKind::Error,
            messages: vec![Message::error("Нет полномочий")
                .with_param_items(vec![
                    RequestHeader {
                        id: 2000000000,
                        ..Default::default()
                    },
                    RequestHeader {
                        id: 2000000001,
                        ..Default::default()
                    },
                ])
                .with_param_description(
                    "Невозможно удалить ЗЦИ, созданный другим пользователем",
                )],
        };
        assert_eq!(messages, expected_messages);
    })
    .await
}
