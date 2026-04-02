use crate::application::calls::check_request_price_info::check_request_price_info;
use crate::presentation::dto::UpdatePriceInformationRequest;
use monolith_service::dto::attachment::{
    Attachment as MonolithAttachment, FoldersCategory,
};
use shared_essential::domain::tables::tcp::PriceInformationRequestType;
use shared_essential::domain::tcp::RequestHeaderRep;
use shared_essential::presentation::dto::response_request::{
    Message, MessageKind, Messages,
};

#[tokio::test]
async fn check_general_request_price_info_fail() {
    let request1 = UpdatePriceInformationRequest {
        header: Default::default(),
        item_list: vec![],
        partner_list: vec![],
        attachment_list: vec![],
    };
    let messages = check_request_price_info(&request1).await;

    let expected_messages = Messages {
        kind: MessageKind::Information,
        messages: vec![
            Message::info("Заполните поле \"Тип ЗЦИ\"")
                .with_fields(vec!["type_request_id".into()]),
            Message::info("Заполните поле \"Предмет ЗЦИ\"")
                .with_fields(vec!["request_subject".into()]),
            Message::info("Заполните поле \"Контактное лицо\"")
                .with_fields(vec!["organizer_name".into()]),
            Message::info("Заполните поле \"Электронный адрес\"")
                .with_fields(vec!["organizer_mail".into()]),
            Message::info("Заполните поле \"Телефон\"")
                .with_fields(vec!["organizer_phone".into()]),
            Message::info("Заполните поле \"Местонахождение\"")
                .with_fields(vec!["organizer_location".into()]),
            Message::info("Прикрепите Техническое задание"),
            Message::info("Прикрепите Договорные документы"),
            Message::info("Заполните данные спецификации"),
        ],
    };

    assert_eq!(messages, expected_messages);
}

#[tokio::test]
async fn check_private_request_price_info_fail() {
    let request = UpdatePriceInformationRequest {
        partner_list: vec![],

        header: RequestHeaderRep {
            request_type_text: None,

            uuid: None,
            id: None,
            plan_uuid: None,
            plan_id: None,
            hierarchy_uuid: None,
            type_request_id: Some(Some(PriceInformationRequestType::Private)),
            request_subject: Some(Some("request_subject".to_owned())),
            start_date: None,
            end_date: None,
            status_id: None,
            customer_id: None,
            currency_id: None,
            organizer_id: Some(Some(1)),
            organizer_name: Some(Some("organizer_name".to_owned())),
            organizer_mail: Some(Some("organizer_mail".to_owned())),
            organizer_phone: Some(Some("organizer_phone".to_owned())),
            organizer_location: Some(Some("organizer_location".to_owned())),
            reason_closing: Some(Some("reason_closing".to_owned())),
            purchasing_trend_id: None,
            created_by: None,
            created_at: None,
            changed_by: None,
            changed_at: None,
        },
        item_list: vec![Default::default()],
        attachment_list: vec![
            MonolithAttachment {
                id: 1,
                category_id: Some(FoldersCategory::TechnicalSpecification),
                kind_id: 2,
                ..Default::default()
            },
            MonolithAttachment {
                id: 11,
                category_id: Some(FoldersCategory::TechnicalSpecification),
                kind_id: 1,
                parent_id: Some(1),
                is_removed: false,
                is_classified: false,
                ..Default::default()
            },
            MonolithAttachment {
                id: 2,
                category_id: Some(FoldersCategory::ContractDocuments),
                kind_id: 2,
                ..Default::default()
            },
            MonolithAttachment {
                id: 12,
                category_id: Some(FoldersCategory::ContractDocuments),
                kind_id: 1,
                parent_id: Some(2),
                is_removed: false,
                is_classified: false,
                ..Default::default()
            },
        ],
    };
    let messages = check_request_price_info(&request).await;

    let expected_messages = Messages {
        kind: MessageKind::Error,
        messages: vec![
            Message::error("Для закрытого ЗЦИ заполните поле \"Обоснование\"")
                .with_fields(vec!["request_type_text".to_string()]),
            Message::info("Заполните данные организаций"),
        ],
    };

    assert_eq!(messages, expected_messages);
}

