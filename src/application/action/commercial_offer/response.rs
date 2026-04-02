use asez2_shared_db::db_item::joined::JoinTo;
use shared_essential::domain::tcp::PartnerWithProposalsSelector;
use shared_essential::presentation::dto::integration::commercial_offer::response::*;
use shared_essential::{
    domain::{
        maths::CurrencyValue,
        tcp::{
            ProposalHeader, ProposalItem, RequestHeader, RequestItem,
            RequestPartner, TCPCheckStatus, TcpGeneralStatus,
        },
    },
    presentation::dto::technical_commercial_proposal::{TcpError, TcpResult},
};

use asez2_shared_db::{
    db_item::{AsezTimestamp, DbUpsert, Select},
    DbItem,
};

use ahash::AHashMap;
use sqlx::PgPool;
use uuid::Uuid;

/// Обрабатывает ТКП, переданный из SAP PI.
///
/// Валидирует и сохраняет заголовок (`ProposalHeader`) и позиции (`ProposalItem`) ТКП в БД.
/// При необходимости создает поставщика (`RequestPartner`).
///
/// # Аргументы
///
/// * `req` — Данные ТКП
/// * `pool` — Пул соединений с БД.
///
/// # Ошибки
///
/// Возвращает `TcpError` в случае:
/// - отсутствия обязательных полей (например, `AsezId`, `currency_id`);
/// - некорректного парсинга `req_number`;
/// - отсутствия данных в БД (`RequestHeader`, `RequestItem`);
/// - ошибок вставки в БД;
/// - ошибок валидации или парсинга полей.
///
/// # Возвращает
///
/// `Ok(())` при успешной обработке, иначе `Err(TcpError)`.
pub async fn commercial_offer_response(
    req: CommercialOfferResponseData,
    pool: &PgPool,
) -> TcpResult<()> {
    let asez_id = req.tcp.supplier.asez_id.ok_or_else(|| {
        tracing::error!(
            "Отсутствует AsezId для req_number {}",
            req.tcp.req_info.req_number
        );
        TcpError::InternalError(format!(
            "Отсутствует AsezId для req_number {}",
            req.tcp.req_info.req_number
        ))
    })?;

    let req_number = req.tcp.req_info.req_number;

    let (request_head_uuid, currency_id) = RequestHeader::select(
        &Select::with_fields([RequestHeader::uuid, RequestHeader::currency_id])
            .eq(RequestHeader::id, req_number),
        pool,
    )
    .await?
    .into_iter()
    .map(|h| (h.uuid, h.currency_id))
    .next()
    .ok_or_else(|| TcpError::not_found(req_number, RequestHeader::TABLE))?;

    let partner = find_or_create_partner(asez_id, request_head_uuid, pool).await?;

    let mut new_proposal_head = build_proposal_header(
        &req,
        request_head_uuid,
        partner.uuid,
        currency_id.ok_or_else(|| {
            TcpError::InternalError(
                "Не удалось получить currency_id из RequestHeader (возможно отсутствует в БД)".to_string(),
            )
        })?,
    );

    let item_select = Select::with_fields([
        RequestItem::uuid,
        RequestItem::number,
        RequestItem::request_uuid,
        RequestItem::description_internal,
        RequestItem::unit_id,
        RequestItem::quantity,
    ])
    .eq(RequestItem::request_uuid, request_head_uuid);

    let request_items_map: AHashMap<i16, RequestItem> =
        RequestItem::select(&item_select, pool)
            .await?
            .into_iter()
            .map(|item| (item.number, item))
            .collect();

    let mut proposal_items = build_proposal_items(
        req.tcp.price_info,
        new_proposal_head.uuid,
        &request_items_map,
    )?;

    let mut tx = pool.begin().await?;
    let _ = RequestPartner::upsert_returning(
        &mut [partner],
        Some(&[RequestPartner::is_public]),
        &mut tx,
    )
    .await?;
    ProposalHeader::insert(&mut new_proposal_head, &mut tx).await?;
    ProposalItem::insert_vec(&mut proposal_items, &mut tx).await?;
    tx.commit().await?;

    Ok(())
}

