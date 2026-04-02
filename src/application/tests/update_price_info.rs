use super::*;

use asez2_shared_db::db_item::{AsezDate, Select};
use asez2_shared_db::uuid;

use monolith_service::dto::attachment::{
    Attachment, FoldersCategory, UpdateHierarchyResponseItem,
};

use shared_essential::domain::tcp::PriceInformationRequestType;
use shared_essential::presentation::dto::general::FeWrapper;
use shared_essential::presentation::dto::response_request::{
    Message, MessageKind, Status,
};
use shared_essential::presentation::dto::UiValue;
use testing::monolith::MockMonolithService;
use uuid::Uuid;

use crate::application::calls::update_price_info::{
    process_update_price_info, QUESTION_ANSWER, STATUS_ID,
};
use crate::presentation::dto::UpdatePriceInformationRequest;

const MIGS: &[&str] = &["update_price_info.sql"];
const USER_ID: i32 = 123;

#[tokio::test]
async fn test_update_price_info_fail_validate() {
    let attachment_list = vec![];

    let header_uuid = uuid!("00000000-0000-0000-0000-000000000001");
    let item_list = vec![];
    let partner_list = vec![];
    let upd_req = UpdatePriceInformationRequest {
        header: RequestHeaderRep {
            uuid: Some(header_uuid),
            // Tests for an error state in the validator.
            type_request_id: Some(Some(PriceInformationRequestType::Private)),
            organizer_name: Some(Some("Kirby".to_string())),
            organizer_mail: Some(Some("a@b.c".to_string())),
            organizer_phone: Some(Some("999".to_string())),
            organizer_location: Some(Some("space".to_string())),
            request_subject: Some(Some("don't help".to_string())),
            // NB: request_type_text is explicitly empty.
            request_type_text: None,
            ..Default::default()
        },
        item_list,
        partner_list,
        attachment_list,
    };
    run_db_test(MIGS, move |pool| async move {
        let (_handle, monolith) = MockMonolithService::new()
            .update_hierarchy(vec![UpdateHierarchyResponseItem {
                uuid: uuid!("00000000-0000-0000-0000-100000000001"),
            }])
            .run()
            .unwrap();

        let res = process_update_price_info(
            USER_ID,
            String::new(),
            upd_req,
            &monolith,
            &pool,
        )
        .await
        .unwrap();

        assert_eq!(res.status, Status::Ok);
        assert!(res.messages.is_error());

        let message = res
            .messages
            .messages
            .iter()
            .find(|x| x.kind == MessageKind::Error)
            .unwrap();
        // We have a warning. God knows why. Probably comes from the
        // "monolyth" mock. So we ignore it.
        assert_eq!(
            &message.text,
            "Для закрытого ЗЦИ заполните поле \"Обоснование\""
        );
    })
    .await
}

