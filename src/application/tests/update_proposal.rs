use asez2_shared_db::db_item::{AsezDate, Select};
use asez2_shared_db::{uuid, DbItem};
use monolith_service::dto::attachment::{
    Attachment, FoldersCategory, UpdateHierarchyResponseItem,
};
use shared_essential::domain::maths::{CurrencyValue, VatId};
use shared_essential::domain::tcp::{
    ProposalHeader, ProposalHeaderRep, ProposalItem, ProposalItemRep,
    RequestPartner,
};
use shared_essential::presentation::dto::response_request::{
    MessageKind, Messages,
};
use shared_essential::presentation::dto::technical_commercial_proposal::TcpError;
use testing::monolith::MockMonolithService;
use uuid::Uuid;

use crate::application::calls::update_proposal::{
    process_update_proposal, UpdateProposalMessage,
};
use crate::presentation::dto::UpdateProposalReq;

use super::run_db_test;

const UPDATE_PROPOSAL_EXTRA_MIGS: &[&str] = &["update_proposal.sql"];
const USER_ID: i32 = 123;

#[tokio::test]
async fn update_proposal_invalid_fields() {
    run_db_test(UPDATE_PROPOSAL_EXTRA_MIGS, |pool| async move {
        let (_handle, monolith) = MockMonolithService::new().run().unwrap();

        let req = UpdateProposalReq {
            supplier_id: None,
            attachment_list: vec![Attachment {
                category_id: Some(FoldersCategory::TechnicalSpecification),
                ..Default::default()
            }],
            header: ProposalHeaderRep {
                uuid: Some(uuid!("00000000-0000-0000-0000-000000000001")),
                start_date: None,
                end_date: None,
                supplier_uuid: None,
                ..Default::default()
            },
            item_list: vec![ProposalItemRep {
                uuid: Some(uuid!("00000000-0000-0000-0000-000000000001")),
                supplier_price: None,
                supplier_vat_id: None,
                is_possibility: Some(true),
                possibility_note: None,
                pay_condition_id: Some(Some(10)),
                prepayment_percent: None,
                request_item_uuid: Some(uuid!(
                    "00000000-0000-0000-0000-000000000001"
                )),
                ..Default::default()
            }],
        };

        let res = process_update_proposal(
            req.clone(),
            String::new(),
            USER_ID,
            &pool,
            &monolith,
        )
        .await;
        let Err(TcpError::Business(messages)) = res else {
            panic!("Была возвращено не то: {:?}", res)
        };

        let expected_messages = Messages {
            kind: MessageKind::Error,
            messages: vec![
                UpdateProposalMessage::missing_supplier_id_field(),
                UpdateProposalMessage::missing_header_field(
                    "Начало срока действия",
                    &req.header,
                ),
                UpdateProposalMessage::missing_header_field(
                    "Окончание срока действия",
                    &req.header,
                ),
                UpdateProposalMessage::missing_item_field(
                    "Цена Организации (без НДС)",
                    &req.item_list[0],
                ),
                UpdateProposalMessage::missing_item_field(
                    "Ставка НДС Организации",
                    &req.item_list[0],
                ),
                UpdateProposalMessage::missing_item_field(
                    "Причина невозможности поставки",
                    &req.item_list[0],
                ),
                UpdateProposalMessage::missing_item_field(
                    "Размер аванса, %",
                    &req.item_list[0],
                ),
                UpdateProposalMessage::not_found_tcp_document(),
            ],
        };
        assert_eq!(expected_messages, messages);
    })
    .await;
}

