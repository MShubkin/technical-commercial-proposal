use super::*;

use asez2_shared_db::uuid;

use shared_essential::presentation::dto::general::ObjectIdentifier;
use shared_essential::presentation::dto::response_request::{
    Message, MessageKind, Messages, ParamItem, Params, Status,
};

use crate::application::calls::price_info_complete::*;

const MIGS: &[&str] = &["price_info_complete.sql"];

#[tokio::test]
async fn test_pre_price_info_complete_errors() {
    let uuids = [
        uuid!("00000000-0000-0000-0000-000000000001"),
        uuid!("00000000-0000-0000-0000-000000000004"),
        uuid!("00000000-0000-0000-0000-000000000005"),
    ]
    .into_iter()
    .map(|uuid| ObjectIdentifier {
        id: 0,
        uuid,
        ..Default::default()
    })
    .collect::<Vec<_>>();
    let err_params = Params {
        description: "Завершение рассмотрения возможно только для ЗЦИ со статусом \"Приём закрыт\" или \"Приём закрыт досрочно\". Скорректируйте выбор".to_string(),
        item_list: [2000000000].into_iter().map(ParamItem::from_id).collect::<Vec<_>>(),
    };
    let ok_params = Params {
        description: "".to_string(),
        item_list: [2000000003, 2000000004]
            .into_iter()
            .map(ParamItem::from_id)
            .collect::<Vec<_>>(),
    };

    run_db_test(MIGS, |pool| async move {
        let res =
            process_price_info_complete(999, uuids.clone(), &pool).await.unwrap();

        assert_eq!(res.status, Status::Ok);
        assert_eq!(res.messages.messages.len(), 2);

        let msg = &res.messages.messages[0];
        assert_eq!(
            msg.text,
            "Статус выбранных ЗЦИ изменен на \"Рассмотрение завершено\""
        );
        assert_eq!(msg.kind, MessageKind::Success);
        assert_eq!(msg.parameters, ok_params);

        let msg = &res.messages.messages[1];
        assert_eq!(msg.text, "Завершение рассмотрения выбранных ЗЦИ невозможно");
        assert_eq!(msg.kind, MessageKind::Error);
        assert_eq!(msg.parameters, err_params);

        assert_eq!(res.data.item_list.len(), 2);
        assert_eq!(res.data.item_list[0].identifier.id, 2000000003);
        assert_eq!(
            res.data.item_list[0].identifier.uuid,
            uuid!("00000000-0000-0000-0000-000000000004"),
        );
        assert_eq!(
            res.data.item_list[0].status_id,
            PriceInformationRequestStatus::Reviewed
        );
        assert_eq!(res.data.item_list[1].identifier.id, 2000000004);
        assert_eq!(
            res.data.item_list[1].identifier.uuid,
            uuid!("00000000-0000-0000-0000-000000000005"),
        );
        assert_eq!(
            res.data.item_list[1].status_id,
            PriceInformationRequestStatus::Reviewed
        );

        // Wrong user
        let res = process_price_info_complete(0, uuids, &pool).await.unwrap();
        assert_eq!(res.status, Status::Ok);

        let expected_messages = Messages {
            kind: MessageKind::Error,
            messages: vec![Message::error("Нет полномочий")
                .with_param_description(
                    "Невозможно закрыть ЗЦИ, созданный другим пользователем",
                )
                .with_param_items([
                    RequestHeader {
                        id: 2000000000,
                        ..Default::default()
                    },
                    RequestHeader {
                        id: 2000000003,
                        ..Default::default()
                    },
                    RequestHeader {
                        id: 2000000004,
                        ..Default::default()
                    },
                ])],
        };
        assert_eq!(res.messages, expected_messages);
    })
    .await
}