#[tokio::test]
async fn test_update_price_info_success_upd() {
    let attachment_list = vec![
        // We skip an attachment since we want to test that we still succeed while we have
        // a warning.
        Attachment {
            uuid: Some(uuid!("00000000-0000-0000-aaaa-000000000002")),
            id: 50,
            category_id: Some(FoldersCategory::TenderDocumentationTemplate),
            ..Default::default()
        },
        Attachment {
            uuid: Some(uuid!("00000000-0000-0000-aaaa-000000000003")),
            id: 50,
            category_id: Some(FoldersCategory::TechnicalSpecification),
            ..Default::default()
        },
    ];

    let header_uuid = uuid!("00000000-0000-0000-0000-000000000001");
    let item_list = vec![
        RequestItemRep {
            uuid: Some(uuid!("00000000-0000-0000-0000-000000000002")),
            delivery_start_date: Some(AsezDate::try_from_yo(2022, 365).unwrap()),
            ..Default::default()
        },
        RequestItemRep {
            uuid: Some(uuid!("00000000-0000-0000-0000-000000000001")),
            delivery_start_date: Some(AsezDate::try_from_yo(2021, 365).unwrap()),
            ..Default::default()
        },
    ];
    let partner_list = vec![
        RequestPartnerRep {
            uuid: Some(uuid!("00000000-0000-0000-0000-100000000001")),
            comment: Some(Some("Have a comment".to_string())),
            ..Default::default()
        },
        // This partner is new.
        RequestPartnerRep {
            comment: Some(Some("Have another comment".to_string())),
            ..Default::default()
        },
    ];
    let upd_req = UpdatePriceInformationRequest {
        header: RequestHeaderRep {
            uuid: Some(header_uuid),
            type_request_id: Some(Some(PriceInformationRequestType::Public)),
            organizer_name: Some(Some("Kirby".to_string())),
            organizer_mail: Some(Some("a@b.c".to_string())),
            organizer_phone: Some(Some("999".to_string())),
            organizer_location: Some(Some("space".to_string())),
            request_subject: Some(Some("don't help".to_string())),
            ..Default::default()
        },
        item_list,
        partner_list,
        attachment_list,
    };
    run_db_test(MIGS, move |pool| async move {
        let (_handle, monolith) = MockMonolithService::new()
            .update_hierarchy(vec![UpdateHierarchyResponseItem {
                uuid: uuid!("00000000-0000-0000-0000-100000000001"),
            }])
            .run()
            .unwrap();

        let headers = RequestHeader::select_all(&*pool).await.unwrap();
        let items = RequestItem::select_all(&*pool).await.unwrap();
        let partners = RequestPartner::select_all(&*pool).await.unwrap();

        assert_eq!(headers.len(), 2);
        assert_eq!(items.len(), 4);
        assert_eq!(partners.len(), 2);
        let mut res = process_update_price_info(
            USER_ID,
            String::new(),
            upd_req,
            &monolith,
            &pool,
        )
        .await
        .unwrap();

        assert_eq!(res.status, Status::Ok);

        let data = res.data;

        // We have info messages from the validator, and yet we have not failed.
        let validator_message = Message::info("Прикрепите Договорные документы");
        assert!(res.messages.messages.iter().any(|x| x == &validator_message));

        let message = res.messages.messages.pop().unwrap();
        // We have a warning. God knows why. Probably comes from the
        // "monolyth" mock. So we ignore it.
        assert_eq!(&message.text, "Обновлен ЗЦИ 2000000000");

        // This is changed.
        assert_ne!(
            data.request_header.changed_at,
            Some(AsezDate::try_from_yo(2024, 1).unwrap().to_timestamp())
        );
        assert_eq!(data.request_header.changed_by, Some(USER_ID));
        // This field remains the same.
        assert_eq!(data.request_header.uuid, Some(header_uuid));
        assert_eq!(data.request_header.status_id, Some(10i16.into()));
        assert_eq!(data.request_header.created_by, Some(999));
        assert_eq!(
            data.request_header.type_request_id,
            Some(Some(PriceInformationRequestType::Public))
        );
        assert_eq!(
            data.request_header.organizer_name,
            Some(Some("Kirby".to_string()))
        );
        assert_eq!(
            data.request_header.organizer_mail,
            Some(Some("a@b.c".to_string()))
        );
        assert_eq!(
            data.request_header.organizer_phone,
            Some(Some("999".to_string()))
        );
        assert_eq!(
            data.request_header.organizer_location,
            Some(Some("space".to_string()))
        );
        assert_eq!(
            data.request_header.created_at,
            Some(AsezDate::try_from_yo(1999, 1).unwrap().to_timestamp())
        );

        let headers = RequestHeader::select_all(&*pool).await.unwrap();

        let i_select = Select::full::<RequestItem>()
            .eq(
                RequestItem::request_uuid,
                uuid!("00000000-0000-0000-0000-000000000001"),
            )
            .add_replace_order_asc(RequestItem::number);
        let items = RequestItem::select(&i_select, &*pool).await.unwrap();

        let p_select = Select::full::<RequestPartner>()
            .eq(
                RequestItem::request_uuid,
                uuid!("00000000-0000-0000-0000-000000000001"),
            )
            .add_replace_order_asc(RequestPartner::number);
        let mut partners = RequestPartner::select(&p_select, &*pool).await.unwrap();

        partners.sort_by(|a, b| a.number.cmp(&b.number));

        assert_eq!(headers.len(), 2);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].uuid, uuid!("00000000-0000-0000-0000-000000000002"));
        assert_eq!(items[1].uuid, uuid!("00000000-0000-0000-0000-000000000001"));

        // Должен был создаться еще один партнер
        let partner1_uuid = uuid!("00000000-0000-0000-0000-100000000001");
        let partner2_uuid = uuid!("00000000-0000-0000-0000-100000000002");
        let new_partner_uuid = partners
            .iter()
            .find(|p| ![partner1_uuid, partner2_uuid].contains(&p.uuid))
            .map(|p| p.uuid)
            .expect("Не был возвращен новый партнер");

        let expected_partners = vec![
            ExpectedPartner {
                uuid: partner1_uuid,
                status_id: Some(TcpGeneralStatus::Created),
                questions: Some([1, 0]),
            },
            ExpectedPartner {
                uuid: partner2_uuid,
                status_id: Some(TcpGeneralStatus::Created),
                questions: Some([2, 1]),
            },
            ExpectedPartner {
                uuid: new_partner_uuid,
                status_id: None,
                questions: None,
            },
        ];
        verify_partners(&data.partner_list, expected_partners);
    })
    .await
}

