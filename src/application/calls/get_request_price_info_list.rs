use ahash::AHashMap;
use sqlx::PgPool;

use asez2_shared_db::db_item::joined::JoinTo;
use asez2_shared_db::db_item::{from_item_with_fields, FilterTree, Select};
use asez2_shared_db::DbItem;
use asez2_shared_db::Value;
use rabbit_services::view_storage::ViewStorageService;
use shared_essential::{
    application::validation::validate_allowed_fields,
    domain::tables::tcp::{
        ProposalHeader, RequestHeaderRep, RequestPartner,
        RequestWithPartnersSelector, TCPCheckStatus,
    },
    presentation::dto::{
        general::{FeWrapper, UserId},
        response_request::*,
        technical_commercial_proposal::{TcpError, TcpResult},
        value::UiValue,
        view_storage::{AllowedFieldsRequest, AllowedFieldsResponse},
    },
};

use crate::application::WORKPLACE;
use crate::presentation::dto::{
    GetRequestPriceInfoListResponse, GetRequestPriceListReq,
};

/// Have some needless constants.
pub(crate) const SUPPLIER_LIST: &str = "supplier_list";
pub(crate) const TKP_IN: &str = "tkp_in";
/// Массив `hierarchy_uuid` по ЗЦИ, который берется из смежных по ЗЦИ записей в
/// ТКП
pub(crate) const HIERARCHY_UUID_LIST: &str = "hierarchy_uuid";
pub(crate) const TKP_DONE: &str = "tkp_done";

/// Получение списка ЗЦИ по Select
/// Route - /rest/technical_commercial_proposal/v1/get/request_price_info_list/
pub(crate) async fn get_request_price_info_list(
    dto: GetRequestPriceListReq,
    user_id: UserId,
    views: ViewStorageService,
    pool: &PgPool,
) -> TcpResult<ApiResponse<GetRequestPriceInfoListResponse, ()>> {
    let afr_dto = AllowedFieldsRequest {
        workplace_id: WORKPLACE.into(),
        section_id: dto.section_id.to_string(),
        user_id: user_id.user_id.to_string(),
    };
    let views_response: AllowedFieldsResponse = views
        .get_allowed_fields(&afr_dto)
        .await
        .map_err(|e| TcpError::InternalError(e.error().to_string()))?;

    let user_id = user_id.user_id;
    validate_allowed_fields(
        &dto.select.field_list,
        &views_response.fields,
        user_id,
    )
    .map_err(TcpError::from)?;

    let db_select = dto.select.try_into()?;
    let data = database_request(db_select, pool).await?;

    Ok(ApiResponse::default().with_data(data.into()))
}

pub(crate) async fn database_request(
    select: Select,
    pool: &PgPool,
) -> TcpResult<Vec<FeWrapper<RequestHeaderRep>>> {
    let origin_filters = select.filter_list.clone();
    let filtered_select = select
        .filtered_copy_for::<RequestHeaderRep>()
        .with_approprtiate_null_position();
    let from_header = from_item_with_fields(&select.field_list);

    let partner_select =
        Select::full::<RequestPartner>().eq(RequestPartner::is_removed, false);

    let requests = RequestWithPartnersSelector::new_with_order(filtered_select)
        .set_suppliers(RequestPartner::join_default().selecting(partner_select))
        .get(pool)
        .await?;

    let partner_uuids =
        requests.iter().flat_map(|x| x.suppliers.iter().map(|x| x.uuid));

    let proposal_select = Select::full::<ProposalHeader>()
        .in_any(ProposalHeader::supplier_uuid, partner_uuids);
    let proposals = ProposalHeader::select(&proposal_select, pool).await?;
    let mut proposal_map = AHashMap::new();

    for prop in proposals {
        proposal_map.entry(prop.supplier_uuid).or_insert(vec![]).push(prop);
    }

    let mut data = requests
        .into_iter()
        .map(|x| {
            let header = from_header(x.header);

            let mut tkp_in = Vec::new();
            let (mut done_count, mut in_count) = (0, 0);
            let mut hierarchy_uuid_list = Vec::new();

            let partner_list = x
                .suppliers
                .iter()
                .map(|x| {
                    let id = x.supplier_id;
                    let proposals = proposal_map.get(&x.uuid);

                    if let Some(proposals) = proposals {
                        tkp_in.push(id);

                        for proposal in proposals {
                            in_count += 1;
                            if matches!(
                                proposal.status_check_id,
                                TCPCheckStatus::Reviewed
                            ) {
                                done_count += 1;
                            }

                            if let Some(hierarchy_uuid) = proposal.hierarchy_uuid {
                                hierarchy_uuid_list
                                    .push(UiValue::from(hierarchy_uuid))
                            }
                        }
                    }

                    id
                })
                .collect::<Vec<_>>();

            let wrapper = FeWrapper::new(header)
                .add_field(SUPPLIER_LIST, partner_list)
                .add_field(
                    HIERARCHY_UUID_LIST,
                    UiValue::VecValue(hierarchy_uuid_list),
                )
                .add_field(TKP_IN, tkp_in)
                .add_field(TKP_DONE, vec![done_count, in_count]);
            Ok(wrapper)
        })
        .collect::<TcpResult<Vec<_>>>()?;

    // Некоторые фильтрации приходится делать на уровне кода,
    // а не на уровне sql
    filter(&mut data, origin_filters);

    Ok(data)
}

/// Отвечает за фильтрацию, которую нельзя сделать на уровне sql, например
/// фильтрация по вычисляемым полям
///
/// Текущие фильтры рассчитаны на логику In и Equals, поэтому
/// тип фильтра мы не проверяем. Проверяем только то что пришел
/// нужный нам тип
fn filter(items: &mut Vec<FeWrapper<RequestHeaderRep>>, filters: FilterTree) {
    // Так как сейчас все вычисляемые поля являются Vec<i64>, то матчится
    // именно по ним
    macro_rules! filter_match {
        ($items:ident, $filter:ident, [$($mode:tt => $field_name:ident),*]) => {
            match $filter.field.as_str() {
                $($field_name => {
                    $items.retain(|i| {
                        let Some(cmp_vals) = i.get_extra_field($field_name) else {
                            return false
                        };
                        filter_match!(@inner $mode $filter.values, cmp_vals)
                    });
                }),*
                _ => {}
            }
        };
        (@inner in_any $filter_values: expr, $cmp_vals: expr) => {
            $filter_values.iter().any(|filter_value| {
                if let (&Value::Int(filter_val), UiValue::VecValue(cmp_vals)) =
                    (filter_value, $cmp_vals)
                {
                    cmp_vals.iter().any(|cmp_val| match cmp_val {
                        UiValue::Int(cmp_val) => *cmp_val == filter_val,
                        _ => false
                    })
                } else {
                    false
                }
            })

        };
        (@inner eq $filter_values: expr, $cmp_vals: expr) => {
            if let UiValue::VecValue(cmp_vals) = $cmp_vals {
                $filter_values.len() == cmp_vals.len() &&
                    $filter_values.iter().zip(cmp_vals.iter()).all(|(f_v, c_v)| {
                        match (f_v, c_v) {
                            (Value::Int(f_v), UiValue::Int(c_v)) => *c_v == *f_v,
                            _ => false
                        }
                    })
            } else {
                false
            }
        }
    }

    for filter in filters.into_filters() {
        filter_match!(items, filter,
            [
                in_any => SUPPLIER_LIST,
                in_any => TKP_IN,
                eq => TKP_DONE
            ]
        )
    }
}
