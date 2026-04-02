use sqlx::PgPool;

use asez2_shared_db::{result::SharedDbError, PgDbOptions};
use env_setup::*;
use trace_setup::TracingKind;

#[derive(Clone, Debug)]
pub struct Env {
    pub url: (String, u16),
    pub rabbit_config: RabbitCfg,
    pub postgres_cfg: PgDbOptions,
    pub logger: TracingKind,
    pub monolith_config: MonolithCfg,
    pub json_config: JsonConfig,
}

impl Env {
    pub fn setup() -> Result<Self, EnvError> {
        let port = try_get!(SRV_PORT, SRV_PORT_DEFAULT_VALUE, u16)?;
        let host =
            var(SRV_HOST).unwrap_or_else(|_| SRV_HOST_DEFAULT_VALUE.to_owned());

        let rabbit_config = RabbitCfg::from_env()?;
        let postgres_cfg = PostgresCfg::from_env()?.into();
        let logger = TracingCfg::from_env()?.tracing_kind;
        let monolith_config = MonolithCfg::from_env()?;
        let json_config = JsonConfig::from_env()?;

        Ok(Self {
            url: (host, port),
            rabbit_config,
            postgres_cfg,
            logger,
            monolith_config,
            json_config,
        })
    }

    pub(crate) async fn setup_postgres_pool(
        &self,
    ) -> Result<PgPool, SharedDbError> {
        self.postgres_cfg.get_pool().await
    }
}
