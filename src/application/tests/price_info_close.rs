use super::*;

use asez2_shared_db::db_item::{AsezTimestamp, Select};
use asez2_shared_db::uuid;

use shared_essential::presentation::dto::general::ObjectIdentifier;
use shared_essential::presentation::dto::response_request::{
    MessageKind, ParamItem, Params, Status,
};

use crate::application::calls::price_info_close::*;
use crate::presentation::dto::{
    PrePriceInfoCloseItem, PrePriceInfoCloseResponse, RequestCloseItem,
};

const MIGS: &[&str] = &["price_info_close.sql"];

#[tokio::test]
async fn test_pre_price_info_close_errors() {
    let uuids = [
        uuid!("00000000-0000-0000-0000-000000000001"),
        uuid!("00000000-0000-0000-0000-000000000002"),
        uuid!("00000000-0000-0000-0000-000000000005"),
        uuid!("00000000-0000-0000-0000-000000000006"),
    ]
    .into_iter()
    .map(|uuid| ObjectIdentifier {
        id: 0,
        uuid,
        ..Default::default()
    })
    .collect::<Vec<_>>();

    run_db_test(MIGS, |pool| async move {
        let res = process_pre_price_info_close(999, uuids, &pool).await.unwrap();

        assert_eq!(res.status, Status::Ok);
        assert_eq!(res.messages.messages.len(), 2);

        let exp_params = Params {
            description: "Досрочное закрытие возможно только для ЗЦИ со статусом \
                \"Приём ТКП\" или \"Ошибка публикации изменений\". Скорректируйте выбор".to_string(),
            item_list: [2000000000, 2000000001].into_iter().map(ParamItem::from_id).collect::<Vec<_>>(),
        };
        let msg = &res.messages.messages[0];
        assert_eq!(msg.text, "Досрочное закрытие выбранных ЗЦИ невозможно");
        assert_eq!(msg.kind, MessageKind::Error);
        assert_eq!(msg.parameters, exp_params);

        let exp_params = Params {
            description: "Невозможно закрыть ЗЦИ, созданное другим пользователем".to_string(),
            item_list: [2000000005].into_iter().map(ParamItem::from_id).collect::<Vec<_>>(),
        };
        let msg = &res.messages.messages[1];
        assert_eq!(msg.text, "Нет полномочий");
        assert_eq!(msg.kind, MessageKind::Error);
        assert_eq!(msg.parameters, exp_params);

        assert!(res.data.item_list.is_empty());
    })
    .await
}

