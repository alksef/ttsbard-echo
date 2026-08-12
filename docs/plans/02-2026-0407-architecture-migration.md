# План миграции архитектуры из app-tts-v2

**Дата:** 2026-04-07
**Номер:** 02
**Статус:** В разработке
**Источник:** D:\RustProjects\app-tts-v2

---

## Цели миграции

Перенос проверенной архитектуры из эталонного проекта (app-tts-v2) в текущий проект (ttsbard-echo) по направлениям:

1. **CSS Архитектура** — полная миграция модульной системы
2. **Структура панелей** — основная структура с табовыми настройками
3. **Работа с конфигом** — полная миграция системы настроек
4. **TypeScript** — полная миграция системы типов

---

## Что такое Composables?

**Composables** во Vue 3 — это функции, использующие Composition API для создания переиспользуемой логики с состоянием.

### Базовый пример

```typescript
// Простой composable
export function useCounter(initialValue: number = 0) {
  const count = ref(initialValue)

  function increment() { count.value++ }
  function decrement() { count.value-- }

  return { count, increment, decrement }
}

// Использование в компоненте
const { count, increment, decrement } = useCounter(10)
```

### Composable vs Обычная функция

| Обычная функция | Composable |
|-----------------|------------|
| Без состояния | С реактивным состоянием (`ref`, `computed`) |
| Одноразовая логика | Сохраняет состояние между вызовами |
| Не может использовать хуки Vue | Использует `watch`, `onMounted`, `onUnmounted` |
| Нет жизненного цикла | Может подписываться на события и очищаться |

### Dependency Injection паттерн

**Проблема текущего подхода:**
```typescript
// Глобальные переменные — плохая практика
const globalSettings = ref<AppSettingsDto | null>(null)

export function useAppSettings() {
  return { settings: globalSettings }
}
```

**Решение с DI:**
```typescript
// 1. Создаём ключ для внедрения
export const APP_SETTINGS_KEY: InjectionKey<AppSettingsContext> =
  Symbol('app-settings')

// 2. Создаём контекст (в App.vue)
export function createAppSettings(): AppSettingsContext {
  const settings = ref<AppSettingsDto | null>(null)
  const isLoading = ref(false)

  let cleanupListeners: (() => void) | null = null

  async function load() {
    isLoading.value = true
    try {
      settings.value = await invoke('get_all_app_settings')
    } finally {
      isLoading.value = false
    }
  }

  // Подписка на события с автоочисткой
  async function setupEventListeners() {
    const unlisten = await listen('settings-changed', load)

    return () => {
      unlisten()
    }
  }

  load()
  setupEventListeners().then((cleanup) => {
    cleanupListeners = cleanup
    onScopeDispose(cleanup)  // Автоочистка при unmount
  })

  return { settings, isLoading, error, reload: load }
}

// 3. Предоставляем контекст (App.vue)
export function provideAppSettings(): AppSettingsContext {
  const context = createAppSettings()
  provide(APP_SETTINGS_KEY, context)  // ← Делаем доступным для потомков
  return context
}

// 4. Внедряем контекст (в любом дочернем компоненте)
export function useAppSettings(): AppSettingsContext {
  const context = inject<AppSettingsContext>(APP_SETTINGS_KEY)

  if (!context) {
    throw new Error('useAppSettings должен использоваться внутри provideAppSettings')
  }

  return context
}
```

### Использование в компонентах

**App.vue (корневой):**
```vue
<script setup lang="ts">
import { provideAppSettings } from './composables/useAppSettings'

const appSettings = provideAppSettings()
</script>

<template>
  <div id="app">
    <SettingsPanel />
  </div>
</template>
```

**SettingsPanel.vue (дочерний):**
```vue
<script setup lang="ts">
import { useAppSettings } from './composables/useAppSettings'

const { settings, isLoading, reload } = useAppSettings()
</script>
```

### Специализированные composables

```typescript
// Извлекаем конкретные части настроек
export function useLoggingSettings(): ComputedRef<LoggingSettingsDto | undefined> {
  const { settings } = useAppSettings()
  return computed(() => settings.value?.logging)
}

export function useTheme(): ComputedRef<Theme> {
  const { settings } = useAppSettings()
  return computed(() => settings.value?.general?.theme ?? 'dark')
}

export function useConnections(): ComputedRef<ConnectionConfig[]> {
  const { settings } = useAppSettings()
  return computed(() => settings.value?.connections ?? [])
}
```

### Жизненный цикл и cleanup

```typescript
export function useWebSocket(url: string) {
  const socket = ref<WebSocket | null>(null)
  const connected = ref(false)

  function connect() {
    socket.value = new WebSocket(url)
    socket.value.onopen = () => { connected.value = true }
  }

  // Важно! Очистка при размонтировании
  onUnmounted(() => {
    socket.value?.close()
  })

  return { connected, connect }
}
```

### Сравнение: текущий vs эталон