#[tokio::test]
async fn update_proposal_success() {
    run_db_test(UPDATE_PROPOSAL_EXTRA_MIGS, |pool| async move {
        let monolith_hierarchy_uuid = Uuid::new_v4();
        let (_handle, monolith) = MockMonolithService::new()
            .update_hierarchy(vec![UpdateHierarchyResponseItem {
                uuid: monolith_hierarchy_uuid,
            }])
            .run()
            .unwrap();

        let req = UpdateProposalReq {
            supplier_id: Some(999),
            attachment_list: vec![Attachment {
                category_id: Some(FoldersCategory::TenderDocumentation),
                ..Default::default()
            }],
            header: ProposalHeaderRep {
                uuid: Some(uuid!("00000000-0000-0000-0000-000000000001")),
                id: Some(6500000000),
                start_date: Some(Some(AsezDate::today())),
                end_date: Some(Some(AsezDate::today())),
                supplier_uuid: Some(Uuid::new_v4()),
                ..Default::default()
            },
            item_list: vec![ProposalItemRep {
                uuid: Some(uuid!("00000000-0000-0000-0000-000000000001")),
                request_item_uuid: Some(uuid!(
                    "00000000-0000-0000-0000-000000000001"
                )),
                supplier_price: Some(Some(1000.into())),
                supplier_vat_id: Some(Some(VatId::R0)),
                is_possibility: Some(true),
                possibility_note: Some(Some(String::from("blah blah blah"))),
                pay_condition_id: Some(Some(10)),
                prepayment_percent: Some(Some(1000.into())),
                supplier_sum_excluded_vat: Some(Some(1000.into())),
                ..Default::default()
            }],
        };

        let (res, messages) = process_update_proposal(
            req.clone(),
            String::new(),
            USER_ID,
            &pool,
            &monolith,
        )
        .await
        .unwrap();

        let expected_messages = Messages {
            kind: MessageKind::Success,
            messages: vec![UpdateProposalMessage::success()],
        };
        assert_eq!(expected_messages, messages);

        let header = ProposalHeader::select_single(
            &Select::full::<ProposalHeader>()
                .eq(ProposalHeader::id, req.header.id.unwrap()),
            &*pool,
        )
        .await
        .unwrap();
        let proposal_item = ProposalItem::select_single(
            &Select::full::<ProposalItem>()
                .eq(ProposalItem::uuid, req.item_list[0].uuid.unwrap()),
            &*pool,
        )
        .await
        .unwrap();

        // Удостовериться, что монолит вернул uuid в иерархии и что он был установлен
        assert_eq!(header.hierarchy_uuid.unwrap(), monolith_hierarchy_uuid);
        // Сумма всех proposal_item
        assert_eq!(
            header.sum_excluded_vat_total,
            req.item_list[0].supplier_sum_excluded_vat.unwrap()
        );
        // Удостовериться, что proposal_item обновляется
        assert_eq!(
            &proposal_item.possibility_note,
            req.item_list[0].possibility_note.as_ref().unwrap()
        );
        assert_eq!(
            &proposal_item.sum_excluded_vat,
            req.item_list[0].supplier_sum_excluded_vat.as_ref().unwrap()
        );
        // Удостовериться, что данные по ЗЦИ возвращаются обновляется
        assert_eq!(res.item_list[0].vat_id, VatId::R15);
        assert_eq!(res.item_list[0].price, CurrencyValue::from(5));
    })
    .await;
}

