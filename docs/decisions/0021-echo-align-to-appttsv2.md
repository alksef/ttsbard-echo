# ADR 0021 — Align ttsbard-echo architecture, config, UI and build/CI/git to app-tts-v2

- **Status:** Accepted
- **Date:** 2026-08-03
- **Decides for:** ttsbard-echo
- **Reference project (read-only sample):** `D:\RustProjects\app-tts-v2`
- **Execution backlog:** `.work/ai/2026-08-03-echo-align-to-appttsv2/`
- **Summary plan:** `docs/plans/03-2026-0803-align-to-appttsv2.md`

## Context

ttsbard-echo — это лёгкий Tauri 2 + Vue 3 клиент (SSE-подключения + плавающее окно +
минимальные настройки). Он production-ready, но архитектурно и инфраструктурно устроен
проще, чем родственный `app-tts-v2` (зрелый TTS-проект с развитыми слоями, контрактным
тестированием, локальной сборкой с external-target safety и GitHub Actions CI).

echo и app-tts-v2 — одна технологическая семья (Tauri 2, Vue 3, TS, Windows-first).
Расхождение в устройстве увеличивает цену дальнейшей разработки echo и повторения
инфраструктурных решений.

## Decision

Привести ttsbard-echo к **тому же каркасу и инфраструктуре**, что app-tts-v2 — как
скелет, без переноса чужого функционального контента.

Переносятся **паттерны и инфраструктура**, не сущности:

1. **Архитектура backend (Rust):** слои `commands/config/dto/events/state/setup`,
   декомпозиция `AppSettings`/менеджеров, AppEvent-контракт и sender-канал
   (соответствует app-tts-v2 ADR-003, ADR-004, ADR-008, ADR-014).
2. **Конфиг ↔ фронтенд:** единая команда получения всех настроек, зеркальные TS-типы,
   composable `useAppSettings` с camelCase↔snake_case mapping.
3. **UI:** layout/Sidebar/SettingsPanel, shared-компоненты, под-панели настроек, CSS-токены
   и темы (соответствует ADR-012).
4. **Инфраструктура сборки/CI/git:** стандартные npm/Cargo/Tauri-команды; локальные build-wrapper'ы не публикуются
   с external-target clean-safety, GitHub Actions (CI на PR + release по тегу), meta-файлы
   (`.gitignore`/`.gitattributes`/`.claudeignore`/`.editorconfig`), npm-скрипты и
   контракто-чекеры IPC/settings-parity **адаптированно** под echo.

## Constraints

- **Не переносится функциональный контент**, которого нет в echo: TTS, аудио-pipeline,
  twitch, webview-сервер, VTube, soundpanel, playback, препроцессор, AI-коррекция текста.
  Они остаются паттернами-образцами, но не появляются как сущности.
- **Продуктовое поведение echo сохраняется** — меняются внутреннее устройство и инфраструктура.
- app-tts-v2 — **только для чтения**.
- Таски исполняются автономным кодогенерирующим агентом (qwen3-coder через локальный Ollama);
  формат задач — observable behavior + границы, без указания реализации.

## Consequences

- Положительные: единый каркас и инфраструктура между echo и app-tts-v2; тиражируемые
  сборка/CI/контракт-чекинг; ниже порог входа для изменений; контракт-чекеры ловят
  рассинхрон Rust↔фронтенд автоматически.
- Отрицательные: значительный объём рефакторинга без новой пользовательской
  функциональности; риск временной рассинхронизации фронт/бэк при сквозных правках
  (митигируется порядком задач и синхронными правками, см. план).
- Нейтральные: contract-чекеры и CI надо поддерживать; `.work/` становится постоянным
  форматом ведения задач echo.

## Resolved questions

- **CodeQL** (`codeql-analysis.yml`) — не переносится в этой сессии; при необходимости
  отдельной сессией позже.
- **`Cargo.toml`** — намеренно вне рамок (зависимости/фичи/lints echo стабильны).
- **Каноническая команда настроек** — `get_all_settings`; разрыв с фронтом устраняется
  задачей 006.
- **Frameless главного окна** — задача 020 (предусловие кастомного titlebar из 009);
  baseline-окно НЕ frameless.
- **SSE-auth** — cookie `webview_auth` (соответствует контракту источника); Authorization/
  query не вводятся.
- Контракто-чекеры speech-contract не применимы (нет speech queue); задача 016 вводит
  только IPC + settings-parity.
