#![allow(unreachable_code)]
use actix_web::http::header::ContentType;
use actix_web::web::{Data, Json, Query};
use actix_web::{web, Error, HttpRequest, HttpResponse, Responder};
use futures::future::ok;
use futures::stream::once;
use sqlx::PgPool;

use monolith_service::http::monolith_token::MonolithToken;
use monolith_service::http::{MonolithHttpService, MONOLITH_TOKEN_COOKIE};

use crate::application::calls::*;
use crate::common::Validate;
use crate::presentation::dto::*;
use rabbit_services::integration::IntegrationService;
use rabbit_services::print_doc::PrintDocService;
use rabbit_services::processing::ProcessingService;
use rabbit_services::view_storage::ViewStorageService;
use shared_essential::common::compression::decompress_bzip;
use shared_essential::presentation::dto::general::UiExportTableReq;
use shared_essential::presentation::dto::technical_commercial_proposal::UiSection;
use shared_essential::presentation::dto::{
    error::AsezError,
    general::{Id, ObjectIdentifier, ObjectIdentifierList, UserId},
    processing::GetPlanResponse,
    response_request::{ApiResponse, ResponseMessage},
    technical_commercial_proposal::create_price_information_request::CreatePriceInformationRequest,
};
use uuid::Uuid;

macro_rules! handle_result {
    ($res:expr,$handle:literal) => {{
        let r = match $res {
            Ok(r) => r,
            Err(e) => {
                tracing::error!(kind = "tcp", "Ошибка в '{}': {}", $handle, e);
                let messages = e.message_response();
                messages.into()
            }
        };
        Json(r)
    }};
}

pub(crate) async fn create_price_request_report_handle() -> impl Responder {
    todo!();
    ""
}

pub(crate) async fn send_for_proposal_handle() -> impl Responder {
    todo!();
    ""
}

pub(crate) async fn complete_handle() -> impl Responder {
    todo!();
    ""
}

pub(crate) async fn tkp_reject_handle() -> impl Responder {
    todo!();
    ""
}

pub(crate) async fn tkp_verified_handle() -> impl Responder {
    todo!();
    ""
}

/// Создание запроса ценовой информации
/// Route - /rest/technical_commercial_proposal/v1/create_price_information_request/
pub(crate) async fn create_price_information_request_handle(
    processing: ProcessingService,
    pool: Data<PgPool>,
    json_create_request: Json<CreatePriceInformationRequest>,
) -> impl Responder {
    tracing::info!(
        kind = "tcp",
        "create_price_information_request. json: {:?}",
        &json_create_request
    );
    if let Some(errors) = json_create_request.validate_json() {
        return Json(GetPlanResponse::default().with_messages(errors));
    }
    let res = create_price_information_request(
        json_create_request.into_inner(),
        processing,
        pool.get_ref(),
    )
    .await;
    handle_result!(res, "create_price_information_request_handle")
}

pub(crate) async fn get_request_price_info_detail_handle(
    data: Data<PgPool>,
    detail: Json<GetRequestPriceInfoDetail>,
    user_id: Query<UserId>,
) -> impl Responder {
    tracing::info!(
        kind = "tcp",
        "get_request_price_info_detail. id:: `{:?}`",
        detail.id
    );
    let detail = detail.into_inner();

    let response =
        get_request_price_info_detail(data.get_ref(), detail, user_id.0).await;
    handle_result!(response, "get_request_price_info_detail")
}

/// Получение списка ЗЦИ по Select
/// Route - /rest/technical_commercial_proposal/v1/get/request_price_info_list/
pub(crate) async fn get_request_price_info_list_handle(
    Json(dto): Json<GetRequestPriceListReq>,
    user_id: Query<UserId>,
    views: ViewStorageService,
    config: Data<PgPool>,
) -> impl Responder {
    tracing::info!(
        kind = "tcp",
        "get_request_price_info_list. user_id: {:?}, request: {:?}",
        &user_id,
        dto
    );
    let res =
        get_request_price_info_list(dto, user_id.0, views, config.get_ref()).await;
    handle_result!(res, "get_request_price_info_list")
}

