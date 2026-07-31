# SERVICE_TEMPLATE — стандарт написания Rust-сервисов

Документ описывает архитектуру, структуру файлов и стиль кода для HTTP-сервисов
на Rust. Эталонная реализация — `task-gateway`; ссылки на реальные файлы даны в
каждом разделе.

Документ нормативный: если код расходится с шаблоном — правится код, а не
шаблон. Раздел 12 перечисляет известные расхождения в текущем репозитории.

**Как читать.** Разделы 1–7 — как устроен сервис. Разделы 8–9 — чеклисты для
типовых задач (новый внешний модуль, новый эндпоинт). Раздел 13 — bootstrap
нового сервиса с нуля.

В шаблонах кода `<module>` / `<Module>` / `<Client>` / `<service>` — плейсхолдеры,
подставляются реальными именами. Такие блоки не компилируются как есть.

---

## 0. Шпаргалка

1. Библиотека + бинарник: логика в `src/lib.rs`, запуск в `src/bin/run_server.rs`.
2. Внешний мир скрыт за трейтом. Трейты — в `src/modules/mod.rs`, реализации —
   в подмодулях. Хендлеры generic по трейтам, не по конкретным клиентам.
3. Подключение внешнего клиента — только через трейт `ServiceConnect`.
4. У каждого модуля свой `enum <Module>Errors`. Наверх — через
   `From<<Module>Errors> for ServerError`. В хендлере `?`, не `match`.
5. Поля структур приватные, доступ через `getset`, сборка через `new(...)`.
6. Конфиг валидируется на старте: `validate()` вызывается из `ServiceConfig::new()`.
   Источники: `config/development.toml` → `config/<run_mode>.toml` → env.
7. Никаких `unwrap()` / `expect()` / `panic!` в `src/` — только в тестах.
8. Публичный контракт описывается `utoipa`-аннотацией рядом с хендлером.
9. Публичные тексты (API, ошибки, Swagger, README, комментарии в коде) — на
   английском. Внутренняя документация для команды — на русском.
10. Тесты вызывают хендлер напрямую с фейковыми реализациями трейтов.

---

## 1. Назначение и область применения

Шаблон рассчитан на сервис, который:

- принимает HTTP-запросы (`axum`);
- ходит в один или несколько внешних ресурсов (брокер, HTTP-сервис, БД, S3);
- отдаёт OpenAPI-документацию, метрики Prometheus и трейсы OpenTelemetry;
- конфигурируется файлами + переменными окружения;
- запускается в Docker.

Если сервис проще (например, чистый воркер без HTTP) — слой `server/` опускается,
остальное сохраняется без изменений.

---

## 2. Технологический стек

Базовый набор зависимостей. Версии — актуальные на момент создания сервиса,
`edition = "2024"`.

| Назначение | Крейт |
| --- | --- |
| HTTP-сервер | `axum` |
| Асинхронный рантайм | `tokio` (features = `["full"]`) |
| Трейты в async | `async-trait` |
| Ошибки модулей | `thiserror` |
| Ошибки в `main` | `anyhow` |
| Сериализация | `serde`, `serde_json` |
| Конфигурация | `config`, `dotenv` |
| Геттеры/сеттеры | `getset` |
| Билдеры | `derive_builder` |
| OpenAPI | `utoipa`, `utoipa-swagger-ui` |
| Логи | `tracing`, `tracing-subscriber`, `tracing-loki` |
| Метрики | `axum-prometheus`, `metrics-exporter-prometheus` |
| Трейсинг | `axum-tracing-opentelemetry` |
| HTTP middleware | `tower-http` (features = `["trace", "cors"]`) |
| Идентификаторы | `uuid` (features = `["v4", "serde"]`) |
| Время | `chrono` (features = `["serde"]`) |
| Ретраи | `backon` |

Клиенты подключаются по потребности: `lapin` (RabbitMQ), `reqwest` (HTTP),
`sqlx` (SQL), и т. д.

Зависимости с features выносятся в отдельные секции `[dependencies.<name>]`, а не
пишутся инлайн — так `Cargo.toml` остаётся читаемым.

---

## 3. Структура каталогов

```text
<service>/
├── Cargo.toml
├── Dockerfile
├── README.md                     # публичное описание сервиса, на английском
├── AGENTS.md                     # краткие правила для агентов + ссылка сюда
├── .env.example                  # список переменных без секретов
├── .env.development
├── .github/workflows/            # pull-request.yml, create_release.yml
├── config/
│   └── development.toml          # база конфигурации, коммитится
├── docker-compose/
│   ├── docker-compose.yaml
│   ├── .env.<service>
│   └── <infra>/                  # конфиги инфраструктуры (rabbitmq, и т. п.)
├── docs/
│   ├── SERVICE_TEMPLATE.md
│   └── Context.drawio / Context.png
├── src/
│   ├── lib.rs                    # объявление модулей + трейт ServiceConnect
│   ├── config.rs                 # ServiceConfig: сборка всей конфигурации
│   ├── errors.rs                 # общие типы + From<> во внешние ошибки
│   ├── logger.rs                 # LoggerConfig + init_logger
│   ├── bin/
│   │   └── run_server.rs         # единственная точка входа
│   ├── modules/
│   │   ├── mod.rs                # трейты-абстракции внешнего мира
│   │   └── <module>/
│   │       ├── mod.rs            # объявление подмодулей
│   │       ├── config.rs         # <Module>Config + validate()
│   │       ├── errors.rs         # <Module>Errors + From<serde_json::Error>
│   │       ├── models/mod.rs     # доменные модели + <Module>Result<T>
│   │       └── <client>/
│   │           ├── mod.rs        # структура клиента + impl ServiceConnect
│   │           ├── core.rs       # impl трейта из modules/mod.rs
│   │           └── errors.rs     # From<ошибка_крейта> for <Module>Errors
│   └── server/
│       ├── mod.rs                # AppState + init_server
│       ├── config.rs             # ServerConfig
│       ├── errors.rs             # ServerError + IntoResponse + From<>
│       ├── swagger.rs            # ApiDoc + трейт SwaggerExample
│       └── router/
│           ├── mod.rs
│           ├── models.rs         # DTO запросов/ответов
│           └── <domain>/
│               ├── mod.rs
│               └── <action>.rs   # один эндпоинт = один файл
└── tests/
    ├── <action>_handler.rs       # тесты хендлеров
    └── <module>_models.rs        # тесты моделей и конфигов
```

