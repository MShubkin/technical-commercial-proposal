use ahash::AHashMap;
use itertools::Itertools;
use sqlx::PgPool;
use uuid::Uuid;

use asez2_shared_db::{
    db_item::{AdaptorableIter, AsezTimestamp, DbUpsert, Select},
    DbAdaptor, DbItem,
};

use monolith_service::{
    dto::attachment::{Attachment, UpdateHierarchyReq, UpdateHierarchyReqItem},
    http::MonolithHttpService,
};

use shared_essential::{
    domain::tcp::{
        OrganizationQuestion, PriceInformationRequestStatus, ProposalHeader,
        RequestHeader, RequestHeaderRep, RequestItem, RequestItemRep,
        RequestPartner, RequestPartnerRep, TcpDbItem,
    },
    presentation::dto::{
        general::FeWrapper,
        response_request::{ApiResponse, Message, Messages},
        technical_commercial_proposal::TcpResult,
    },
};

use super::{
    check_request_price_info::check_request_price_info,
    get_price_information_request_info_details_by_uuid,
};
use crate::presentation::dto::{
    UpdatePriceInformationRequest, UpdatePriceInformationResponse,
};

pub(crate) const STATUS_ID: &str = "status_id";
pub(crate) const QUESTION_ANSWER: &str = "question_answer";

/// Данные, которые требуются для создания записи
type CreateRequest =
    (RequestHeader, Vec<RequestItem>, Vec<RequestPartner>, Vec<Attachment>);
/// Данные, которые требуются для обновления записи. (Данные, Массив uuid request_item, которые надо удалить)
type MergeUpdateRequest = (CreateRequest, Vec<Uuid>);

/// Итоговые данные после совершения действия
type ActionOutput = (RequestHeader, Vec<RequestItem>, Vec<RequestPartner>);

pub(crate) async fn process_update_price_info(
    user_id: i32,
    monolith_token: String,
    req: UpdatePriceInformationRequest,
    monolith_service: &MonolithHttpService,
    pool: &PgPool,
) -> TcpResult<ApiResponse<UpdatePriceInformationResponse, ()>> {
    // If the precheck is not empty, then we
    let mut messages = check_request_price_info(&req).await;
    if messages.is_error() {
        return Ok(messages.into());
    }

    // If the header does not have a uuid, create a new request. If it does, we update it.
    let (res, (proposals, questions), action) = if let Some(uuid) = req.header.uuid
    {
        let old_price_info =
            get_price_information_request_info_details_by_uuid(uuid, pool).await?;

        let merge_request = merge_request(
            user_id,
            req,
            old_price_info.header,
            old_price_info.items,
            old_price_info.suppliers,
        )?;

        let res = update(
            merge_request,
            user_id,
            monolith_token,
            &mut messages,
            monolith_service,
            pool,
        )
        .await?;

        (
            res,
            (
                old_price_info.proposal_headers,
                old_price_info.organization_questions,
            ),
            "Обновлен",
        )
    } else {
        let update_req = decompose_new(user_id, req)?;

        let supplier_uuids = update_req.2.iter().map(|r| r.uuid);
        let proposal_select = Select::full::<ProposalHeader>()
            .in_any(ProposalHeader::supplier_uuid, supplier_uuids.clone());
        let question_select = Select::full::<OrganizationQuestion>()
            .in_any(ProposalHeader::supplier_uuid, supplier_uuids);

        let questions =
            OrganizationQuestion::select(&question_select, pool).await?;
        let proposals = ProposalHeader::select(&proposal_select, pool).await?;

        let res = create(
            update_req,
            user_id,
            monolith_token,
            &mut messages,
            monolith_service,
            pool,
        )
        .await?;

        (res, (proposals, questions), "Создан")
    };

    let res = build_response(res, proposals, questions)?;

    let text =
        format!("{action} ЗЦИ {}", res.request_header.id.unwrap_or_default());
    messages.add_prepared_message(Message::success(text));

    Ok((res, messages).into())
}