#[tokio::test]
async fn test_update_price_info_success_new() {
    let attachment_list = vec![
        Attachment {
            // uuid: Some(uuid!("00000000-0000-0000-aaaa-000000000001")),
            id: 50,
            category_id: Some(FoldersCategory::ContractDocuments),
            ..Default::default()
        },
        Attachment {
            // uuid: Some(uuid!("00000000-0000-0000-aaaa-000000000002")),
            id: 50,
            category_id: Some(FoldersCategory::TenderDocumentationTemplate),
            ..Default::default()
        },
        Attachment {
            // uuid: Some(uuid!("00000000-0000-0000-aaaa-000000000003")),
            id: 50,
            category_id: Some(FoldersCategory::TechnicalSpecification),
            ..Default::default()
        },
    ];

    let item_list = vec![
        RequestItemRep {
            delivery_start_date: Some(AsezDate::try_from_yo(2021, 365).unwrap()),
            ..Default::default()
        },
        RequestItemRep {
            delivery_start_date: Some(AsezDate::try_from_yo(2022, 365).unwrap()),
            ..Default::default()
        },
    ];
    let partner_list = vec![
        RequestPartnerRep {
            comment: Some(Some("Have a comment".to_string())),
            ..Default::default()
        },
        // This partner is new.
        RequestPartnerRep {
            comment: Some(Some("Have another comment".to_string())),
            ..Default::default()
        },
    ];
    let upd_req = UpdatePriceInformationRequest {
        header: RequestHeaderRep {
            type_request_id: Some(Some(PriceInformationRequestType::Public)),
            organizer_name: Some(Some("Kirby".to_string())),
            organizer_mail: Some(Some("a@b.c".to_string())),
            organizer_phone: Some(Some("999".to_string())),
            organizer_location: Some(Some("space".to_string())),
            request_subject: Some(Some("don't help".to_string())),
            ..Default::default()
        },
        item_list,
        partner_list,
        attachment_list,
    };
    run_db_test(MIGS, |pool| async move {
        let (_handle, monolith) = MockMonolithService::new()
            .update_hierarchy(vec![UpdateHierarchyResponseItem {
                uuid: uuid!("00000000-0000-0000-0000-100000000001"),
            }])
            .run()
            .unwrap();

        let headers = RequestHeader::select_all(&*pool).await.unwrap();
        let items = RequestItem::select_all(&*pool).await.unwrap();
        let partners = RequestPartner::select_all(&*pool).await.unwrap();

        assert_eq!(headers.len(), 2);
        assert_eq!(items.len(), 4);
        assert_eq!(partners.len(), 2);
        let mut res = process_update_price_info(
            USER_ID,
            String::new(),
            upd_req,
            &monolith,
            &pool,
        )
        .await
        .unwrap();

        assert_eq!(res.status, Status::Ok);

        let data = res.data;
        let message = res.messages.messages.pop().unwrap();
        // We have a warning. God knows why. Probably comes from the
        // "monolyth" mock. So we ignore it.
        assert_eq!(&message.text, "Создан ЗЦИ 2000000002");

        // This is changed.
        assert_ne!(
            data.request_header.changed_at,
            Some(AsezDate::try_from_yo(2024, 1).unwrap().to_timestamp())
        );
        assert_ne!(
            data.request_header.created_at,
            Some(AsezDate::try_from_yo(1999, 1).unwrap().to_timestamp())
        );
        assert_eq!(data.request_header.changed_by, Some(USER_ID));
        // This field remains the same.
        assert_eq!(data.request_header.id, Some(2000000002));
        assert_eq!(
            data.request_header.status_id,
            Some(PriceInformationRequestStatus::TcpProject)
        );
        assert_eq!(data.request_header.created_by, Some(USER_ID));
        assert_eq!(
            data.request_header.type_request_id,
            Some(Some(PriceInformationRequestType::Public))
        );
        assert_eq!(
            data.request_header.organizer_name,
            Some(Some("Kirby".to_string()))
        );
        assert_eq!(
            data.request_header.organizer_mail,
            Some(Some("a@b.c".to_string()))
        );
        assert_eq!(
            data.request_header.organizer_phone,
            Some(Some("999".to_string()))
        );
        assert_eq!(
            data.request_header.organizer_location,
            Some(Some("space".to_string()))
        );
        let headers = RequestHeader::select_all(&*pool).await.unwrap();

        let i_select = Select::full::<RequestItem>().not_in_any(
            RequestItem::request_uuid,
            [
                uuid!("00000000-0000-0000-0000-000000000001"),
                uuid!("00000000-0000-0000-0000-000000000002"),
            ],
        );
        let items = RequestItem::select(&i_select, &*pool).await.unwrap();

        let p_select = Select::full::<RequestPartner>().not_in_any(
            RequestPartner::request_uuid,
            [
                uuid!("00000000-0000-0000-0000-000000000001"),
                uuid!("00000000-0000-0000-0000-000000000002"),
            ],
        );
        let mut partners = RequestPartner::select(&p_select, &*pool).await.unwrap();

        partners.sort_by(|a, b| a.number.cmp(&b.number));

        assert_eq!(headers.len(), 3);
        assert_eq!(items.len(), 2, "{:#?}", items);
        assert_eq!(items[0].number, 1);
        assert_eq!(items[1].number, 2);

        // Создается и заголовок, и два новых партнера, поэтому старых партнеров не должно сущестововать
        let expected_partners = partners
            .iter()
            .map(|p| ExpectedPartner {
                uuid: p.uuid,
                status_id: None,
                questions: None,
            })
            .collect();
        verify_partners(&data.partner_list, expected_partners);
    })
    .await
}

