# 008 — Воспроизводимый CI и release gate

- **Статус:** active, реализация в рабочем дереве
- **Приоритет:** P0
- **Зависимости:** нет
- **Блокирует:** 009, 010, 011, 013

## Результат

Каждый PR и push в основную ветку получает одинаковый набор автоматических
проверок, Rust-зависимости разрешаются воспроизводимо, а релизный тег не может
опубликовать commit без успешного CI.

## Проблема

До начала roadmap CI запускался только для pull request, `Cargo.lock` был
исключён из Git, а release workflow не проверял результат CI основной ветки.
Ручной запуск release workflow также пытался трактовать имя ветки как semver.

## Объём работ

1. Запускать `ci.yml` для PR, push в `main`/`master` и вручную.
2. Добавить read-only permissions, отмену устаревших CI-прогонов и таймауты.
3. Использовать Node.js 22 и npm lockfile через `npm ci`.
4. Хранить `src-tauri/Cargo.lock` в Git и выполнять Cargo-команды с `--locked`.
5. Проверять formatting, Clippy, Rust tests, frontend contracts/build и Windows
   Tauri debug build.
6. Перед релизом ждать успешный push-запуск CI для того же commit.
7. Сохранить рабочий `workflow_dispatch`: версия берётся из репозитория, release
   создаётся только по валидному тегу `vX.Y.Z`.
8. Актуализировать документацию сборки и release-процесс.

## Не входит

- CodeQL, Dependabot, подпись бинарников и автообновление.
- Перенос Piper/espeak/libclang-этапов из `app-tts-v2`: у Echo нет этих
  зависимостей.
- Оптимизация продолжительности Tauri bundle smoke.

## Приёмка

- Новый commit в `main` или `master` создаёт CI run.
- Повторный push в ту же ветку отменяет устаревший незавершённый run.
- Изменение lockfile, не соответствующее manifests, обнаруживается `--locked`.
- Тег на commit без успешного CI не создаёт GitHub Release.
- Тег `vX.Y.Z` на проверенном commit создаёт Windows artifacts и prerelease.
- Ручной запуск успешно собирает текущую версию и не создаёт release.

## Верификация

- `npm test`;
- `npm run build`;
- `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`;
- `cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets --all-features -- -D warnings`;
- `cargo test --manifest-path src-tauri/Cargo.toml --locked --all-targets`;
- успешный push-run GitHub Actions;
- ручной `workflow_dispatch`;
- тестовый release tag после завершения остальных проверок.

## Завершение

После зелёного удалённого CI перенести документ в `completed/` и записать ссылки
на CI run и проверенный release run. Локальная зелёная сборка не заменяет эту
проверку.