/// When updating, as always, the BE cannot guarantee which fields will be
/// delivered to the backend, and that they will be consistent. Therefore we
/// first retrieve the item from the database, merge with the new item and only
/// then update.
async fn update(
    merge_request: MergeUpdateRequest,
    user_id: i32,
    monolith_token: String,
    messages: &mut Messages,
    monolith_service: &MonolithHttpService,
    pool: &PgPool,
) -> TcpResult<ActionOutput> {
    let (
        (mut header, mut add_or_update_items, mut partners, attachments),
        delete_items,
    ) = merge_request;

    // Если на стороне монолита произошла ошибка, то и действие
    // дальше не имеет смысла.
    header.hierarchy_uuid = update_hierarchy(
        header.hierarchy_uuid,
        attachments,
        user_id,
        monolith_token,
        messages,
        monolith_service,
    )
    .await?;

    let mut tx = pool.begin().await?;

    let header = header.update_returning::<_, &str>(None, None, &mut tx).await?;

    let add_or_update_items =
        RequestItem::upsert_returning(&mut add_or_update_items, None, &mut tx)
            .await?;

    if !delete_items.is_empty() {
        RequestItem::delete_by_uuids(&delete_items, &mut tx).await?;
    }
    let partners =
        RequestPartner::upsert_returning(&mut partners, None, &mut tx).await?;

    tx.commit().await?;

    Ok((header, add_or_update_items, partners))
}

async fn create(
    update_req: CreateRequest,
    user_id: i32,
    monolith_token: String,
    messages: &mut Messages,
    monolith_service: &MonolithHttpService,
    pool: &PgPool,
) -> TcpResult<ActionOutput> {
    let (mut header, mut items, mut partners, attachments) = update_req;

    // TODO: Updating hierarchy first is not ideal, since if we fail later
    // we can't undo here. However, if we update the DB first, we
    // 1. Hold the transaction for too long, and
    // 2. We have to update the request table again anyway!
    header.hierarchy_uuid = update_hierarchy(
        header.hierarchy_uuid,
        attachments,
        user_id,
        monolith_token,
        messages,
        monolith_service,
    )
    .await?;
    header.status_id = PriceInformationRequestStatus::TcpProject;

    let mut tx = pool.begin().await?;

    let header = header.insert_returning(&mut tx).await?;
    let items = RequestItem::insert_vec_returning(&mut items, &mut tx).await?;
    let partners =
        RequestPartner::insert_vec_returning(&mut partners, &mut tx).await?;

    tx.commit().await?;

    Ok((header, items, partners))
}

async fn update_hierarchy(
    hierarchy_uuid: Option<Uuid>,
    item_list: Vec<Attachment>,
    user_id: i32,
    monolith_token: String,
    messages: &mut Messages,
    monolith_service: &MonolithHttpService,
) -> TcpResult<Option<Uuid>> {
    // Optimisation to not call the monolith if we don't have to?
    if item_list.is_empty() {
        return Ok(None);
    }
    let hierarchy_list = vec![UpdateHierarchyReqItem {
        uuid: hierarchy_uuid,
        item_list,
    }];
    let req = UpdateHierarchyReq { hierarchy_list };

    let response =
        monolith_service.update_hierarchy(req, monolith_token, user_id).await?;

    messages.add_messages(response.messages.into());

    let uuid = response.data.hierarchy_list.into_iter().map(|x| x.uuid).next();
    Ok(uuid)
}

/// Here we decompose the request and set *_at, *_by and `uuid` fields.
fn decompose_new(
    user_id: i32,
    request: UpdatePriceInformationRequest,
) -> TcpResult<CreateRequest> {
    let UpdatePriceInformationRequest {
        header,
        item_list,
        partner_list,
        attachment_list,
    } = request;

    let mut header = header.into_item()?;

    header.uuid = Uuid::new_v4();
    header.created_by = user_id;
    header.changed_by = user_id;
    header.created_at = AsezTimestamp::now();
    header.changed_at = header.created_at;

    let items = item_list
        .into_iter()
        .enumerate()
        .map(|(n, x)| {
            let mut item = x.into_item()?;
            item.uuid = Uuid::new_v4();
            item.request_uuid = header.uuid;
            item.number = n as i16 + 1;

            Ok(item)
        })
        .collect::<TcpResult<Vec<_>>>()?;

    // We assume that we have less than 32767 partners.
    let partners = partner_list
        .into_iter()
        .enumerate()
        .map(|(n, x)| {
            let mut partner = x.into_item()?;
            partner.uuid = Uuid::new_v4();
            partner.request_uuid = header.uuid;
            partner.number = n as i16 + 1;

            Ok(partner)
        })
        .collect::<TcpResult<Vec<_>>>()?;

    Ok((header, items, partners, attachment_list))
}