/// Получить запись из proposal_head по id.
/// Route - /rest/technical_commercial_proposal/v1/get/proposal_detail/
pub(crate) async fn get_proposal_detail_handle(
    Query(user_id): Query<UserId>,
    Json(proposal_id): Json<Id>,
    pg_pool: Data<PgPool>,
) -> impl Responder {
    tracing::info!(
        kind = "tcp",
        "get_proposal_detail. user_id: {user_id:?}, proposal_id: {proposal_id:?}"
    );
    let res = get_proposal_detail(user_id, proposal_id.id, pg_pool.get_ref()).await;
    handle_result!(res, "get_proposal_detail")
}

/// Получение предметов закупки ЗЦИ по uuid Организации Предметов закупки
/// Route - /rest/technical_commercial_proposal/v1/get/purchasing_subject_by_group_uuid/{uuid}/
pub(crate) async fn get_purchasing_subject_by_group_uuid_handle(
    path: web::Path<Uuid>,
    pg_pool: Data<PgPool>,
) -> impl Responder {
    tracing::info!(
        kind = "tcp",
        "get_purchasing_subject_by_group_uuid. uuid:: `{:?}`",
        &path
    );
    let response =
        get_purchasing_subject_by_group_uuid(pg_pool.get_ref(), path.into_inner())
            .await;

    handle_result!(response, "get_purchasing_subject_by_group_uuid")
}

/// Получение актуального списка организаций по идентификатору "Предмета закупки"
/// Route - /rest/technical_commercial_proposal/v1/get/organizations/{uuid_subject}
pub(crate) async fn get_organizations_handle(
    path: web::Path<Uuid>,
    pg_pool: Data<PgPool>,
) -> impl Responder {
    tracing::info!(kind = "tcp", "get_organizations. uuid_subject:: `{:?}`", &path);
    let response = get_organizations(pg_pool.get_ref(), path.into_inner()).await;
    handle_result!(response, "get_organizations")
}

/// Получение актуальных записей справочника "Группа Предметов закупки"
/// Route - /rest/technical_commercial_proposal/v1/get/purchasing_subject_group
pub(crate) async fn get_purchasing_subject_group_handle(
    user_id: Query<UserId>,
    pg_pool: Data<PgPool>,
) -> impl Responder {
    tracing::info!(
        kind = "tcp",
        "get_purchasing_subject_group. user_id: {user_id:?}"
    );
    let response = get_purchasing_subject_group(
        pg_pool.get_ref(),
        user_id.into_inner().user_id,
    )
    .await;
    handle_result!(response, "get_purchasing_subject_group")
}

/// Получение списка ЗЦИ по uuid ППЗ
/// Route - /rest/technical_commercial_proposal/v1/get_price_information_request_by_plan_uuid/{uuid}/
pub(crate) async fn get_price_information_request_by_plan_uuid_handle(
    path: web::Path<String>,
    data: Data<PgPool>,
) -> impl Responder {
    tracing::info!(
        kind = "tcp",
        "get_price_information_request_list. uuid:: `{:?}`",
        &path
    );
    let response = get_price_information_request_by_plan_uuid(
        path.into_inner(),
        data.get_ref(),
    )
    .await;
    handle_result!(response, "get_price_information_request_list")
}

/// Route - /rest/technical_commercial_proposal/v1/get_price_information_request_by_plan_uuid_vec/
pub(crate) async fn get_price_information_request_by_plan_uuid_vec_handle(
    uuids: Json<Vec<Uuid>>,
    data: Data<PgPool>,
) -> impl Responder {
    tracing::info!(
        kind = "tcp",
        "get_price_information_request_by_plan_uuid_vec. uuids:: `{:?}`",
        &uuids
    );
    let uuids = uuids.into_inner();
    let response =
        get_price_information_request_by_plan_uuid_vec(uuids, data.get_ref()).await;

    handle_result!(response, "get_price_information_request_by_plan_uuid_vec")
}

/// Получение списка ТКП
/// Route - /rest/technical_commercial_proposal/v1/get_tkp_by_request_uuid/{uuid}/
pub(crate) async fn get_tkp_by_request_uuid_handle(
    path: web::Path<String>,
    data: Data<PgPool>,
) -> impl Responder {
    tracing::info!(kind = "tcp", "get_tkp_by_request_uuid. path:: `{:?}`", &path);

    let response = get_tkp_by_request_uuid(path.into_inner(), data.get_ref()).await;
    handle_result!(response, "get_tkp_by_request_uuid")
}

