# Инструкции для работы с проектом

## Структура проекта

- `src/` — frontend на Vue 3 и TypeScript.
- `src-tauri/src/` — backend на Rust и Tauri.
- `src-tauri/src/commands/` — Tauri-команды frontend/backend.
- `src-tauri/src/config/` — настройки, DTO и валидация.
- `src-tauri/src/connections/` — SSE-подключения и их менеджер.
- `docs/` — актуальная документация проекта.
- `docs/user/` — инструкция пользователя.
- `docs/development/` — инструкции разработчика.
- `docs/reference/` — технические контракты и справочник.
- `docs/roadmap/completed/` — архив закрытых roadmap-задач.

Исторические материалы находятся в `docs/plans/` и не являются текущими инструкциями.

## Команды

Установить зависимости и запустить frontend:

```powershell
npm ci
npm run dev
```

Проверить проект:

```powershell
npm run build
npm test
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

`npm test` проверяет IPC- и settings-контракты. Полная Tauri-сборка выполняется на Windows:

```powershell
npm run tauri build
```

## Версионирование

Версия должна быть одинаковой в следующих файлах:

- `package.json`;
- `package-lock.json`;
- `src-tauri/Cargo.toml`;
- `src-tauri/tauri.conf.json`;
- `src/version.ts`.

Для синхронного обновления версии используйте:

```powershell
npm run version -- 0.1.0
```

Релизные теги имеют формат `vX.Y.Z`, например `v0.1.0`. Workflow `build.yml` собирает Windows-пакеты и создаёт GitHub Release для тегов `v*`.

## CI

- `ci.yml` запускает форматирование Rust, Clippy, frontend-проверки, Rust-тесты и Windows debug build.
- `build.yml` запускает Windows release build по тегу и публикует NSIS/MSI-артефакты.

Перед коммитом проверяйте `git status`, не добавляйте `node_modules/`, `dist/`, `target/`, `.vite/`, `.work/`, локальные конфиги и файлы с секретами.