Правила:

- **Один эндпоинт — один файл.** Файл называется по имени хендлера
  (`publish_message.rs` → `pub async fn publish_message`).
- **Один внешний ресурс — один каталог в `modules/`.** Каталог назван по роли в
  системе (`broker`, `state_manager`), а не по технологии.
- **Одна реализация — один вложенный каталог** внутри модуля, названный по
  технологии (`rabbitmq`, `webhook_manager`). Так к абстракции добавляется
  вторая реализация без переписывания вызывающего кода.
- `mod.rs` содержит только объявления `pub mod` и, при необходимости, главную
  структуру уровня. Логика — в соседних файлах.

---

## 4. Слои и правила зависимостей

```text
bin/run_server.rs      сборка: config → logger → клиенты → AppState → Router
        │
        ▼
server/                HTTP: роутер, хендлеры, DTO, Swagger, ServerError
        │  (знает только про трейты из modules/mod.rs)
        ▼
modules/mod.rs         трейты-абстракции: BrokerProducer, StateManager, ...
        │
        ▼
modules/<module>/      конфиг, ошибки, доменные модели модуля
        │
        ▼
modules/<module>/<client>/   конкретная технология: lapin, reqwest, sqlx
```

Направление зависимостей — строго вниз.

- `server/` **не импортирует** `RabbitMQProducer`, `WebhookManager`, `reqwest`,
  `lapin`. Только трейты и доменные модели модулей.
- `modules/` **не импортирует** ничего из `server/`, кроме одного исключения:
  преобразование `From<<Module>Errors> for ServerError` живёт в
  `server/errors.rs` — то есть на стороне сервера, а не модуля.
- Конкретную реализацию выбирает **только** `bin/run_server.rs`. Это
  единственное место, где сервис знает, что брокер — именно RabbitMQ.

Практический критерий: замена RabbitMQ на Kafka должна затрагивать
`modules/broker/kafka/*` и одну строку в `run_server.rs`. Если приходится трогать
хендлеры — слои нарушены.

---

## 5. Шаблоны файлов

### 5.1 `src/lib.rs`

Объявляет модули и общий трейт подключения к внешним сервисам.

```rust
pub mod config;
mod errors;
pub mod logger;
pub mod modules;
pub mod server;

#[async_trait::async_trait]
pub trait ServiceConnect {
    type Config;
    type Error;
    type Client;

    async fn connect(config: &Self::Config) -> Result<Self::Client, Self::Error>;
}
```

`ServiceConnect` — единый вход для всех внешних клиентов: одна сигнатура
подключения, одинаковый вид кода в `main`, единообразная обработка ошибок старта.

Живой пример: [src/lib.rs](../src/lib.rs).

### 5.2 `src/modules/mod.rs` — абстракции

Один трейт на один внешний ресурс. Методы описывают **намерение** домена
(`publish`, `create_task`), а не транспорт (`send_amqp_frame`, `post_json`).

```rust
use crate::modules::{
    <module>::models::{<Module>Result, <Payload>},
};

pub mod <module>;

#[async_trait::async_trait]
pub trait <Module>Trait {
    async fn <action>(&self, payload: <Payload>) -> <Module>Result<<Output>>;
}
```

Требования к трейту:

- методы `async`, через `#[async_trait::async_trait]`;
- возвращают `<Module>Result<T>`, а не `anyhow::Result`;
- принимают доменные модели модуля, а не DTO из `server/router/models.rs`;
- `&self`, не `&mut self` — клиенты хранятся в `Arc` и шарятся между задачами.

Живой пример: [src/modules/mod.rs](../src/modules/mod.rs).

### 5.3 `src/modules/<module>/models/mod.rs` — модели и алиас результата

```rust
use getset::Getters;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::modules::<module>::errors::<Module>Errors;

pub type <Module>Result<T> = Result<T, <Module>Errors>;

#[derive(Serialize, Deserialize, Getters, Debug, Clone, PartialEq)]
#[getset(get = "pub")]
pub struct <Payload> {
    task_id: Uuid,
    user_id: String,
    payload: serde_json::Value,
}

impl <Payload> {
    pub fn new(task_id: Uuid, user_id: String, payload: serde_json::Value) -> Self {
        Self { task_id, user_id, payload }
    }
}
```

Алиас `<Module>Result<T>` объявляется **в `models/mod.rs`**, рядом с моделями,
а не в `errors.rs`.

**Newtype для доменных строк.** Строковый идентификатор с семантикой (ключ
маршрутизации, имя сервиса) оборачивается в newtype с `#[serde(transparent)]` —
типобезопасность внутри, обычная строка в JSON:

```rust
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct TaskType(String);

impl TaskType {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
```

Живой пример: [src/modules/broker/models/mod.rs](../src/modules/broker/models/mod.rs).

### 5.4 `src/modules/<module>/config.rs`