/// Получение списка ТКП
/// Route - /rest/technical_commercial_proposal/v1/get_tkp_by_request_uuid_vec/
pub(crate) async fn get_tkp_by_request_uuid_vec_handle(
    request_uuids: Json<Vec<Uuid>>,
    data: Data<PgPool>,
) -> impl Responder {
    tracing::info!(
        kind = "tcp",
        "get_tkp_by_request_uuid_vec. request_uuids:: `{:?}`",
        &request_uuids
    );
    let uuids = request_uuids.into_inner();

    let response = get_tkp_by_request_uuid_vec(uuids, data.get_ref()).await;
    handle_result!(response, "get_tkp_by_request_uuid_vec")
}

pub async fn update_proposal_handler(
    req: HttpRequest,
    Json(dto): Json<UpdateProposalReq>,
    user_id: Query<UserId>,
    db_pool: Data<PgPool>,
    monolith_service: Data<MonolithHttpService>,
) -> TcpHttpResponse<UpdateProposalResponseData> {
    tracing::info!(kind = "tcp", "update_proposal. dto: `{:?}`", &dto);
    let user_id = user_id.into_inner().user_id;
    let token = req
        .cookie(MONOLITH_TOKEN_COOKIE)
        .map(|token| token.value().to_owned())
        .unwrap_or_default();

    let (data, messages) =
        process_update_proposal(dto, token, user_id, &db_pool, &monolith_service)
            .await?;

    Ok(Json((data, messages).into()))
}

pub(crate) async fn pre_price_info_close_handle(
    user_id: Query<UserId>,
    Json(req): Json<ObjectIdentifierList>,
    db_pool: Data<PgPool>,
) -> TcpHttpResponse<PrePriceInfoCloseResponse> {
    let user_id = user_id.user_id;
    let item_list = req.item_list;
    tracing::info!(
        kind = "tcp",
        "pre_request/request_price_info_close. User id: {:?}, request: {:?}",
        user_id,
        item_list
    );
    process_pre_price_info_close(user_id, item_list, &db_pool)
        .await
        .map(Json)
        .map_err(AsezError::from)
}

pub(crate) async fn price_info_close_handle(
    user_id: Query<UserId>,
    Json(req): Json<PriceInfoCloseRequest>,
    db_pool: Data<PgPool>,
) -> TcpHttpResponse<PriceInfoCloseResponse> {
    let user_id = user_id.user_id;
    let item_list = req.item_list;
    tracing::info!(
        kind = "tcp",
        "action/request_price_info_close. User id: {:?}, request: {:?}",
        user_id,
        item_list
    );
    process_price_info_close(user_id, item_list, &db_pool)
        .await
        .map(Json)
        .map_err(AsezError::from)
}

/// "/action/proposal_apply_pricing_consider"
#[tracing::instrument(skip_all)]
pub(crate) async fn proposal_apply_pricing_handle(
    user_id: Query<UserId>,
    monolith_token: MonolithToken,
    Json(req): Json<ApplyPricingProposal>,
    monolith_service: Data<MonolithHttpService>,
    db_pool: Data<PgPool>,
) -> TcpHttpResponse<ApplyPricingProposalResponse> {
    let user_id = user_id.user_id;
    tracing::info!(
        kind = "tcp",
        "action/proposal_apply_pricing_consider. User ID = {:?}, req = {:?}",
        user_id,
        req
    );
    let monolith = monolith_service.into_inner();
    let token = monolith_token.into_inner();
    process_apply_proposal_pricing(user_id, token, req, &monolith, db_pool.as_ref())
        .await
        .map(Json)
        .map_err(AsezError::from)
}

/// Проверка ЗЦИ на наличие ошибок перед сохранением
/// route - "/check/request_price_info/"
pub(crate) async fn check_request_price_info_handle(
    Json(request): Json<UpdatePriceInformationRequest>,
) -> TcpHttpResponse<()> {
    tracing::info!(kind = "siem", "[SIEM] ЗЦИ: Проверка на наличие ощибок");
    let mut response = ApiResponse::<(), ()>::default();
    let messages = check_request_price_info(&request).await;
    response.messages = messages;
    Ok(web::Json(response))
}