| Текущий проект | Эталонный проект |
|----------------|------------------|
| 1 базовый composable | 11 специализированных |
| Глобальные переменные | Dependency injection |
| Нет cleanup | `onScopeDispose` cleanup |
| Прямой `invoke` | Инкапсулированные вызовы |
| Дублирование кода | Переиспользуемые функции |

### Преимущества composables

| Преимущество | Описание |
|--------------|----------|
| **Переиспользование** | Логика один раз, используется везде |
| **Тестирование** | Легко тестировать изолированно |
| **Чистота компонентов** | Компоненты становятся проще |
| **Состояние** | Сохраняют состояние между ререндерами |
| **Cleanup** | Автоматическая очистка при unmount |
| **Tree-shaking** | Unused код удаляется при сборке |

---

## Часть 1. CSS Архитектура (полная миграция)

### Текущее состояние
- Один файл `src/style.css` (207 строк)
- Всё вперемешку: переменные, сброс, стили, анимации
- Дублирование цветов для тем

### Целевое состояние
```
src/styles/
├── index.css      (точка входа, импорты)
├── variables.css  (188 строк — все CSS переменные)
├── base.css       (60 строк — сброс, body, scrollbar)
└── gradients.css  (61 строка — градиенты с color-mix)
```

### Задачи

#### 1.1 Создать структуру директорий
- [ ] Создать `src/styles/` директорию
- [ ] Создать файлы `index.css`, `variables.css`, `base.css`, `gradients.css`

#### 1.2 Миграция переменных (`variables.css`)
Перенести и расширить систему CSS переменных:

**Новые категории переменных:**
```css
/* Семантические цвета для UI элементов */
--success-bg, --success-bg-weak, --success-text, --success-border
--warning-bg, --warning-bg-weak, --warning-text, --warning-border
--danger-bg, --danger-bg-weak, --danger-text, --danger-border
--info-bg, --info-bg-weak, --info-text, --info-border

/* Toast notifications */
--toast-bg, --toast-border, --toast-error-border, --toast-error-bg
--toast-warning-border, --toast-warning-bg
--toast-success-border, --toast-success-bg

/* Sidebar специфичные */
--sidebar-bg-top, --sidebar-bg-bottom
--sidebar-btn-bg, --sidebar-btn-hover-bg, --sidebar-btn-active-bg
--indicator-gradient-start, --indicator-gradient-end, --indicator-shadow

/* Status indicators */
--status-connected, --status-connected-glow
--status-disconnected, --status-disconnected-glow

/* Cards */
--card-active-border, --card-active-bg
--card-error-border, --card-error-bg

/* Buttons */
--btn-accent-bg, --btn-accent-bg-hover
--btn-neutral-bg, --btn-neutral-hover
--btn-disabled-bg

/* Form elements */
--select-bg, --select-bg-hover
--input-bg-strong, --range-bg

/* Tables */
--table-header-bg

/* Misc */
--kbd-bg, --kbd-shadow
--output-bg-dark
```

**Из файла `src/style.css` перенести:**
- Все RGB переменные
- Все производные цвета
- Эффекты (shadow)

**Из эталона добавить:**
- Семантические переменные для сообщений (success/warning/danger/info)
- Специфичные переменные для компонентов
- Переменные для toast notifications

#### 1.3 Миграция базовых стилей (`base.css`)
- [ ] Сброс стилей (`* { margin: 0; padding: 0; box-sizing: border-box; }`)
- [ ] HTML и BODY стили
- [ ] Фоновые градиенты body
- [ ] Стили скроллбара

#### 1.4 Миграция градиентов (`gradients.css`)
- [ ] Градиенты с использованием `color-mix()`
- [ ] Fallback для браузеров без `color-mix()`
- [ ] Grid pattern
- [ ] Кнопочные градиенты

**Градиенты для переноса:**
```css
--app-gradient-bg: linear-gradient(135deg, ...)
--app-gradient-glow: radial-gradient(...)
--app-gradient-line: linear-gradient(...)
--grid-pattern: ...
--btn-accent-gradient: ...
--success-gradient: ...
--danger-gradient: ...
```

#### 1.5 Обновление точек входа
- [ ] Обновить `src/style.css` — импорт из `styles/index.css`
- [ ] Убрать дублирующиеся стили

#### 1.6 Проверка
- [ ] Проверить отображение в тёмной теме
- [ ] Проверить отображение в светлой теме
- [ ] Проверить все цвета UI элементов

---

## Часть 2. Структура панелей

### Текущее состояние
```
src/components/
├── ConnectionsPanel.vue
├── FloatingPanel.vue
├── SettingsPanel.vue  (без табов)
└── Sidebar.vue
```

### Целевое состояние
```
src/components/
├── shared/                    (новое)
│   ├── StatusMessage.vue
│   ├── TestResult.vue
│   └── InputWithToggle.vue
├── settings/                  (новое)
│   ├── SettingsGeneral.vue
│   ├── SettingsConnections.vue (новое)
│   └── SettingsAppearance.vue  (новое)
├── ConnectionsPanel.vue        (обновить)
├── FloatingPanel.vue
├── SettingsPanel.vue           (с табами!)
└── Sidebar.vue
```