```rust
use getset::Getters;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct <Module>Config {
    address: String,
    <field>: String,
}

impl <Module>Config {
    pub fn new(address: impl Into<String>, <field>: impl Into<String>) -> Self {
        Self { address: address.into(), <field>: <field>.into() }
    }

    /// Called from ServiceConfig::new(); the service must not start with an
    /// invalid configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.address.trim().is_empty() {
            return Err("<module>.address must not be empty".to_string());
        }
        Ok(())
    }
}
```

Правила:

- поля приватные, читаются геттерами `getset`;
- `new(...)` принимает `impl Into<String>` — конструктор удобно вызывать из
  тестов литералами;
- `validate()` возвращает `Result<(), String>` с **точным** текстом ошибки,
  включающим путь к полю (`<module>.routes[3].exchange must not be empty`);
- `Debug` выводится только там, где в конфиге нет секретов. Для полей с
  паролями/токенами `Debug` реализуется вручную с маскированием.

Живой пример с проверкой коллекции: [src/modules/broker/config.rs](../src/modules/broker/config.rs).

### 5.5 `src/modules/<module>/errors.rs`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum <Module>Errors {
    #[error("<Module> configuration error: {0}")]
    ConfigurationError(String),
    #[error("<Module> is unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("IO Error: {0}")]
    IOError(String),
    #[error("Deserialize Error {0}")]
    DeserializeError(String),
    #[error("Serialize Error {0}")]
    SerializeError(String),
    #[error("Not Found Error: {0}")]
    NotFoundError(String),
    #[error("Another Error: {0}")]
    AnotherError(String),
}

impl From<serde_json::Error> for <Module>Errors {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializeError(format!("Serde Serialization error: {}", err))
    }
}
```

Правила:

- варианты несут `String`, а не исходную ошибку — модуль не протекает наружу
  типами своих зависимостей;
- набор вариантов — по **семантике сбоя** (недоступен, не найден, не авторизован,
  ошибка сериализации), а не по типу исключения;
- `AnotherError` — обязательный catch-all для непредусмотренных случаев;
- тексты — на английском.

Живой пример: [src/modules/broker/errors.rs](../src/modules/broker/errors.rs).

### 5.6 `src/modules/<module>/<client>/mod.rs` — клиент и подключение

Структура клиента + реализация `ServiceConnect`. Поля — в `Arc`, чтобы клиент
дёшево клонировался и шарился между задачами.

```rust
use std::sync::Arc;

use crate::ServiceConnect;
use crate::modules::<module>::config::<Module>Config;
use crate::modules::<module>::errors::<Module>Errors;

pub mod core;
pub mod errors;

pub struct <Client> {
    config: Arc<<Module>Config>,
    connection: Arc<<ExternalClient>>,
}

#[async_trait::async_trait]
impl ServiceConnect for <Client> {
    type Config = <Module>Config;
    type Error = <Module>Errors;
    type Client = Self;

    async fn connect(config: &Self::Config) -> Result<Self::Client, Self::Error> {
        tracing::debug!("Creating <Client>...");
        config.validate().map_err(<Module>Errors::ConfigurationError)?;

        let connection = <ExternalClient>::connect(config.address()).await?;

        tracing::info!(address = ?config.address(), "Connected to <Module>");
        Ok(Self {
            config: Arc::new(config.to_owned()),
            connection: Arc::new(connection),
        })
    }
}
```

Живые примеры: [src/modules/broker/rabbitmq/mod.rs](../src/modules/broker/rabbitmq/mod.rs),
[src/modules/state_manager/mod.rs](../src/modules/state_manager/mod.rs).

### 5.7 `src/modules/<module>/<client>/core.rs` — реализация трейта

Файл содержит **только** `impl <Module>Trait for <Client>`. Вся работа с внешним
крейтом сосредоточена здесь.

```rust
use crate::modules::<Module>Trait;
use crate::modules::<module>::errors::<Module>Errors;
use crate::modules::<module>::models::{<Module>Result, <Payload>};
use crate::modules::<module>::<client>::<Client>;

#[async_trait::async_trait]
impl <Module>Trait for <Client> {
    async fn <action>(&self, payload: <Payload>) -> <Module>Result<<Output>> {
        // 1. Достать нужную часть конфигурации, ошибку — через ok_or_else.
        // 2. Выполнить внешний вызов, ошибки крейта — через `?`
        //    (работает благодаря From<> в errors.rs).
        // 3. Проверить ответ и вернуть доменный результат.
        // 4. tracing::info! на успешном завершении значимой операции.
        todo!()
    }
}
```

Живой пример: [src/modules/broker/rabbitmq/core.rs](../src/modules/broker/rabbitmq/core.rs).

### 5.8 `src/modules/<module>/<client>/errors.rs` — мост к ошибкам крейта

Перевод ошибок внешнего крейта в семантику модуля. Здесь — и только здесь —
допустим `match` по типам ошибок зависимости.

```rust
impl From<<CrateError>> for <Module>Errors {
    fn from(err: <CrateError>) -> Self {
        match err.kind() {
            <Kind>::Auth(error) => Self::Unauthorized(format!("Unauthorized: {}", error)),
            <Kind>::Io(_) => Self::IOError(format!("IO Error: {}", err)),
            _ => Self::AnotherError(format!("Another Error: {}", err)),
        }
    }
}
```

Для HTTP-клиента ошибка классифицируется по статусу и типу сбоя — см.
[src/modules/state_manager/errors.rs](../src/modules/state_manager/errors.rs):
`404 → NotFoundError`, `408/429/502/503/504 → ServiceUnavailable`,
`is_timeout() || is_connect() → ServiceUnavailable`, `is_decode() → DeserializeError`.

### 5.9 `src/server/errors.rs` — единая ошибка API

`ServerError` — единственный тип ошибки, который видит HTTP-слой.

```rust
pub type ServerResult<T> = Result<T, ServerError>;