/// Предзапрос данных вопрос/ответ для формирования модального окна
/// route - "/pre_request/organization_question./"
pub(crate) async fn pre_organization_question(
    user_id: Query<UserId>,
    monolith_token: MonolithToken,
    Json(req): Json<PreOrganizationQuestionReq>,
    monolith_service: Data<MonolithHttpService>,
    db_pool: Data<PgPool>,
) -> TcpHttpResponse<PreOrganizationQuestionResponseData> {
    tracing::info!(
        kind = "tcp",
        "pre_request/organization_question. User ID = {:?}, req = {:?}",
        user_id,
        req
    );
    let (res, messages) = process_pre_organization_question(
        req,
        user_id.user_id,
        monolith_token.into_inner(),
        &db_pool,
        &monolith_service,
    )
    .await?;

    Ok(Json(ApiResponse::default().with_data(res).with_messages(messages)))
}

/// действие завершение рассмотрения
/// route - "/action/request_price_info_complete/"
pub(crate) async fn price_info_complete_handle(
    user_id: Query<UserId>,
    Json(req): Json<ObjectIdentifierList>,
    db_pool: Data<PgPool>,
) -> TcpHttpResponse<PriceInfoCompleteResponse> {
    let user_id = user_id.user_id;
    let item_list = req.item_list;
    tracing::info!(
        kind = "tcp",
        "action/request_price_info_complete. User id: {:?}, request: {:?}",
        user_id,
        item_list
    );
    process_price_info_complete(user_id, item_list, &db_pool)
        .await
        .map(Json)
        .map_err(AsezError::from)
}

/// Действие проверка добавления партнёра.
/// route - "/сheck/add_partner/"
pub(crate) async fn check_add_partner_handle(
    user_id: Query<UserId>,
    Json(req): Json<CheckPartnerReq>,
    db_pool: Data<PgPool>,
) -> TcpHttpResponse<CheckAddPartnerResponse> {
    let user_id = user_id.user_id;
    tracing::info!(
        kind = "tcp",
        "сheck/add_partner/. User id: {:?}, request: {:?}",
        user_id,
        req
    );
    process_check_add_partner(user_id, req, &db_pool)
        .await
        .map(Json)
        .map_err(AsezError::from)
}

/// Действие проверка удаления партнёра.
/// route - "/сheck/delete_partner/"
pub(crate) async fn check_delete_partner_handle(
    user_id: Query<UserId>,
    Json(req): Json<CheckPartnerReq>,
    db_pool: Data<PgPool>,
) -> TcpHttpResponse<CheckDeletePartnerResponse> {
    let user_id = user_id.user_id;
    tracing::info!(
        kind = "tcp",
        "сheck/delete_partner/. User id: {:?}, request: {:?}",
        user_id,
        req
    );
    process_check_delete_partner(user_id, req, &db_pool)
        .await
        .map(Json)
        .map_err(AsezError::from)
}

/// Действие удаления ЗЦИ.
/// route - /rest/technical_commercial_proposal/v1/delete/request_price_info/
pub(crate) async fn delete_price_info_handle(
    user_id: Query<UserId>,
    Json(req): Json<ObjectIdentifierList>,
    db_pool: Data<PgPool>,
) -> TcpHttpResponse<DeletePriceInfoResponse> {
    let user_id = user_id.user_id;
    tracing::info!(
        kind = "tcp",
        "/delete/request_price_info/. User id: {:?}, request: {:?}",
        user_id,
        req
    );
    let req = req.item_list;
    process_delete_price_info(user_id, req, &db_pool)
        .await
        .map(Json)
        .map_err(AsezError::from)
}

/// Обновление ЗЦИ.
/// route - /rest/technical_commercial_proposal/v1/update/request_price_info/
pub(crate) async fn update_price_info_handle(
    user_id: Query<UserId>,
    monolith_token: MonolithToken,
    Json(req): Json<UpdatePriceInformationRequest>,
    monolith_service: Data<MonolithHttpService>,
    db_pool: Data<PgPool>,
) -> TcpHttpResponse<UpdatePriceInformationResponse> {
    let user_id = user_id.user_id;
    tracing::info!(
        kind = "tcp",
        "/update/request_price_info/. User id: {:?}, request: {:?}",
        user_id,
        req
    );
    process_update_price_info(
        user_id,
        monolith_token.into_inner(),
        req,
        &monolith_service,
        &db_pool,
    )
    .await
    .map(Json)
    .map_err(AsezError::from)
}