### Задачи

#### 2.1 Создать структуру директорий
- [ ] Создать `src/components/shared/`
- [ ] Создать `src/components/settings/`

#### 2.2 Обновить SettingsPanel с табовой системой

**Новая структура `SettingsPanel.vue`:**

```typescript
type TabType = 'general' | 'connections' | 'appearance'
const activeTab = ref<TabType>('general')
```

**Табы:**
1. **General** — общие настройки (тема, горячие клавиши, etc.)
2. **Connections** — настройки соединений (из текущей SettingsPanel)
3. **Appearance** — настройки внешнего вида (прозрачность, цвета)

#### 2.3 Создать табовые компоненты

**SettingsGeneral.vue:**
- Переключатель темы (тёмная/светлая)
- Горячие клавиши (включить/выключить)
- Логирование (включить/выключить, уровень)
- Исключение из захвата экрана

**SettingsConnections.vue:**
- Список соединений
- Добавление/редактирование/удаление
- Access token для соединений

**SettingsAppearance.vue:**
- Прозрачность floating panel
- Цвет фона floating panel
- Click-through режим
- Размеры и отступы

#### 2.4 Создать shared компоненты

**StatusMessage.vue:**
```typescript
props: {
  type: 'success' | 'warning' | 'error' | 'info'
  message: string
  timeout?: number
}
```

**TestResult.vue:**
```typescript
props: {
  success: boolean
  message: string
  details?: string
}
```

**InputWithToggle.vue:**
```typescript
props: {
  modelValue: string
  enabled: boolean
  label: string
  placeholder?: string
}
```

#### 2.5 Обновить App.vue
- [ ] Добавить новые типы панелей
- [ ] Обновить переключение панелей
- [ ] Добавить ErrorToasts компонент (если нужен)

#### 2.6 Проверка
- [ ] Проверить переключение табов
- [ ] Проверить сохранение настроек в каждом табе
- [ ] Проверить отображение сообщений об ошибках/успехе

---

## Часть 3. Работа с конфигом

### Текущее состояние
```rust
// src-tauri/src/config/settings.rs
pub struct AppSettings {
    pub connections: Vec<ConnectionConfig>,
    pub logging: LoggingSettings,
    pub theme: Theme,
}

pub struct SettingsManager {
    pub fn load(&self) -> AppSettings
    pub fn save(&self, settings: &AppSettings) -> Result<()>
    pub fn add_connection(&self, connection: ConnectionConfig) -> Result<()>
    pub fn remove_connection(&self, id: &str) -> Result<()>
    pub fn set_theme(&self, theme: Theme) -> Result<()>
}
```

### Целевое состояние

#### 3.1 Расширить AppSettings
```rust
pub struct AppSettings {
    // Connections
    pub connections: Vec<ConnectionConfig>,

    // Logging
    pub logging: LoggingSettings,

    // Theme
    pub theme: Theme,

    // Appearance (новое)
    pub appearance: AppearanceSettings,

    // Hotkeys (новое)
    pub hotkeys: HotkeySettings,

    // General (новое)
    pub general: GeneralSettings,
}

pub struct AppearanceSettings {
    pub floating_opacity: u8,
    pub floating_bg_color: String,
    pub floating_clickthrough: bool,
}

pub struct HotkeySettings {
    pub enabled: bool,
    pub toggle_window: Option<String>,
}

pub struct GeneralSettings {
    pub exclude_from_capture: bool,
    pub check_updates: bool,
}
```

#### 3.2 Атомарные обновления полей
Добавить методы для обновления отдельных полей без полной перезагрузки:

```rust
impl SettingsManager {
    // Theme
    pub fn set_theme(&self, theme: Theme) -> Result<()>

    // Appearance
    pub fn set_floating_opacity(&self, opacity: u8) -> Result<()>
    pub fn set_floating_bg_color(&self, color: String) -> Result<()>
    pub fn set_floating_clickthrough(&self, clickthrough: bool) -> Result<()>

    // Hotkeys
    pub fn set_hotkey_enabled(&self, enabled: bool) -> Result<()>
    pub fn set_toggle_window_hotkey(&self, hotkey: Option<String>) -> Result<()>

    // General
    pub fn set_exclude_from_capture(&self, exclude: bool) -> Result<()>

    // Logging
    pub fn set_logging_enabled(&self, enabled: bool) -> Result<()>
    pub fn set_logging_level(&self, level: String) -> Result<()>

    // Connections
    pub fn add_connection(&self, connection: ConnectionConfig) -> Result<()>
    pub fn remove_connection(&self, id: &str) -> Result<()>
    pub fn update_connection(&self, id: &str, updated: ConnectionConfig) -> Result<()>
}
```

#### 3.3 JSON Pointer для атомарных обновлений
Добавить возможность обновлять отдельные поля:

```rust
impl SettingsManager {
    pub fn update_field<T: Serialize>(&self, json_pointer: &str, value: &T) -> Result<()> {
        let mut settings = self.load();
        // Update field at JSON pointer path
        // Example: "/appearance/floating_opacity"
        self.save(&settings)
    }
}
```