#[tokio::test]
async fn test_pre_price_info_close_success() {
    let uuids = [
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

    let exp = PrePriceInfoCloseResponse {
        item_list: vec![
            PrePriceInfoCloseItem {
                header: RequestHeaderRep {
                    uuid: Some(uuid!("00000000-0000-0000-0000-000000000004")),
                    id: Some(2000000003),
                    request_subject: Some(Some("help".to_owned())),
                    ..Default::default()
                },
                supplier_list: vec![5, 4, 3, 2],
            },
            PrePriceInfoCloseItem {
                header: RequestHeaderRep {
                    uuid: Some(uuid!("00000000-0000-0000-0000-000000000005")),
                    id: Some(2000000004),
                    request_subject: Some(Some("help".to_owned())),
                    ..Default::default()
                },
                supplier_list: vec![1],
            },
        ],
    };
    run_db_test(MIGS, |pool| async move {
        let res = process_pre_price_info_close(999, uuids, &pool).await.unwrap();

        assert_eq!(res.status, Status::Ok);
        assert!(res.messages.is_empty());

        assert_eq!(exp, res.data, "{:#?}\n{:#?}", exp, res.data);
    })
    .await
}

#[tokio::test]
async fn test_price_info_close_success() {
    let status = PriceInformationRequestStatus::EntryClosedEarly;
    let uuids_in = [
        uuid!("00000000-0000-0000-0000-000000000004"),
        uuid!("00000000-0000-0000-0000-000000000005"),
    ];
    let uuids = uuids_in
        .iter()
        .map(|uuid| ObjectIdentifier::new(0, *uuid))
        .map(|identifier| RequestCloseItem {
            identifier,
            reason_closing: "Test it to the limit!".to_string(),
        })
        .collect::<Vec<_>>();

    let exp = vec![
        RequestHeaderRep {
            uuid: Some(uuid!("00000000-0000-0000-0000-000000000004")),
            id: Some(2000000003),
            status_id: Some(status),
            ..Default::default()
        },
        RequestHeaderRep {
            uuid: Some(uuid!("00000000-0000-0000-0000-000000000005")),
            id: Some(2000000004),
            status_id: Some(status),
            ..Default::default()
        },
    ];
    run_db_test(MIGS, move |pool| async move {
        let res = process_price_info_close(999, uuids, &pool).await.unwrap();

        assert_eq!(res.status, Status::Ok);
        assert_eq!(res.messages.messages.len(), 1);
        let text = &res.messages.messages[0].text;

        assert_eq!(text, "Статус ЗЦИ изменен на \"Прием закрыт досрочно\"");

        assert_eq!(
            exp, res.data.item_list,
            "{:#?}\n{:#?}",
            exp, res.data.item_list
        );

        let select = Select::default()
            .in_any("uuid", uuids_in)
            .add_replace_order_asc("uuid");
        let mut reqs = RequestHeader::select(&select, &*pool).await.unwrap();

        let last_time = AsezTimestamp::try_from("2024-01-01 00:00:00").unwrap();
        assert_eq!(reqs.len(), 2);
        let req = reqs.pop().unwrap();
        assert_eq!(req.status_id, status);
        assert_ne!(req.changed_at, last_time);
        assert_eq!(req.changed_by, 999);
        assert_eq!(req.reason_closing, Some("Test it to the limit!".to_string()));
        let req = reqs.pop().unwrap();
        assert_eq!(req.status_id, status);
        assert_ne!(req.changed_at, last_time);
        assert_eq!(req.changed_by, 999);
        assert_eq!(req.reason_closing, Some("Test it to the limit!".to_string()));
    })
    .await
}

#[tokio::test]
async fn test_price_info_close_fail() {
    let uuids_in = [
        uuid!("00000000-0000-0000-0000-000000000001"),
        uuid!("00000000-0000-0000-0000-000000000002"),
        uuid!("00000000-0000-0000-0000-000000000005"),
        uuid!("00000000-0000-0000-0000-000000000006"),
    ];
    let uuids = uuids_in
        .iter()
        .map(|uuid| ObjectIdentifier::new(0, *uuid))
        .map(|identifier| RequestCloseItem {
            identifier,
            reason_closing: "Test it to the limit!".to_string(),
        })
        .collect::<Vec<_>>();

    run_db_test(MIGS, move |pool| async move {
        let res = process_price_info_close(999, uuids, &pool).await.unwrap();

        assert_eq!(res.status, Status::Ok);
        assert_eq!(res.messages.messages.len(), 2);

        let exp_params = Params {
            description: "Досрочное закрытие возможно только для ЗЦИ со статусом \
                \"Приём ТКП\" или \"Ошибка публикации изменений\". Скорректируйте выбор".to_string(),
            item_list: [2000000000, 2000000001].into_iter().map(ParamItem::from_id).collect::<Vec<_>>(),
        };
        let msg = &res.messages.messages[0];
        assert_eq!(msg.text, "Досрочное закрытие выбранных ЗЦИ невозможно");
        assert_eq!(msg.kind, MessageKind::Error);
        assert_eq!(msg.parameters, exp_params);

        let exp_params = Params {
            description: "Невозможно закрыть ЗЦИ, созданное другим пользователем".to_string(),
            item_list: [2000000005].into_iter().map(ParamItem::from_id).collect::<Vec<_>>(),
        };
        let msg = &res.messages.messages[1];
        assert_eq!(msg.text, "Нет полномочий");
        assert_eq!(msg.kind, MessageKind::Error);
        assert_eq!(msg.parameters, exp_params);

        assert!(res.data.item_list.is_empty());
    })
    .await
}
