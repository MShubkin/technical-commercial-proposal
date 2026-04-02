use crate::presentation::dto::{ApproveProposalReq, ApproveProposalResponseData};

use asez2_shared_db::db_item::{joined::JoinTo, AsezTimestamp, Select};
use asez2_shared_db::DbAdaptor;

use monolith_service::dto::attachment::{
    FoldersCategory, GetHierarchyReq, GetHierarchyResponseData,
};
use monolith_service::http::MonolithHttpService;
use shared_essential::domain::tcp::{
    JoinedProposalHeaderProposalItem as JoinedProposal, ProposalHeader,
    ProposalHeaderRep, ProposalItem, ProposalWithItemsSelector, TcpGeneralStatus,
};

use shared_essential::presentation::dto::{
    response_request::{Message, Messages, ParamItem},
    technical_commercial_proposal::{TcpError, TcpResult},
};

use sqlx::PgPool;

const UPDATE_FIELDS: &[&str] = &[
    ProposalHeader::changed_at,
    ProposalHeader::changed_by,
    ProposalHeader::status_id,
    ProposalHeader::receive_date,
];

const RESPONSE_FIELDS: &[&str] =
    &[ProposalHeader::uuid, ProposalHeader::id, ProposalHeader::status_id];

pub(crate) async fn process_approve_proposal(
    user_id: i32,
    monolith_token: String,
    req: ApproveProposalReq,
    monolith_service: &MonolithHttpService,
    pool: &PgPool,
) -> TcpResult<(ApproveProposalResponseData, Messages)> {
    let uuids = req.item_list.iter().map(|x| x.uuid);
    let proposal =
        Select::full::<ProposalHeader>().in_any(ProposalHeader::uuid, uuids);
    let items = Select::full::<ProposalItem>();

    let proposals = ProposalWithItemsSelector::new(proposal)
        .set_items(ProposalItem::join_default().selecting(items))
        .get(pool)
        .await?;

    let mut messages = Messages::default();

    check_header_fields(&proposals, &mut messages);
    check_item_fields(&proposals, &mut messages);

    let monolith_res = get_hierarchy_from_monolith(
        &proposals,
        monolith_token,
        monolith_service,
        user_id,
    )
    .await?;

    check_attachment_fields(&monolith_res, &mut messages);

    if messages.is_error() {
        return Err(TcpError::Business(messages));
    }

    let changed_at = AsezTimestamp::now();
    let updatable = proposals
        .into_iter()
        .map(|p| {
            let mut x = p.header;
            x.changed_at = changed_at;
            x.changed_by = user_id;
            // Установить (ТКП получено)
            x.status_id = TcpGeneralStatus::Received;
            x.receive_date = x.receive_date.or(Some(changed_at));
            x
        })
        .collect::<Vec<_>>();

    let mut tx = pool.begin().await?;

    let updated = ProposalHeaderRep::update_vec_returning::<Vec<_>>(
        &updatable,
        Some(UPDATE_FIELDS),
        Some(RESPONSE_FIELDS),
        &mut tx,
    )
    .await?;

    tx.commit().await?;

    for proposal in &updated {
        if let Some(id) = proposal.id {
            messages.add_prepared_message(ApproveProposalMessage::success(id));
        }
    }

    Ok((ApproveProposalResponseData { item_list: updated }, messages))
}

/// Проверить заполнение полей заголовка
fn check_header_fields(data: &[JoinedProposal], messages: &mut Messages) {
    let headers: Vec<&ProposalHeader> = data.iter().map(|p| &p.header).collect();

    headers.iter().for_each(|header| {
        [
            //TODO: Возможно supplier_uuid должен быть Option<Uuid>
            (header.supplier_uuid == uuid::Uuid::nil(), "Организация"),
            (header.start_date.is_none(), "Начало срока действия"),
            (header.end_date.is_none(), "Окончание срока действия"),
            (header.hierarchy_uuid.is_none(), "UUID иерархии"),
        ]
        .iter()
        .filter(|(is_empty, _)| *is_empty)
        .for_each(|(_, name)| {
            messages.add_prepared_message(
                ApproveProposalMessage::missing_header_field(name, header),
            );
        });
    });
}

