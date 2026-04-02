//! Модуль отвечает за описание контрактов обращения к сервису
use actix_web::web::Json;
use monolith_service::dto::attachment::Attachment as MonolithAttachment;
use serde::{Deserialize, Serialize};
use shared_essential::{
    domain::{
        maths::{CurrencyValue, VatId},
        tables::tcp::{
            PriceInformationRequestStatus, ProposalHeader, ProposalHeaderRep,
            ProposalItem, ProposalItemRep, RequestHeader, RequestHeaderRep,
            RequestItem, RequestItemRep, RequestPartner, RequestPartnerRep,
            TCPReviewResult,
        },
        tcp::{
            OrganizationQuestionRep, PartnerSubjectPurchasedRep,
            RequestSubjectPurchasedRep,
        },
        Section,
    },
    presentation::dto::{
        general::{FeWrapper, Metadata, ObjectIdentifier, UiSelect},
        response_request::{
            ApiResponse, ApiResponseData, ApiResponseDataWrapper, PaginatedData,
            ParamItem,
        },
        technical_commercial_proposal::UiSection,
        AsezResult,
    },
};
use uuid::Uuid;

/// Тайп альяс для HTTP ответа пользователю
pub type TcpHttpResponse<T> = AsezResult<Json<ApiResponse<T, ()>>>;

pub type GetRequestPriceListReq = GeneralGetRequestPriceListReq<UiSection>;

/// Запрос на получение планов
#[derive(Deserialize, Serialize, Debug)]
pub struct GeneralGetRequestPriceListReq<T: Into<Section>> {
    /// UI секция пользователя
    pub section_id: T,
    #[serde(flatten)]
    /// Селект для запроса определенных полей
    pub select: UiSelect,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetRequestPriceInfoDetail {
    pub id: i64,
}

/// Детальная информация ЗЦИ
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub(crate) struct PriceInformationDetail {
    /// Заголовок ЗЦИ
    #[serde(flatten)]
    pub(crate) request_header: RequestHeaderRep,
    /// Позиции ЗЦИ
    pub(crate) item_list: Vec<FeWrapper<RequestItemRep>>,
    /// Поставщики
    pub(crate) partner_list: Vec<FeWrapper<RequestPartnerRep>>,
}

impl ApiResponseData for PriceInformationDetail {}

/// Стуктура ЗЦИ
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub(crate) struct PriceInformationRequest {
    /// Заголовок ЗЦИ
    pub(crate) header: RequestHeader,
    /// Позиции ЗЦИ
    pub(crate) items: Vec<RequestItem>,
    /// Поставщики
    pub(crate) suppliers: Option<Vec<RequestPartner>>,
}

/// Структура ТКП
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
pub(crate) struct TechnicalCommercialProposal {
    /// Заколовок
    pub(crate) header: ProposalHeader,
    /// Позиции
    pub(crate) items: Vec<ProposalItem>,
}
/// Запрос на сохранение (обновление) ТКП
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UpdateProposalReq {
    pub supplier_id: Option<i32>,
    #[serde(flatten)]
    pub header: ProposalHeaderRep,
    pub item_list: Vec<ProposalItemRep>,
    pub attachment_list: Vec<MonolithAttachment>,
}

/// Ответ на [сохранение-обновление ТКП](UpdateProposalReq)
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct UpdateProposalResponseData {
    #[serde(flatten)]
    pub header: ProposalHeaderRep,
    pub item_list: Vec<UpdateProposalResponseItem>,
}
/// Элемент [`UpdateProposalResponseData::item_list`]
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct UpdateProposalResponseItem {
    /// Позиция ТКП
    #[serde(flatten)]
    pub proposal_item: ProposalItemRep,
    /// Валюта ЗЦИ
    pub vat_id: VatId,
    /// Цена ЗЦИ
    pub price: CurrencyValue,
}
impl ApiResponseData for UpdateProposalResponseData {}

/// Сущность которая существует потому что кому то очень надо склеить две
/// сущности. (по ручке "/get/proposal_detail")
#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub(crate) struct GetProposalItem {
    #[serde(flatten)]
    pub(crate) proposal_item: ProposalItemRep,
    /// From request item
    pub(crate) price: CurrencyValue,
    /// From request item
    pub(crate) vat_id: VatId,
    /// Метадата для фронтенда
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _meta: Option<Metadata>,
}