#[tokio::test]
async fn check_request_price_info_success() {
    let request = UpdatePriceInformationRequest {
        header: RequestHeaderRep {
            uuid: None,
            id: None,
            plan_uuid: None,
            plan_id: None,
            hierarchy_uuid: None,
            type_request_id: Some(Some(PriceInformationRequestType::Private)),
            request_subject: Some(Some("request_subject".to_owned())),
            start_date: None,
            end_date: None,
            status_id: None,
            customer_id: None,
            currency_id: None,
            request_type_text: Some(Some("request_type_text".to_owned())),
            organizer_id: Some(Some(1)),
            organizer_name: Some(Some("organizer_name".to_owned())),
            organizer_mail: Some(Some("organizer_mail".to_owned())),
            organizer_phone: Some(Some("organizer_phone".to_owned())),
            organizer_location: Some(Some("organizer_location".to_owned())),
            reason_closing: Some(Some("reason_closing".to_owned())),
            purchasing_trend_id: None,
            created_by: None,
            created_at: None,
            changed_by: None,
            changed_at: None,
        },
        item_list: vec![Default::default()],
        partner_list: vec![Default::default()],
        attachment_list: vec![
            MonolithAttachment {
                id: 1,
                category_id: Some(FoldersCategory::TechnicalSpecification),
                kind_id: 2,
                ..Default::default()
            },
            MonolithAttachment {
                id: 11,
                category_id: Some(FoldersCategory::TechnicalSpecification),
                kind_id: 1,
                parent_id: Some(1),
                is_removed: false,
                is_classified: false,
                ..Default::default()
            },
            MonolithAttachment {
                id: 2,
                category_id: Some(FoldersCategory::ContractDocuments),
                kind_id: 2,
                ..Default::default()
            },
            MonolithAttachment {
                id: 12,
                category_id: Some(FoldersCategory::ContractDocuments),
                kind_id: 1,
                parent_id: Some(2),
                is_removed: false,
                is_classified: false,
                ..Default::default()
            },
        ],
    };

    let messages = check_request_price_info(&request).await;
    assert!(messages.messages.is_empty());
}

#[tokio::test]
async fn check_request_price_info_attachment_fail() {
    let mut request = UpdatePriceInformationRequest {
        header: RequestHeaderRep {
            uuid: None,
            id: None,
            plan_uuid: None,
            plan_id: None,
            hierarchy_uuid: None,
            type_request_id: Some(Some(PriceInformationRequestType::Private)),
            request_subject: Some(Some("request_subject".to_owned())),
            start_date: None,
            end_date: None,
            status_id: None,
            customer_id: None,
            currency_id: None,
            request_type_text: Some(Some("request_type_text".to_owned())),
            organizer_id: Some(Some(1)),
            organizer_name: Some(Some("organizer_name".to_owned())),
            organizer_mail: Some(Some("organizer_mail".to_owned())),
            organizer_phone: Some(Some("organizer_phone".to_owned())),
            organizer_location: Some(Some("organizer_location".to_owned())),
            reason_closing: Some(Some("reason_closing".to_owned())),
            purchasing_trend_id: None,
            created_by: None,
            created_at: None,
            changed_by: None,
            changed_at: None,
        },
        item_list: vec![Default::default()],
        partner_list: vec![Default::default()],
        attachment_list: vec![],
    };

    let expected_messages = Messages {
        kind: MessageKind::Information,
        messages: vec![
            Message::info("Прикрепите Техническое задание"),
            Message::info("Прикрепите Договорные документы"),
        ],
    };

    // only folders
    request.attachment_list = vec![
        MonolithAttachment {
            id: 1,
            category_id: Some(FoldersCategory::TechnicalSpecification),
            kind_id: 2,
            ..Default::default()
        },
        MonolithAttachment {
            id: 2,
            category_id: Some(FoldersCategory::ContractDocuments),
            kind_id: 2,
            ..Default::default()
        },
    ];
    let messages = check_request_price_info(&request).await;
    assert_eq!(messages, expected_messages);

    // files is removed or is_classified
    request.attachment_list = vec![
        MonolithAttachment {
            id: 1,
            category_id: Some(FoldersCategory::TechnicalSpecification),
            kind_id: 2,
            ..Default::default()
        },
        MonolithAttachment {
            id: 11,
            category_id: Some(FoldersCategory::TechnicalSpecification),
            kind_id: 1,
            parent_id: Some(1),
            is_removed: true, // error
            is_classified: false,
            ..Default::default()
        },
        MonolithAttachment {
            id: 2,
            category_id: Some(FoldersCategory::ContractDocuments),
            kind_id: 2,
            ..Default::default()
        },
        MonolithAttachment {
            id: 12,
            category_id: Some(FoldersCategory::ContractDocuments),
            kind_id: 1,
            parent_id: Some(2),
            is_removed: false,
            is_classified: true, // error
            ..Default::default()
        },
    ];
    let messages = check_request_price_info(&request).await;
    assert_eq!(messages, expected_messages);
}
