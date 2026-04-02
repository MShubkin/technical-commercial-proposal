use asez2_shared_db::{
    db_item::{
        AdaptorableIter, AsezTimestamp, DbUpdateByFilter, DbUpsert, Filter,
        FilterTree, Select,
    },
    DbAdaptor, DbItem,
};
use shared_essential::{
    domain::tcp::{
        PartnerSubjectPurchased, PartnerSubjectPurchasedRep,
        RequestSubjectPurchased, RequestSubjectPurchasedRep,
    },
    presentation::dto::{
        general::UserId,
        response_request::{
            ApiResponse, ApiResponseDataWrapper, Message, Messages,
        },
        technical_commercial_proposal::TcpResult,
    },
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::presentation::dto::{
    ActionOrganizationsRequest, ActionPurchasingSubjectGroupRequest,
    ActionPurchasingSubjectRequest, ActionSubjectItem, GetOrganizationsResponse,
    GetPurchasingSubjectsResponse, UpdateOrganizationsReq,
    UpdateOrganizationsResponse, UpdatePurchasingSubjectGroupReq,
    UpdatePurchasingSubjectReq,
};

pub(crate) const REQ_SUBJ_RET_FIELDS: &[&str] = &[
    RequestSubjectPurchased::uuid,
    RequestSubjectPurchased::id,
    RequestSubjectPurchased::contract_subject_purchase_text,
    RequestSubjectPurchased::is_removed,
];

pub(crate) const P_SUBJ_RET_FIELDS: &[&str] = &[
    PartnerSubjectPurchased::uuid,
    PartnerSubjectPurchased::supplier_id,
    PartnerSubjectPurchased::is_removed,
];

/// Получение предметов закупки ЗЦИ по uuid Организации Предметов закупки
/// Route - /rest/technical_commercial_proposal/v1/get/purchasing_subject_by_group_uuid/{uuid}/
pub(crate) async fn get_purchasing_subject_by_group_uuid(
    pool: &PgPool,
    parent_uuid: Uuid,
) -> TcpResult<ApiResponse<GetPurchasingSubjectsResponse, ()>> {
    let item_list = RequestSubjectPurchasedRep::select(
        &Select::with_fields(REQ_SUBJ_RET_FIELDS)
            .eq(RequestSubjectPurchased::parent_uuid, parent_uuid)
            .eq(RequestSubjectPurchased::is_removed, false),
        pool,
    )
    .await?;
    Ok((GetPurchasingSubjectsResponse { item_list }, Messages::default()).into())
}

/// Получение актуального списка организаций по идентификатору "Предмета закупки"
/// Route - /rest/technical_commercial_proposal/v1/get/organizations/{uuid_subject}
pub(crate) async fn get_organizations(
    pool: &PgPool,
    uuid_subject: Uuid,
) -> TcpResult<ApiResponse<GetOrganizationsResponse, ()>> {
    let partner_subjects = PartnerSubjectPurchased::select(
        &Select::full::<PartnerSubjectPurchased>()
            .eq(PartnerSubjectPurchased::uuid_subject, uuid_subject)
            .eq(PartnerSubjectPurchased::is_removed, false),
        pool,
    )
    .await?;

    let item_list = partner_subjects
        .into_iter()
        .adaptors_with_fields(P_SUBJ_RET_FIELDS)
        .collect();

    Ok((GetOrganizationsResponse { item_list }, Messages::default()).into())
}

/// Получение актуальных записей справочника "Группа Предметов закупки"
/// Route - /rest/technical_commercial_proposal/v1/get/purchasing_subject_group
pub(crate) async fn get_purchasing_subject_group(
    pool: &PgPool,
    user_id: i32,
) -> TcpResult<ApiResponse<GetPurchasingSubjectsResponse, ()>> {
    let item_list = RequestSubjectPurchasedRep::select(
        &Select::with_fields(REQ_SUBJ_RET_FIELDS)
            .eq(RequestSubjectPurchased::created_by, user_id)
            .eq(RequestSubjectPurchased::hierarchy_id, 1)
            .eq(RequestSubjectPurchased::is_removed, false),
        pool,
    )
    .await?;

    Ok((GetPurchasingSubjectsResponse { item_list }, Messages::default()).into())
}

/// Удаление Организации из списка
/// /rest/technical_commercial_proposal/v1/action/organizations_remove
pub(crate) async fn organizations_remove(
    pool: &PgPool,
    ActionOrganizationsRequest {
        item: ActionSubjectItem { uuid, .. },
    }: ActionOrganizationsRequest,
) -> TcpResult<ApiResponse<(), ()>> {
    let removed_value = PartnerSubjectPurchased {
        uuid,
        is_removed: true,
        ..Default::default()
    };

    match removed_value
        .update(Some(&[PartnerSubjectPurchased::is_removed]), pool)
        .await?
    {
        // No rows with such uuid in DB
        0 => Ok(((), Message::error("Ошибка при удалении Организации")).into()),
        _ => Ok(((), Message::success("Организация удалена")).into()),
    }
}

/// Удаление Группы Предметов закупки
/// Route - /rest/technical_commercial_proposal/v1/action/purchasing_subject_group_remove
pub(crate) async fn purchasing_subject_group_remove(
    pool: &PgPool,
    ActionPurchasingSubjectGroupRequest {
        item: ActionSubjectItem { uuid, .. },
    }: ActionPurchasingSubjectGroupRequest,
) -> TcpResult<ApiResponse<(), ()>> {
    let mut tx = pool.begin().await?;
    let req_subjects = RequestSubjectPurchased {
        is_removed: true,
        ..Default::default()
    }
    .update_by_filter_returning(
        &[RequestSubjectPurchased::is_removed],
        &FilterTree::from(Filter::eq(
            RequestSubjectPurchased::hierarchy_uuid,
            uuid,
        )),
        Some(&[RequestSubjectPurchased::uuid]),
        &mut tx,
    )
    .await?;

    let subject_uuids = req_subjects.iter().map(|x| x.uuid).collect::<Vec<_>>();
    if subject_uuids.is_empty() {
        return Ok((
            (),
            Message::error("Ошибка базы данных: Не найденa желаемая Группa Предметов закупки при попытке удаления"),
        )
            .into());
    }

    PartnerSubjectPurchased {
        is_removed: true,
        ..Default::default()
    }
    .update_by_filter(
        &[PartnerSubjectPurchased::is_removed],
        &FilterTree::from(Filter::in_any(
            PartnerSubjectPurchased::uuid_subject,
            subject_uuids,
        )),
        &mut tx,
    )
    .await?;
    tx.commit().await?;

    Ok(((), Message::success("Группа Предметов закупки удалена")).into())
}

/// Удаление Предмета закупки
/// Route - /rest/technical_commercial_proposal/v1/action/purchasing_subject_remove
pub(crate) async fn purchasing_subject_remove(
    pool: &PgPool,
    ActionPurchasingSubjectRequest {
        item: ActionSubjectItem { uuid, .. },
    }: ActionPurchasingSubjectRequest,
) -> TcpResult<ApiResponse<(), ()>> {
    let mut tx = pool.begin().await?;
    let request_subject_changed = (RequestSubjectPurchased {
        uuid,
        is_removed: true,
        ..Default::default()
    })
    .update_returning(
        Some(&[RequestSubjectPurchased::is_removed]),
        Some(&[RequestSubjectPurchased::parent_uuid]),
        &mut tx,
    )
    .await?;

    if request_subject_changed.parent_uuid.is_none() {
        return Ok((
            (),
            Message::error(
                "Выбрана группа Предметов закупки вместо Предмета закупки",
            ),
        )
            .into());
    }

    PartnerSubjectPurchased {
        is_removed: true,
        ..Default::default()
    }
    .update_by_filter(
        &[PartnerSubjectPurchased::is_removed],
        &FilterTree::from(Filter::eq(PartnerSubjectPurchased::uuid_subject, uuid)),
        &mut tx,
    )
    .await?;

    tx.commit().await?;

    Ok(((), Message::success("Предмет закупки удален")).into())
}

/// Создание записи Шаблонов заключений Экспертов АЦ
/// /rest/technical_commercial_proposal/v1/update/organizations/
pub(crate) async fn organizations_update(
    pool: &PgPool,
    UserId { user_id }: UserId,
    req: UpdateOrganizationsReq,
) -> TcpResult<ApiResponse<UpdateOrganizationsResponse, ()>> {
    const RESPONSE_FIELDS: &[&str] = &[
        PartnerSubjectPurchased::uuid,
        PartnerSubjectPurchased::uuid_subject,
        PartnerSubjectPurchased::supplier_id,
    ];

    if req.uuid_subject.is_none() || req.supplier_id.is_none() {
        return Ok(
            Message::error("Обязательные поля не присутствуют в запросе").into()
        );
    }

    let mut req = req.into_item()?;

    let now = AsezTimestamp::now();
    req.uuid = Uuid::new_v4();
    req.is_removed = false;
    req.created_by = user_id;
    req.created_at = now;
    req.changed_by = user_id;
    req.changed_at = now;

    let new = req.insert_returning(pool).await?;

    Ok((
        ApiResponseDataWrapper::from(PartnerSubjectPurchasedRep::from_item(
            new,
            Some(RESPONSE_FIELDS),
        )),
        Message::success("Организация добавлена"),
    )
        .into())
}

/// Создание группы Предмета закупки АЦ
/// /rest/technical_commercial_proposal/v1/update/purchasing_subject_group/
pub(crate) async fn purchasing_subject_group_update(
    pool: &PgPool,
    user_id: UserId,
    UpdatePurchasingSubjectGroupReq { uuid, text }: UpdatePurchasingSubjectGroupReq,
) -> TcpResult<ApiResponse<(), ()>> {
    upsert_purchasing_subject(pool, user_id, text, uuid, None).await
}

/// Обновление Предметов Закупки
/// /rest/technical_commercial_proposal/v1/update/purchasing_subject/
pub(crate) async fn purchasing_subject_update(
    pool: &PgPool,
    user_id: UserId,
    UpdatePurchasingSubjectReq {
        uuid,
        parent_uuid,
        text,
    }: UpdatePurchasingSubjectReq,
) -> TcpResult<ApiResponse<(), ()>> {
    upsert_purchasing_subject(pool, user_id, text, uuid, Some(parent_uuid)).await
}

async fn upsert_purchasing_subject(
    pool: &PgPool,
    UserId { user_id }: UserId,
    text: String,
    uuid: Option<Uuid>,
    parent_uuid: Option<Uuid>,
) -> TcpResult<ApiResponse<(), ()>> {
    const UPDATE_FIELDS: &[&str] = &[
        RequestSubjectPurchased::is_removed,
        RequestSubjectPurchased::hierarchy_id,
        RequestSubjectPurchased::hierarchy_uuid,
        RequestSubjectPurchased::contract_subject_purchase_text,
        RequestSubjectPurchased::changed_at,
        RequestSubjectPurchased::changed_by,
    ];

    let mut tx = pool.begin().await?;

    let now = AsezTimestamp::now();
    let uuid = uuid.unwrap_or(Uuid::new_v4());
    let hierarchy_id = if parent_uuid.is_none() { 1 } else { 2 };

    let record = RequestSubjectPurchased {
        uuid,
        parent_uuid,
        hierarchy_id,
        hierarchy_uuid: parent_uuid.unwrap_or(uuid),
        is_removed: false,
        contract_subject_purchase_text: text,

        changed_by: user_id,
        changed_at: now,

        // Only for insert
        created_by: user_id,
        created_at: now,

        ..Default::default()
    };

    // NB: may be used in RecordCtx, but `field_history` table not present in TCP
    let updated = RequestSubjectPurchased::upsert_returning(
        &mut [record],
        Some(UPDATE_FIELDS),
        &mut tx,
    )
    .await?;

    let (new_text, changed_text, err_text) = if parent_uuid.is_none() {
        (
            "Новая группа Предметов закупки создана",
            "Группа Предметов закупки изменена",
            "Ошибка при создании новой группы Предметов закупки",
        )
    } else {
        (
            "Новый Предмет закупки создан",
            "Предмет закупки изменен",
            "Ошибка при создании нового Предмета закупки",
        )
    };

    let Some(updated) = updated.first() else {
        // Must not occured, just in case
        return Ok(Message::error(err_text.to_string()).into());
    };

    tx.commit().await?;

    Ok(if updated.created_at == updated.changed_at {
        Message::success(new_text.to_string()).into()
    } else {
        Message::success(changed_text.to_string()).into()
    })
}