#[tokio::test]
async fn upsert_proposal_success() {
    run_db_test(UPDATE_PROPOSAL_EXTRA_MIGS, |pool| async move {
        let monolith_hierarchy_uuid = Uuid::new_v4();
        let (_handle, monolith) = MockMonolithService::new()
            .update_hierarchy(vec![UpdateHierarchyResponseItem {
                uuid: monolith_hierarchy_uuid,
            }])
            .run()
            .unwrap();

        let req = UpdateProposalReq {
            supplier_id: Some(999),
            attachment_list: vec![Attachment {
                category_id: Some(FoldersCategory::TenderDocumentation),
                ..Default::default()
            }],
            header: ProposalHeaderRep {
                start_date: Some(Some(AsezDate::today())),
                end_date: Some(Some(AsezDate::today())),
                supplier_uuid: Some(Uuid::new_v4()),
                ..Default::default()
            },
            item_list: vec![
                ProposalItemRep {
                    uuid: Some(uuid!("00000000-0000-0000-0000-000000000001")),
                    request_item_uuid: Some(uuid!(
                        "00000000-0000-0000-0000-000000000001"
                    )),
                    supplier_price: Some(Some(1000.into())),
                    supplier_vat_id: Some(Some(VatId::R0)),
                    is_possibility: Some(true),
                    possibility_note: Some(Some(String::from("blah blah blah"))),
                    pay_condition_id: Some(Some(10)),
                    prepayment_percent: Some(Some(1000.into())),
                    supplier_sum_excluded_vat: Some(Some(1000.into())),
                    ..Default::default()
                },
                ProposalItemRep {
                    request_item_uuid: Some(uuid!(
                        "00000000-0000-0000-0000-000000000002"
                    )),
                    supplier_price: Some(Some(1000.into())),
                    supplier_vat_id: Some(Some(VatId::R0)),
                    is_possibility: Some(true),
                    possibility_note: Some(Some(String::from("blah blah blah"))),
                    pay_condition_id: Some(Some(10)),
                    prepayment_percent: Some(Some(1000.into())),
                    supplier_sum_excluded_vat: Some(Some(1000.into())),
                    ..Default::default()
                },
            ],
        };

        let (res, messages) = process_update_proposal(
            req.clone(),
            String::new(),
            USER_ID,
            &pool,
            &monolith,
        )
        .await
        .unwrap();

        let expected_messages = Messages {
            kind: MessageKind::Success,
            messages: vec![UpdateProposalMessage::success()],
        };
        assert_eq!(expected_messages, messages);

        let header = ProposalHeader::select_single(
            &Select::full::<ProposalHeader>()
                .eq(ProposalHeader::id, res.header.id.unwrap()),
            &*pool,
        )
        .await
        .unwrap();
        let proposal_item = ProposalItem::select(
            &Select::full::<ProposalItem>()
                .eq(ProposalItem::proposal_uuid, res.header.uuid.unwrap()),
            &*pool,
        )
        .await
        .unwrap();

        // Удостовериться, что монолит вернул uuid в иерархии и что он был установлен
        assert_eq!(header.hierarchy_uuid.unwrap(), monolith_hierarchy_uuid);
        // Сумма всех proposal_item
        assert_eq!(
            header.sum_excluded_vat_total.unwrap(),
            req.item_list[0].supplier_sum_excluded_vat.unwrap().unwrap()
                + req.item_list[1].supplier_sum_excluded_vat.unwrap().unwrap()
        );
        // Удостовериться, что proposal_item обновляется и создается
        assert_eq!(
            &proposal_item[0].possibility_note,
            req.item_list[0].possibility_note.as_ref().unwrap()
        );
        assert_ne!(proposal_item[0].uuid, Uuid::default());
        assert_eq!(
            &proposal_item[1].possibility_note,
            req.item_list[1].possibility_note.as_ref().unwrap()
        );
        assert_ne!(proposal_item[1].uuid, Uuid::default());
        assert_eq!(
            &proposal_item[0].sum_excluded_vat,
            req.item_list[0].supplier_sum_excluded_vat.as_ref().unwrap()
        );
        assert_eq!(
            &proposal_item[1].sum_excluded_vat,
            req.item_list[0].supplier_sum_excluded_vat.as_ref().unwrap()
        );
        // Удостовериться, что данные по ЗЦИ возвращаются обновляется
        assert_eq!(res.item_list[0].vat_id, VatId::R15);
        assert_eq!(res.item_list[0].price, CurrencyValue::from(5));

        assert_eq!(res.item_list[1].vat_id, VatId::R11);
        assert_eq!(res.item_list[1].price, CurrencyValue::from(0.05));
    })
    .await;
}

/// Проверка на ошибку при работе с публичным партнером
#[tokio::test]
async fn insert_request_partner_fail() {
    run_db_test(UPDATE_PROPOSAL_EXTRA_MIGS, |pool| async move {
        let (_handle, monolith) = MockMonolithService::new().run().unwrap();

        let request_uuid = uuid!("00000000-0000-0000-0000-000000000001");
        let req = UpdateProposalReq {
            // Уже существующий публичный партнер
            supplier_id: Some(1),
            attachment_list: vec![Attachment {
                category_id: Some(FoldersCategory::TenderDocumentation),
                ..Default::default()
            }],
            header: ProposalHeaderRep {
                supplier_uuid: None,

                start_date: Some(Some(AsezDate::today())),
                end_date: Some(Some(AsezDate::today())),
                request_uuid: Some(request_uuid),
                ..Default::default()
            },
            item_list: vec![],
        };

        let res = process_update_proposal(
            req.clone(),
            String::new(),
            USER_ID,
            &pool,
            &monolith,
        )
        .await;
        let Err(TcpError::Business(messages)) = res else {
            panic!("Была возвращено не то: {:?}", res)
        };

        let expected_messages = Messages {
            kind: MessageKind::Error,
            messages: vec![UpdateProposalMessage::no_authority()],
        };
        assert_eq!(expected_messages, messages);
    })
    .await;
}

