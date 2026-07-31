# AGENTS.md

Task Gateway — HTTP-шина задач на Rust (axum + lapin + reqwest).

## Обязательно к прочтению

Перед написанием кода прочитай **[docs/SERVICE_TEMPLATE.md](docs/SERVICE_TEMPLATE.md)** —
это стандарт архитектуры и стиля сервиса. Он описывает структуру каталогов,
слои, шаблоны файлов, конвенции ошибок, конфигурации, логирования, Swagger и
тестов. Новый код должен соответствовать этому шаблону, а не привносить
альтернативную структуру.

Готовые чеклисты:

- добавление нового внешнего модуля — раздел 8;
- добавление нового эндпоинта — раздел 9.

## Ключевые правила (коротко)

1. Хендлеры generic по трейтам модулей (`BrokerProducer`, `StateManager`), не по
   конкретным клиентам.
2. У каждого модуля свой `enum XxxErrors`; наверх — только через
   `From<XxxErrors> for ServerError`. В хендлере — `?`, не `match`.
3. Поля структур приватные, доступ через `getset`, сборка через `new(...)`.
4. Конфиг валидируется на старте (`validate()` в `ServiceConfig::new()`),
   источники: `config/development.toml` → `config/<run_mode>.toml` → env
   `TASK_GATEWAY__<SECTION>__<FIELD>`.
5. Внешний клиент подключается через трейт `ServiceConnect`.
6. Никаких `unwrap()` / `expect()` в `src/` (кроме тестов).
7. Публичный контракт описывается `utoipa`-аннотациями рядом с хендлером.
8. Публичные тексты (API, ошибки, README) — на английском.

## Команды

```bash
cargo fmt
cargo clippy --all-targets
cargo test
cargo run --bin run_server
```
