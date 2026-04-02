use asez2_shared_db::uuid;

use crate::application::calls::process_pre_organization_question;

use crate::presentation::dto::{
    OrganizationQuestionItemReq, PreOrganizationQuestionReq,
};

use super::run_db_test_with_monolith;

const ORGANIZATION_QUESTION_EXTRA_MIGS: &[&str] = &["organization_question.sql"];
const USER_ID: i32 = 123;

#[tokio::test]
async fn pre_organization_question() {
    run_db_test_with_monolith(
        ORGANIZATION_QUESTION_EXTRA_MIGS,
        |pool, monolith| async move {
            let req = PreOrganizationQuestionReq {
                item_list: vec![
                    OrganizationQuestionItemReq {
                        supplier_id: 1,
                        request_uuid: uuid!("00000000-0000-0000-0000-000000000001"),
                    },
                    OrganizationQuestionItemReq {
                        supplier_id: 2,
                        request_uuid: uuid!("00000000-0000-0000-0000-000000000002"),
                    },
                ],
            };

            let (data, messages) = process_pre_organization_question(
                req,
                USER_ID,
                String::new(),
                &pool,
                &monolith,
            )
            .await
            .unwrap();

            assert!(messages.is_empty(), "Сообщений не должно быть");

            assert_eq!(data.item_list.len(), 2);
            // TODO: сейчас логика завязана на реализации monolyth и возвращаемых от него данных
            let attachments = &data
                .item_list
                .iter()
                .find(|i| {
                    i.organization_question.uuid.unwrap()
                        == uuid!("10000000-0000-0000-0000-000000000001")
                })
                .unwrap()
                .attachment_list;
            assert_eq!(attachments.len(), 2, "Монолит вернул не те аттачменты");
        },
    )
    .await;
}