#[derive(Debug, Error, Serialize, ToSchema)]
pub enum ServerError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Not found error: {0}")]
    NotFound(String),
    #[error("Unauthorized request: {0}")]
    Unauthorized(String),
    #[error("Service is unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Internal server error: {0}")]
    InternalError(String),
    // ...
}

impl ServerError {
    /// Единственное место, где ошибка превращается в HTTP-статус.
    pub fn status_code(&self) -> (String, StatusCode) {
        match self {
            ServerError::BadRequest(msg) => (msg.to_owned(), StatusCode::BAD_REQUEST),
            ServerError::NotFound(msg) => (msg.to_owned(), StatusCode::NOT_FOUND),
            // ...
        }
    }
}

impl From<<Module>Errors> for ServerError {
    fn from(err: <Module>Errors) -> Self {
        tracing::error!("Error: {err}", err = err.to_string());   // лог на границе
        match err {
            <Module>Errors::ServiceUnavailable(err) => Self::ServiceUnavailable(err),
            <Module>Errors::NotFoundError(err) => Self::NotFound(err),
            <Module>Errors::AnotherError(err) => Self::InternalError(err),
            // ...
        }
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }

        let (msg, status) = self.status_code();
        let mut resp = Json(ErrorResponse { message: msg }).into_response();
        *resp.status_mut() = status;
        resp
    }
}
```

Правила:

- ошибка логируется **один раз** — в `From<...> for ServerError`. Не логировать
  повторно в хендлере;
- тело ошибки всегда `{"message": "..."}` — форма зафиксирована в
  `ApiErrorResponse` и покрыта тестом;
- маппинг статусов — только в `status_code()`. В хендлерах статусы не задаются.

Живой пример: [src/server/errors.rs](../src/server/errors.rs).

### 5.10 `src/server/mod.rs` — состояние и роутер

`AppState` generic по трейтам модулей — это то, что делает хендлеры тестируемыми
без поднятия инфраструктуры.

```rust
pub struct AppState<A, B>
where
    A: <TraitA>,
    B: <TraitB>,
{
    module_a: Arc<A>,
    module_b: Arc<B>,
    <runtime_setting>: <Type>,
}

impl<A, B> AppState<A, B>
where
    A: <TraitA>,
    B: <TraitB>,
{
    pub fn new(module_a: Arc<A>, module_b: Arc<B>, <runtime_setting>: <Type>) -> Self {
        AppState { module_a, module_b, <runtime_setting> }
    }

    pub fn module_a(&self) -> &A {
        self.module_a.as_ref()
    }
}

pub fn init_server<A, B>(app: AppState<A, B>) -> Router
where
    A: <TraitA> + Send + Sync + 'static,
    B: <TraitB> + Send + Sync + 'static,
{
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();
    let openapi = swagger::api_doc(&app.<runtime_setting>);

    let app_arc = Arc::new(app);
    Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi))
        .route("/", get(Html("<a href=\"/docs\">DOCUMENTATION</a>")))
        .route("/api/v1/<domain>/<action>", post(router::<domain>::<action>::<action>))
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .layer(prometheus_layer)
        .with_state(app_arc)
}
```

Правила:

- пути версионируются: `/api/v<N>/<domain>/<action>`;
- `/docs` — Swagger UI, `/api-docs/openapi.json` — спецификация, `/metrics` —
  Prometheus, `/` — редирект-ссылка на документацию;
- `/metrics` объявляется **после** `DefaultBodyLimit`, чтобы лимит тела не
  применялся к нему;
- состояние оборачивается в `Arc` один раз, в `init_server`.

Живой пример: [src/server/mod.rs](../src/server/mod.rs).

### 5.11 `src/server/router/models.rs` — DTO

DTO слоя API отделены от доменных моделей модулей. Здесь живут `ToSchema`,
`IntoParams` и примеры для Swagger.

```rust
#[derive(Deserialize, Getters, Debug, Clone, PartialEq, IntoParams)]
#[into_params(parameter_in = Query)]
#[getset(get = "pub")]
pub struct <Action>Query {
    task_id: String,
}

#[derive(Serialize, Deserialize, Getters, Debug, Clone, PartialEq, ToSchema)]
#[schema(example = json!({ "field": "value" }))]
#[getset(get = "pub")]
pub struct <Action>Request {
    /// Doc-комментарий попадает в Swagger — писать по-английски.
    #[schema(example = "12345", nullable = false)]
    user_id: Option<String>,
}

#[derive(Serialize, Deserialize, Getters, Debug, Clone, PartialEq, ToSchema)]
#[getset(get = "pub")]
pub struct ApiErrorResponse {
    #[schema(example = "Service is unavailable")]
    message: String,
}
```

Примеры в `#[schema(example = ...)]` — обязательны для всех публичных полей:
Swagger должен быть пригоден для копипасты в `curl` без правок.

Живой пример: [src/server/router/models.rs](../src/server/router/models.rs).

### 5.12 `src/server/router/<domain>/<action>.rs` — хендлер

```rust
#[utoipa::path(
    post,
    path = "/api/v1/<domain>/<action>",
    request_body = <Action>Request,
    tags = ["<Domain>"],
    description = "Что делает эндпоинт и что означает успешный ответ.",
    responses(
        (status = 200, description = "...", body = <Action>Response),
        (status = 400, description = "...", body = ApiErrorResponse),
        (status = 404, description = "...", body = ApiErrorResponse),
        (status = 500, description = "...", body = ApiErrorResponse),
        (status = 503, description = "...", body = ApiErrorResponse)
    )
)]
pub async fn <action><A, B>(
    State(state): State<Arc<AppState<A, B>>>,
    Json(payload): Json<<Action>Request>,
) -> ServerResult<impl IntoResponse>
where
    A: <TraitA> + Send + Sync,
    B: <TraitB> + Send + Sync,
{
    let domain_payload = <Payload>::new(/* из DTO */);

    let result = state.module_a().<action>(domain_payload).await?;

    Ok(Json(<Action>Response::new(result)).into_response())
}
```