fn merge_request(
    user_id: i32,
    req: UpdatePriceInformationRequest,
    old_header: RequestHeader,
    old_items: Vec<RequestItem>,
    old_partners: Vec<RequestPartner>,
) -> TcpResult<MergeUpdateRequest> {
    let UpdatePriceInformationRequest {
        header,
        item_list,
        partner_list,
        attachment_list,
    } = req;

    let mut header = header.into_item_merged(old_header)?;
    header.changed_at = AsezTimestamp::now();
    header.changed_by = user_id;
    let header_uuid = header.uuid;

    let attachments = attachment_list;

    let mut old_items =
        old_items.into_iter().map(|x| (x.uuid, x)).collect::<AHashMap<_, _>>();

    let mut number = 1;
    // We assume that items arrive in the order that the user wishes to
    // display them.
    let add_or_update_items = item_list
        .into_iter()
        .sorted_by_key(|item| item.number)
        .map(|new| {
            let item =
                if let Some(old) = new.uuid.and_then(|x| old_items.remove(&x)) {
                    new.into_item_merged(old)?
                } else {
                    let mut item = new.into_item()?;
                    item.uuid = Uuid::new_v4();
                    item.request_uuid = header_uuid;
                    item
                };
            Ok(item.numerate(&mut number))
        })
        .collect::<TcpResult<Vec<_>>>()?;

    let delete_items = old_items.keys().cloned().collect_vec();

    let mut old_partners = old_partners
        .into_iter()
        .map(|x| (x.uuid, x))
        .collect::<AHashMap<_, _>>();
    let mut number = 1;

    let partners = partner_list
        .into_iter()
        .map(|new| {
            let item =
                if let Some(old) = new.uuid.and_then(|x| old_partners.remove(&x)) {
                    new.into_item_merged(old)?
                } else {
                    let mut item = new.into_item()?;
                    item.uuid = Uuid::new_v4();
                    item.request_uuid = header_uuid;
                    item
                };
            Ok(item.numerate(&mut number))
        })
        .collect::<TcpResult<Vec<_>>>()?
        .into_iter()
        // We then update the remaining items, placing them at the bottom of
        // the item list to avoid potential conflict in numeration.
        .chain(old_partners.into_iter().map(|(_, item)| item.numerate(&mut number)))
        .collect();

    Ok(((header, add_or_update_items, partners, attachments), delete_items))
}

fn build_response(
    output: ActionOutput,
    proposals: Vec<ProposalHeader>,
    questions: Vec<OrganizationQuestion>,
) -> TcpResult<UpdatePriceInformationResponse> {
    let (header, items, partners) = output;

    let proposal_map = proposals
        .iter()
        .map(|x| (x.supplier_uuid, x))
        .collect::<AHashMap<_, _>>();
    // Из за ошибок выборки появляются дубликаты
    let mut questions_map = AHashMap::new();
    for s in questions.iter().unique_by(|x| x.uuid) {
        questions_map.entry(s.supplier_uuid).or_insert(vec![]).push(s);
    }

    let request_header = RequestHeaderRep::from_item::<&str>(header, None);
    let item_list = items
        .into_iter()
        .adaptors::<RequestItemRep>()
        .map(FeWrapper::new)
        .collect();
    let partner_list = partners
        .into_iter()
        .map(|x| {
            let supplier_uuid = x.uuid;
            let r = RequestPartnerRep::from_item::<&str>(x, None);
            let mut wrapper = FeWrapper::new(r);

            if let Some(pr) = proposal_map.get(&supplier_uuid) {
                wrapper = wrapper.add_field(STATUS_ID, pr.status_id as i16);
            }

            if let Some(questions) =
                questions_map.get(&supplier_uuid).map(|question| {
                    let q = question.len() as i64;
                    let a =
                        question.iter().filter(|x| x.answer_created_at.is_some());
                    vec![q, a.count() as i64]
                })
            {
                wrapper = wrapper.add_field(QUESTION_ANSWER, questions);
            }

            wrapper
        })
        .collect();

    Ok(UpdatePriceInformationResponse {
        request_header,
        item_list,
        partner_list,
    })
}