#### 3.4 Event-driven обновления
Добавить события для обновления настроек:

```rust
// Events
pub enum SettingsEvent {
    ThemeChanged(Theme),
    AppearanceChanged(AppearanceSettings),
    HotkeysChanged(HotkeySettings),
    GeneralChanged(GeneralSettings),
    LoggingChanged(LoggingSettings),
    ConnectionsChanged(Vec<ConnectionConfig>),
}
```

#### 3.5 Добавить Tauri команды
```rust
#[tauri::command]
async fn update_theme(theme: String) -> Result<(), String>

#[tauri::command]
async fn set_floating_opacity(opacity: u8) -> Result<(), String>

#[tauri::command]
async fn set_floating_clickthrough(clickthrough: bool) -> Result<(), String>

#[tauri::command]
async fn set_hotkey_enabled(enabled: bool) -> Result<(), String>

#[tauri::command]
async fn set_logging_enabled(enabled: bool) -> Result<(), String>

#[tauri::command]
async fn set_logging_level(level: String) -> Result<(), String>

#[tauri::command]
async fn set_exclude_from_capture(exclude: bool) -> Result<(), String>
```

#### 3.6 Миграция с валидацией
Добавить валидацию при загрузке настроек:

```rust
impl AppSettings {
    pub fn with_defaults() -> Self {
        Self {
            connections: Vec::new(),
            logging: LoggingSettings::default(),
            theme: Theme::Dark,
            appearance: AppearanceSettings::default(),
            hotkeys: HotkeySettings::default(),
            general: GeneralSettings::default(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        // Validate opacity range
        if self.appearance.floating_opacity > 100 {
            return Err("floating_opacity must be 0-100".to_string());
        }
        // Validate hex color
        // Validate hotkey format
        Ok(())
    }
}
```

#### 3.7 Проверка
- [ ] Проверить сохранение всех настроек
- [ ] Проверить атомарные обновления
- [ ] Проверить валидацию
- [ ] Проверить события

---

## Часть 4. TypeScript типы

### Текущее состояние
```typescript
// src/types.ts
export type Theme = 'dark' | 'light'

export interface ConnectionConfig {
  id: string
  name: string
  url: string
  enabled: boolean
  access_token?: string
}

export interface FloatingSettings {
  opacity: number
  bgColor: string
  clickthrough: boolean
}
```

### Целевое состояние
```
src/types/
├── settings.ts    (264 строки — полная система типов)
├── types.ts       (остальные типы)
└── index.ts       (экспорт всех типов)
```

### Задачи

#### 4.1 Создать структуру директорий
- [ ] Создать `src/types/`
- [ ] Создать `settings.ts`, `types.ts`, `index.ts`

#### 4.2 Миграция типов настроек (`settings.ts`)

**Полный список типов:**

```typescript
// ============================================================================
// Theme
// ============================================================================
export type Theme = 'dark' | 'light'

// ============================================================================
// Connections
// ============================================================================
export interface ConnectionConfig {
  id: string
  name: string
  url: string
  enabled: boolean
  access_token?: string
}

// ============================================================================
// Logging
// ============================================================================
export interface LoggingSettingsDto {
  enabled: boolean
  level: string
  module_levels: Record<string, string>
}

// ============================================================================
// Appearance
// ============================================================================
export interface AppearanceSettingsDto {
  floating_opacity: number
  floating_bg_color: string
  floating_clickthrough: boolean
}

// ============================================================================
// Hotkeys
// ============================================================================
export interface HotkeySettingsDto {
  enabled: boolean
  toggle_window?: string
}

// ============================================================================
// General
// ============================================================================
export interface GeneralSettingsDto {
  exclude_from_capture: boolean
  check_updates: boolean
  theme?: Theme
}

// ============================================================================
// Windows
// ============================================================================
export interface WindowPositionDto {
  x?: number
  y?: number
}

export interface FloatingWindowSettingsDto {
  x?: number
  y?: number
  opacity: number
  bg_color: string
  clickthrough: boolean
}

export interface WindowsSettingsDto {
  main: WindowPositionDto
  floating: FloatingWindowSettingsDto
}

// ============================================================================
// Main Settings DTO
// ============================================================================
export interface AppSettingsDto {
  connections: ConnectionConfig[]
  logging: LoggingSettingsDto
  appearance: AppearanceSettingsDto
  hotkeys: HotkeySettingsDto
  general: GeneralSettingsDto
  windows: WindowsSettingsDto
}

// ============================================================================
// Injection Key
// ============================================================================
import { InjectionKey, Ref } from 'vue'

export interface AppSettingsContext {
  settings: Ref<AppSettingsDto | null>
  isLoading: Ref<boolean>
  error: Ref<string | null>
  reload: () => Promise<void>
  cleanup?: () => void
}

export const APP_SETTINGS_KEY: InjectionKey<AppSettingsContext> =
  Symbol('app-settings')
```