/// Проверить заполнение полей позиций
fn check_item_fields(data: &[JoinedProposal], messages: &mut Messages) {
    let items: Vec<&ProposalItem> =
        data.iter().flat_map(|p| p.items.iter()).collect();

    items.iter().for_each(|item| {
        [
            (
                item.is_possibility && item.price.is_none(),
                "Цена Организации (без НДС)",
            ),
            (
                item.is_possibility && item.vat_id.is_none(),
                "Ставка НДС Организации",
            ),
            (
                !item.is_possibility
                    && item
                        .possibility_note
                        .as_ref()
                        .map_or(true, |note| note.is_empty()),
                "Причина невозможности поставки",
            ),
            (
                item.pay_condition_id == Some(10)
                    && item.prepayment_percent.is_none(),
                "Размер аванса, %",
            ),
        ]
        .into_iter()
        .filter_map(|(is_absent, field)| is_absent.then_some(field))
        .for_each(|absent_field| {
            messages.add_prepared_message(
                ApproveProposalMessage::missing_item_field(absent_field, item),
            );
        })
    });
}

async fn get_hierarchy_from_monolith(
    data: &[JoinedProposal],
    monolith_token: String,
    monolith_service: &MonolithHttpService,
    user_id: i32,
) -> TcpResult<GetHierarchyResponseData> {
    let monolith_req = GetHierarchyReq {
        hierarchy_list: data
            .iter()
            .filter_map(|p| p.header.hierarchy_uuid)
            .collect(),
    };

    let monolith_res = monolith_service
        .get_hierarchy(monolith_req, monolith_token, user_id)
        .await?;

    if monolith_res.data.hierarchy_list.is_empty() {
        return Err(TcpError::MonolithError(
            "Монолит вернул пустой список иерархий".to_string(),
        ));
    }

    Ok(monolith_res.data)
}

/// Проверить наличие документа ТКП
fn check_attachment_fields(
    items: &GetHierarchyResponseData,
    messages: &mut Messages,
) {
    let has_valid_file = items
        .hierarchy_list
        .iter()
        .flat_map(|h| &h.item_list)
        .filter(|item| {
            item.kind_id == 2
                && item.category_id == Some(FoldersCategory::TenderDocumentation)
        })
        .any(|folder| {
            items.hierarchy_list.iter().flat_map(|h| &h.item_list).any(|file| {
                file.kind_id == 1
                    && file.parent_id == Some(folder.id)
                    && !file.is_removed
                    && !file.is_classified
            })
        });

    if !has_valid_file {
        messages
            .add_prepared_message(ApproveProposalMessage::not_found_tcp_document());
    }
}

pub struct ApproveProposalMessage;

impl ApproveProposalMessage {
    pub fn success(id: i64) -> Message {
        let text = format!("ТКП {} подтверждено", id);
        Message::success(text)
    }

    pub fn missing_header_field(
        field_name: &str,
        header: &ProposalHeader,
    ) -> Message {
        Message::error(format!("Заполните поле {}", field_name)).with_param_item(
            ParamItem {
                uuid: Some(header.uuid),
                id: header.id.to_string(),
                ..Default::default()
            },
        )
    }

    pub fn missing_item_field(field_name: &str, item: &ProposalItem) -> Message {
        Message::error(format!("Заполните поле {}", field_name)).with_param_item(
            ParamItem {
                uuid: Some(item.uuid),
                ..Default::default()
            },
        )
    }

    pub fn not_found_tcp_document() -> Message {
        Message::error(String::from("Прикрепите Документ ТКП"))
    }
}
