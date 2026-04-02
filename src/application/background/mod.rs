pub(crate) mod request_header_closing;

use futures::Future;
use std::time::Duration;
use tokio::time::sleep;

use humantime::format_duration;
use shared_essential::presentation::dto::technical_commercial_proposal::TcpResult;

use crate::infrastructure::GlobalConfig;
use request_header_closing::request_header_closing;

/// Конфигурация и запуск фоновых задач
pub(crate) fn start_background_tasks(config: &GlobalConfig) {
    let GlobalConfig { db_pool, .. } = config.clone();
    let tech_user = config.env.monolith_config.tech_user_id;

    [PeriodicTask::new(
        "Закрытие ЗЦИ",
        Duration::from_secs(60 * 60),
        move || request_header_closing(db_pool.clone(), tech_user),
    )]
    .into_iter()
    .for_each(PeriodicTask::run);
}

// Фоновая задача, которая выполняется раз в какое то время
struct PeriodicTask<F> {
    name: &'static str,
    interval: Duration,
    task: F,
}

impl<F> PeriodicTask<F> {
    fn new(name: &'static str, interval: Duration, task: F) -> Self {
        Self {
            name,
            interval,
            task,
        }
    }

    fn run<Fut>(self)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = TcpResult<()>> + Send + 'static,
    {
        tokio::spawn(async move {
            let Self {
                interval,
                name,
                task,
            } = self;

            loop {
                tracing::info!("Выполнение фоновой периодичной задачи {}", name);

                if let Err(error) = task().await {
                    tracing::info!("Ошибка запуска задачи `{}`: {}", name, error);
                }

                tracing::info!(
                    "Следущее выполнение задачи `{}` запланировано через {:?}",
                    name,
                    format_duration(interval)
                );

                sleep(interval).await;
            }
        });
    }
}
