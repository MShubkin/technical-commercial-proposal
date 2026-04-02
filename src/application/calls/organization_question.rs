use ahash::AHashMap;
use asez2_shared_db::db_item::{from_item_with_fields, Filter, FilterTree, Select};
use asez2_shared_db::DbItem;
use monolith_service::dto::attachment::GetHierarchyReq;
use monolith_service::http::MonolithHttpService;
use shared_essential::domain::tcp::OrganizationQuestion;
use shared_essential::presentation::dto::response_request::Messages;
use shared_essential::presentation::dto::technical_commercial_proposal::TcpResult;
use sqlx::PgPool;

use crate::presentation::dto::{
    OrganizationQuestionResponseItem, PreOrganizationQuestionReq,
    PreOrganizationQuestionResponseData,
};

const RETURN_FIELDS: &[&str] = &[
    OrganizationQuestion::uuid,
    OrganizationQuestion::question_text,
    OrganizationQuestion::question_created_at,
    OrganizationQuestion::answer_question_text,
    OrganizationQuestion::answer_created_at,
    OrganizationQuestion::answer_published_at,
];

pub async fn process_pre_organization_question(
    req: PreOrganizationQuestionReq,
    user_id: i32,
    token: String,
    pool: &PgPool,
    monolith: &MonolithHttpService,
) -> TcpResult<(PreOrganizationQuestionResponseData, Messages)> {
    let mut messages = Messages::default();

    let filters = req.item_list.iter().map(|i| {
        FilterTree::and_from_list([
            Filter::eq(OrganizationQuestion::supplier_id, i.supplier_id),
            Filter::eq(OrganizationQuestion::request_uuid, i.request_uuid),
        ])
    });

    let organization_question_select = Select::with_fields(RETURN_FIELDS)
        .set_filter_tree(FilterTree::or_from_list(filters));
    let organization_questions =
        OrganizationQuestion::select(&organization_question_select, pool).await?;

    let hierarchy_list = organization_questions.iter().map(|i| i.uuid).collect();
    let attachments = monolith
        .get_hierarchy(GetHierarchyReq { hierarchy_list }, token, user_id)
        .await?;
    messages.add_messages(attachments.messages.into());

    let mut attachments_by_question = attachments
        .data
        .hierarchy_list
        .into_iter()
        .map(|a| (a.uuid, a.item_list))
        .collect::<AHashMap<_, _>>();

    let from_item = from_item_with_fields(RETURN_FIELDS);
    let item_list = organization_questions
        .into_iter()
        .map(|organization_question| {
            let attachment_list = attachments_by_question
                .remove(&organization_question.uuid)
                .unwrap_or_default();
            let organization_question = from_item(organization_question);

            OrganizationQuestionResponseItem {
                organization_question,
                attachment_list,
            }
        })
        .collect();

    Ok((PreOrganizationQuestionResponseData { item_list }, messages))
}