#### 4.3 Создать index.ts
```typescript
export * from './settings'
export * from './types'
```

#### 4.4 Обновить импорты в компонентах
Заменить:
```typescript
import type { ConnectionConfig } from './types'
```
На:
```typescript
import type { ConnectionConfig } from '@/types'
```

#### 4.5 Проверка
- [ ] Проверить типы во всех компонентах
- [ ] Проверить соответствие с Rust DTO

---

## Часть 5. Composables

### Текущее состояние
```typescript
// src/composables/useAppSettings.ts
export function useAppSettings() {
  return {
    settings: globalSettings,
    isLoading: globalIsLoading,
    error: globalError,
    reload: reloadSettings,
    setTheme,
  }
}
```

### Целевое состояние

#### 5.1 Обновить useAppSettings с dependency injection

```typescript
// src/composables/useAppSettings.ts
import { ref, provide, inject, onScopeDispose } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { AppSettingsDto, AppSettingsContext } from '@/types'
import { APP_SETTINGS_KEY } from '@/types'

// Create context (for root component)
export function createAppSettings(): AppSettingsContext {
  const settings = ref<AppSettingsDto | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  let cleanupListeners: (() => void) | null = null

  async function load() { /* ... */ }
  async function reload() { /* ... */ }

  async function setupEventListeners() {
    const unlistenSettingsChanged = await listen('settings-changed', reload)
    const unlistenThemeChanged = await listen('theme-changed', reload)
    const unlistenConnectionsChanged = await listen('connections-changed', reload)

    return () => {
      unlistenSettingsChanged()
      unlistenThemeChanged()
      unlistenConnectionsChanged()
    }
  }

  load()
  setupEventListeners().then((cleanup) => {
    cleanupListeners = cleanup
    onScopeDispose(cleanup)
  })

  return {
    settings,
    isLoading,
    error,
    reload,
    cleanup: () => cleanupListeners?.()
  }
}

// Provide app settings (App.vue)
export function provideAppSettings(): AppSettingsContext {
  const context = createAppSettings()
  provide(APP_SETTINGS_KEY, context)
  return context
}

// Inject app settings (child components)
export function useAppSettings(): AppSettingsContext {
  const context = inject<AppSettingsContext>(APP_SETTINGS_KEY)
  if (!context) {
    throw new Error('useAppSettings must be used within a component that provides app settings')
  }
  return context
}
```

#### 5.2 Специализированные composables

```typescript
// src/composables/useAppSettings.ts (продолжение)

export function useConnectionsSettings(): ComputedRef<ConnectionConfig[]> {
  const { settings } = useAppSettings()
  return computed(() => settings.value?.connections ?? [])
}

export function useLoggingSettings(): ComputedRef<LoggingSettingsDto | undefined> {
  const { settings } = useAppSettings()
  return computed(() => settings.value?.logging)
}

export function useAppearanceSettings(): ComputedRef<AppearanceSettingsDto | undefined> {
  const { settings } = useAppSettings()
  return computed(() => settings.value?.appearance)
}

export function useHotkeySettings(): ComputedRef<HotkeySettingsDto | undefined> {
  const { settings } = useAppSettings()
  return computed(() => settings.value?.hotkeys)
}

export function useGeneralSettings(): ComputedRef<GeneralSettingsDto | undefined> {
  const { settings } = useAppSettings()
  return computed(() => settings.value?.general)
}

export function useWindowsSettings(): ComputedRef<WindowsSettingsDto | undefined> {
  const { settings } = useAppSettings()
  return computed(() => settings.value?.windows)
}
```

#### 5.3 Обновить App.vue

```vue
<script setup lang="ts">
import { provideAppSettings } from './composables/useAppSettings'

// Create and provide app settings context
const appSettings = provideAppSettings()

// Watch for theme changes
watch(() => appSettings.settings.value?.general?.theme, (newTheme) => {
  const theme = newTheme || 'dark'
  localStorage.setItem('app-theme', theme)
  document.documentElement.setAttribute('data-theme', theme)
}, { immediate: true })
</script>
```

#### 5.4 Обновить компоненты настроек

**SettingsGeneral.vue:**
```typescript
import { useGeneralSettings, useLoggingSettings } from '@/composables/useAppSettings'

const generalSettings = useGeneralSettings()
const loggingSettings = useLoggingSettings()
```

**SettingsConnections.vue:**
```typescript
import { useConnectionsSettings } from '@/composables/useAppSettings'

const connections = useConnectionsSettings()
```

**SettingsAppearance.vue:**
```typescript
import { useAppearanceSettings } from '@/composables/useAppSettings'

const appearance = useAppearanceSettings()
```

#### 5.5 Проверка
- [ ] Проверить provide/inject
- [ ] Проверить специализированные composables
- [ ] Проверить event listeners cleanup
- [ ] Проверить HMR compatibility

---

## Порядок выполнения

Рекомендуемый порядок для минимизации конфликтов и тестирования:

### Фаза 1: CSS и типы (независимые изменения)
1. **CSS Архитектура** — Часть 1
   - Создать структуру директорий
   - Перенести переменные, базовые стили, градиенты
   - Обновить импорты
   - Тест: проверка отображения

2. **TypeScript типы** — Часть 4
   - Создать структуру директорий
   - Создать `settings.ts` с полными типами
   - Создать `index.ts`
   - Обновить импорты
   - Тест: проверка типов

### Фаза 2: Composables и состояние
3. **Composables** — Часть 5
   - Обновить `useAppSettings` с DI
   - Добавить специализированные composables
   - Обновить `App.vue`
   - Тест: проверка работы composables

### Фаза 3: Backend конфигурация
4. **Работа с конфигом** — Часть 3
   - Расширить `AppSettings`
   - Добавить атомарные обновления
   - Добавить Tauri команды
   - Добавить события
   - Тест: проверка сохранения настроек

### Фаза 4: UI компоненты
5. **Структура панелей** — Часть 2
   - Создать структуру директорий
   - Создать shared компоненты
   - Создать settings табы
   - Обновить SettingsPanel
   - Обновить остальные компоненты
   - Тест: проверка UI

---

## Файловая структура после миграции

```
ttsbard-echo/
├── src/
│   ├── styles/                 (новое)
│   │   ├── index.css
│   │   ├── variables.css
│   │   ├── base.css
│   │   └── gradients.css
│   ├── types/                  (новое)
│   │   ├── settings.ts
│   │   ├── types.ts
│   │   └── index.ts
│   ├── components/
│   │   ├── shared/             (новое)
│   │   │   ├── StatusMessage.vue
│   │   │   ├── TestResult.vue
│   │   │   └── InputWithToggle.vue
│   │   ├── settings/           (новое)
│   │   │   ├── SettingsGeneral.vue
│   │   │   ├── SettingsConnections.vue
│   │   │   └── SettingsAppearance.vue
│   │   ├── ConnectionsPanel.vue
│   │   ├── FloatingPanel.vue
│   │   ├── SettingsPanel.vue   (обновлён)
│   │   └── Sidebar.vue
│   ├── composables/
│   │   └── useAppSettings.ts   (обновлён)
│   ├── App.vue                 (обновлён)
│   └── main.ts
└── src-tauri/
    └── src/
        └── config/
            ├── settings.rs     (обновлён)
            └── dto.rs          (обновлён)
```

---

## Проверка и тестирование

### CSS проверка
- [ ] Тёмная тема отображается корректно
- [ ] Светлая тема отображается корректно
- [ ] Все семантические цвета применяются
- [ ] Градиенты отображаются корректно
- [ ] Скроллбар стилизован

### TypeScript проверка
- [ ] Нет ошибок компиляции TypeScript
- [ ] Типы соответствуют Rust DTO
- [ ] Все импорты работают

### Composables проверка
- [ ] `provideAppSettings` создаёт контекст
- [ ] `useAppSettings` внедряет контекст
- [ ] Специализированные composables возвращают правильные типы
- [ ] Event listeners очищаются при unmount
- [ ] HMR работает корректно

### Config проверка
- [ ] Все настройки сохраняются
- [ ] Атомарные обновления работают
- [ ] Валидация работает
- [ ] События отправляются корректно
- [ ] Настройки загружаются при старте

### UI проверка
- [ ] Все панели переключаются
- [ ] Табы в SettingsPanel работают
- [ ] Shared компоненты отображаются
- [ ] Сообщения об ошибках/успехе показываются
- [ ] Настройки применяются немедленно

---

## Примечания

1. **CSS переменные:** Использовать семантические имена вместо цветовых (например, `--success-bg` вместо `--green-bg`)

2. **TypeScript:** Использовать `export const` для enum-like констант для лучшей типизации

3. **Composables:** Всегда предоставлять cleanup функцию для event listeners

4. **Config:** Использовать JSON pointer для атомарных обновлений полей

5. **Events:** Отправлять события только для изменившихся полей, не для всех настроек

6. **Тестирование:** После каждой фазы запускать полное тестирование

---

## Часть 6. Дополнительные паттерны (опционально)

### 6.1 Переиспользуемые компоненты для карточек

**Проблема:** ConnectionsPanel.vue содержит 715 строк, дублирование UI

**Решение:** Создать `ProviderCard.vue` компонент

```vue
<!-- src/components/shared/ProviderCard.vue -->
<script setup lang="ts">
interface Props {
  title: string
  icon?: Component
  active?: boolean
  expanded?: boolean
  disabled?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  active: false,
  expanded: false,
  disabled: false,
})

const emit = defineEmits<{
  (e: 'toggle'): void
  (e: 'select'): void
}>()
</script>

<template>
  <div class="provider-card" :class="{ active, disabled }">
    <div class="card-header" @click="emit('select')">
      <input type="radio" :checked="active" />
      <component v-if="icon" :is="icon" :size="18" />
      <span class="card-title">{{ title }}</span>
      <span class="expand-icon" @click.stop="emit('toggle')">
        {{ expanded ? '▼' : '▶' }}
      </span>
    </div>
    <div v-if="expanded" class="card-content">
      <slot />
    </div>
  </div>
</template>
```