#[tokio::test]
async fn test_update_price_info_success_upd_with_delete_items() {
    let attachment_list = vec![
        // We skip an attachment since we want to test that we still succeed while we have
        // a warning.
        Attachment {
            uuid: Some(uuid!("00000000-0000-0000-aaaa-000000000002")),
            id: 50,
            category_id: Some(FoldersCategory::TenderDocumentationTemplate),
            ..Default::default()
        },
        Attachment {
            uuid: Some(uuid!("00000000-0000-0000-aaaa-000000000003")),
            id: 50,
            category_id: Some(FoldersCategory::TechnicalSpecification),
            ..Default::default()
        },
    ];

    let header_uuid = uuid!("00000000-0000-0000-0000-000000000001");
    let item_list = vec![
        // just one item for update. item with uuid 00000000-0000-0000-0000-000000000002 must be deleted
        RequestItemRep {
            uuid: Some(uuid!("00000000-0000-0000-0000-000000000001")),
            delivery_start_date: Some(AsezDate::try_from_yo(2021, 365).unwrap()),
            ..Default::default()
        },
    ];
    let partner_list = vec![
        RequestPartnerRep {
            uuid: Some(uuid!("00000000-0000-0000-0000-100000000001")),
            comment: Some(Some("Have a comment".to_string())),
            ..Default::default()
        },
        // This partner is new.
        RequestPartnerRep {
            comment: Some(Some("Have another comment".to_string())),
            ..Default::default()
        },
    ];
    let upd_req = UpdatePriceInformationRequest {
        header: RequestHeaderRep {
            uuid: Some(header_uuid),
            type_request_id: Some(Some(PriceInformationRequestType::Public)),
            organizer_name: Some(Some("Kirby".to_string())),
            organizer_mail: Some(Some("a@b.c".to_string())),
            organizer_phone: Some(Some("999".to_string())),
            organizer_location: Some(Some("space".to_string())),
            request_subject: Some(Some("don't help".to_string())),
            ..Default::default()
        },
        item_list,
        partner_list,
        attachment_list,
    };
    run_db_test(MIGS, move |pool| async move {
        let (_handle, monolith) = MockMonolithService::new()
            .update_hierarchy(vec![UpdateHierarchyResponseItem {
                uuid: uuid!("00000000-0000-0000-0000-100000000001"),
            }])
            .run()
            .unwrap();

        let headers = RequestHeader::select_all(&*pool).await.unwrap();
        let items = RequestItem::select_all(&*pool).await.unwrap();
        let partners = RequestPartner::select_all(&*pool).await.unwrap();

        assert_eq!(headers.len(), 2);
        assert_eq!(items.len(), 4);
        assert_eq!(partners.len(), 2);
        let mut res = process_update_price_info(
            USER_ID,
            String::new(),
            upd_req,
            &monolith,
            &pool,
        )
        .await
        .unwrap();

        assert_eq!(res.status, Status::Ok);

        let data = res.data;

        // We have info messages from the validator, and yet we have not failed.
        let validator_message = Message::info("Прикрепите Договорные документы");
        assert!(res.messages.messages.iter().any(|x| x == &validator_message));

        let message = res.messages.messages.pop().unwrap();
        // We have a warning. God knows why. Probably comes from the
        // "monolyth" mock. So we ignore it.
        assert_eq!(&message.text, "Обновлен ЗЦИ 2000000000");

        // This is changed.
        assert_ne!(
            data.request_header.changed_at,
            Some(AsezDate::try_from_yo(2024, 1).unwrap().to_timestamp())
        );
        assert_eq!(data.request_header.changed_by, Some(USER_ID));
        // This field remains the same.
        assert_eq!(data.request_header.uuid, Some(header_uuid));
        assert_eq!(data.request_header.status_id, Some(10i16.into()));
        assert_eq!(data.request_header.created_by, Some(999));
        assert_eq!(
            data.request_header.type_request_id,
            Some(Some(PriceInformationRequestType::Public))
        );
        assert_eq!(
            data.request_header.organizer_name,
            Some(Some("Kirby".to_string()))
        );
        assert_eq!(
            data.request_header.organizer_mail,
            Some(Some("a@b.c".to_string()))
        );
        assert_eq!(
            data.request_header.organizer_phone,
            Some(Some("999".to_string()))
        );
        assert_eq!(
            data.request_header.organizer_location,
            Some(Some("space".to_string()))
        );
        assert_eq!(
            data.request_header.created_at,
            Some(AsezDate::try_from_yo(1999, 1).unwrap().to_timestamp())
        );
        let headers = RequestHeader::select_all(&*pool).await.unwrap();

        let i_select = Select::full::<RequestItem>()
            .eq(
                RequestItem::request_uuid,
                uuid!("00000000-0000-0000-0000-000000000001"),
            )
            .add_replace_order_asc(RequestItem::number);
        let items = RequestItem::select(&i_select, &*pool).await.unwrap();

        let p_select = Select::full::<RequestPartner>()
            .eq(
                RequestItem::request_uuid,
                uuid!("00000000-0000-0000-0000-000000000001"),
            )
            .add_replace_order_asc(RequestPartner::number);
        let mut partners = RequestPartner::select(&p_select, &*pool).await.unwrap();

        partners.sort_by(|a, b| a.number.cmp(&b.number));

        assert_eq!(headers.len(), 2);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].uuid, uuid!("00000000-0000-0000-0000-000000000001"));

        // Должен был создаться еще один партнер
        let partner1_uuid = uuid!("00000000-0000-0000-0000-100000000001");
        let partner2_uuid = uuid!("00000000-0000-0000-0000-100000000002");
        let new_partner_uuid = partners
            .iter()
            .find(|p| ![partner1_uuid, partner2_uuid].contains(&p.uuid))
            .map(|p| p.uuid)
            .expect("Не был возвращен новый партнер");

        let expected_partners = vec![
            ExpectedPartner {
                uuid: partner1_uuid,
                status_id: Some(TcpGeneralStatus::Created),
                questions: Some([1, 0]),
            },
            ExpectedPartner {
                uuid: partner2_uuid,
                status_id: Some(TcpGeneralStatus::Created),
                questions: Some([2, 1]),
            },
            ExpectedPartner {
                uuid: new_partner_uuid,
                status_id: None,
                questions: None,
            },
        ];
        verify_partners(&data.partner_list, expected_partners);
    })
    .await
}

