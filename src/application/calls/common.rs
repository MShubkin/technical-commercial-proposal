use ahash::AHashMap;
use std::collections::HashMap;

use crate::presentation::dto::{
    PriceInformationRequest, TechnicalCommercialProposal,
};
use asez2_shared_db::{
    db_item::{joined::JoinTo, Select},
    result::SharedDbError,
    DbItem,
};
use shared_essential::{
    domain::tables::tcp::{
        JoinedPriceInformationInfoDetail, JoinedPriceInformationInfoDetailSelector,
        ProposalHeader, ProposalItem, RequestHeader, RequestItem, RequestPartner,
        TcpDbItem,
    },
    presentation::dto::{
        response_request::{Message, Messages, ParamItem},
        technical_commercial_proposal::{TcpError, TcpResult},
    },
};
use sqlx::PgPool;
use uuid::Uuid;

/// Получение детальной информации ЗЦИ по request_header.id
pub(crate) async fn get_price_information_request_info_details_by_id(
    id: i64,
    pool: &PgPool,
) -> TcpResult<JoinedPriceInformationInfoDetail> {
    let header_select =
        Select::full::<RequestHeader>().eq(RequestHeader::id, id).take_first();
    let item_select =
        Select::full::<RequestItem>().add_replace_order_asc(RequestItem::uuid);
    get_price_information_request_info_details(header_select, item_select, pool)
        .await?
        .ok_or_else(|| {
            TcpError::RecordNotFound(
                id.to_string(),
                RequestHeader::TABLE.to_string(),
            )
        })
}

/// Получение детальной информации ЗЦИ по request_header.uuid
pub(crate) async fn get_price_information_request_info_details_by_uuid(
    uuid: Uuid,
    pool: &PgPool,
) -> TcpResult<JoinedPriceInformationInfoDetail> {
    let header_select = Select::full::<RequestHeader>()
        .eq(RequestHeader::uuid, uuid)
        .take_first();
    let item_select =
        Select::full::<RequestItem>().add_replace_order_asc(RequestItem::uuid);
    get_price_information_request_info_details(header_select, item_select, pool)
        .await?
        .ok_or_else(|| {
            TcpError::RecordNotFound(
                uuid.to_string(),
                RequestHeader::TABLE.to_string(),
            )
        })
}

/// Получение детальной информации ЗЦИ для экспорта
pub(crate) async fn get_price_information_request_info_details_export(
    header_select: Select,
    item_select: Select,
    pool: &PgPool,
) -> TcpResult<JoinedPriceInformationInfoDetail> {
    get_price_information_request_info_details(header_select, item_select, pool)
        .await?
        .ok_or_else(|| {
            TcpError::Export(
                "Записи для детальной информации ЗЦИ не найдены".to_string(),
            )
        })
}

/// Получение детальной информации ЗЦИ
///
/// Позволяет делать выборку либо по request_header.id, либо по request_header.uuid
async fn get_price_information_request_info_details(
    header_select: Select,
    item_select: Select,
    pool: &PgPool,
) -> TcpResult<Option<JoinedPriceInformationInfoDetail>> {
    Ok(JoinedPriceInformationInfoDetailSelector::new(header_select)
        .set_items(
            RequestItem::join_default()
                .distinct_aggr(!item_select.order_list.is_empty())
                .selecting(item_select),
        )
        .set_suppliers(
            RequestPartner::join_default()
                .selecting(Select::full::<RequestPartner>()),
        )
        .distinct()
        .get(pool)
        .await?
        .pop())
}

/// Сохранение в БД ЗЦИ. Возвращаются номера созданных ЗЦИ
pub(crate) async fn insert_price_information_request(
    pool: &PgPool,
    request_list: Vec<PriceInformationRequest>,
) -> TcpResult<Vec<i64>> {
    let mut headers = Vec::with_capacity(request_list.len());
    let mut items = Vec::with_capacity(request_list.len());
    let mut partners = Vec::with_capacity(request_list.len());

    for req in request_list {
        headers.push(req.header);
        items.extend(req.items.into_iter());
        partners.extend(req.suppliers.unwrap_or_default().into_iter());
    }
    let mut tx = pool.begin().await?;
    let created_req_ids =
        RequestHeader::insert_vec_returning(&mut headers, &mut tx)
            .await?
            .into_iter()
            .map(|x| x.id)
            .collect::<Vec<_>>();
    RequestItem::insert_vec(&mut items, &mut tx).await?;
    RequestPartner::insert_vec(&mut partners, &mut tx).await?;

    tx.commit().await?;
    Ok(created_req_ids)
}

