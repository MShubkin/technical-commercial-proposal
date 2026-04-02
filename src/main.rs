//#![allow(dead_code)]

use std::sync::Arc;

use actix_web::middleware::{NormalizePath, TrailingSlash};
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use monolith_service::http::MonolithHttpDriver;
use shared_essential::presentation::dto::Source;
use tracing::info;

use crate::application::background::start_background_tasks;
use crate::infrastructure::web::setup_cors;
use crate::infrastructure::{
    rabbit::{declare_queues, start_rabbit_listener},
    setup_rabbit_adapter, setup_routers, Env, GlobalConfig,
};

mod application;
mod common;
mod domain;
mod infrastructure;
mod presentation;
#[cfg(test)]
mod tests;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let env = Env::setup()?;

    let _logger_guard = env.logger.initiate_log(
        "srm",
        &env.url.0,
        env.url.1,
        &["infra", "broker", "tcp", "siem"],
    )?;

    info!(kind = "infra", "Setup rabbit adapter...");
    let rabbit_adapter = setup_rabbit_adapter(&env.rabbit_config).await?;
    declare_queues(&rabbit_adapter).await?;

    info!(kind = "infra", "Setup postgress pool");
    let db_pool = env.setup_postgres_pool().await?;

    let monolith_driver =
        MonolithHttpDriver::basic_driver(env.monolith_config.url.clone())?;
    info!(kind = "infra", "Настройка monolith драйвера");

    let url = env.url.clone();
    let global_config =
        Arc::new(GlobalConfig::new(env, rabbit_adapter, monolith_driver, db_pool));

    tracing::info!(kind = "infra", "Запуск фоновых задач...");
    start_background_tasks(global_config.as_ref());

    info!(kind = "infra", "Register consumer");
    start_rabbit_listener(global_config.clone()).await?;

    info!(kind = "infra", "TKP service started at {}:{}", url.0, url.1);

    HttpServer::new(move || {
        App::new()
            .app_data(Source::TechnicalCommercialProposal)
            .app_data(Data::from(global_config.clone()))
            .app_data(Data::from(global_config.rabbit_adapter()))
            .app_data(Data::from(global_config.monolith_service()))
            .app_data(Data::from(global_config.db_pool()))
            .app_data(global_config.env.json_config.for_actix_web())
            .wrap(setup_cors())
            .wrap(NormalizePath::new(TrailingSlash::Always))
            .wrap(http_middleware::default_cookie_decoder())
            .configure(setup_routers)
    })
    .bind(url.clone())?
    .run()
    .await
    .expect("Starting service error");

    info!(kind = "infra", "TKP service is stopped");

    Ok(())
}
