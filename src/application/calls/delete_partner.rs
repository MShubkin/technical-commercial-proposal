use crate::presentation::dto::{
    CheckDeletePartnerResponse, CheckPartnerItem, CheckPartnerReq, SupplierId,
};
use ahash::AHashSet;
use itertools::Itertools;

use asez2_shared_db::db_item::Select;
use shared_essential::domain::tcp::{
    JoinedRequestHeaderRequestPartner, JoinedRequestPartnerProposalHeader,
    PartnerWithProposalsSelector, PriceInformationRequestStatus, ProposalHeader,
    RequestHeader, RequestPartner, RequestWithPartnersSelector, TcpGeneralStatus,
};
use shared_essential::presentation::dto::{
    response_request::ApiResponse, technical_commercial_proposal::TcpResult,
};

use asez2_shared_db::db_item::joined::JoinTo;
use asez2_shared_db::DbItem;
use shared_essential::presentation::dto::response_request::{
    Message, Messages, PaginatedData,
};
use sqlx::PgPool;
use uuid::Uuid;

pub(crate) async fn process_check_delete_partner(
    _user_id: i32,
    req: CheckPartnerReq,
    pool: &PgPool,
) -> TcpResult<ApiResponse<CheckDeletePartnerResponse, ()>> {
    let mut messages = Messages::default();
    let mut response_items = Vec::new();

    let request_supplier_ids: AHashSet<i32> =
        req.item_list.iter().map(|item| item.supplier_id).collect();

    let requests = get_request_with_partners(&req, pool).await?;

    let (mut statuses_90_150, mut statuses_70_100): (Vec<_>, Vec<_>) =
        requests.into_iter().partition(|x| {
            [
                PriceInformationRequestStatus::AcceptingIncomingTCPs,
                PriceInformationRequestStatus::ErrorPublishingChanges,
            ]
            .contains(&x.header.status_id)
        });

    let (is_public, is_not_public): (Vec<_>, Vec<_>) = if let Some(request) =
        statuses_70_100.pop()
    {
        // Если статус ЗЦИ 70 или 100 - проставляем признак is_removed и отправляем на фронт сообщение об успешном удалении всех организацийй из запроса
        let request_partners = request
            .suppliers
            .iter()
            .filter(|item| request_supplier_ids.contains(&item.supplier_id))
            .cloned()
            .map(|mut partner| {
                partner.is_removed = true;
                partner
            })
            .collect::<Vec<_>>();
        let mut tx = pool.begin().await?;
        RequestPartner::update_vec(
            &request_partners,
            Some(&[RequestPartner::is_removed]),
            &mut tx,
        )
        .await?;
        tx.commit().await?;

        response_items = convert_suppliers_to_response_items(&req.item_list, true);
        messages.add_prepared_message(DeletePartnerMessage::is_deleted_success(
            &response_items,
        ));
        return Ok((PaginatedData::from(response_items), messages).into());
    } else if let Some(request) = statuses_90_150.pop() {
        // Если статус ЗЦИ 90 или 150 - выполняем проверки
        request.suppliers.into_iter().partition(|x| x.is_public)
    } else {
        // Операция удаления доступна только на статусах ЗЦИ: 70, 90, 100, 150
        // На всякий случай делаем проверку:
        // Если статус ЗЦИ отличается от значений: 70, 90, 100, 150  - выдаём сообщение о невозможности выполнения операции
        response_items = convert_suppliers_to_response_items(&req.item_list, false);
        messages.add_prepared_message(
            DeletePartnerMessage::is_deletion_prohibited(&response_items),
        );
        return Ok((PaginatedData::from(response_items), messages).into());
    };

    // Отдельно проверяем для статусов ЗЦИ 90, 150, какие организации из запроса были добавлены только на фронте
    // Для них удаление разрешено без проверок
    let request_suppliers: Vec<_> = is_public
        .iter()
        .chain(&is_not_public)
        .map(|item| item.supplier_id)
        .collect();
    let front_suppliers: Vec<_> = req
        .item_list
        .iter()
        .filter(|x| !request_suppliers.contains(&x.supplier_id))
        .cloned()
        .collect();

    if !front_suppliers.is_empty() {
        let front_check_items =
            convert_suppliers_to_response_items(&front_suppliers, true);
        messages.add_prepared_message(DeletePartnerMessage::is_deleted_success(
            &front_check_items,
        ));
        response_items.extend(front_check_items);
    }

    if !is_public.is_empty() {
        let public_items =
            convert_request_partners_to_response_items(&is_public, false);
        messages.add_prepared_message(
            DeletePartnerMessage::is_published_etp_error(&public_items),
        );
        response_items.extend(public_items);
    }

    if !is_not_public.is_empty() {
        let partners_with_proposals =
            get_partners_with_proposals(req.uuid, &is_not_public, pool).await?;
        // Сheck availability tcp with status Received(20)
        let partners_with_received_proposals = partners_with_proposals
            .iter()
            .filter(|x| {
                x.proposals
                    .iter()
                    .any(|item| item.status_id == TcpGeneralStatus::Received)
            })
            .collect_vec();

        if partners_with_received_proposals.is_empty() {
            let partners: Vec<_> = partners_with_proposals
                .iter()
                .map(|item| &item.partner)
                .cloned()
                .map(|mut partner| {
                    partner.is_removed = true;
                    partner
                })
                .collect();
            let proposals: Vec<_> = partners_with_proposals
                .into_iter()
                .flat_map(|x| x.proposals)
                .map(|mut proposal| {
                    proposal.status_id = TcpGeneralStatus::Deleted;
                    proposal
                })
                .collect();

            let mut tx = pool.begin().await?;

            RequestPartner::update_vec(
                &partners,
                Some(&[RequestPartner::is_removed]),
                &mut tx,
            )
            .await?;

            ProposalHeader::update_vec(
                &proposals,
                Some(&[ProposalHeader::status_id]),
                &mut tx,
            )
            .await?;

            tx.commit().await?;

            let items = convert_request_partners_to_response_items(&partners, true);

            messages.add_prepared_message(
                DeletePartnerMessage::is_deleted_success(&items),
            );
            response_items.extend(items);
        } else {
            let items = partners_with_received_proposals
                .iter()
                .map(|item| CheckPartnerItem {
                    supplier_id: item.partner.supplier_id,
                    is_allowed: false,
                })
                .collect_vec();
            messages.add_prepared_message(
                DeletePartnerMessage::is_tcp_exist_error(&items),
            );
            response_items.extend(items);
        }
    }

    let paginated = PaginatedData::from(response_items);
    Ok((paginated, messages).into())
}