/// Получение списка ЗЦИ по uuid ППЗ
pub(crate) async fn get_price_information_requests_by_plan_uuid(
    pool: &PgPool,
    uuid_ppz: Uuid,
) -> TcpResult<Vec<PriceInformationRequest>> {
    let rh_select = Select::full::<RequestHeader>()
        .in_any(RequestHeader::plan_uuid, [uuid_ppz]);
    let headers = RequestHeader::select(&rh_select, pool).await?;

    let h_uuids = headers.iter().map(|x| x.uuid).collect::<Vec<_>>();
    let (mut item_map, mut supplier_map) = (AHashMap::new(), AHashMap::new());

    RequestItem::get_by_request_uuids(&h_uuids, pool)
        .await?
        .into_iter()
        .for_each(|i| {
            item_map.entry(i.request_uuid).or_insert(vec![]).push(i);
        });
    RequestPartner::get_by_request_uuids(&h_uuids, pool)
        .await?
        .into_iter()
        .for_each(|i| {
            supplier_map.entry(i.request_uuid).or_insert(vec![]).push(i);
        });

    tracing::info!(kind = "infra", "request headers count: {:?}", headers.len());
    let list = headers
        .into_iter()
        .map(|header| PriceInformationRequest {
            items: item_map.remove(&header.uuid).unwrap_or_default(),
            suppliers: supplier_map.remove(&header.uuid),
            header,
        })
        .collect::<Vec<_>>();

    Ok(list)
}

/// Получение списка ЗЦИ по списку uuid ППЗ
pub(crate) async fn get_price_information_requests_by_plan_uuid_vec(
    pool: &PgPool,
    uuid_ppz: Vec<Uuid>,
) -> TcpResult<HashMap<Uuid, Vec<PriceInformationRequest>>> {
    let rh_select =
        Select::full::<RequestHeader>().in_any(RequestHeader::plan_uuid, uuid_ppz);
    let headers = RequestHeader::select(&rh_select, pool).await?;

    let request_uuids: Vec<Uuid> =
        headers.iter().map(|header| header.uuid).collect();
    let headers_map_by_plan_uuid: AHashMap<Uuid, Vec<RequestHeader>> =
        headers.into_iter().fold(AHashMap::new(), |mut hash_map, header| {
            if let Some(plan_uuid) = header.plan_uuid {
                hash_map.entry(plan_uuid).or_default().push(header);
            }
            hash_map
        });
    let items = RequestItem::get_by_request_uuids(&request_uuids, pool).await?;
    let items_map_by_request_uuid: AHashMap<Uuid, Vec<RequestItem>> =
        items.into_iter().fold(AHashMap::new(), |mut hash_map, item| {
            hash_map.entry(item.request_uuid).or_default().push(item);
            hash_map
        });
    let suppliers =
        RequestPartner::get_by_request_uuids(&request_uuids, pool).await?;
    let suppliers_map_by_request_uuid: AHashMap<Uuid, Vec<RequestPartner>> =
        suppliers.into_iter().fold(AHashMap::new(), |mut hash_map, supplier| {
            hash_map.entry(supplier.request_uuid).or_default().push(supplier);
            hash_map
        });

    let result_map: HashMap<Uuid, Vec<PriceInformationRequest>> =
        headers_map_by_plan_uuid.into_iter().fold(
            HashMap::new(),
            |mut hash_map, value| {
                let mut request_vec: Vec<PriceInformationRequest> =
                    Vec::with_capacity(value.1.len());
                value.1.into_iter().for_each(|header| {
                    let mut request: PriceInformationRequest =
                        PriceInformationRequest {
                            header: header.clone(),
                            ..Default::default()
                        };
                    if let Some(items) = items_map_by_request_uuid.get(&header.uuid)
                    {
                        request.items = items.clone();
                    }
                    if let Some(suppliers) =
                        suppliers_map_by_request_uuid.get(&header.uuid)
                    {
                        request.suppliers = Some(suppliers.clone());
                    }
                    request_vec.push(request);
                });

                hash_map.insert(value.0, request_vec);
                hash_map
            },
        );

    Ok(result_map)
}