/// Ответ (дата) по ручке "/get/proposal_detail"
#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub(crate) struct GetProposalDataResponse {
    #[serde(flatten)]
    pub header: ProposalHeaderRep,
    pub created_by: i32,
    /// Номер ЗЦИ
    pub request_id: i64,
    /// request_partner.id
    pub supplier_id: i32,
    pub item_list: Vec<GetProposalItem>,
}
impl ApiResponseData for GetProposalDataResponse {}

/// Поле item внутри запросов Организаций, а так же Предметов закупки и их Групп
#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub(crate) struct ActionSubjectItem {
    /// Id записи (не используется)
    pub id: i32,
    /// Уникальный идентификатор записи
    pub uuid: Uuid,
}

/// Запрос на удаление Организации
#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub(crate) struct ActionOrganizationsRequest {
    pub(crate) item: ActionSubjectItem,
}

/// Запрос на удаление Предмета закупки
#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub(crate) struct ActionPurchasingSubjectRequest {
    pub(crate) item: ActionSubjectItem,
}

/// Запрос на удаление Группы Предметов закупки, а так же самих прежметов
#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub(crate) struct ActionPurchasingSubjectGroupRequest {
    pub(crate) item: ActionSubjectItem,
}

// Ответ (дата) по ручке "/get/purchasing_subjects_by_group_uuid"
#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub(crate) struct GetPurchasingSubjectsResponse {
    pub(crate) item_list: Vec<RequestSubjectPurchasedRep>,
}
impl ApiResponseData for GetPurchasingSubjectsResponse {}

// Ответ (дата) по ручке "/get/organizations"
#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub(crate) struct GetOrganizationsResponse {
    pub(crate) item_list: Vec<PartnerSubjectPurchasedRep>,
}
impl ApiResponseData for GetOrganizationsResponse {}

/// Объект, ответа (дата) по ручке "/pre_request/request_price_info_close"
#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub(crate) struct PrePriceInfoCloseItem {
    #[serde(flatten)]
    pub(crate) header: RequestHeaderRep,
    pub(crate) supplier_list: Vec<i32>,
}

/// Ответ (дата) по ручке "/pre_request/request_price_info_close"
#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub(crate) struct PrePriceInfoCloseResponse {
    pub(crate) item_list: Vec<PrePriceInfoCloseItem>,
}
impl ApiResponseData for PrePriceInfoCloseResponse {}

/// Запросы: 1. Обновление ЗЦИ 2.Проверка на наличие ошибок перед сохранением ЗЦИ
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub(crate) struct UpdatePriceInformationRequest {
    #[serde(flatten)]
    pub header: RequestHeaderRep,
    pub item_list: Vec<RequestItemRep>,
    pub partner_list: Vec<RequestPartnerRep>,
    pub attachment_list: Vec<MonolithAttachment>,
}

/// Запрос /rest/technical_commercial_proposal/v1/update/organizations/
pub(crate) type UpdateOrganizationsReq = PartnerSubjectPurchasedRep;

/// Ответ /rest/technical_commercial_proposal/v1/update/organizations/
pub(crate) type UpdateOrganizationsResponse =
    ApiResponseDataWrapper<PartnerSubjectPurchasedRep>;

/// Запрос /rest/technical_commercial_proposal/v1/update/purchasing_subject_group/
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub(crate) struct UpdatePurchasingSubjectGroupReq {
    pub uuid: Option<Uuid>,
    pub text: String,
}