/// Проверка на корректность добавления request_partner записи при создании ProposalHeader
#[tokio::test]
async fn insert_request_partner_success() {
    run_db_test(UPDATE_PROPOSAL_EXTRA_MIGS, |pool| async move {
        let (_handle, monolith) = MockMonolithService::new()
            .update_hierarchy(vec![UpdateHierarchyResponseItem {
                uuid: Uuid::new_v4(),
            }])
            .run()
            .unwrap();

        let request_uuid = uuid!("00000000-0000-0000-0000-000000000002");
        let req = UpdateProposalReq {
            // Новый партнер
            supplier_id: Some(3),
            attachment_list: vec![Attachment {
                category_id: Some(FoldersCategory::TenderDocumentation),
                ..Default::default()
            }],
            header: ProposalHeaderRep {
                supplier_uuid: None,

                start_date: Some(Some(AsezDate::today())),
                end_date: Some(Some(AsezDate::today())),
                request_uuid: Some(request_uuid),
                ..Default::default()
            },
            item_list: vec![],
        };

        let (res, messages) = process_update_proposal(
            req.clone(),
            String::new(),
            USER_ID,
            &pool,
            &monolith,
        )
        .await
        .unwrap();

        let expected_messages = Messages {
            kind: MessageKind::Success,
            messages: vec![UpdateProposalMessage::success()],
        };
        assert_eq!(expected_messages, messages);

        let header = ProposalHeader::select_single(
            &Select::full::<ProposalHeader>()
                .eq(ProposalHeader::id, res.header.id.unwrap()),
            &*pool,
        )
        .await
        .unwrap();
        let partners = RequestPartner::select(
            &Select::full::<RequestPartner>()
                .eq(RequestPartner::request_uuid, request_uuid)
                .add_replace_order_asc(RequestPartner::number),
            &*pool,
        )
        .await
        .unwrap();

        assert_eq!(partners.len(), 2);
        let expected_partner_values = [
            RequestPartner {
                supplier_id: 2,
                number: 1,
                request_uuid,
                ..Default::default()
            },
            RequestPartner {
                supplier_id: 3,
                number: 2,
                request_uuid,

                ..Default::default()
            },
        ];
        partners.iter().zip(expected_partner_values).for_each(
            |(partner, expected_partner)| {
                assert!(
                    partner.supplier_id == expected_partner.supplier_id
                        && partner.number == expected_partner.number
                        && partner.request_uuid == expected_partner.request_uuid,
                    "Факт: {:?}\nОжидается: {:?}",
                    partner,
                    expected_partner
                )
            },
        );

        assert_eq!(header.supplier_uuid, partners[1].uuid)
    })
    .await;
}

/// Проверка на корректность обновления proposal_head при существовующем партнере
#[tokio::test]
async fn update_header_with_request_partner_on_upsert() {
    run_db_test(UPDATE_PROPOSAL_EXTRA_MIGS, |pool| async move {
        let monolith_hierarchy_uuid = Uuid::new_v4();
        let (_handle, monolith) = MockMonolithService::new()
            .update_hierarchy(vec![UpdateHierarchyResponseItem {
                uuid: monolith_hierarchy_uuid,
            }])
            .run()
            .unwrap();

        let request_uuid = uuid!("00000000-0000-0000-0000-000000000002");
        // Уже существующий непубличный партнер
        let supplier_id = 2;

        let req = UpdateProposalReq {
            supplier_id: Some(supplier_id),
            attachment_list: vec![Attachment {
                category_id: Some(FoldersCategory::TenderDocumentation),
                ..Default::default()
            }],
            header: ProposalHeaderRep {
                supplier_uuid: None,

                start_date: Some(Some(AsezDate::today())),
                end_date: Some(Some(AsezDate::today())),
                request_uuid: Some(request_uuid),
                ..Default::default()
            },
            item_list: vec![],
        };

        let (res, messages) = process_update_proposal(
            req.clone(),
            String::new(),
            USER_ID,
            &pool,
            &monolith,
        )
        .await
        .unwrap();

        let expected_messages = Messages {
            kind: MessageKind::Success,
            messages: vec![UpdateProposalMessage::success()],
        };
        assert_eq!(expected_messages, messages);

        let header = ProposalHeader::select_single(
            &Select::full::<ProposalHeader>()
                .eq(ProposalHeader::id, res.header.id.unwrap()),
            &*pool,
        )
        .await
        .unwrap();
        let partners = RequestPartner::select(
            &Select::full::<RequestPartner>()
                .eq(RequestPartner::request_uuid, request_uuid)
                .add_replace_order_asc(RequestPartner::number),
            &*pool,
        )
        .await
        .unwrap();

        assert_eq!(partners.len(), 1);
        assert_eq!(header.supplier_uuid, partners[0].uuid)
    })
    .await;
}
