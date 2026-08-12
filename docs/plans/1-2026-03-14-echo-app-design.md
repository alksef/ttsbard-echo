# ttsbard-echo Application Design

**Date:** 2026-03-14
**Status:** Approved
**Author:** Claude (brainstorming session)

## Overview

Tauri 2 desktop application for connecting to external WebView Source (HTTP/SSE) servers and displaying connection events in a floating panel. The application serves as an SSE client that receives events from external sources and displays them in a minimal UI.

## Architecture

### High-Level Diagram

```
┌─────────────────┐     SSE      ┌──────────────────┐
│ External Server │ ──────────>  │ ttsbard-echo     │
│ (HTTP/SSE)      │   Events     │                  │
└─────────────────┘              │ ┌──────────────┐ │
                                 │ │ SSE Client   │ │
                                 │ └──────────────┘ │
                                 │         ↓        │
                                 │ ┌──────────────┐ │
                                 │ │ Event System │ │
                                 │ └──────────────┘ │
                                 │         ↓        │
                                 │ ┌──────────────┐ │
                                 │ │ Floating UI  │ │
                                 │ └──────────────┘ │
                                 └──────────────────┘
```

### Technology Stack

- **Frontend:** Vue 3 + TypeScript + Vite
- **Backend:** Rust + Tauri 2
- **Storage:** JSON file in %APPDATA%
- **Logging:** tracing + tracing-subscriber
- **Concurrency:** tokio + parking_lot

## Components

### Frontend Components

| Component | File | Responsibility |
|-----------|------|----------------|
| App.vue | src/App.vue | Main layout, sidebar routing |
| Sidebar.vue | src/components/Sidebar.vue | 2-button sidebar (Connections, Settings) |
| ConnectionsPanel.vue | src/components/ConnectionsPanel.vue | Floating window content |
| SettingsPanel.vue | src/components/SettingsPanel.vue | Settings management UI |

### Backend Modules

| Module | File | Responsibility |
|--------|------|----------------|
| Config | config/mod.rs | Settings management, validation |
| State | state.rs | Application state (Arc-wrapped) |
| Events | events.rs | Event enums (AppEvent, ConnectionStatus) |
| Event Loop | event_loop.rs | Event processing handlers |
| Connections | connections/ | SSE client implementation |
| Floating | floating.rs | Floating window management |
| Commands | commands/ | Tauri commands (frontend API) |
| Setup | setup.rs | App initialization |

## Data Structures

### Settings (settings.json)

```rust
pub struct AppSettings {
    pub connections: Vec<ConnectionConfig>,
    pub windows: WindowsSettings,
    pub logging: LoggingSettings,
}

pub struct ConnectionConfig {
    pub id: String,
    pub name: String,
    pub url: String,
    pub enabled: bool,
}

pub struct WindowsSettings {
    pub main: WindowPosition,
    pub floating: FloatingWindowSettings,
}

pub struct FloatingWindowSettings {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub opacity: u8,        // 10-100
    pub bg_color: String,   // hex #RRGGBB
    pub clickthrough: bool,
}
```

### Events

```rust
pub enum AppEvent {
    ConnectionAdded(String),
    ConnectionRemoved(String),
    ConnectionStatusChanged(ConnectionId, ConnectionStatus),
    MessageReceived(ConnectionId, String),
    FloatingWindowToggled,
    FloatingAppearanceChanged,
}

pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}
```

### State

```rust
pub struct AppState {
    pub event_sender: Arc<Mutex<Option<Sender<AppEvent>>>>,
    pub connections: Arc<RwLock<HashMap<ConnectionId, ConnectionState>>>,
    pub settings_manager: Arc<RwLock<SettingsManager>>,
    pub windows_manager: Arc<RwLock<WindowsManager>>,
    pub runtime: Arc<tokio::runtime::Runtime>,
}

pub struct ConnectionState {
    pub config: ConnectionConfig,
    pub status: ConnectionStatus,
    pub last_message: Option<String>,
    pub last_message_time: Option<DateTime<Utc>>,
}
```

## UI Design

### Main Window

```
┌─────────────────────────────────────┐
│ ┌─────────┬                         │
│ │ Sidebar │ Main Content           │
│ │         │                         │
│ │ - Conn  │ [Settings Panel]        │
│ │ - Sett  │                         │
│ └─────────┘                         │
└─────────────────────────────────────┘
```

### Floating Panel

```
┌─────────────────────────────────────┐
│ ПОДКЛЮЧЕНИЯ                          │
├─────────────────────────────────────┤
│ [🌐] Source 1    ● Connected         │
│   Последнее: Привет от источника...  │
├─────────────────────────────────────┤
│ [🌐] Source 2    ○ Disconnected      │
│   Нет соединения                     │
├─────────────────────────────────────┤
│ [+] Добавить подключение            │
└─────────────────────────────────────┘
```

## User Flows

### Adding a Connection

1. User clicks "Настройки" in sidebar
2. User clicks "Добавить подключение"
3. User enters name and URL
4. Connection added to settings.json
5. SSE client attempts connection
6. Status updates in floating panel

### Viewing Messages

1. User clicks "Подключения" in sidebar
2. Floating panel appears at saved position
3. Connection statuses displayed
4. Messages appear when received
5. Panel saves position on close

## Key Patterns

### Settings Management

- **In-memory cache:** RwLock<AppSettings> for fast reads
- **Write-through:** Save to disk on every update
- **Validation:** Invalid values corrected on load
- **Atomic updates:** Individual field setters

### Event System

- **Enum-based:** Type-safe, serializable events
- **MPSC channel:** Internal event routing
- **Tauri emit:** Frontend notification
- **Handler pattern:** Process events in event_loop.rs

### Window Management

- **Position persistence:** Save on hide, restore on show
- **Settings sync:** Apply appearance on show
- **Platform features:** Windows API for capture exclusion

## Dependencies

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
tracing-appender = "0.2"
parking_lot = "0.12"
dirs = "5"
reqwest = { version = "0.12", features = ["json"] }
eventsource-client = "0.12"
```

## File Locations

| Type | Location |
|------|----------|
| Settings | %APPDATA%\ttsbard-echo\settings.json |
| Windows | %APPDATA%\ttsbard-echo\windows.json |
| Logs | %APPDATA%\ttsbard-echo\logs\ttsbard-echo.log |

## Security Considerations

- URLs validated before connection
- Error messages don't expose sensitive data
- Click-through mode toggleable
- No credential storage (URLs only)

## Success Criteria

- [ ] Connect to external SSE servers
- [ ] Display connection status in real-time
- [ ] Show messages in floating panel
- [ ] Persist window position and appearance
- [ ] Add/remove/edit connections
- [ ] Log events to file
- [ ] Handle connection errors gracefully