/// Запрос /rest/technical_commercial_proposal/v1/update/purchasing_subject/
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub(crate) struct UpdatePurchasingSubjectReq {
    pub uuid: Option<Uuid>,
    pub parent_uuid: Uuid,
    pub text: String,
}

/// Входное ДТО для хттп ручке по "/action/proposal_apply_pricing_consider/"
#[derive(Deserialize, Debug)]
pub(crate) struct ApplyPricingProposal {
    pub(crate) is_apply_pricing_consider: Option<bool>,
    pub(crate) item_list: Vec<ObjectIdentifier>,
}

/// Возврат из хттп ручке по "/action/proposal_apply_pricing_consider/"
pub(crate) type ApplyPricingProposalResponse = PaginatedData<ProposalHeaderRep>;
/// Возврат для - /rest/technical_commercial_proposal/v1/get/request_price_info_list/
pub(crate) type GetRequestPriceInfoListResponse =
    PaginatedData<FeWrapper<RequestHeaderRep>>;

#[derive(Deserialize, Debug)]
pub struct PreOrganizationQuestionReq {
    pub item_list: Vec<OrganizationQuestionItemReq>,
}
#[derive(Deserialize, Debug)]
pub struct OrganizationQuestionItemReq {
    pub supplier_id: i32,
    pub request_uuid: Uuid,
}
#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct PreOrganizationQuestionResponseData {
    pub item_list: Vec<OrganizationQuestionResponseItem>,
}
impl ApiResponseData for PreOrganizationQuestionResponseData {}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct OrganizationQuestionResponseItem {
    #[serde(flatten)]
    pub organization_question: OrganizationQuestionRep,
    pub attachment_list: Vec<MonolithAttachment>,
}

/// Сущность которую принимает "/action/request_price_info_close"
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RequestCloseItem {
    #[serde(flatten)]
    pub(crate) identifier: ObjectIdentifier,
    pub(crate) reason_closing: String,
}

/// Сущность ответа (дата) по ручке "/pre_request/request_price_info_close"
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PriceInfoCloseRequest {
    pub(crate) item_list: Vec<RequestCloseItem>,
}

/// Ответ по ручке "/pre_request/request_price_info_close"
pub(crate) type PriceInfoCloseResponse = PaginatedData<RequestHeaderRep>;

/// Сущность которую отдаёт "/action/request_price_info_complete/"
/// ТОДО: Может быть более универсальная.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct PriceInfoCompleteItem {
    #[serde(flatten)]
    pub(crate) identifier: ObjectIdentifier,
    pub(crate) status_id: PriceInformationRequestStatus,
}

/// Ответ по ручке - "/action/request_price_info_complete/"
pub(crate) type PriceInfoCompleteResponse = PaginatedData<PriceInfoCompleteItem>;

/// Потому что нельзя просто прислать масиф чисел. Надо для каждого писать "supply_id".
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SupplierId {
    pub(crate) supplier_id: i32,
}

/// Запрос по ручке - "/сheck/add_partner/"
/// Запрос по ручке - "/сheck/delete_partner/"
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CheckPartnerReq {
    #[allow(dead_code)]
    pub(crate) id: i64,
    pub(crate) uuid: Uuid,
    pub(crate) item_list: Vec<SupplierId>,
}

/// Сущность которую принимает "/сheck/add_partner/"
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Serialize)]
pub(crate) struct CheckPartnerItem {
    pub(crate) supplier_id: i32,
    pub(crate) is_allowed: bool,
}

/// Ответ по ручке - "/сheck/add_partner/"
pub(crate) type CheckAddPartnerResponse = PaginatedData<CheckPartnerItem>;

/// Ответ по ручке - "/сheck/delete_partner/"
pub(crate) type CheckDeletePartnerResponse = PaginatedData<CheckPartnerItem>;

/// Ответ по пучке - "/delete/request_price_info/"
pub(crate) type DeletePriceInfoResponse = PaginatedData<PriceInfoCompleteItem>;