/// Чтение списка ТКП по uuid ЗЦИ
pub(crate) async fn get_tkp_by_uuid_request(
    pool: &PgPool,
    uuid_request: Uuid,
) -> TcpResult<Vec<TechnicalCommercialProposal>> {
    let headers =
        ProposalHeader::get_by_request_uuids(&[uuid_request], pool).await?;

    let h_uuids = headers.iter().map(|x| x.uuid).collect::<Vec<_>>();
    let mut item_map = AHashMap::new();

    get_proposal_items_by_tkp_uuids(&h_uuids, pool)
        .await?
        .into_iter()
        .for_each(|i| {
            item_map.entry(i.proposal_uuid).or_insert(vec![]).push(i);
        });

    let list = headers
        .into_iter()
        .map(|header| TechnicalCommercialProposal {
            items: item_map.remove(&header.uuid).unwrap_or_default(),
            header,
        })
        .collect::<Vec<_>>();
    Ok(list)
}

/// Чтение списка ТКП по списку uuid ЗЦИ
pub(crate) async fn get_tkp_by_uuid_request_vec(
    pool: &PgPool,
    uuid_request: Vec<Uuid>,
) -> TcpResult<HashMap<Uuid, Vec<TechnicalCommercialProposal>>> {
    let tkp_headers =
        ProposalHeader::get_by_request_uuids(&uuid_request, pool).await?;

    let tkp_uuids: Vec<Uuid> =
        tkp_headers.iter().map(|tkp_header| tkp_header.uuid).collect();

    let tkp_map_by_request_uuid: AHashMap<Uuid, Vec<ProposalHeader>> = tkp_headers
        .into_iter()
        .fold(AHashMap::new(), |mut hash_map, tkp_header| {
            hash_map.entry(tkp_header.request_uuid).or_default().push(tkp_header);
            hash_map
        });

    let items = get_proposal_items_by_tkp_uuids(&tkp_uuids, pool).await?;
    let items_map_by_tkp_uuid: AHashMap<Uuid, Vec<ProposalItem>> =
        items.into_iter().fold(AHashMap::new(), |mut hash_map, item| {
            hash_map.entry(item.proposal_uuid).or_default().push(item);
            hash_map
        });

    let result_map: HashMap<Uuid, Vec<TechnicalCommercialProposal>> =
        tkp_map_by_request_uuid.into_iter().fold(
            HashMap::new(),
            |mut hash_map, value| {
                let mut request_vec: Vec<TechnicalCommercialProposal> =
                    Vec::with_capacity(value.1.len());
                value.1.into_iter().for_each(|tkp_header| {
                    let mut tkp: TechnicalCommercialProposal =
                        TechnicalCommercialProposal {
                            header: tkp_header.clone(),
                            ..Default::default()
                        };

                    if let Some(items) = items_map_by_tkp_uuid.get(&tkp_header.uuid)
                    {
                        tkp.items = items.clone();
                    }

                    request_vec.push(tkp);
                });
                hash_map.insert(value.0, request_vec);
                hash_map
            },
        );

    Ok(result_map)
}

/// Exists for dry.
async fn get_proposal_items_by_tkp_uuids(
    uuids: &[Uuid],
    pool: &PgPool,
) -> Result<Vec<ProposalItem>, SharedDbError> {
    let a_sel =
        Select::full::<ProposalItem>().in_any(ProposalItem::proposal_uuid, uuids);
    ProposalItem::select(&a_sel, pool).await
}

/// Сообщение из успеха не ошибок
pub(crate) fn messages_from_success_and_errors(
    success_msg_text: &str,
    error_msg_text: &str,
    err_param_description: &str,
    ok_param_list: Vec<ParamItem>,
    err_param_list: Vec<ParamItem>,
) -> Messages {
    let mut messages = Messages::default();
    // После обработки всего массива item_list:
    // Для успешно обработанных сформировать сообщение типа Success
    if !ok_param_list.is_empty() {
        let ok_message =
            Message::success(success_msg_text).with_parameters(ok_param_list);
        messages.add_prepared_message(ok_message);
    }
    // Для id для которых проверка не была выполнена сформировать сообщение типа Error:
    if !err_param_list.is_empty() {
        let err_message = Message::error(error_msg_text)
            .with_parameters(err_param_list)
            .with_param_description(err_param_description);
        messages.add_prepared_message(err_message);
    }
    messages
}