struct ExpectedPartner {
    uuid: Uuid,
    status_id: Option<TcpGeneralStatus>,
    questions: Option<[i64; 2]>,
}

fn verify_partners(
    partners: &[FeWrapper<RequestPartnerRep>],
    expected_partners: Vec<ExpectedPartner>,
) {
    assert_eq!(
        partners.len(),
        expected_partners.len(),
        "Было возвращено не то количество партнеров"
    );

    expected_partners.iter().for_each(|expected_partner| {
        let partner_uuid = expected_partner.uuid;
        let returned_partner = partners
            .iter()
            .find(|p| p.entity.uuid.expect("Не был возвращен uuid") == partner_uuid)
            .unwrap_or_else(|| {
                panic!("Не найден партнер {} в ответе", partner_uuid)
            });

        assert_eq!(
            returned_partner.extra_fields.get(STATUS_ID).cloned(),
            expected_partner.status_id.map(|s| UiValue::from(s as i16)),
            "Не совпадает значение по status_id у {}",
            partner_uuid
        );
        assert_eq!(
            returned_partner.extra_fields.get(QUESTION_ANSWER).cloned(),
            expected_partner.questions.map(|q| UiValue::from(q.to_vec())),
            "Не совпадает значение по status_id у {}",
            partner_uuid
        );
    })
}
