use amqprs::BasicProperties;
use serde::Serialize;
use std::fmt::Debug;
use std::sync::Arc;
use tracing::{error, info, Instrument};

use amqprs::channel::{
    BasicConsumeArguments, BasicPublishArguments, QueueDeclareArguments,
};
use broker::rabbit::{RabbitAdapter, RabbitConsumer};
use broker::{BrokerAdapter, Consumer, Publisher};

use rabbit_services::{properties::get_rabbit_span, TCP_QUEUE};

use shared_essential::presentation::dto::technical_commercial_proposal::*;
use shared_essential::presentation::dto::{AsezError, AsezResult};

use crate::infrastructure::GlobalConfig;
use crate::presentation::rabbit_handlers::*;

macro_rules! handle_action {
    ($fun: expr, $dto: expr, $properties: expr, $config: expr) => {{
        let config = $config.clone();
        let span = get_rabbit_span(&$properties);

        tokio::spawn(
            async move {
                let res = $fun($dto, &config.db_pool).await;

                let reply_to = $properties.reply_to().map(|s| s.as_str());
                if let Err(error) =
                    publish_response(res, reply_to, config.broker_adapter.clone())
                        .await
                {
                    error!(kind = "tcp", "Publish error: {}", error);
                }
            }
            .instrument(span),
        );
    }};
}

pub async fn start_rabbit_listener(config: Arc<GlobalConfig>) -> TcpResult<()> {
    let mut tcp_consumer =
        register_consumer(&config.broker_adapter, TCP_QUEUE).await?;

    tokio::spawn(async move {
        loop {
            tokio::select! {
                tcp_request = tcp_consumer.consume() => {
                     match tcp_request {
                        Ok(rabbit_message) => {
                          info!(kind = "tcp", "Процессинг действия: {:?}", &rabbit_message);
                          match rabbit_message.content {
                            TcpDataAction::CommercialOfferRequestConfirmation(dto) => handle_action!(handle_commercial_offer_request_confirmation, dto, rabbit_message.properties, &config),
                            TcpDataAction::CommercialOfferResponse(dto) => handle_action!(handle_commercial_offer_response, dto, rabbit_message.properties, &config),
                            TcpDataAction::CommercialOfferAddDocResponse(dto) => handle_action!(handle_commercial_offer_add_doc_response, dto, rabbit_message.properties, &config),
                          }
                        }
                        Err(err) => {
                            error!(kind = "tcp", "Ошибка при получении сообщения: {:?}", err);
                        }
                    }
                }
            }
        }
    });

    Ok(())
}

pub async fn declare_queues(adapter: &RabbitAdapter) -> TcpResult<()> {
    adapter
        .declare_queue(
            QueueDeclareArguments::default()
                .queue(TCP_QUEUE.into())
                .durable(true)
                .finish(),
        )
        .await?;
    Ok(())
}

pub async fn register_consumer(
    adapter: &RabbitAdapter,
    queue_name: &str,
) -> TcpResult<RabbitConsumer> {
    let consumer = adapter
        .register_consumer(BasicConsumeArguments::new(queue_name, "tcp-consumer"))
        .await?;
    Ok(consumer)
}

pub(crate) async fn publish_response<R>(
    search_result: TcpResult<R>,
    reply_to: Option<&str>,
    config: Arc<RabbitAdapter>,
) -> TcpResult<()>
where
    R: Debug + Serialize + Send + Sync,
{
    let asez_result: AsezResult<R> = search_result.map_err(AsezError::new);

    let reply_to = reply_to.unwrap_or_default();
    let basic_props = BasicProperties::default()
        .with_content_type("application/json")
        .with_persistence(true)
        .finish();

    let publish_props = BasicPublishArguments::new("", reply_to);

    let basic_publisher =
        config.register_publisher(basic_props, publish_props).await?;

    info!(kind = "tcp", "Отправление ответа: {:?}", &asez_result);

    basic_publisher.publish(&asez_result).await?;

    Ok(())
}
