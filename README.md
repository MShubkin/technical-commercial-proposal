# Модуль «Технико-коммерческое предложение» (ТКП)

Микросервис системы АСЭЗ 2.0, реализующий полный жизненный цикл запросов ценовой информации (ЗЦИ) и технико-коммерческих предложений (ТКП) от поставщиков.

## Содержание

- [Описание](#описание)
- [Технологии](#технологии)
- [Структура проекта](#структура-проекта)
- [Настройка окружения](#настройка-окружения)
- [Запуск](#запуск)
- [API](#api)
- [RabbitMQ](#rabbitmq)
- [Мониторинг](#мониторинг)

---

## Описание

Модуль отвечает за:

- Создание и управление запросами ценовой информации (ЗЦИ)
- Приём и проверку технико-коммерческих предложений от поставщиков
- Публикацию и завершение ЗЦИ
- Управление партнёрами, предметами закупки и группами
- Экспорт данных в табличные форматы
- Фоновое закрытие устаревших запросов

---

## Технологии

| Компонент        | Технология                           |
|------------------|--------------------------------------|
| Язык             | Rust 2021 edition                    |
| HTTP-фреймворк   | Actix-web                            |
| Async runtime    | Tokio                                |
| База данных      | PostgreSQL (SQLx)                    |
| Брокер сообщений | RabbitMQ (amqprs)                    |
| Логирование      | tracing / CEF-формат                 |
| Контейнеризация  | Docker (Astra Linux 1.7.5)           |

---

## Структура проекта

```
src/
├── main.rs                        # Точка входа
├── application/
│   ├── calls/                     # Бизнес-логика (use-cases)
│   ├── action/                    # Обработчики действий (ТКП)
│   ├── background/                # Фоновые задачи
│   └── tests/                     # Тесты
├── presentation/
│   ├── http_calls.rs              # HTTP-обработчики (30+)
│   ├── rabbit_handlers.rs         # Обработчики RabbitMQ
│   └── dto.rs                     # DTO запросов/ответов
├── infrastructure/
│   ├── web.rs                     # Роутинг и CORS
│   ├── rabbit.rs                  # Настройка RabbitMQ-консьюмера
│   ├── env.rs                     # Инициализация окружения
│   └── service_interaction/       # Клиенты внешних сервисов
└── domain/
    └── mod.rs                     # Обогащение доменных данных
```

Миграции базы данных находятся в директории `migrations/` (формат SQLx).

---

## Настройка окружения

Скопируйте `.env` и задайте переменные:

```env
# Сервер
SRV_HOST=0.0.0.0
SRV_PORT=3078
SRV_THREAD_COUNT=7

# PostgreSQL
POSTGRES_VHOST=localhost
POSTGRES_PORT=5432
POSTGRES_DB=postgres
POSTGRES_USER=postgres
POSTGRES_PASSWORD=postgres
POSTGRES_MIN_CONNECTIONS=1
POSTGRES_MAX_CONNECTIONS=4
POSTGRES_TIMEOUT_S=10
POSTGRES_CONNECTION_REFRESH_S=3600

# RabbitMQ
RABBITMQ_HOST=localhost
RABBITMQ_VHOST=/
RABBITMQ_PORT=5672
RABBITMQ_USERNAME=guest
RABBITMQ_PASSWORD=guest
RABBITMQ_RETRIES=10
RABBITMQ_INTERVAL_MS=100

# Внешние сервисы
MONOLITH_BASE_URL=http://localhost:8080
MASTER_DATA_BASE_URL=http://localhost:9071
MONOLITH_TECH_USER_ID=10200
PLAN_DB_HOST=127.0.0.1
PLAN_DB_PORT=3004

# Логирование: cef (продакшн) | dev | любое другое значение — без логгера
LOGGER_MODE=cef
LOGGER_DIR=logs
LOGGER_FILE=log.cef
RUST_LOG=debug
```

---

## Запуск

### Локально

```bash
# Применить миграции
sqlx migrate run

# Собрать и запустить
cargo run -p technical-commercial-proposal
```

### Docker

```bash
docker build \
  --build-arg GIT_COMMIT_ID=$(git rev-parse HEAD) \
  -t technical-commercial-proposal .

docker run --env-file .env -p 3078:3078 technical-commercial-proposal
```

### Сборка релизного бинаря

```bash
cargo build -p technical-commercial-proposal --release --locked
```

---

## API

Все маршруты расположены под префиксом `/v1`.

### Запросы ценовой информации (ЗЦИ)

| Метод  | Маршрут                                              | Описание                          |
|--------|------------------------------------------------------|-----------------------------------|
| POST   | `/v1/create_price_information_request`               | Создать ЗЦИ                       |
| POST   | `/v1/get/request_price_info_list/`                   | Список ЗЦИ                        |
| POST   | `/v1/get/request_price_info_detail/`                 | Детали ЗЦИ                        |
| POST   | `/v1/update/request_price_info/`                     | Обновить ЗЦИ                      |
| DELETE | `/v1/delete/request_price_info/`                     | Удалить ЗЦИ                       |
| POST   | `/v1/action/request_price_info_publication/`         | Опубликовать ЗЦИ                  |
| POST   | `/v1/action/request_price_info_close/`               | Закрыть ЗЦИ                       |
| POST   | `/v1/action/request_price_info_complete/`            | Завершить рассмотрение ЗЦИ        |
| POST   | `/v1/pre_request/request_price_info_close/`          | Предварительная проверка закрытия |
| POST   | `/v1/check/request_price_info/`                      | Проверить ЗЦИ на ошибки           |

### Технико-коммерческие предложения (ТКП)

| Метод | Маршрут                                              | Описание                              |
|-------|------------------------------------------------------|---------------------------------------|
| POST  | `/v1/get/proposal_detail/`                           | Детали ТКП                            |
| POST  | `/v1/get/proposal_list_by_object_id/`                | ТКП по объекту закупки                |
| POST  | `/v1/get/proposal_items_for_pricing/`                | Позиции ТКП для ценообразования       |
| POST  | `/v1/get/technical_commercial_proposal/`             | Получить ТКП                          |
| POST  | `/v1/update/proposal/`                               | Обновить ТКП                          |
| POST  | `/v1/action/proposal_approve/`                       | Утвердить ТКП                         |
| POST  | `/v1/action/proposal_apply_pricing_consider/`        | Применить ценообразование             |
| POST  | `/v1/send_for_proposal_price_request/`               | Отправить ТКП на ценообразование      |
| POST  | `/v1/complete_price_request/`                        | Завершить ценовой запрос              |
| POST  | `/v1/tkp_reject/`                                    | Отклонить ТКП                         |
| POST  | `/v1/tkp_verified/`                                  | Подтвердить ТКП                       |

### Партнёры

| Метод | Маршрут                    | Описание                    |
|-------|----------------------------|-----------------------------|
| POST  | `/v1/check/add_partner/`   | Проверить добавление партнёра |
| POST  | `/v1/check/delete_partner/`| Проверить удаление партнёра |

### Настройки

| Метод | Маршрут                                        | Описание                             |
|-------|------------------------------------------------|--------------------------------------|
| POST  | `/v1/update/organizations/`                    | Обновить организации                 |
| POST  | `/v1/action/organizations_remove/`             | Удалить организации                  |
| GET   | `/v1/get/organizations/{uuid_subject}/`        | Получить организации по предмету     |
| POST  | `/v1/update/purchasing_subject_group/`         | Обновить группу предметов закупки    |
| POST  | `/v1/action/purchasing_subject_group_remove/`  | Удалить группу предметов закупки     |
| GET   | `/v1/get/purchasing_subject_group/`            | Список групп предметов закупки       |
| POST  | `/v1/update/purchasing_subject/`               | Обновить предмет закупки             |
| POST  | `/v1/action/purchasing_subject_remove/`        | Удалить предмет закупки              |
| GET   | `/v1/get/purchasing_subject_by_group_uuid/{uuid}/` | Предметы закупки по группе       |

### Экспорт и отчёты

| Метод | Маршрут                    | Описание            |
|-------|----------------------------|---------------------|
| POST  | `/v1/export/table/`        | Экспорт таблицы     |
| POST  | `/v1/create_report/`       | Создать отчёт       |

### Вспомогательные маршруты

| Метод | Маршрут                                                  | Описание                            |
|-------|----------------------------------------------------------|-------------------------------------|
| POST  | `/v1/get_price_information_request_by_plan_uuid/{uuid}/` | ЗЦИ по UUID плана                   |
| POST  | `/v1/get_price_information_request_by_plan_uuid_vec/`    | ЗЦИ по списку UUID планов           |
| POST  | `/v1/get_tkp_by_request_uuid/{uuid}/`                    | ТКП по UUID запроса                 |
| POST  | `/v1/get_tkp_by_request_uuid_vec/`                       | ТКП по списку UUID запросов         |
| POST  | `/v1/pre_request/organization_question/`                 | Вопрос организации (предзапрос)     |

---

## RabbitMQ

### Очереди, с которыми сервис взаимодействует

| Очередь      | Направление | Описание                                         |
|--------------|-------------|--------------------------------------------------|
| `processing` | Входящие    | Получение данных из части процессинга (сервис `processing`) |

Сервис не создаёт собственных очередей.

---

## Мониторинг

| Маршрут              | Описание                  |
|----------------------|---------------------------|
| `GET /monitoring/test`   | Healthcheck — возвращает `200 OK` |
| `GET /monitoring/config` | Текущая конфигурация сервера      |

---

## Авторы

Nikolay Galko, Ibragim Kusov