/// Создание или обновление записи Шаблонов заключений Экспертов АЦ
/// /rest/technical_commercial_proposal/v1/update/organizations/
pub(crate) async fn update_organizations_handler(
    Query(user_id): Query<UserId>,
    Json(req): Json<UpdateOrganizationsReq>,
    pg_pool: Data<PgPool>,
) -> impl Responder {
    tracing::info!(kind = "tcp", "update_organizations. req: `{:?}`", &req);
    let response = organizations_update(&pg_pool, user_id, req).await;
    handle_result!(response, "organizations_update")
}

/// Создание группы Предмета закупки АЦ
/// /rest/technical_commercial_proposal/v1/update/purchasing_subject_group/
pub(crate) async fn update_purchasing_subject_group_handler(
    Query(user_id): Query<UserId>,
    Json(req): Json<UpdatePurchasingSubjectGroupReq>,
    pg_pool: Data<PgPool>,
) -> impl Responder {
    tracing::info!(
        kind = "tcp",
        "update_purchasing_subject_group. req: `{:?}`",
        &req
    );
    let response = purchasing_subject_group_update(&pg_pool, user_id, req).await;
    handle_result!(response, "purchasing_subject_group_update")
}

/// Обновление Предметов Закупки
/// /rest/technical_commercial_proposal/v1/update/purchasing_subject/
pub(crate) async fn update_purchasing_subject_handler(
    Query(user_id): Query<UserId>,
    Json(req): Json<UpdatePurchasingSubjectReq>,
    pg_pool: Data<PgPool>,
) -> impl Responder {
    tracing::info!(kind = "tcp", "update_purchasing_subject. req: `{:?}`", &req);
    let response = purchasing_subject_update(&pg_pool, user_id, req).await;
    handle_result!(response, "purchasing_subject_update")
}

/// Действие по Подтверждению ТКП
/// route - "/action/proposal_approve/"
pub async fn proposal_approve_handler(
    req: HttpRequest,
    Json(dto): Json<ApproveProposalReq>,
    user_id: Query<UserId>,
    db_pool: Data<PgPool>,
    monolith_service: Data<MonolithHttpService>,
) -> TcpHttpResponse<ApproveProposalResponseData> {
    tracing::info!(kind = "tcp", "approve_proposal. dto: `{:?}`", &dto);
    let user_id = user_id.into_inner().user_id;
    let token = req
        .cookie(MONOLITH_TOKEN_COOKIE)
        .map(|token| token.value().to_owned())
        .unwrap_or_default();

    let (data, messages) =
        process_approve_proposal(user_id, token, dto, &monolith_service, &db_pool)
            .await?;

    Ok(Json((data, messages).into()))
}

/// Удаление организации из списка
/// /rest/technical_commercial_proposal/v1/action/organizations_remove
pub(crate) async fn organizations_remove_handler(
    Json(req): Json<ActionOrganizationsRequest>,
    pg_pool: Data<PgPool>,
) -> impl Responder {
    tracing::info!(kind = "tcp", "organizations_remove. request: `{:?}`", &req);
    let response = organizations_remove(pg_pool.get_ref(), req).await;
    handle_result!(response, "organizations_remove")
}

/// Удаление группы Предметов закупки
/// Route - /rest/technical_commercial_proposal/v1/action/purchasing_subject_group_remove
pub(crate) async fn purchasing_subject_group_remove_handler(
    Json(req): Json<ActionPurchasingSubjectGroupRequest>,
    pg_pool: Data<PgPool>,
) -> impl Responder {
    tracing::info!(
        kind = "tcp",
        "purchasing_subject_group_remove. request: `{:?}`",
        &req
    );
    let response = purchasing_subject_group_remove(pg_pool.get_ref(), req).await;
    handle_result!(response, "purchasing_subject_group_remove")
}

/// Удаление предмета закупки
/// Route - /rest/technical_commercial_proposal/v1/action/purchasing_subject_remove
pub(crate) async fn purchasing_subject_remove_handler(
    Json(req): Json<ActionPurchasingSubjectRequest>,
    pg_pool: Data<PgPool>,
) -> impl Responder {
    tracing::info!(
        kind = "tcp",
        "purchasing_subject_remove. request: `{:?}`",
        &req
    );
    let response = purchasing_subject_remove(pg_pool.get_ref(), req).await;
    handle_result!(response, "purchasing_subject_remove")
}