Правила:

- сигнатура generic по трейтам — иначе хендлер нельзя протестировать без
  реального брокера/БД;
- порядок экстракторов: `State` → `HeaderMap` / `Query` / `Path` → `Json` (тело
  всегда последним — требование `axum`);
- возвращаемый тип — всегда `ServerResult<impl IntoResponse>`;
- ошибки модулей поднимаются через `?`; `match` по ошибке модуля в хендлере
  запрещён — маппинг живёт в `From<> for ServerError`;
- хендлер отвечает за: валидацию входа → сборку доменной модели → вызов модулей
  → сборку ответа. Работы с внешним крейтом в хендлере нет;
- вспомогательные функции разбора/валидации — приватные `fn` в том же файле,
  возвращают `ServerResult<T>`;
- утвердительный успех: `Ok(Json(...).into_response())`, статус по умолчанию 200.

Живые примеры: [src/server/router/broker/publish_message.rs](../src/server/router/broker/publish_message.rs),
[src/server/router/tasks/cancel_task.rs](../src/server/router/tasks/cancel_task.rs).

### 5.13 `src/server/swagger.rs`

```rust
#[derive(OpenApi)]
#[openapi(
    info(title = "<Service> API", version = "1.0.0", description = "..."),
    tags(
        (name = "<Domain>", description = "..."),
    ),
    components(schemas(<Action>Request, <Action>Response, ApiErrorResponse, Successful)),
    paths(<action>, <other_action>)
)]
pub(super) struct ApiDoc;

/// Документ строится функцией, а не константой: параметры, зависящие от
/// конфигурации (имена заголовков, лимиты), подставляются в рантайме.
pub(super) fn api_doc(<runtime_setting>: &<Type>) -> OpenApiDocument {
    let mut document = ApiDoc::openapi();
    // патч документа по актуальной конфигурации
    document
}

/// Единый способ описывать примеры ответов в Swagger.
pub trait SwaggerExample {
    type Example;

    fn example(value: Option<&str>) -> Self::Example;
}
```

`ApiDoc` и `api_doc` — `pub(super)`: документ собирается только в `init_server`.
Если документ патчится по конфигурации, патч покрывается unit-тестом в этом же
файле (`#[cfg(test)] mod tests`).

Живой пример: [src/server/swagger.rs](../src/server/swagger.rs).

### 5.14 `src/bin/run_server.rs` — точка входа

Единственный файл, знающий конкретные реализации. Порядок фиксирован.

```rust
#[tokio::main(worker_threads = 8)]
async fn main() -> anyhow::Result<()> {
    // 1. Конфигурация — первым делом, до логгера.
    let config = ServiceConfig::new()?;

    // 2. Логгер.
    logger::init_logger(config.logger())?;

    // 3. Внешние клиенты через ServiceConnect.
    let module_a = Arc::new(<ClientA>::connect(config.module_a()).await?);
    let module_b = Arc::new(<ClientB>::connect(config.module_b()).await?);

    // 4. Состояние приложения.
    let server_app = AppState::new(module_a, module_b, <runtime_setting>);

    // 5. Роутер + слои: trace → cors → otel.
    let app = <service>::server::init_server(server_app)
        .layer(trace_layer)
        .layer(cors_layer)
        .layer(OtelAxumLayer::default());

    // 6. Запуск.
    let listener = TcpListener::bind(config.server().address()).await?;
    tracing::info!(address = format!("http://{}", config.server().address()), "Running server on");

    if let Err(err) = axum::serve(listener, app).await {
        tracing::error!(err = ?err, "failed to stop http server");
    };

    Ok(())
}
```

Правила:

- `main` возвращает `anyhow::Result<()>`; ошибки старта поднимаются через `?` и
  роняют процесс — сервис не стартует в полуработоспособном состоянии;
- `anyhow::Context` добавляется там, где текст ошибки крейта не подсказывает,
  какую переменную окружения чинить:
  `.context("SERVICE__SERVER__USER_ID_HEADER contains an invalid HTTP header name")?`;
- никакой бизнес-логики в `main` — только сборка.

Живой пример: [src/bin/run_server.rs](../src/bin/run_server.rs).

---

## 6. Конвенции кода

### 6.1 Структуры и доступ к полям

- Поля **всегда приватные**.
- Чтение — `getset`: `#[getset(get = "pub")]`, для `Copy`-типов —
  `#[derive(CopyGetters)]` + `#[getset(get_copy = "pub")]`.
- Мутация — `#[getset(set = "pub")]`, точечно и только там, где нужна.
- Ручные геттеры пишутся, только если нужна логика (`state_manager()` возвращает
  `&S` из `Arc<S>`).
- Конструктор `new(...)` — обязателен для каждой публичной структуры. Строковые
  параметры принимаются как `impl Into<String>`.
- Для структур с числом полей > 5 или большим количеством опциональных —
  `derive_builder::Builder`.
- `Default` реализуется явно, когда у типа есть осмысленное значение по умолчанию.

### 6.2 Порядок derive

Единый порядок для читаемости: `Serialize, Deserialize, Getters, Setters,
Default, PartialEq, Debug, Clone, ToSchema`.

### 6.3 Запреты

