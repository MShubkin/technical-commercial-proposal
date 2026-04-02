use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

use rabbit_services::processing::ProcessingService;
use shared_essential::domain::CompletePlanRep;
use shared_essential::presentation::dto::AsezResult;
use shared_essential::presentation::dto::processing::GetPlanResponse;
use shared_essential::presentation::dto::response_request::*;
use shared_essential::presentation::dto::technical_commercial_proposal::{TcpError, TcpResult};
use shared_essential::presentation::dto::technical_commercial_proposal::create_price_information_request::CreatePriceInformationRequest;

use crate::application::calls::insert_price_information_request;
use crate::common::Utils;
use crate::domain::enrich_request_with_plan_data;
use crate::infrastructure::service_interaction::processing::get_processing_plans;
use crate::presentation::dto::PriceInformationRequest;

mod mapper;
mod validate;

/// Создание запроса ценовой информации
/// Route - /rest/technical_commercial_proposal/v1/create_price_information_request/
pub(crate) async fn create_price_information_request(
    create_request: CreatePriceInformationRequest,
    processing: ProcessingService,
    pool: &PgPool,
) -> TcpResult<GetPlanResponse> {
    let mut response = Default::default();
    let mut requests =
        PriceInformationRequest::create_price_information_request_from_json(
            create_request,
        )?;
    // TODO: PlansRequest нужен user_id, хотя бы как заглушка. Но в TKP он недоступен
    // без аутентификации. Можно сделать как в estimated-commission, но надо продумать нужно ли.
    let null_user = -1;
    //Получение данных ППЗ из сервиса процессинга
    let plans_response_result =
        get_plans_data(&requests, null_user, processing).await;
    let plan_vec =
        process_plans_response(&mut response, plans_response_result).await?;
    //Заполнение структуры ЗЦИ данными ППЗ
    enrich_request_with_plan_data(&mut requests, &plan_vec.item_list);

    // Сохранение в базе ЗЦИ
    let request_numbers = insert_price_information_request(pool, requests).await?;
    fill_success_response(&mut response, request_numbers);
    Ok(response)
}

/// Обработка ответа из сервиса процессинга
async fn process_plans_response(
    service_response: &mut GetPlanResponse,
    plans_response: AsezResult<GetPlanResponse>,
) -> TcpResult<PaginatedData<CompletePlanRep>> {
    match plans_response {
        Ok(api_response) => {
            if !api_response.messages.is_empty() {
                service_response.messages = api_response.messages;

                error!(
                    kind = "tcp",
                    "Ошибка получения данных из сервиса процессинга {:?}",
                    &service_response.messages
                );

                Err(TcpError::InternalError(format!(
                    "Ошибка получения данных из сервиса процессинга {:?}",
                    &service_response.messages
                )))
            } else {
                let total = api_response.data.total;
                let item_list = api_response
                    .data
                    .item_list
                    .into_iter()
                    .map(|x| CompletePlanRep {
                        plan: x.plan,
                        items: x.items,
                    })
                    .collect::<_>();
                Ok(PaginatedData { total, item_list })
            }
        }
        Err(error) => {
            error!(kind = "tcp", "process_plans_response {:?}", error);

            let tcp_error = TcpError::InternalError(format!(
                "Ошибка получения данных из сервиса процессинга {:?}",
                &service_response.messages
            ));
            Err(tcp_error)
        }
    }
}

/// Получение данных ППЗ из сервиса процессинга
async fn get_plans_data(
    requests: &[PriceInformationRequest],
    user_id: i32,
    processing: ProcessingService,
) -> AsezResult<GetPlanResponse> {
    let uuids: Vec<Uuid> = requests
        .iter()
        .map(|el| el.header.plan_uuid.unwrap_or_default())
        .collect();
    get_processing_plans(uuids, user_id, processing).await
}

fn fill_success_response<T: ApiResponseData>(
    response: &mut ApiResponse<T, ()>,
    request_numbers: Vec<i64>,
) {
    response.status = Status::Ok;
    response.messages.add_message(
        MessageKind::Information,
        format!(
            "Успешно созданы запрос(ы): {}",
            Utils::convert_vec_i64_to_string(&request_numbers)
        ),
    );
}