/// Действие публикации ЗЦИ
/// route - "/get/proposal_list_by_object_id/"
pub(crate) async fn get_proposals_by_object_id_handle(
    user_id: Query<UserId>,
    Json(req): Json<ObjectIdentifier>,
    db_pool: Data<PgPool>,
) -> TcpHttpResponse<GetProposalPriceDataByIdResponse> {
    let user_id = user_id.user_id;
    tracing::info!(
        kind = "tcp",
        "action/request_price_info_publication. User id: {:?}, request: {:?}",
        user_id,
        req
    );

    let (data, messages) =
        get_proposals_by_object_id(user_id, req, &db_pool).await?;

    Ok(Json((data, messages).into()))
}

// Получение данных ППЗ/ДС позиций ТКП
// route - "/get/proposal_items_for_pricing/"
pub(crate) async fn get_proposal_items_for_pricing_handle(
    Query(UserId { user_id }): Query<UserId>,
    Json(req): Json<ProposalItemsForPricingRequest>,
    db_pool: Data<PgPool>,
) -> TcpHttpResponse<ProposalItemsForPricingResponse> {
    tracing::info!(
        kind = "request",
        "get/proposal_items_for_pricing. User id: {}, request: {:?}",
        user_id,
        req
    );

    let res = get_proposal_items_for_pricing(req, &db_pool).await?;

    Ok(Json(res.into()))
}

/// Действие публикации ЗЦИ
/// route - "/action/request_price_info_publication/"
pub(crate) async fn price_info_publication_handle(
    user_id: Query<UserId>,
    monolith_token: MonolithToken,
    Json(req): Json<PriceInfoPublicationReq>,
    monolith_service: Data<MonolithHttpService>,
    db_pool: Data<PgPool>,
    integration_service: IntegrationService,
) -> TcpHttpResponse<PriceInfoPublicationResponse> {
    let user_id = user_id.user_id;
    tracing::info!(
        kind = "tcp",
        "action/request_price_info_publication. User id: {:?}, request: {:?}",
        user_id,
        req
    );

    let res = process_publication_price_info(
        user_id,
        monolith_token.into_inner(),
        req,
        &monolith_service,
        &db_pool,
        integration_service,
    )
    .await?;

    Ok(Json(res.into()))
}

/// Формирование электронной таблицы (Excel или P7) на основании ракурса, фильтров и сортировок.
/// route - /rest/technical_commercial_proposal/v1/export/table/
pub(crate) async fn export_table_handler(
    query: Query<UserId>,
    monolith_token: MonolithToken,
    Json(request): Json<UiExportTableReq<UiSection>>,
    pool: Data<PgPool>,
    print_doc: PrintDocService,
) -> actix_web::Result<HttpResponse> {
    let user_id = query.0.user_id;
    tracing::info!(
        kind = "tcp",
        "/export/table/. User id: {:?}, request: {:?}",
        user_id,
        request
    );

    let (export_response, messages) = process_export_table(
        request,
        user_id,
        monolith_token.into_inner(),
        &pool,
        &print_doc,
    )
    .await?;

    tracing::debug!(
        kind = "tcp",
        "export_table_handler:ExportResponse messages: {:?}",
        messages
    );

    let body = once(ok::<_, Error>(web::Bytes::from(
        decompress_bzip(export_response.byte_buf.as_slice()).unwrap_or(Vec::new()),
    )));

    Ok(HttpResponse::Ok()
        .append_header((
            "Content-Disposition",
            "attachment; filename=\"export_table.xlsx\"".to_string(),
        ))
        .content_type(ContentType::octet_stream())
        .streaming(body))
}

/// Получение данных от ЗЦИ/ТКП. Используется при открытии Модуля АЦ из UI
/// route - /get/get_technical_commercial_proposal/
pub(crate) async fn get_technical_commercial_proposal(
    user_id: Query<UserId>,
    Json(req): Json<ObjectIdentifierList>,
    db_pool: Data<PgPool>,
) -> TcpHttpResponse<GetTechnicalCommercialProposalResponse> {
    let user_id = user_id.user_id;
    let item_list = req.item_list;
    tracing::info!(
        kind = "tcp",
        "get/technical_commercial_proposal. User id: {:?}, request: {:?}",
        user_id,
        item_list
    );
    process_get_technical_commercial_proposal(user_id, item_list, &db_pool)
        .await
        .map(Json)
        .map_err(AsezError::from)
}