**Использование:**
```vue
<ProviderCard
  title="OpenAI TTS"
  :icon="Bot"
  :active="provider === 'openai'"
  :expanded="expandedCards.has('openai')"
  @select="selectProvider('openai')"
  @toggle="toggleCard('openai')"
>
  <!-- Card content -->
</ProviderCard>
```

### 6.2 Улучшенная система сообщений

**Текущее состояние:** Встроенная в каждый компонент, дублирование кода

**Целевое состояние:** Переиспользуемый `StatusMessage.vue`

```vue
<!-- src/components/shared/StatusMessage.vue -->
<script setup lang="ts">
interface Props {
  message: string
  type?: 'success' | 'error' | 'info'
  autoHide?: boolean
  autoHideDelay?: number
  dismissible?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  type: 'info',
  autoHide: true,
  autoHideDelay: 3000,
  dismissible: true,
})

const emit = defineEmits<{
  (e: 'dismiss'): void
}>()

// Auto-hide logic
let timeoutId: ReturnType<typeof setTimeout> | null = null
if (props.autoHide && props.message) {
  timeoutId = setTimeout(() => emit('dismiss'), props.autoHideDelay)
}

onUnmounted(() => {
  if (timeoutId !== null) clearTimeout(timeoutId)
})
</script>

<template>
  <Transition name="fade-slide">
    <div v-if="message" class="status-message" :class="type">
      <component :is="iconMap[type]" :size="16" />
      <span>{{ message }}</span>
      <button v-if="dismissible" class="status-close" @click="emit('dismiss')">
        <X :size="14" />
      </button>
    </div>
  </Transition>
</template>

<style scoped>
.status-message {
  position: fixed;
  top: 20px;
  left: calc(50% + 100px);
  transform: translateX(-50%);
  padding: 0.4rem 0.75rem;
  border-radius: 8px;
  z-index: 1000;
  backdrop-filter: blur(10px);
}

.status-message.success {
  background: var(--success-bg);
  border: 1px solid var(--success-border);
  color: var(--success-text);
}

.status-message.error {
  background: var(--danger-bg);
  border-left: 4px solid var(--status-disconnected);
  color: var(--danger-text);
}
</style>
```

### 6.3 Расширенная система событий

**Текущее состояние:** 6 событий

**Целевое состояние:** 20+ событий для гранулярных обновлений

```rust
// src-tauri/src/events.rs
pub enum AppEvent {
    // Theme
    ThemeChanged(Theme),

    // Floating Window
    FloatingAppearanceChanged,
    ClickthroughChanged(bool),
    ShowFloatingWindow,
    HideFloatingWindow,
    UpdateFloatingText(String),

    // Connections
    ConnectionsChanged,
    ConnectionStatusChanged(String, ConnectionStatus),
    MessageReceived(String, String),

    // Settings
    SettingsChanged,
    LoggingChanged(LoggingSettings),
    AppearanceChanged(AppearanceSettings),
    HotkeysChanged(HotkeySettings),

    // General
    BackendReady,
    AppQuit,
}

impl AppEvent {
    pub fn to_tauri_event(&self) -> &'static str {
        match self {
            AppEvent::ThemeChanged(_) => "theme-changed",
            AppEvent::FloatingAppearanceChanged => "floating-appearance-changed",
            AppEvent::ClickthroughChanged(_) => "clickthrough-changed",
            AppEvent::ShowFloatingWindow => "show-floating-window",
            AppEvent::HideFloatingWindow => "hide-floating-window",
            AppEvent::UpdateFloatingText(_) => "update-floating-text",
            AppEvent::ConnectionsChanged => "connections-changed",
            AppEvent::ConnectionStatusChanged(_, _) => "connection-status-changed",
            AppEvent::MessageReceived(_, _) => "message-received",
            AppEvent::SettingsChanged => "settings-changed",
            AppEvent::LoggingChanged(_) => "logging-changed",
            AppEvent::AppearanceChanged(_) => "appearance-changed",
            AppEvent::HotkeysChanged(_) => "hotkeys-changed",
            AppEvent::BackendReady => "backend-ready",
            AppEvent::AppQuit => "app-quit",
        }
    }
}
```

### 6.4 Специализированные Tauri команды

**Текущее состояние:** 4 команды

**Целевое состояние:** 15+ команд для атомарных обновлений