- `unwrap()`, `expect()`, `panic!`, `unimplemented!`, `todo!` в `src/` —
  запрещены. В `tests/` — разрешены и предпочтительны.
- `unsafe` — только в изолированных функциях с комментарием, объясняющим
  инвариант (пример: установка `RUST_LOG` в [src/logger.rs](../src/logger.rs)).
- `.clone()` «на всякий случай». Клонирование — осознанное, обычно `Arc::clone`.
- Прокидывание типов внешних крейтов через границы модулей.
- Магические числа и строки — выносятся в `const` в начало файла:
  `const CONFIG_PREFIX: &str = "SERVICE";`.

### 6.4 Именование

| Сущность | Правило | Пример |
| --- | --- | --- |
| Крейт | `snake_case` | `task_gateway` |
| Трейт-абстракция модуля | роль | `BrokerProducer`, `StateManager` |
| Клиент | технология + роль | `RabbitMQProducer`, `WebhookManager` |
| Enum ошибок модуля | `<Module>Errors`, мн. ч. | `PublisherErrors` |
| Алиас результата | `<Module>Result<T>` | `BrokerResult<T>` |
| Конфиг | `<Module>Config` | `MessageBrokerConfig` |
| Хендлер | глагол + существительное | `publish_message`, `cancel_task` |
| DTO запроса/ответа | `<Name>Request` / `<Name>Response` | `MessageRequest` |
| Тест | утверждение целиком | `publish_message_returns_task_key_from_broker` |

### 6.5 Логирование

- `tracing::debug!` — начало операции с внешним ресурсом
  (`"Creating channel..."`).
- `tracing::info!` — успешное завершение значимой операции, установка
  соединения, старт сервера.
- `tracing::error!` — только на границе преобразования ошибки в `ServerError`.
- Структурированные поля вместо интерполяции:
  `tracing::info!(exchange = exchange, routing = routing, "Published")`.
- В логи не попадают пользовательские payload'ы, токены и заголовки авторизации.

### 6.6 Язык

- Английский: имена, doc-комментарии, тексты ошибок, Swagger, README, коммиты.
- Русский: внутренняя документация (`AGENTS.md`, этот файл), обсуждения.
- Смешивать в одном артефакте нельзя. Ответы API — только английский.

---

## 7. Конфигурация, логирование, наблюдаемость

### 7.1 Схема конфигурации

`ServiceConfig` — плоская композиция конфигов слоёв. Имя поля = имя секции в TOML
= сегмент в имени переменной окружения.

```rust
const CONFIG_PREFIX: &str = "<SERVICE>";
const SERVICE_RUN_MODE: &str = "<SERVICE>__RUN_MODE";
const DEV_FILE_CONFIG_PATH: &str = "./config/development.toml";

#[derive(Builder, Clone, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct ServiceConfig {
    <module_a>: <ModuleA>Config,
    <module_b>: <ModuleB>Config,
    server: ServerConfig,
    logger: LoggerConfig,
}

impl ServiceConfig {
    pub fn new() -> Result<Self, ConfigError> {
        dotenv().ok();

        let run_mode = std::env::var(SERVICE_RUN_MODE).unwrap_or("development".into());
        dotenv::from_filename(format!(".env.{}", run_mode)).ok();

        let settings = Config::builder()
            .add_source(File::with_name(DEV_FILE_CONFIG_PATH))
            .add_source(
                File::with_name(&format!("./config/{}", run_mode))
                    .format(FileFormat::Toml)
                    .required(false),
            )
            .add_source(
                Environment::with_prefix(CONFIG_PREFIX)
                    .prefix_separator("__")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        let config: Self = settings.try_deserialize()?;
        config.<module_a>.validate().map_err(ConfigError::Message)?;
        config.<module_b>.validate().map_err(ConfigError::Message)?;
        Ok(config)
    }
}
```

### 7.2 Приоритет источников

От низшего к высшему:

1. `config/development.toml` — база, коммитится, содержит значения по умолчанию
   и структуры, неудобные для env (массивы таблиц: маршруты, лимиты);
2. `config/<run_mode>.toml` — переопределения для среды, `required(false)`;
   `production.toml` в `.gitignore`;
3. `.env.<run_mode>` — локальные значения разработчика;
4. переменные окружения `<SERVICE>__<SECTION>__<FIELD>` — высший приоритет,
   основной способ конфигурации в Docker.

Разделитель — двойное подчёркивание `__`, чтобы имена полей с одинарным
подчёркиванием (`user_id_header`) разбирались корректно:
`SERVICE__SERVER__USER_ID_HEADER` → `server.user_id_header`.

Все переменные перечисляются в `.env.example` **без значений-секретов**.

### 7.3 Валидация

- Каждый `<Module>Config` имеет `validate(&self) -> Result<(), String>`.
- Все `validate()` вызываются из `ServiceConfig::new()` — до подключения к
  внешним ресурсам.
- Ошибка валидации содержит путь к полю и требование; она попадает в лог старта
  и роняет процесс.
- Дополнительно `validate()` вызывается в `ServiceConnect::connect()` — клиент
  не должен полагаться на то, что кто-то проверил конфиг за него.

### 7.4 Логгер

`LoggerConfig { use_loki: bool, level: String, address: String }`.
`init_logger` собирает `FmtSubscriber` для локальной разработки и
`registry + loki_layer + fmt_layer` для окружений с Loki. Уровень берётся из
`RUST_LOG`, а если переменная не задана — из конфига.
См. [src/logger.rs](../src/logger.rs).

### 7.5 Наблюдаемость