async fn get_request_with_partners(
    req: &CheckPartnerReq,
    pool: &PgPool,
) -> TcpResult<Vec<JoinedRequestHeaderRequestPartner>> {
    let supplier_ids = req.item_list.iter().map(|x| x.supplier_id);
    let request_select = Select::full::<RequestHeader>()
        .eq(RequestHeader::uuid, req.uuid)
        .in_any(
            RequestHeader::status_id,
            [
                PriceInformationRequestStatus::AcceptingIncomingTCPs,
                PriceInformationRequestStatus::ErrorPublishingChanges,
                PriceInformationRequestStatus::TcpProject,
                PriceInformationRequestStatus::TransferToEtpError,
            ],
        );
    let partners_select = Select::full::<RequestPartner>()
        .in_any(RequestPartner::supplier_id, supplier_ids)
        .eq(RequestPartner::is_removed, false);
    let result = RequestWithPartnersSelector::new(request_select)
        .set_suppliers(RequestPartner::join_default().selecting(partners_select))
        .get(pool)
        .await?;
    Ok(result)
}

async fn get_partners_with_proposals(
    request_uuid: Uuid,
    partners: &[RequestPartner],
    pool: &PgPool,
) -> TcpResult<Vec<JoinedRequestPartnerProposalHeader>> {
    let supplier_ids = partners.iter().map(|x| x.supplier_id);
    let request_partner_select = Select::full::<RequestPartner>()
        .in_any(RequestPartner::supplier_id, supplier_ids)
        .eq(RequestPartner::request_uuid, request_uuid)
        .eq(RequestPartner::is_removed, false);
    let partners = PartnerWithProposalsSelector::new(request_partner_select)
        .get(pool)
        .await?;
    Ok(partners)
}

pub(crate) struct DeletePartnerMessage;

impl DeletePartnerMessage {
    pub fn is_published_etp_error(partners: &[CheckPartnerItem]) -> Message {
        Message::error(
            "Информация опубликована на ЭТП ГПБ. Удаление невозможно".to_string(),
        )
        .with_param_items(partners)
    }

    pub fn is_tcp_exist_error(partners: &[CheckPartnerItem]) -> Message {
        Message::error(
            "Удаление организации невозможно. Имеется подтвержденное ТКП"
                .to_string(),
        )
        .with_param_items(partners)
    }
    pub fn is_deleted_success(partners: &[CheckPartnerItem]) -> Message {
        Message::success("Выбранные записи удалены".to_string())
            .with_param_items(partners)
    }
    pub fn is_deletion_prohibited(partners: &[CheckPartnerItem]) -> Message {
        Message::error("Удаление организации невозможно. Статус ЗЦИ не равен одному из статусов: 70, 90, 100, 150".to_string())
            .with_param_items(partners)
    }
}

pub(crate) fn convert_request_partners_to_response_items(
    partners: &[RequestPartner],
    is_allowed: bool,
) -> Vec<CheckPartnerItem> {
    partners
        .iter()
        .map(|item| CheckPartnerItem {
            supplier_id: item.supplier_id,
            is_allowed,
        })
        .collect()
}

pub(crate) fn convert_suppliers_to_response_items(
    suppliers: &[SupplierId],
    is_allowed: bool,
) -> Vec<CheckPartnerItem> {
    suppliers
        .iter()
        .map(|item| CheckPartnerItem {
            supplier_id: item.supplier_id,
            is_allowed,
        })
        .collect()
}