```rust
// src-tauri/src/commands/settings.rs

#[tauri::command]
async fn update_theme(theme: Theme) -> Result<(), String> { /* ... */ }

#[tauri::command]
async fn set_floating_opacity(opacity: u8) -> Result<(), String> { /* ... */ }

#[tauri::command]
async fn set_floating_bg_color(color: String) -> Result<(), String> { /* ... */ }

#[tauri::command]
async fn set_floating_clickthrough(enabled: bool) -> Result<(), String> { /* ... */ }

#[tauri::command]
async fn set_hotkey_enabled(enabled: bool) -> Result<(), String> { /* ... */ }

#[tauri::command]
async fn set_hotkey_toggle_window(hotkey: Option<String>) -> Result<(), String> { /* ... */ }

#[tauri::command]
async fn set_logging_enabled(enabled: bool) -> Result<(), String> { /* ... */ }

#[tauri::command]
async fn set_logging_level(level: String) -> Result<(), String> { /* ... */ }

#[tauri::command]
async fn set_exclude_from_capture(exclude: bool) -> Result<(), String> { /* ... */ }

#[tauri::command]
async fn set_check_updates(enabled: bool) -> Result<(), String> { /* ... */ }

#[tauri::command]
async fn add_connection(config: ConnectionConfig) -> Result<String, String> { /* ... */ }

#[tauri::command]
async fn remove_connection(id: String) -> Result<(), String> { /* ... */ }

#[tauri::command]
async fn update_connection(id: String, config: ConnectionConfig) -> Result<(), String> { /* ... */ }

#[tauri::command]
async fn connect_connection(id: String) -> Result<(), String> { /* ... */ }

#[tauri::command]
async fn disconnect_connection(id: String) -> Result<(), String> { /* ... */ }
```

### 6.5 Backend-ready паттерн

**Проблема:** Frontend может загружаться раньше backend

**Решение:** Реализовать backend-ready handshake

```rust
// src-tauri/src/lib.rs
#[tauri::command]
async fn is_backend_ready(state: State<'_, AppState>) -> bool {
    state.backend_ready.load(Ordering::Relaxed)
}

#[tauri::command]
async fn confirm_backend_ready(state: State<'_, AppState>) {
    if !state.backend_ready.load(Ordering::Relaxed) {
        state.backend_ready.store(true, Ordering::Relaxed);
        state.emit_event(AppEvent::BackendReady);
    }
}

// В setup()
app.emit("backend-ready", ());
```

```typescript
// src/composables/useAppSettings.ts
const MAX_RETRIES = 50
const RETRY_INTERVAL_MS = 100

async function waitForBackendReady(): Promise<boolean> {
  for (let i = 0; i < MAX_RETRIES; i++) {
    try {
      const ready = await invoke<boolean>('is_backend_ready')
      if (ready) return true
    } catch (e) {
      console.warn('Failed to check backend ready:', e)
    }

    // Try to confirm (will emit event if ready)
    try {
      await invoke('confirm_backend_ready')
    } catch {
      // Ignore errors
    }

    await new Promise(resolve => setTimeout(resolve, RETRY_INTERVAL_MS))
  }
  return false
}
```

---

## Приложение A: Таблица соответствия событий

| Текущий проект | Эталонный проект | Описание |
|----------------|------------------|----------|
| `SettingsChanged` | `settings-changed` | Общие изменения настроек |
| `ConnectionStatusChanged(id, status)` | `connection-status-changed` | Статус соединения |
| `MessageReceived(id, msg)` | `message-received` | Получено сообщение |
| — | `theme-changed` | Изменена тема |
| — | `floating-appearance-changed` | Изменён внешний вид floating |
| — | `clickthrough-changed` | Изменён click-through |
| — | `backend-ready` | Backend готов |
| — | `logging-changed` | Изменены настройки логов |
| — | `appearance-changed` | Изменены настройки внешнего вида |
| — | `hotkeys-changed` | Изменены горячие клавиши |

---

## Приложение B: Сравнение команд Tauri

| Текущий проект | Эталонный проект | Новое |
|----------------|------------------|-------|
| `get_settings` | `get_all_app_settings` | |
| `save_logging_settings` | `save_logging_settings` | |
| `update_theme` | `update_theme` | |
| — | `set_floating_opacity` | ✅ |
| — | `set_floating_bg_color` | ✅ |
| — | `set_floating_clickthrough` | ✅ |
| — | `set_hotkey_enabled` | ✅ |
| — | `set_exclude_from_capture` | ✅ |
| `add_connection` | `add_connection` | |
| `remove_connection` | `remove_connection` | |
| — | `update_connection` | ✅ |
| — | `connect_connection` | ✅ |
| — | `disconnect_connection` | ✅ |
| — | `is_backend_ready` | ✅ |
| — | `confirm_backend_ready` | ✅ |

---

## Приложение C: Чек-лист миграции

### Фаза 1: Фундамент
- [ ] CSS модульная структура
- [ ] TypeScript типы настроек
- [ ] Dependency injection для настроек

### Фаза 2: Backend
- [ ] Расширенные AppSettings
- [ ] Атомарные обновления
- [ ] Новые Tauri команды
- [ ] Расширенные события

### Фаза 3: Frontend
- [ ] Shared компоненты (StatusMessage, ProviderCard)
- [ ] Settings табы
- [ ] Специализированные composables
- [ ] Backend-ready handshake

### Фаза 4: Полировка
- [ ] Тестирование всех фич
- [ ] Документация
- [ ] Оптимизация производительности