| Аспект | Механизм | Точка |
| --- | --- | --- |
| Метрики | `axum-prometheus` | `GET /metrics` |
| Трейсы | `axum-tracing-opentelemetry` | `OtelAxumLayer` в `main` |
| HTTP-логи | `tower_http::trace` | `TraceLayer` в `main` |
| Логи | `tracing` + Loki | `init_logger` |
| Документация | `utoipa-swagger-ui` | `GET /docs` |

Этот набор подключается в каждом сервисе — он не опционален.

---

## 8. Чеклист: добавление нового внешнего модуля

Задача: сервису нужен новый внешний ресурс (например, S3, Redis, сторонний API).

1. **Каталог.** `src/modules/<module>/` с файлами `mod.rs`, `config.rs`,
   `errors.rs`, `models/mod.rs`. Имя каталога — по роли, не по технологии.
2. **Модели.** В `models/mod.rs` — доменные структуры и
   `pub type <Module>Result<T> = Result<T, <Module>Errors>;`.
3. **Ошибки.** В `errors.rs` — `enum <Module>Errors` с вариантами по семантике
   сбоя и обязательным `AnotherError`. Плюс `From<serde_json::Error>`, если
   модуль сериализует.
4. **Конфиг.** В `config.rs` — `<Module>Config` с приватными полями, `new(...)`
   и `validate()`.
5. **Трейт.** В `src/modules/mod.rs` — `pub mod <module>;` и трейт с доменными
   методами, возвращающими `<Module>Result<T>`.
6. **Реализация.** `src/modules/<module>/<client>/`:
   - `mod.rs` — структура клиента (`Arc`-поля) + `impl ServiceConnect`;
   - `core.rs` — `impl <Module>Trait for <Client>`;
   - `errors.rs` — `From<ошибка_крейта> for <Module>Errors`.
7. **Подъём ошибок.** В `src/server/errors.rs` — `impl From<<Module>Errors> for
   ServerError` с `tracing::error!` и маппингом на варианты `ServerError`.
8. **Конфигурация сервиса.** Поле в `ServiceConfig`, вызов `validate()` в
   `ServiceConfig::new()`, секция в `config/development.toml`, переменные в
   `.env.example` / `.env.development` / `docker-compose/.env.<service>`.
9. **Состояние.** Новый generic-параметр в `AppState` и `init_server`, поле в
   `Arc`, геттер.
10. **Сборка.** В `run_server.rs` — `<Client>::connect(config.<module>()).await?`
    и передача в `AppState::new`.
11. **Тесты.** `tests/<module>_models.rs` — сериализация моделей и валидация
    конфига (включая негативные кейсы).
12. **Документация.** Секция в `README.md`: назначение модуля, переменные,
    поведение при недоступности.

Проверка: `cargo fmt && cargo clippy --all-targets && cargo test`.

---

## 9. Чеклист: добавление нового эндпоинта

1. **DTO.** В `src/server/router/models.rs` — `<Action>Request` / `<Action>Response`
   / `<Action>Query` с `ToSchema` / `IntoParams`, `#[schema(example = ...)]` на
   каждом публичном поле и `new(...)` для ответа.
2. **Файл хендлера.** `src/server/router/<domain>/<action>.rs`; для нового домена
   — каталог с `mod.rs`, объявленным в `src/server/router/mod.rs`.
3. **Хендлер.** Generic по трейтам модулей, возвращает
   `ServerResult<impl IntoResponse>`, ошибки — через `?`.
4. **Swagger.** `#[utoipa::path(...)]` над хендлером: `path`, `tags`,
   `request_body`/`params`, `description`, и **полный** список `responses` —
   все статусы, которые реально может вернуть `status_code()` для этой цепочки
   ошибок.
5. **Регистрация в ApiDoc.** В `src/server/swagger.rs` — импорт хендлера,
   добавление в `paths(...)`, новых схем — в `components(schemas(...))`, при
   необходимости новый `tag`.
6. **Маршрут.** В `init_server` — `.route("/api/v1/<domain>/<action>", post(...))`
   рядом с остальными, до `DefaultBodyLimit`.
7. **Тесты.** `tests/<action>_handler.rs`: успешный путь, каждая ветка ошибки,
   проверка побочных эффектов через фейк-реализации (см. раздел 10).
8. **README.** Эндпоинт, пример запроса и ответа, семантика успешного ответа.

---

## 10. Тесты

### 10.1 Расположение

- `tests/<action>_handler.rs` — интеграционные тесты хендлеров;
- `tests/<module>_models.rs` — сериализация моделей, валидация конфигов;
- `#[cfg(test)] mod tests` внутри файла — только для внутренней логики, не
  доступной снаружи (например, патч Swagger-документа).

Тесты в `tests/` работают с крейтом через его публичный API — это заодно
проверяет, что нужное действительно `pub`.

### 10.2 Фейковые реализации

Хендлеры тестируются без инфраструктуры: в тестовом файле объявляются структуры,
реализующие трейты модулей. Три канонические роли:

```rust
/// Не должен вызываться в этом сценарии.
struct NoopBroker;

#[async_trait::async_trait]
impl BrokerProducer for NoopBroker {
    async fn publish(&self, _payload: PublishMessage) -> BrokerResult<String> {
        unreachable!("cancel_task must not publish broker messages")
    }
}

/// Записывает вызовы — для проверки побочных эффектов.
#[derive(Clone, Default)]
struct RecordingStateManager {
    created_tasks: Arc<Mutex<Vec<TaskState>>>,
}

/// Возвращает ошибку — для проверки её подъёма в HTTP-статус.
struct UnavailableStateManager;
```

Фабрика состояния выносится в конец файла:

```rust
fn test_state<S: StateManager>(state_manager: S) -> State<Arc<AppState<NoopBroker, S>>> {
    State(Arc::new(AppState::new(
        Arc::new(NoopBroker),
        Arc::new(state_manager),
        HeaderName::from_static("x-user-id"),
    )))
}

async fn response_json(response: Response) -> serde_json::Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()
}
```