/// Ответ по - "/update/request_price_info/"
pub(crate) type UpdatePriceInformationResponse = PriceInformationDetail;

/// Запрос на подтверждение ТКП
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ApproveProposalReq {
    pub item_list: Vec<ObjectIdentifier>,
}
/// Ответ на [подтверждение ТКП](ApproveProposalReq)
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct ApproveProposalResponseData {
    pub item_list: Vec<ProposalHeaderRep>,
}
impl ApiResponseData for ApproveProposalResponseData {}

/// Запрос для последующей публикации на ЭТП ГПБ
pub(crate) type PriceInfoPublicationReq = ObjectIdentifier;

/// Ответ по ручке - "/action/request_price_info_publication/"
#[derive(Deserialize, Serialize, Debug, Default)]
pub(crate) struct PriceInfoPublicationResponse {
    pub item_list: Vec<RequestHeaderRep>,
}
impl ApiResponseData for PriceInfoPublicationResponse {}

impl AsRef<CheckPartnerItem> for CheckPartnerItem {
    fn as_ref(&self) -> &Self {
        self
    }
}
impl From<&CheckPartnerItem> for ParamItem {
    fn from(value: &CheckPartnerItem) -> Self {
        ParamItem {
            id: value.supplier_id.to_string(),
            ..Default::default()
        }
    }
}

/// Внутренний объект возврата по "/get/proposal_list_by_object_id/"
#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
pub(crate) struct ProposalPricingItem {
    pub(crate) uuid: Uuid,
    pub(crate) id: i64,
    pub(crate) supplier_id: i32,
    pub(crate) supplier_vat_id: i32,
    pub(crate) result_id: TCPReviewResult,
    #[serde(rename = "supplier_sum_excluded_vat_total")]
    pub(crate) sum_excluded_vat: CurrencyValue,
    #[serde(rename = "supplier_sum_included_vat_total")]
    pub(crate) sum_included_vat: CurrencyValue,
}

/// Запрос по "/get/proposal_items_for_pricing"
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub(crate) struct ProposalItemsForPricingRequest {
    /// Uuid ТКП
    pub uuid: Uuid,
    /// Идентификатор ТКП
    pub id: i64,
}

/// Предмет ответа на "/get/proposal_items_for_pricing"
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub(crate) struct ProposalItemForPricing {
    pub plan_item_uuid: Uuid,
    pub proposal_price: CurrencyValue,
}

/// Ответ на "/get/proposal_items_for_pricing"
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub(crate) struct ProposalItemsForPricingResponse {
    pub item_list: Vec<ProposalItemForPricing>,
}

impl ApiResponseData for ProposalItemsForPricingResponse {}

/// Объект возврата по "/get/proposal_list_by_object_id/"
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub(crate) struct GetProposalPriceDataByIdResponse {
    pub(crate) item_list: Vec<ProposalPricingItem>,
}
impl ApiResponseData for GetProposalPriceDataByIdResponse {}

#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub(crate) struct GetTechnicalCommercialProposalItemResponse {
    #[serde(flatten)]
    pub(crate) proposal_header: ProposalHeaderRep,
    #[serde(flatten)]
    pub(crate) partner: RequestPartnerRep,
    #[serde(flatten)]
    pub(crate) request_header: RequestHeaderRep,
    pub(crate) position_list: Vec<GetTechnicalCommercialProposalPosition>,
}
/// Объект возврата по "/get/technical_commercial_proposal/"
#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub(crate) struct GetTechnicalCommercialProposalResponse {
    pub item_list: Vec<GetTechnicalCommercialProposalItemResponse>,
}

#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub(crate) struct GetTechnicalCommercialProposalPosition {
    #[serde(flatten)]
    pub proposal_item: ProposalItemRep,
    #[serde(flatten)]
    pub request_item: RequestItemRep,
}

impl ApiResponseData for GetTechnicalCommercialProposalResponse {}
