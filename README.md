# Geek Metaverse Image Generation

Rust-сервис для генерации и редактирования изображений через LLM/image APIs.

## Возможности

- Генерация изображения и возврат результата как файла
- Генерация изображения и возврат пути к сохраненному файлу
- Редактирование изображения по `multipart/form-data` с загрузкой одного или нескольких исходников
- Поддержка нескольких провайдеров: OpenAI, OpenRouter, xAI
- Prometheus-метрики и tracing-логи

## Поддерживаемые модели

Сервис ожидает значение `model` в одном из следующих форматов:

- `openai::gpt-image-1-mini`
- `openai::gpt-image-1`
- `openai::gpt-image-1.5`
- `openrouter::google/gemini-2.5-flash-image`
- `openrouter::google/gemini-3-pro-image-preview`
- `openrouter::google/gemini-3.1-flash-image-preview`
- `openrouter::black-forest-labs/flux.2-klein-4b`
- `openrouter::black-forest-labs/flux.2-max`
- `openrouter::black-forest-labs/flux.2-pro`
- `openrouter::black-forest-labs/flux.2-flex`
- `xai::grok-imagine-image`
- `xai::grok-imagine-image-pro`

## Структура API

После запуска сервис поднимает:

- `GET /docs` - интерактивная документация Swagger
- `GET /api-docs/openapi.json` - OpenAPI схема
- `GET /metrics` - Prometheus-метрики
- `POST /api/v1/images/generate/file` - генерация изображения с ответом в виде PNG
- `POST /api/v1/images/generate/url` - генерация изображения с ответом в виде JSON с путем
- `POST /api/v1/images/edit/file` - редактирование изображения с ответом в виде PNG

## Сохранение файлов

Сервис сохраняет результаты на локальный диск.

Генерация:

```text
images/generation/<universe>/<user_id>/<image_name>.png
```

Редактирование:

```text
images/editing/<universe>/<user_id>/<task_id>/raw/<uploaded_file_name>.png
images/editing/<universe>/<user_id>/<image_name>.png
```

## Конфигурация

По умолчанию сервис читает:

1. `./config/development.toml`
2. `./config/<RUN_MODE>.toml`, если задан `GEEK_METAVERSE_IMAGE_GENERATION__RUN_MODE`
3. переменные окружения с префиксом `GEEK_METAVERSE_IMAGE_GENERATION__`

Базовый конфиг лежит в [config/development.toml]

Примеры полезных переменных окружения:

```bash
export GEEK_METAVERSE_IMAGE_GENERATION__RUN_MODE=development
export GEEK_METAVERSE_IMAGE_GENERATION__SERVER__ADDRESS=0.0.0.0:10001
export GEEK_METAVERSE_IMAGE_GENERATION__LLM_CLIENT__OPENAI__API_KEY=sk-...
export GEEK_METAVERSE_IMAGE_GENERATION__LLM_CLIENT__OPENROUTER__API_KEY=sk-or-v1-...
export GEEK_METAVERSE_IMAGE_GENERATION__LLM_CLIENT__XAI__API_KEY=...
export GEEK_METAVERSE_IMAGE_GENERATION__LOGGER__LEVEL=debug
```

Ключевые секции конфига:

- `server.address` - адрес HTTP-сервера
- `llm_client.openai.*` - доступ к OpenAI
- `llm_client.openrouter.*` - доступ к OpenRouter
- `llm_client.xai.*` - доступ к xAI
- `logger.*` - уровень логирования и Loki-настройки

## Локальный запуск

Требования:

- Rust toolchain
- доступные API-ключи хотя бы для одного провайдера

Запуск:

```bash
cargo run
```

После старта сервис по умолчанию доступен на `http://0.0.0.0:10001`, документация - на `http://0.0.0.0:10001/docs`.

## Docker

Сборка образа:

```bash
docker build -t geek-metaverse-image-generation:1.0.0 .
```

Запуск контейнера:

```bash
docker run --rm \
  -p 10001:10001 \
  -v "$(pwd)/images:/app/images" \
  -e GEEK_METAVERSE_IMAGE_GENERATION__LLM_CLIENT__OPENAI__API_KEY=sk-... \
  geek-metaverse-image-generation:1.0.0
```

В репозитории также есть compose-файл [docker-compose/docker-compose.yaml], но перед использованием стоит сверить его с текущим `server.address`: в конфиге по умолчанию используется порт `10001`, а в compose сейчас проброшен `10008`.

## Примеры запросов

Генерация с возвратом файла:

```bash
curl -X POST http://localhost:10001/api/v1/images/generate/file \
  -H 'Content-Type: application/json' \
  -d '{
    "image_name": "loksodon-druid",
    "prompt": "Нарисуй локсодона-друида с посохом из кости",
    "user_id": 122333,
    "universe": "dnd",
    "model": "openai::gpt-image-1-mini"
  }' \
  --output loksodon-druid.png
```

Генерация с возвратом пути:

```bash
curl -X POST http://localhost:10001/api/v1/images/generate/url \
  -H 'Content-Type: application/json' \
  -d '{
    "image_name": "city",
    "prompt": "Фэнтезийный город на вершине утеса",
    "user_id": 42,
    "universe": "metaverse",
    "model": "openrouter::google/gemini-2.5-flash-image"
  }'
```

Редактирование изображения:

```bash
curl -X POST http://localhost:10001/api/v1/images/edit/file \
  -F 'image_name=clown-edit' \
  -F 'prompt=Добавь клоуна рядом' \
  -F 'user_id=42' \
  -F 'universe=dnd' \
  -F 'model=openai::gpt-image-1-mini' \
  -F 'images[]=@./example.png' \
  --output clown-edit.png
```

Логирование настраивается через секцию `logger`. Если `use_loki = true`, сервис отправляет логи в Loki; иначе использует стандартный `tracing` output в stdout.

## Технологии

- Rust 
- Axum
- async-openai
- Prometheus metrics
- Tracing / Loki