### 10.3 Стиль теста

Хендлер вызывается как обычная функция — с экстракторами, без поднятия сервера:

```rust
#[tokio::test]
async fn <action>_returns_not_found_for_unknown_route() {
    let request: <Action>Request = serde_json::from_value(json!({ /* ... */ })).unwrap();
    let state = test_state(SuccessfulBroker);

    let error = match <action>(state, Json(request)).await {
        Ok(_) => panic!("<action> should reject an unknown route"),
        Err(error) => error,
    };
    let response = error.into_response();

    assert_eq!(response.status(), 404);
    assert_eq!(
        response_json(response).await,
        json!({ "message": "Unknown task type: audio.generate" })
    );
}
```

Правила:

- имя теста — утверждение целиком: `<subject>_<expected>_<condition>`;
- вход собирается из `json!` через `serde_json::from_value` — это одновременно
  проверяет, что публичный JSON-контракт разбирается;
- проверяется и статус, и **тело целиком** через `json!`, а не отдельные поля;
- проверяются побочные эффекты: что записалось в `RecordingX`;
- негативные кейсы обязательны — каждая ветка ошибки в хендлере покрывается;
- `unwrap()` в тестах допустим и предпочтителен: падение теста и есть сигнал.

Живые примеры: [tests/publish_message_handler.rs](../tests/publish_message_handler.rs),
[tests/cancel_task_handler.rs](../tests/cancel_task_handler.rs),
[tests/broker_models.rs](../tests/broker_models.rs).

---

## 11. Инфраструктура

### 11.1 Dockerfile

Многостадийная сборка с `cargo-chef` для кеширования зависимостей:
`chef` → `planner` (recipe.json) → `builder` (`cargo chef cook --release`, затем
`cargo install --bins --path .`) → runtime на `ubuntu:24.04` с `openssl` и
`ca-certificates`. В финальный образ копируются только бинарник и `config/`.
См. [Dockerfile](../Dockerfile).

### 11.2 docker-compose

`docker-compose/` содержит `docker-compose.yaml`, `.env.<service>` и конфиги
инфраструктуры. Сервис берёт переменные через `env_file`, инфраструктурные
контейнеры получают `healthcheck` и ограничения ресурсов.

### 11.3 CI

`.github/workflows/pull-request.yml` — цепочка
`check → fmt → build → audit → tests`; `check` прогоняется и с
`--all-features`. `.github/workflows/create_release.yml` — сборка и публикация
образа в `ghcr.io` по semver-тегам.

### 11.4 Команды разработчика

```bash
cargo fmt
cargo clippy --all-targets
cargo test
cargo run --bin run_server
```

Эти четыре команды прогоняются перед каждым коммитом.

---

## 12. Известные расхождения текущего кода с шаблоном

Фиксируются явно, чтобы не тиражировались в новых сервисах.

| Место | Расхождение | Как правильно |
| --- | --- | --- |
| [src/config.rs](../src/config.rs) | `unimplemented!()` в ветке `.env.task-gateway` | ветка должна загружать файл, как остальные, либо не существовать |
| [src/modules/state_manager/mod.rs](../src/modules/state_manager/mod.rs) | структура клиента лежит в `mod.rs` модуля, а не в каталоге реализации | как в `broker/rabbitmq/mod.rs` — структура в каталоге реализации |
| [src/modules/state_manager/config.rs](../src/modules/state_manager/config.rs) | нет `validate()`; есть `#[serde(alias = "create_task_endpont")]` для совместимости с опечаткой | `validate()` + вызов из `ServiceConfig::new()`; алиас убрать после миграции конфигов |
| [src/modules/state_manager/models/mod.rs](../src/modules/state_manager/models/mod.rs) | `unwrap()` в `Default for TaskState` | константный `Uuid` или `Uuid::nil()` |
| [src/server/mod.rs](../src/server/mod.rs) | русский текст и незакрытый тег в HTML корневого маршрута | английский текст, валидный HTML |
| [src/errors.rs](../src/errors.rs) + [src/server/errors.rs](../src/server/errors.rs) | два типа успешного ответа: `Successful` и `Success` | один тип на сервис |
| [src/bin/run_server.rs](../src/bin/run_server.rs) | дублирующийся `tracing::info!` о старте | один лог старта |
| [config/development.toml](../config/development.toml) | `llm_mode` в секции `server` не имеет поля в `ServerConfig` | конфиг-файл и структура синхронны |

---

## 13. Bootstrap нового сервиса

1. `cargo new <service> --lib`, добавить `edition = "2024"`.
2. Скопировать в `Cargo.toml` базовый набор из раздела 2 (без клиентов, которых
   нет в новом сервисе).
3. Создать каркас: `src/lib.rs` (раздел 5.1), `src/logger.rs`, `src/errors.rs`,
   `src/config.rs` (раздел 7.1), `src/bin/run_server.rs` (раздел 5.14).
4. Создать `src/server/` — `mod.rs`, `config.rs`, `errors.rs`, `swagger.rs`,
   `router/{mod.rs,models.rs}`.
5. Создать `src/modules/mod.rs` (пока пустой набор трейтов).
6. Добавить `config/development.toml`, `.env.example`, `.env.development`,
   `.gitignore` (`/target`, `/config/production.toml`, `.env.development`).
7. Скопировать `Dockerfile`, `.dockerignore`, `.github/workflows/`.
8. Добавить `AGENTS.md` со ссылкой на этот документ и краткими правилами.
9. Дальше — по чеклистам разделов 8 и 9 на каждый модуль и эндпоинт.