/// Находит существующего `RequestPartner` по `asez_id` и `request_uuid`,
/// либо создает нового, если такого ещё нет.
///
/// # Аргументы
///
/// * `asez_id` — идентификатор поставщика в АСЭЗ.
/// * `req_head_uuid` — UUID заголовка ЗЦИ (`RequestHeader`).
/// * `pool` — пул соединений с PostgreSQL.
///
/// # Возвращает
///
/// UUID существующего или вновь созданного `RequestPartner`.
///
/// # Ошибки
///
/// Возвращает `TcpError`, если возникают ошибки при обращении к базе данных.
pub(crate) async fn find_or_create_partner(
    asez_id: i32,
    req_head_uuid: Uuid,
    pool: &PgPool,
) -> TcpResult<RequestPartner> {
    let partner_select =
        Select::with_fields([RequestPartner::uuid, RequestPartner::is_public])
            .eq(RequestPartner::supplier_id, asez_id)
            .eq(RequestPartner::request_uuid, req_head_uuid)
            .eq(RequestPartner::is_removed, false);

    let partners = PartnerWithProposalsSelector::new(partner_select)
        .set_proposals(
            ProposalHeader::join_default()
                .selecting(Select::full::<ProposalHeader>()),
        )
        .get(pool)
        .await?;

    if let Some(mut list) = partners.into_iter().next() {
        if let Some(proposal) = list.proposals.first() {
            tracing::error!(
                kind = "tcp",
                "От данной организации уже создано ТКП (id = {})",
                proposal.id
            );
            return Err(TcpError::InternalError(
                "От данной организации уже создано ТКП".to_string(),
            ));
        } else {
            list.partner.is_public = true;
            return Ok(list.partner);
        }
    }

    let max_number = RequestPartner::select(
        &Select::with_fields([RequestPartner::number])
            .eq(RequestPartner::request_uuid, req_head_uuid),
        pool,
    )
    .await?
    .iter()
    .map(|p| p.number)
    .max()
    .unwrap_or(0);

    let new_partner = RequestPartner {
        uuid: Uuid::new_v4(),
        request_uuid: req_head_uuid,
        supplier_id: asez_id,
        number: max_number + 1,
        is_public: true,
        ..Default::default()
    };

    Ok(new_partner)
}

/// Строит структуру `ProposalHeader` на основе входных данных.
///
/// # Аргументы
///
/// * `req` — исходные данные.
/// * `request_uuid` — UUID `RequestHeader`.
/// * `partner_uuid` — UUID `RequestPartner`.
/// * `currency_id` — ID валюты.
///
/// # Возвращает
///
/// Новый экземпляр `ProposalHeader` с предзаполненными полями.
fn build_proposal_header(
    req: &CommercialOfferResponseData,
    request_uuid: Uuid,
    partner_uuid: Uuid,
    currency_id: i16,
) -> ProposalHeader {
    let sum_excluded_vat_total =
        Some(req.tcp.price_info.iter().map(|p| p.cost).sum::<f64>().into());
    let now = AsezTimestamp::now();

    ProposalHeader {
        uuid: Uuid::new_v4(),
        etp_id: Some(req.tcp.tcp_id),
        request_uuid,
        hierarchy_uuid: Some(req.hierarchy_uuid),
        supplier_uuid: partner_uuid,
        start_date: Some(req.tcp.date_start_proposal),
        end_date: Some(req.tcp.date_end_proposal),
        currency_id: currency_id as i32,
        created_by: req.user_id,
        created_at: now,
        changed_at: now,
        changed_by: req.user_id,
        status_id: TcpGeneralStatus::Received,
        status_check_id: TCPCheckStatus::Review,
        receive_date: Some(now),
        sum_excluded_vat_total,
        contact_phone: req.tcp.supplier.contact_phone.clone(),
        ..Default::default()
    }
}

/// Строит вектор `ProposalItem`.
///
/// # Аргументы
///
/// * `price_info` — Позиции ТКП.
/// * `proposal_uuid` — UUID заголовка.
/// * `request_items_map` — отображение `number -> RequestItem` для поиска позиций.
///
/// # Возвращает
///
/// Вектор `ProposalItem`, готовых к инсерту в БД.
///
/// # Ошибки
///
/// Возвращает `TcpError`, если:
/// - не найдена позиция по номеру;
/// - не удалось распарсить `terms_of_payment`;
/// - ставка НДС указана некорректно.
fn build_proposal_items(
    price_info: Vec<PriceInfo>,
    proposal_uuid: Uuid,
    request_items_map: &AHashMap<i16, RequestItem>,
) -> TcpResult<Vec<ProposalItem>> {
    price_info
        .into_iter()
        .map(|p| {
            let pos_nr = p.price_info_pos_nr as i16;
            let request_item = request_items_map.get(&pos_nr).ok_or_else(|| {
                TcpError::InternalError(format!(
                    "Не найдена позиция ЗЦИ с номером {}",
                    pos_nr
                ))
            })?;

            let is_impossible = p.impossible_to_do.unwrap_or(false);

            Ok(ProposalItem {
                uuid: Uuid::new_v4(),
                number: pos_nr,
                proposal_uuid,
                request_item_uuid: request_item.uuid,
                description_internal: request_item.description_internal.clone(),
                quantity: request_item.quantity,
                unit_id: request_item.unit_id as i32,
                price: is_impossible.then_some(p.price.into()),
                vat_id: p.vat_id,
                sum_excluded_vat: is_impossible.then_some(p.cost.into()),
                sum_included_vat: is_impossible.then_some(p.cost_nds.into()),
                manufacturer: is_impossible.then_some(p.manufacturer),
                mark: p.product_mark,
                pay_condition_id: p.pay_condition_id,
                prepayment_percent: p.prepayment_percent.map(CurrencyValue::from),
                delivery_condition: p.terms_of_delivery,
                execution_percent: p.execution_percent.map(CurrencyValue::from),
                is_possibility: is_impossible,
                possibility_note: p.cause_impossible,
                analog_description: p.analog_description,
                delivery_period: p.delivery_period,
            })
        })
        .collect()
}
