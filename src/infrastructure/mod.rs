use broker::rabbit::RabbitAdapter;
use monolith_service::http::{MonolithHttpDriver, MonolithHttpService};
use monolith_service::MonolithService;
/// ! Infrastructure is a layer that works with third–party
/// ! libraries, frameworks, and so on.
use sqlx::PgPool;
use std::sync::Arc;

mod env;
pub use env::Env;

pub use shared_essential::infrastructure::rabbit::setup_rabbit_adapter;

pub mod web;
pub(crate) use web::setup_routers;

pub(crate) mod service_interaction;

pub(crate) mod rabbit;

/// Global config for whole service
#[derive(Clone, Debug)]
pub struct GlobalConfig {
    pub env: Env,
    pub db_pool: Arc<PgPool>,
    pub broker_adapter: Arc<RabbitAdapter>,
    pub monolith_service: Arc<MonolithHttpService>,
}

impl GlobalConfig {
    pub fn new(
        env: Env,
        broker_adapter: RabbitAdapter,
        monolith_driver: MonolithHttpDriver,
        db_pool: PgPool,
    ) -> Self {
        Self {
            env,
            db_pool: Arc::new(db_pool),
            broker_adapter: Arc::new(broker_adapter),
            monolith_service: Arc::new(MonolithService::new(monolith_driver)),
        }
    }

    pub fn rabbit_adapter(&self) -> Arc<RabbitAdapter> {
        self.broker_adapter.clone()
    }

    pub fn monolith_service(&self) -> Arc<MonolithHttpService> {
        self.monolith_service.clone()
    }

    pub fn db_pool(&self) -> Arc<PgPool> {
        self.db_pool.clone()
    }
}
