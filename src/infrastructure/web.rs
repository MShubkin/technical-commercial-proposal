use actix_cors::Cors;
use actix_web::web::{self, delete, get, post, Json, ServiceConfig};
use tracing_actix_web::TracingLogger;

use crate::presentation::http_calls::*;
use http_middleware::{
    rabbit::DefaultRabbitProperties, tracing_fields::AsezTracingFields,
    AsezSessionWatcher,
};
use igg_tracing::ServiceRootSpanBuilder;
use shared_essential::infrastructure::server_config::ServerConfig;

/// Максимальное время (в секундах), в течение которого этот запрос CORS может быть кэширован
const MAX_AGE_CORS_CACHE: usize = 3600;

/// Настройка http-маршрутов
pub(crate) fn setup_routers(cfg: &mut ServiceConfig) {
    let api_scope = web::scope("/v1")
        .wrap(AsezSessionWatcher)
        .wrap(TracingLogger::<ServiceRootSpanBuilder>::new())
        .wrap(AsezTracingFields)
        .wrap(DefaultRabbitProperties)
        .service(
            web::scope("/action")
                .route(
                    "/proposal_apply_pricing_consider/",
                    post().to(proposal_apply_pricing_handle),
                )
                .route(
                    "/request_price_info_close/",
                    post().to(price_info_close_handle),
                )
                .route(
                    "/request_price_info_complete/",
                    post().to(price_info_complete_handle),
                )
                .route(
                    "/request_price_info_publication/",
                    post().to(price_info_publication_handle),
                )
                .route("/proposal_approve/", post().to(proposal_approve_handler))
                .route(
                    "/organizations_remove/",
                    post().to(organizations_remove_handler),
                )
                .route(
                    "/purchasing_subject_group_remove/",
                    post().to(purchasing_subject_group_remove_handler),
                )
                .route(
                    "/purchasing_subject_remove/",
                    post().to(purchasing_subject_remove_handler),
                ),
        )
        .service(
            web::scope("/check")
                .route("/add_partner/", post().to(check_add_partner_handle))
                .route(
                    "/request_price_info/",
                    post().to(check_request_price_info_handle),
                )
                .route("/delete_partner/", post().to(check_delete_partner_handle)),
        )
        .service(
            web::scope("/delete").route(
                "/request_price_info/",
                delete().to(delete_price_info_handle),
            ),
        )
        .service(
            web::scope("/get")
                .route(
                    "/request_price_info_list/",
                    post().to(get_request_price_info_list_handle),
                )
                .route(
                    "/request_price_info_detail/",
                    post().to(get_request_price_info_detail_handle),
                )
                .route(
                    "/proposal_list_by_object_id/",
                    post().to(get_proposals_by_object_id_handle),
                )
                .route(
                    "/proposal_items_for_pricing/",
                    post().to(get_proposal_items_for_pricing_handle),
                )
                .route("/proposal_detail/", post().to(get_proposal_detail_handle))
                .route(
                    "/technical_commercial_proposal/",
                    post().to(get_technical_commercial_proposal),
                )
                .route(
                    "/purchasing_subject_by_group_uuid/{uuid}/",
                    get().to(get_purchasing_subject_by_group_uuid_handle),
                )
                .route(
                    "/organizations/{uuid_subject}/",
                    get().to(get_organizations_handle),
                )
                .route(
                    "/purchasing_subject_group/",
                    get().to(get_purchasing_subject_group_handle),
                ),
        )
        .service(
            web::scope("/pre_request")
                .route(
                    "/request_price_info_close/",
                    post().to(pre_price_info_close_handle),
                )
                .route(
                    "/organization_question/",
                    post().to(pre_organization_question),
                ),
        )
        .service(
            web::scope("/update")
                .route("/proposal/", post().to(update_proposal_handler))
                .route("/request_price_info/", post().to(update_price_info_handle))
                .route("/organizations/", post().to(update_organizations_handler))
                .route(
                    "/purchasing_subject_group/",
                    post().to(update_purchasing_subject_group_handler),
                )
                .route(
                    "/purchasing_subject/",
                    post().to(update_purchasing_subject_handler),
                ),
        )
        .service(
            web::scope("/export").route("/table/", post().to(export_table_handler)),
        )
        .route(
            "/create_price_information_request",
            post().to(create_price_information_request_handle),
        )
        .route(
            "/get_price_information_request_by_plan_uuid/{uuid}/",
            post().to(get_price_information_request_by_plan_uuid_handle),
        )
        .route(
            "/get_price_information_request_by_plan_uuid_vec/",
            post().to(get_price_information_request_by_plan_uuid_vec_handle),
        )
        .route(
            "/get_tkp_by_request_uuid/{uuid}/",
            post().to(get_tkp_by_request_uuid_handle),
        )
        .route(
            "/get_tkp_by_request_uuid_vec/",
            post().to(get_tkp_by_request_uuid_vec_handle),
        )
        .route(
            "/send_for_proposal_price_request/",
            post().to(send_for_proposal_handle),
        )
        .route("/complete_price_request/", post().to(complete_handle))
        .route("/tkp_reject/", post().to(tkp_reject_handle))
        .route("/tkp_verified/", post().to(tkp_verified_handle))
        .route("/create_report/", post().to(create_price_request_report_handle));
    let monitoring_scope = web::scope("/monitoring")
        .route("/config", get().to(config_handler))
        .route("/test", get().to(healthcheck_handler))
        .route("/config/", get().to(config_handler))
        .route("/test/", get().to(healthcheck_handler));
    cfg.service(api_scope).service(monitoring_scope);
}

/// Настройка CORS политики сервиса
pub fn setup_cors() -> Cors {
    Cors::default()
        .allow_any_origin()
        .allow_any_header()
        .allow_any_method()
        .supports_credentials()
        // кэши
        // https://blog.gelin.ru/2018/12/cors.html
        .disable_vary_header()
        .max_age(MAX_AGE_CORS_CACHE)
}

/// Хендлер для проверки, жив ли сервис или нет
pub async fn healthcheck_handler() -> String {
    "Technical Commercial Proposal is alive".into()
}

/// Получение конфигурации сервера
pub async fn config_handler() -> Json<ServerConfig> {
    let server_cfg = ServerConfig::new();
    Json(server_cfg)
}
