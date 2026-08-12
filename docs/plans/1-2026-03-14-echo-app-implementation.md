# ttsbard-echo Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Tauri 2 desktop application that connects to external HTTP/SSE servers and displays events in a floating panel.

**Architecture:** Vue 3 frontend with Rust/Tauri 2 backend. SSE clients connect to external servers, events flow through internal channel to UI. Settings persisted to JSON in %APPDATA%.

**Tech Stack:** Vue 3, TypeScript, Vite, Rust, Tauri 2, tokio, tracing, serde, eventsource-client

---

## Task 1: Initialize Tauri 2 Project Structure

**Files:**
- Create: `package.json`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/src/main.rs`
- Create: `vite.config.ts`
- Create: `tsconfig.json`

**Step 1: Create package.json**

```bash
cat > package.json << 'EOF'
{
  "name": "ttsbard-echo",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2",
    "lucide-vue-next": "^0.577.0",
    "vue": "^3.5.30"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2",
    "@vitejs/plugin-vue": "^6.0.4",
    "typescript": "~5.9.3",
    "vite": "^8.0.0",
    "vue-tsc": "^3.2.5"
  }
}
EOF
```

**Step 2: Create Cargo.toml**

```bash
cat > src-tauri/Cargo.toml << 'EOF'
[package]
name = "ttsbard-echo"
version = "0.1.0"
edition = "2021"

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
chrono = { version = "0.4", features = ["serde"] }

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
EOF
```

**Step 3: Create tauri.conf.json**

```bash
cat > src-tauri/tauri.conf.json << 'EOF'
{
  "$schema": "https://schema.tauri.app/config/2.0.0",
  "productName": "ttsbard-echo",
  "version": "0.1.0",
  "identifier": "com.ttsbard.echo",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "ttsbard-echo",
        "width": 900,
        "height": 700,
        "resizable": true,
        "decorations": true
      },
      {
        "label": "floating",
        "title": "Connections",
        "width": 350,
        "height": 400,
        "resizable": false,
        "decorations": false,
        "alwaysOnTop": true,
        "skipTaskbar": true,
        "visible": false,
        "transparent": true
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": []
  }
}
EOF
```

**Step 4: Create main.rs**

```bash
cat > src-tauri/src/main.rs << 'EOF'
// Prevents additional console window on Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    ttsbard_echo::run()
}
EOF
```

**Step 5: Create vite.config.ts**

```bash
cat > vite.config.ts << 'EOF'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src')
    }
  },
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**']
    }
  }
})
EOF
```

**Step 6: Create tsconfig.json**

```bash
cat > tsconfig.json << 'EOF'
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "preserve",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["src/**/*.ts", "src/**/*.d.ts", "src/**/*.tsx", "src/**/*.vue"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
EOF
```

**Step 7: Create tsconfig.node.json**

```bash
cat > tsconfig.node.json << 'EOF'
{
  "compilerOptions": {
    "composite": true,
    "skipLibCheck": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowSyntheticDefaultImports": true
  },
  "include": ["vite.config.ts"]
}
EOF
```

**Step 8: Install dependencies**

```bash
npm install
cd src-tauri && cargo fetch
```

**Step 9: Commit**

```bash
git add .
git commit -m "feat: initialize Tauri 2 project structure with Vue 3"
```

---

## Task 2: Create Logging System

**Files:**
- Create: `src-tauri/src/lib.rs` (logging setup)
- Modify: `src-tauri/Cargo.toml` (add tracing dependencies - already done in Task 1)

**Step 1: Create lib.rs with logging setup**

```bash
cat > src-tauri/src/lib.rs << 'EOF'
use std::path::PathBuf;
use tauri::Manager;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Clone)]
pub struct LogGuard {
    _guard: Option<Box<WorkerGuard>>,
}

// Leak the guard to keep logging alive for program duration
fn leak_guard(guard: WorkerGuard) -> LogGuard {
    LogGuard {
        _guard: Some(Box::leak(Box::new(guard))),
    }
}

pub fn init_logging() -> LogGuard {
    let config_dir = dirs::config_dir()
        .expect("Failed to get config dir")
        .join("ttsbard-echo");

    let log_dir = config_dir.join("logs");
    std::fs::create_dir_all(&log_dir)
        .expect("Failed to create log directory");

    let log_file = log_dir.join("ttsbard-echo.log");
    let file_appender = tracing_appender::rolling::never(log_dir, "ttsbard-echo.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::from_default_env()
        .add_directive("ttsbard_echo=debug".parse().unwrap())
        .add_directive("tauri=warn".parse().unwrap());

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(fmt::layer().with_writer(non_blocking))
        .init();

    tracing::info!("Logging initialized. Log file: {:?}", log_file);

    leak_guard(guard)
}

pub fn run() {
    let _log_guard = init_logging();

    tauri::Builder::default()
        .setup(|app| {
            tracing::info!("Application starting");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
EOF
```

**Step 2: Test build**

```bash
cd src-tauri && cargo build
```

Expected: SUCCESS with logging initialized

**Step 3: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: add tracing-based logging system"
```

---

## Task 3: Create Config Module

**Files:**
- Create: `src-tauri/src/config/mod.rs`
- Create: `src-tauri/src/config/settings.rs`
- Create: `src-tauri/src/config/windows.rs`
- Create: `src-tauri/src/config/validation.rs`
- Create: `src-tauri/src/config/constants.rs`
- Create: `src-tauri/src/config/dto.rs`

**Step 1: Create config/mod.rs**

```bash
mkdir -p src-tauri/src/config
cat > src-tauri/src/config/mod.rs << 'EOF'
pub mod constants;
pub mod dto;
pub mod settings;
pub mod validation;
pub mod windows;

pub use settings::SettingsManager;
pub use windows::WindowsManager;
pub use dto::AppSettingsDto;
pub use validation::{is_valid_hex_color, validate_opacity};
pub use constants::{DEFAULT_FLOATING_BG_COLOR, DEFAULT_FLOATING_OPACITY, DEFAULT_LOG_LEVEL};
EOF
```

**Step 2: Create config/constants.rs**

```bash
cat > src-tauri/src/config/constants.rs << 'EOF'
pub const DEFAULT_FLOATING_OPACITY: u8 = 90;
pub const DEFAULT_FLOATING_BG_COLOR: &str = "#1e1e1e";
pub const DEFAULT_LOG_LEVEL: &str = "info";
EOF
```

**Step 3: Create config/validation.rs**

```bash
cat > src-tauri/src/config/validation.rs << 'EOF'
pub fn is_valid_hex_color(color: &str) -> bool {
    color.len() == 7
        && color.starts_with('#')
        && color[1..].chars().all(|c| c.is_ascii_hexdigit())
}

pub fn validate_opacity(opacity: u8) -> u8 {
    opacity.clamp(10, 100)
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid hex color format: {0}")]
    InvalidColor(String),
    #[error("Invalid opacity: {0}")]
    InvalidOpacity(String),
}
EOF
```

**Step 4: Create config/settings.rs**

```bash
cat > src-tauri/src/config/settings.rs << 'EOF'
use crate::config::{constants::DEFAULT_FLOATING_BG_COLOR, validation::is_valid_hex_color};
use anyhow::Result;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub id: String,
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSettings {
    pub enabled: bool,
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub module_levels: HashMap<String, String>,
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for LoggingSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            level: default_log_level(),
            module_levels: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub connections: Vec<ConnectionConfig>,
    #[serde(default)]
    pub logging: LoggingSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            connections: Vec::new(),
            logging: LoggingSettings::default(),
        }
    }
}

pub struct SettingsManager {
    config_dir: PathBuf,
    cache: Arc<RwLock<AppSettings>>,
}

impl SettingsManager {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get config dir"))?
            .join("ttsbard-echo");

        std::fs::create_dir_all(&config_dir)?;

        let settings_file = config_dir.join("settings.json");
        let settings = if settings_file.exists() {
            let content = std::fs::read_to_string(&settings_file)?;
            serde_json::from_str(&content)?
        } else {
            let settings = AppSettings::default();
            let content = serde_json::to_string_pretty(&settings)?;
            std::fs::write(&settings_file, content)?;
            settings
        };

        Ok(Self {
            config_dir,
            cache: Arc::new(RwLock::new(settings)),
        })
    }

    pub fn load(&self) -> AppSettings {
        self.cache.read().clone()
    }

    pub fn save(&self, settings: &AppSettings) -> Result<()> {
        let settings_file = self.config_dir.join("settings.json");
        let content = serde_json::to_string_pretty(settings)?;
        std::fs::write(&settings_file, content)?;
        *self.cache.write() = settings.clone();
        Ok(())
    }

    pub fn add_connection(&self, connection: ConnectionConfig) -> Result<()> {
        let mut settings = self.load();
        settings.connections.push(connection);
        self.save(&settings)
    }

    pub fn remove_connection(&self, id: &str) -> Result<()> {
        let mut settings = self.load();
        settings.connections.retain(|c| c.id != id);
        self.save(&settings)
    }

    pub fn update_connection(&self, id: &str, updated: ConnectionConfig) -> Result<()> {
        let mut settings = self.load();
        if let Some(conn) = settings.connections.iter_mut().find(|c| c.id == id) {
            *conn = updated;
            self.save(&settings)
        } else {
            Err(anyhow::anyhow!("Connection not found: {}", id))
        }
    }
}
EOF
```

**Step 5: Create config/windows.rs**

```bash
cat > src-tauri/src/config/windows.rs << 'EOF'
use crate::config::{constants::DEFAULT_FLOATING_BG_COLOR, constants::DEFAULT_FLOATING_OPACITY, validation::validate_opacity};
use anyhow::Result;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WindowPosition {
    pub x: Option<i32>,
    pub y: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloatingWindowSettings {
    #[serde(default)]
    pub position: WindowPosition,
    #[serde(default = "default_opacity")]
    pub opacity: u8,
    #[serde(default = "default_bg_color")]
    pub bg_color: String,
    #[serde(default)]
    pub clickthrough: bool,
}

fn default_opacity() -> u8 {
    DEFAULT_FLOATING_OPACITY
}

fn default_bg_color() -> String {
    DEFAULT_FLOATING_BG_COLOR.to_string()
}

impl Default for FloatingWindowSettings {
    fn default() -> Self {
        Self {
            position: WindowPosition::default(),
            opacity: default_opacity(),
            bg_color: default_bg_color(),
            clickthrough: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WindowsSettings {
    #[serde(default)]
    pub floating: FloatingWindowSettings,
}

pub struct WindowsManager {
    config_dir: PathBuf,
    cache: Arc<RwLock<WindowsSettings>>,
}

impl WindowsManager {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get config dir"))?
            .join("ttsbard-echo");

        std::fs::create_dir_all(&config_dir)?;

        let windows_file = config_dir.join("windows.json");
        let settings = if windows_file.exists() {
            let content = std::fs::read_to_string(&windows_file)?;
            serde_json::from_str(&content)?
        } else {
            let settings = WindowsSettings::default();
            let content = serde_json::to_string_pretty(&settings)?;
            std::fs::write(&windows_file, content)?;
            settings
        };

        Ok(Self {
            config_dir,
            cache: Arc::new(RwLock::new(settings)),
        })
    }

    pub fn load(&self) -> WindowsSettings {
        self.cache.read().clone()
    }

    pub fn save(&self, settings: &WindowsSettings) -> Result<()> {
        let windows_file = self.config_dir.join("windows.json");
        let content = serde_json::to_string_pretty(settings)?;
        std::fs::write(&windows_file, content)?;
        *self.cache.write() = settings.clone();
        Ok(())
    }

    pub fn get_floating_opacity(&self) -> u8 {
        self.cache.read().floating.opacity
    }

    pub fn set_floating_opacity(&self, value: u8) -> Result<()> {
        let mut settings = self.load();
        settings.floating.opacity = validate_opacity(value);
        self.save(&settings)
    }

    pub fn get_floating_bg_color(&self) -> String {
        self.cache.read().floating.bg_color.clone()
    }

    pub fn set_floating_bg_color(&self, color: String) -> Result<()> {
        let mut settings = self.load();
        settings.floating.bg_color = color;
        self.save(&settings)
    }

    pub fn get_floating_clickthrough(&self) -> bool {
        self.cache.read().floating.clickthrough
    }

    pub fn set_floating_clickthrough(&self, enabled: bool) -> Result<()> {
        let mut settings = self.load();
        settings.floating.clickthrough = enabled;
        self.save(&settings)
    }

    pub fn get_floating_position(&self) -> (Option<i32>, Option<i32>) {
        let pos = &self.cache.read().floating.position;
        (pos.x, pos.y)
    }

    pub fn set_floating_position(&self, x: Option<i32>, y: Option<i32>) -> Result<()> {
        let mut settings = self.load();
        settings.floating.position.x = x;
        settings.floating.position.y = y;
        self.save(&settings)
    }
}
EOF
```

**Step 6: Create config/dto.rs**

```bash
cat > src-tauri/src/config/dto.rs << 'EOF'
use serde::{Deserialize, Serialize};
use crate::config::settings::{ConnectionConfig, LoggingSettings};
use crate::config::windows::{FloatingWindowSettings, WindowPosition};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettingsDto {
    pub connections: Vec<ConnectionConfig>,
    pub logging: LoggingSettingsDto,
    pub floating: FloatingWindowDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSettingsDto {
    pub enabled: bool,
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloatingWindowDto {
    pub opacity: u8,
    pub bg_color: String,
    pub clickthrough: bool,
}

impl From<(crate::config::settings::AppSettings, crate::config::windows::WindowsSettings)> for AppSettingsDto {
    fn from((app, win): (crate::config::settings::AppSettings, crate::config::windows::WindowsSettings)) -> Self {
        Self {
            connections: app.connections,
            logging: LoggingSettingsDto {
                enabled: app.logging.enabled,
                level: app.logging.level,
            },
            floating: FloatingWindowDto {
                opacity: win.floating.opacity,
                bg_color: win.floating.bg_color,
                clickthrough: win.floating.clickthrough,
            },
        }
    }
}
EOF
```

**Step 7: Test build**

```bash
cd src-tauri && cargo build
```

Expected: SUCCESS

**Step 8: Commit**

```bash
git add src-tauri/src/config
git commit -m "feat: add config module with settings and windows management"
```

---

## Task 4: Create Event System

**Files:**
- Create: `src-tauri/src/events.rs`
- Create: `src-tauri/src/event_loop.rs`

**Step 1: Create events.rs**

```bash
cat > src-tauri/src/events.rs << 'EOF'
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

impl fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionStatus::Disconnected => write!(f, "Disconnected"),
            ConnectionStatus::Connecting => write!(f, "Connecting"),
            ConnectionStatus::Connected => write!(f, "Connected"),
            ConnectionStatus::Error(e) => write!(f, "Error: {}", e),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    ConnectionAdded(String),
    ConnectionRemoved(String),
    ConnectionStatusChanged(String, ConnectionStatus),
    MessageReceived(String, String),
    FloatingWindowToggled,
    FloatingAppearanceChanged,
}
EOF
```

**Step 2: Create event_loop.rs**

```bash
cat > src-tauri/src/event_loop.rs << 'EOF'
use crate::events::AppEvent;
use tauri::{AppHandle, Emitter};
use tracing::debug;

pub struct EventHandler {
    app_handle: AppHandle,
}

impl EventHandler {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    pub fn process_event(&self, event: AppEvent) {
        match &event {
            AppEvent::ConnectionAdded(id) => {
                debug!("Connection added: {}", id);
                let _ = self.app_handle.emit("connection-added", id);
            }
            AppEvent::ConnectionRemoved(id) => {
                debug!("Connection removed: {}", id);
                let _ = self.app_handle.emit("connection-removed", id);
            }
            AppEvent::ConnectionStatusChanged(id, status) => {
                debug!("Connection status changed: {} -> {}", id, status);
                let _ = self.app_handle.emit("connection-status-changed", (id, status.to_string()));
            }
            AppEvent::MessageReceived(id, message) => {
                debug!("Message received from {}: {}", id, message);
                let _ = self.app_handle.emit("message-received", (id, message));
            }
            AppEvent::FloatingWindowToggled => {
                debug!("Floating window toggled");
                let _ = self.app_handle.emit("floating-window-toggled", ());
            }
            AppEvent::FloatingAppearanceChanged => {
                debug!("Floating appearance changed");
                let _ = self.app_handle.emit("floating-appearance-changed", ());
            }
        }
    }
}
EOF
```

**Step 3: Test build**

```bash
cd src-tauri && cargo build
```

Expected: SUCCESS

**Step 4: Commit**

```bash
git add src-tauri/src/events.rs src-tauri/src/event_loop.rs
git commit -m "feat: add event system with handler"
```

---

## Task 5: Create State Management

**Files:**
- Create: `src-tauri/src/state.rs`

**Step 1: Create state.rs**

```bash
cat > src-tauri/src/state.rs << 'EOF'
use crate::config::{SettingsManager, WindowsManager};
use crate::events::{AppEvent, ConnectionStatus};
use parking_lot::{Mutex, RwLock};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub struct ConnectionState {
    pub id: String,
    pub status: ConnectionStatus,
    pub last_message: Option<String>,
}

pub struct AppState {
    pub event_sender: Arc<Mutex<Option<Sender<AppEvent>>>>,
    pub connections: Arc<RwLock<HashMap<String, ConnectionState>>>,
    pub settings_manager: Arc<RwLock<SettingsManager>>,
    pub windows_manager: Arc<RwLock<WindowsManager>>,
    pub runtime: Arc<tokio::runtime::Runtime>,
}

impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            event_sender: Arc::new(Mutex::new(None)),
            connections: Arc::new(RwLock::new(HashMap::new())),
            settings_manager: Arc::new(RwLock::new(SettingsManager::new()?)),
            windows_manager: Arc::new(RwLock::new(WindowsManager::new()?)),
            runtime: Arc::new(tokio::runtime::Runtime::new()?),
        })
    }

    pub fn emit_event(&self, event: AppEvent) {
        if let Some(sender) = self.event_sender.lock().as_ref() {
            let _ = sender.try_send(event);
        }
    }

    pub fn set_event_sender(&self, sender: Sender<AppEvent>) {
        *self.event_sender.lock() = Some(sender);
    }
}
EOF
```

**Step 2: Test build**

```bash
cd src-tauri && cargo build
```

Expected: SUCCESS

**Step 3: Commit**

```bash
git add src-tauri/src/state.rs
git commit -m "feat: add application state management"
```

---

## Task 6: Create SSE Client Module

**Files:**
- Create: `src-tauri/src/connections/mod.rs`
- Create: `src-tauri/src/connections/client.rs`
- Create: `src-tauri/src/connections/manager.rs`

**Step 1: Create connections/mod.rs**

```bash
mkdir -p src-tauri/src/connections
cat > src-tauri/src/connections/mod.rs << 'EOF'
pub mod client;
pub mod manager;

pub use manager::ConnectionManager;
EOF
```

**Step 2: Create connections/client.rs**

```bash
cat > src-tauri/src/connections/client.rs << 'EOF'
use crate::events::{AppEvent, ConnectionStatus};
use crate::state::AppState;
use eventsource_client as es;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tracing::{error, info, warn};

pub struct SSEClient {
    id: String,
    url: String,
    state: Arc<AppState>,
}

impl SSEClient {
    pub fn new(id: String, url: String, state: Arc<AppState>) -> Self {
        Self { id, url, state }
    }

    pub async fn connect(&self) -> anyhow::Result<()> {
        info!("Connecting to {} for {}", self.url, self.id);

        self.state.emit_event(AppEvent::ConnectionStatusChanged(
            self.id.clone(),
            ConnectionStatus::Connecting,
        ));

        let client = es::ClientBuilder::for_url(&self.url)?
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build client: {}", e))?;

        let mut stream = client.stream();
        let id = self.id.clone();
        let state = self.state.clone();

        tokio::spawn(async move {
            info!("SSE stream started for {}", id);

            while let Ok(Some(event)) = stream.next().await {
                match event {
                    es::SSEEvent::Message(msg) => {
                        let text = msg.data;
                        state.emit_event(AppEvent::MessageReceived(id.clone(), text));
                    }
                    es::SSEEvent::Open => {
                        state.emit_event(AppEvent::ConnectionStatusChanged(
                            id.clone(),
                            ConnectionStatus::Connected,
                        ));
                    }
                    es::SSEEvent::Error(e) => {
                        warn!("SSE error for {}: {}", id, e);
                        state.emit_event(AppEvent::ConnectionStatusChanged(
                            id.clone(),
                            ConnectionStatus::Error(e.to_string()),
                        ));
                    }
                }
            }

            state.emit_event(AppEvent::ConnectionStatusChanged(
                id,
                ConnectionStatus::Disconnected,
            ));
        });

        Ok(())
    }
}
EOF
```

**Step 3: Create connections/manager.rs**

```bash
cat > src-tauri/src/connections/manager.rs << 'EOF'
use crate::config::ConnectionConfig;
use crate::connections::client::SSEClient;
use crate::events::{AppEvent, ConnectionStatus};
use crate::state::AppState;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::task::JoinHandle;

pub struct ConnectionManager {
    state: Arc<AppState>,
    handles: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
}

impl ConnectionManager {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            handles: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start_connection(&self, config: ConnectionConfig) -> anyhow::Result<()> {
        if !config.enabled {
            return Ok(());
        }

        let client = SSEClient::new(config.id.clone(), config.url.clone(), self.state.clone());
        client.connect().await?;

        Ok(())
    }

    pub fn stop_connection(&self, id: &str) {
        let mut handles = self.handles.write();
        if let Some(handle) = handles.remove(id) {
            handle.abort();
        }
    }

    pub async fn start_all(&self) -> anyhow::Result<()> {
        let settings = self.state.settings_manager.read();
        let connections = settings.load().connections;

        for config in connections {
            if config.enabled {
                if let Err(e) = self.start_connection(config).await {
                    tracing::warn!("Failed to start connection: {}", e);
                }
            }
        }

        Ok(())
    }
}
EOF
```

**Step 4: Test build**

```bash
cd src-tauri && cargo build
```

Expected: SUCCESS

**Step 5: Commit**

```bash
git add src-tauri/src/connections
git commit -m "feat: add SSE client and connection manager"
```

---

## Task 7: Create Floating Window Management

**Files:**
- Create: `src-tauri/src/floating.rs`

**Step 1: Create floating.rs**

```bash
cat > src-tauri/src/floating.rs << 'EOF'
use crate::events::AppEvent;
use crate::state::AppState;
use tauri::{AppHandle, Emitter, Manager};
use tracing::debug;

pub fn show_floating_window(app_handle: &AppHandle, state: &AppState) -> tauri::Result<()> {
    debug!("Showing floating window");

    if let Some(window) = app_handle.get_webview_window("floating") {
        // Apply saved position
        let windows_manager = state.windows_manager.read();
        let (x, y) = windows_manager.get_floating_position();

        if let Some(pos_x) = x {
            if let Some(pos_y) = y {
                debug!("Applying saved position: ({}, {})", pos_x, pos_y);
                let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                    x: pos_x,
                    y: pos_y,
                }));
            }
        }

        // Apply clickthrough
        if windows_manager.get_floating_clickthrough() {
            debug!("Applying clickthrough mode");
            let _ = window.set_ignore_cursor_events(true);
        }

        window.show()?;
        state.emit_event(AppEvent::FloatingWindowToggled);
    }

    Ok(())
}

pub fn hide_floating_window(app_handle: &AppHandle, state: &AppState) -> tauri::Result<()> {
    debug!("Hiding floating window");

    if let Some(window) = app_handle.get_webview_window("floating") {
        // Save current position
        if let Ok(outer_pos) = window.outer_position() {
            let x = outer_pos.x;
            let y = outer_pos.y;
            debug!("Saving position: ({}, {})", x, y);
            let windows_manager = state.windows_manager.read();
            let _ = windows_manager.set_floating_position(Some(x), Some(y));
        }

        window.hide()?;
        state.emit_event(AppEvent::FloatingWindowToggled);
    }

    Ok(())
}

pub fn toggle_floating_window(app_handle: &AppHandle, state: &AppState) -> tauri::Result<()> {
    if let Some(window) = app_handle.get_webview_window("floating") {
        if window.is_visible()? {
            hide_floating_window(app_handle, state)?;
        } else {
            show_floating_window(app_handle, state)?;
        }
    }
    Ok(())
}

pub fn update_floating_appearance(app_handle: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app_handle.get_webview_window("floating") {
        window.emit("floating-appearance-update", ())?;
    }
    Ok(())
}
EOF
```

**Step 2: Test build**

```bash
cd src-tauri && cargo build
```

Expected: SUCCESS

**Step 3: Commit**

```bash
git add src-tauri/src/floating.rs
git commit -m "feat: add floating window management"
```

---

## Task 8: Create Tauri Commands

**Files:**
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/connections.rs`
- Create: `src-tauri/src/commands/settings.rs`
- Create: `src-tauri/src/commands/windows.rs`

**Step 1: Create commands/mod.rs**

```bash
mkdir -p src-tauri/src/commands
cat > src-tauri/src/commands/mod.rs << 'EOF'
pub mod connections;
pub mod settings;
pub mod windows;
EOF
```

**Step 2: Create commands/connections.rs**

```bash
cat > src-tauri/src/commands/connections.rs << 'EOF'
use crate::config::ConnectionConfig;
use crate::state::AppState;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn add_connection(
    state: State<'_, AppState>,
    name: String,
    url: String,
) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    let config = ConnectionConfig {
        id: id.clone(),
        name,
        url,
        enabled: true,
    };

    state.settings_manager.write()
        .add_connection(config.clone())
        .map_err(|e| e.to_string())?;

    // Start the connection
    let manager = crate::connections::ConnectionManager::new(state.inner().clone());
    manager.start_connection(config).await
        .map_err(|e| e.to_string())?;

    Ok(id)
}

#[tauri::command]
pub async fn remove_connection(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    state.settings_manager.write()
        .remove_connection(&id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_connections(state: State<'_, AppState>) -> Result<Vec<ConnectionConfig>, String> {
    Ok(state.settings_manager.read().load().connections)
}
EOF
```

**Step 3: Create commands/settings.rs**

```bash
cat > src-tauri/src/commands/settings.rs << 'EOF'
use crate::config::dto::AppSettingsDto;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn get_all_settings(state: State<'_, AppState>) -> Result<AppSettingsDto, String> {
    let app = state.settings_manager.read().load();
    let win = state.windows_manager.read().load();
    Ok(AppSettingsDto::from((app, win)))
}

#[tauri::command]
pub fn save_logging_settings(
    state: State<'_, AppState>,
    enabled: bool,
    level: String,
) -> Result<(), String> {
    let mut settings = state.settings_manager.read().load();
    settings.logging.enabled = enabled;
    settings.logging.level = level;
    state.settings_manager.write()
        .save(&settings)
        .map_err(|e| e.to_string())
}
EOF
```

**Step 4: Create commands/windows.rs**

```bash
cat > src-tauri/src/commands/windows.rs << 'EOF'
use crate::state::AppState;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn toggle_floating_window(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    crate::floating::toggle_floating_window(&app, &state)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn show_floating_window(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    crate::floating::show_floating_window(&app, &state)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn hide_floating_window(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    crate::floating::hide_floating_window(&app, &state)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_floating_opacity(
    state: State<'_, AppState>,
    value: u8,
) -> Result<(), String> {
    state.windows_manager.write()
        .set_floating_opacity(value)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_floating_bg_color(
    state: State<'_, AppState>,
    color: String,
) -> Result<(), String> {
    state.windows_manager.write()
        .set_floating_bg_color(color)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_clickthrough(
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<bool, String> {
    state.windows_manager.write()
        .set_floating_clickthrough(enabled)
        .map_err(|e| e.to_string())?;
    Ok(enabled)
}

#[tauri::command]
pub fn get_floating_opacity(state: State<'_, AppState>) -> u8 {
    state.windows_manager.read().get_floating_opacity()
}

#[tauri::command]
pub fn get_floating_bg_color(state: State<'_, AppState>) -> String {
    state.windows_manager.read().get_floating_bg_color()
}

#[tauri::command]
pub fn is_clickthrough_enabled(state: State<'_, AppState>) -> bool {
    state.windows_manager.read().get_floating_clickthrough()
}
EOF
```

**Step 5: Add uuid dependency to Cargo.toml**

```bash
# Add to dependencies in Cargo.toml
echo 'uuid = { version = "1.0", features = ["v4"] }' >> src-tauri/Cargo.toml
```

**Step 6: Test build**

```bash
cd src-tauri && cargo build
```

Expected: SUCCESS

**Step 7: Commit**

```bash
git add src-tauri/src/commands src-tauri/Cargo.toml
git commit -m "feat: add Tauri commands for connections, settings, and windows"
```

---

## Task 9: Create Setup and Integration

**Files:**
- Create: `src-tauri/src/setup.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Create setup.rs**

```bash
cat > src-tauri/src/setup.rs << 'EOF'
use crate::connections::ConnectionManager;
use crate::event_loop::EventHandler;
use crate::events::AppEvent;
use crate::state::AppState;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::mpsc;
use tracing::info;

pub async fn init_app(app_handle: &AppHandle) -> anyhow::Result<()> {
    info!("Initializing application");

    // Create app state
    let state = AppState::new()?;
    let state_arc = Arc::new(state);

    // Store state in app
    app_handle.manage(state_arc.clone());

    // Create event channel
    let (tx, mut rx) = mpsc::channel::<AppEvent>(100);
    state_arc.set_event_sender(tx);

    // Start event loop
    let event_handler = EventHandler::new(app_handle.clone());
    let state_for_loop = state_arc.clone();

    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            event_handler.process_event(event);
        }
    });

    // Start all enabled connections
    let manager = ConnectionManager::new(state_arc.clone());
    manager.start_all().await?;

    info!("Application initialized successfully");

    Ok(())
}
EOF
```

**Step 2: Update lib.rs with setup integration**

```bash
cat > src-tauri/src/lib.rs << 'EOF'
mod commands;
mod config;
mod connections;
mod event_loop;
mod events;
mod floating;
mod setup;
mod state;

use std::path::PathBuf;
use tauri::Manager;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Clone)]
pub struct LogGuard {
    _guard: Option<Box<WorkerGuard>>,
}

fn leak_guard(guard: WorkerGuard) -> LogGuard {
    LogGuard {
        _guard: Some(Box::leak(Box::new(guard))),
    }
}

pub fn init_logging() -> LogGuard {
    let config_dir = dirs::config_dir()
        .expect("Failed to get config dir")
        .join("ttsbard-echo");

    let log_dir = config_dir.join("logs");
    std::fs::create_dir_all(&log_dir)
        .expect("Failed to create log directory");

    let log_file = log_dir.join("ttsbard-echo.log");
    let file_appender = tracing_appender::rolling::never(log_dir, "ttsbard-echo.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::from_default_env()
        .add_directive("ttsbard_echo=debug".parse().unwrap())
        .add_directive("tauri=warn".parse().unwrap());

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(fmt::layer().with_writer(non_blocking))
        .init();

    tracing::info!("Logging initialized. Log file: {:?}", log_file);

    leak_guard(guard)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _log_guard = init_logging();

    tauri::Builder::default()
        .setup(|app| {
            // Initialize app on tokio runtime
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = setup::init_app(&handle).await {
                    tracing::error!("Failed to initialize app: {}", e);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Connections
            commands::connections::add_connection,
            commands::connections::remove_connection,
            commands::connections::get_connections,
            // Settings
            commands::settings::get_all_settings,
            commands::settings::save_logging_settings,
            // Windows
            commands::windows::toggle_floating_window,
            commands::windows::show_floating_window,
            commands::windows::hide_floating_window,
            commands::windows::set_floating_opacity,
            commands::windows::set_floating_bg_color,
            commands::windows::set_clickthrough,
            commands::windows::get_floating_opacity,
            commands::windows::get_floating_bg_color,
            commands::windows::is_clickthrough_enabled,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
EOF
```

**Step 3: Test build**

```bash
cd src-tauri && cargo build
```

Expected: SUCCESS

**Step 4: Commit**

```bash
git add src-tauri/src/setup.rs src-tauri/src/lib.rs
git commit -m "feat: add setup and integrate all modules"
```

---

## Task 10: Create Frontend - Vue App Structure

**Files:**
- Create: `src/main.ts`
- Create: `src/App.vue`
- Create: `index.html`
- Create: `src/vite-env.d.ts`

**Step 1: Create index.html**

```bash
cat > index.html << 'EOF'
<!doctype html>
<html lang="ru">
  <head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/svg+xml" href="/vite.svg" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>ttsbard-echo</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
EOF
```

**Step 2: Create main.ts**

```bash
mkdir -p src
cat > src/main.ts << 'EOF'
import { createApp } from 'vue'
import './style.css'
import App from './App.vue'

createApp(App).mount('#app')
EOF
```

**Step 3: Create style.css**

```bash
cat > src/style.css << 'EOF'
:root {
  font-family: Inter, system-ui, Avenir, Helvetica, Arial, sans-serif;
  line-height: 1.5;
  font-weight: 400;
  color: #ffffff;
  background-color: #111417;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  min-width: 320px;
  min-height: 100vh;
}

#app {
  width: 100%;
  height: 100vh;
}
EOF
```

**Step 4: Create vite-env.d.ts**

```bash
cat > src/vite-env.d.ts << 'EOF'
/// <reference types="vite/client" />
EOF
```

**Step 5: Create App.vue**

```bash
cat > src/App.vue << 'EOF'
<script setup lang="ts">
import { ref } from 'vue'
import Sidebar from './components/Sidebar.vue'
import SettingsPanel from './components/SettingsPanel.vue'

const currentPanel = ref<'main' | 'settings'>('main')
</script>

<template>
  <div class="app">
    <Sidebar v-model:panel="currentPanel" />
    <main class="main-content">
      <SettingsPanel v-if="currentPanel === 'settings'" />
      <div v-else class="welcome">
        <h1>ttsbard-echo</h1>
        <p>Выберите действие в сайдбаре</p>
      </div>
    </main>
  </div>
</template>

<style scoped>
.app {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

.main-content {
  flex: 1;
  padding: 2rem;
  overflow-y: auto;
}

.welcome {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--color-text-secondary);
}

.welcome h1 {
  font-size: 2rem;
  margin-bottom: 0.5rem;
  color: var(--color-text-primary);
}
</style>
EOF
```

**Step 6: Test dev server**

```bash
npm run dev
```

Expected: Dev server running on http://localhost:5173

**Step 7: Commit**

```bash
git add index.html src src/style.css
git commit -m "feat: add Vue 3 frontend structure"
```

---

## Task 11: Create Frontend - Sidebar Component

**Files:**
- Create: `src/components/Sidebar.vue`
- Create: `src/types.ts`

**Step 1: Create types.ts**

```bash
mkdir -p src/components
cat > src/types.ts << 'EOF'
export interface ConnectionConfig {
  id: string
  name: string
  url: string
  enabled: boolean
}

export interface FloatingSettings {
  opacity: number
  bgColor: string
  clickthrough: boolean
}
EOF
```

**Step 2: Create Sidebar.vue**

```bash
cat > src/components/Sidebar.vue << 'EOF'
<script setup lang="ts">
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Globe, Settings } from 'lucide-vue-next'

type Panel = 'main' | 'settings'

const props = defineProps<{
  panel: Panel
}>()

const emit = defineEmits<{
  'update:panel': [panel: Panel]
}>()

function setPanel(panel: Panel) {
  emit('update:panel', panel)
}

async function toggleFloating() {
  try {
    await invoke('toggle_floating_window')
  } catch (e) {
    console.error('Failed to toggle floating window:', e)
  }
}
</script>

<template>
  <aside class="sidebar">
    <nav class="sidebar-nav">
      <button
        class="sidebar-button"
        @click="toggleFloating"
        title="Подключения"
      >
        <Globe :size="20" class="sidebar-icon" />
        <span class="sidebar-button-label">Подключения</span>
      </button>

      <button
        class="sidebar-button"
        :class="{ 'sidebar-button-active': props.panel === 'settings' }"
        @click="setPanel('settings')"
        title="Настройки"
      >
        <Settings :size="20" class="sidebar-icon" />
        <span class="sidebar-button-label">Настройки</span>
      </button>
    </nav>
  </aside>
</template>

<style scoped>
.sidebar {
  flex: 0 0 200px;
  width: 200px;
  min-width: 200px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.03), transparent 22%),
              linear-gradient(180deg, rgba(17, 19, 26, 0.98) 0%, rgba(14, 16, 22, 0.96) 100%);
  color: var(--color-text-primary);
  display: flex;
  flex-direction: column;
  box-shadow: inset -1px 0 0 rgba(255, 255, 255, 0.06);
}

.sidebar-nav {
  display: flex;
  flex-direction: column;
  padding: 1rem 0;
  gap: 0.5rem;
}

.sidebar-button {
  width: 100%;
  padding: 0.75rem 1rem;
  border: 1px solid transparent;
  background: rgba(255, 255, 255, 0.01);
  color: var(--color-text-secondary);
  cursor: pointer;
  text-align: left;
  transition: all 0.18s ease;
  display: flex;
  align-items: center;
  gap: 0.75rem;
  border-radius: 0;
}

.sidebar-button:hover {
  background: rgba(255, 255, 255, 0.06);
  color: var(--color-text-primary);
  border-color: rgba(255, 255, 255, 0.08);
}

.sidebar-button-active {
  background: rgba(255, 255, 255, 0.09) !important;
  border-color: rgba(255, 255, 255, 0.08) !important;
  color: var(--color-text-primary) !important;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
}

.sidebar-icon {
  min-width: 20px;
}

.sidebar-button-label {
  flex: 1;
  font-size: 0.92rem;
  font-weight: 600;
}
</style>
EOF
```

**Step 3: Update App.vue to use v-model**

```bash
cat > src/App.vue << 'EOF'
<script setup lang="ts">
import { ref } from 'vue'
import Sidebar from './components/Sidebar.vue'
import SettingsPanel from './components/SettingsPanel.vue'

const currentPanel = ref<'main' | 'settings'>('main')
</script>

<template>
  <div class="app">
    <Sidebar v-model:panel="currentPanel" />
    <main class="main-content">
      <SettingsPanel v-if="currentPanel === 'settings'" />
      <div v-else class="welcome">
        <h1>ttsbard-echo</h1>
        <p>Нажмите "Подключения" для показа плавающей панели</p>
      </div>
    </main>
  </div>
</template>

<style scoped>
.app {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

.main-content {
  flex: 1;
  padding: 2rem;
  overflow-y: auto;
}

.welcome {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--color-text-secondary);
}

.welcome h1 {
  font-size: 2rem;
  margin-bottom: 0.5rem;
  color: var(--color-text-primary);
}
</style>
EOF
```

**Step 4: Commit**

```bash
git add src/components/Sidebar.vue src/types.ts src/App.vue
git commit -m "feat: add sidebar component with navigation"
```

---

## Task 12: Create Frontend - Settings Panel

**Files:**
- Create: `src/components/SettingsPanel.vue`

**Step 1: Create SettingsPanel.vue**

```bash
cat > src/components/SettingsPanel.vue << 'EOF'
<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ConnectionConfig, FloatingSettings } from '../types'

const connections = ref<ConnectionConfig[]>([])
const floatingSettings = ref<FloatingSettings>({
  opacity: 90,
  bgColor: '#1e1e1e',
  clickthrough: false
})

const newConnectionName = ref('')
const newConnectionUrl = ref('')

async function loadSettings() {
  try {
    const settings = await invoke<any>('get_all_settings')
    connections.value = settings.connections || []
    floatingSettings.value = {
      opacity: settings.floating.opacity,
      bgColor: settings.floating.bg_color,
      clickthrough: settings.floating.clickthrough
    }
  } catch (e) {
    console.error('Failed to load settings:', e)
  }
}

async function addConnection() {
  if (!newConnectionName.value || !newConnectionUrl.value) return

  try {
    await invoke('add_connection', {
      name: newConnectionName.value,
      url: newConnectionUrl.value
    })
    newConnectionName.value = ''
    newConnectionUrl.value = ''
    await loadSettings()
  } catch (e) {
    console.error('Failed to add connection:', e)
  }
}

async function removeConnection(id: string) {
  try {
    await invoke('remove_connection', { id })
    await loadSettings()
  } catch (e) {
    console.error('Failed to remove connection:', e)
  }
}

async function saveFloatingOpacity() {
  try {
    await invoke('set_floating_opacity', { value: floatingSettings.value.opacity })
  } catch (e) {
    console.error('Failed to save opacity:', e)
  }
}

async function saveFloatingBgColor() {
  try {
    await invoke('set_floating_bg_color', { color: floatingSettings.value.bgColor })
  } catch (e) {
    console.error('Failed to save color:', e)
  }
}

async function toggleClickthrough() {
  try {
    const enabled = await invoke<boolean>('set_clickthrough', {
      enabled: !floatingSettings.value.clickthrough
    })
    floatingSettings.value.clickthrough = enabled
  } catch (e) {
    console.error('Failed to toggle clickthrough:', e)
  }
}

onMounted(() => {
  loadSettings()
})
</script>

<template>
  <div class="settings-panel">
    <h1>Настройки</h1>

    <section class="section">
      <h2>Подключения</h2>
      <div class="connection-list">
        <div v-for="conn in connections" :key="conn.id" class="connection-item">
          <div class="connection-info">
            <strong>{{ conn.name }}</strong>
            <code>{{ conn.url }}</code>
          </div>
          <button @click="removeConnection(conn.id)" class="btn-danger">Удалить</button>
        </div>
      </div>
      <div class="add-connection">
        <input
          v-model="newConnectionName"
          type="text"
          placeholder="Название"
          class="input"
        />
        <input
          v-model="newConnectionUrl"
          type="text"
          placeholder="URL (например, http://localhost:8080/events)"
          class="input"
        />
        <button @click="addConnection" class="btn-primary">Добавить</button>
      </div>
    </section>

    <section class="section">
      <h2>Внешний вид</h2>
      <div class="setting-row">
        <label>Прозрачность: {{ floatingSettings.opacity }}%</label>
        <input
          v-model.number="floatingSettings.opacity"
          type="range"
          min="10"
          max="100"
          @change="saveFloatingOpacity"
          class="slider"
        />
      </div>
      <div class="setting-row">
        <label>Цвет фона:</label>
        <input
          v-model="floatingSettings.bgColor"
          type="color"
          @change="saveFloatingBgColor"
          class="color-input"
        />
        <input
          v-model="floatingSettings.bgColor"
          type="text"
          @blur="saveFloatingBgColor"
          class="input"
          maxlength="7"
        />
      </div>
      <div class="setting-row">
        <label>
          <input
            type="checkbox"
            :checked="floatingSettings.clickthrough"
            @change="toggleClickthrough"
          />
          Пропускать клики (click-through)
        </label>
      </div>
      <div class="preview" :style="{ backgroundColor: floatingSettings.bgColor, opacity: floatingSettings.opacity / 100 }">
        Предпросмотр
      </div>
    </section>
  </div>
</template>

<style scoped>
.settings-panel {
  max-width: 800px;
  margin: 0 auto;
}

h1 {
  margin-bottom: 2rem;
}

.section {
  margin-bottom: 2rem;
  padding: 1.5rem;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
}

.section h2 {
  margin-top: 0;
  margin-bottom: 1rem;
  font-size: 1.2rem;
}

.connection-list {
  margin-bottom: 1rem;
}

.connection-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  margin-bottom: 0.5rem;
}

.connection-info {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.connection-info code {
  font-size: 0.85rem;
  color: var(--color-text-secondary);
  font-family: monospace;
}

.add-connection {
  display: flex;
  gap: 0.5rem;
}

.input {
  flex: 1;
  padding: 0.5rem;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.05);
  color: white;
}

.btn-primary, .btn-danger {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 500;
}

.btn-primary {
  background: var(--color-accent, #1d8cff);
  color: white;
}

.btn-danger {
  background: rgba(255, 111, 105, 0.2);
  color: #ffb8b4;
}

.setting-row {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 1rem;
}

.setting-row label {
  min-width: 150px;
}

.slider {
  flex: 1;
}

.color-input {
  width: 50px;
  height: 36px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  cursor: pointer;
  padding: 0;
}

.preview {
  margin-top: 1rem;
  padding: 1rem;
  border-radius: 8px;
  text-align: center;
  min-height: 60px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: white;
  font-weight: 500;
  text-shadow: 0 1px 2px rgba(0,0,0,0.5);
}
</style>
EOF
```

**Step 2: Commit**

```bash
git add src/components/SettingsPanel.vue
git commit -m "feat: add settings panel with connections and appearance"
```

---

## Task 13: Create Frontend - Floating Panel

**Files:**
- Create: `src-tauri/floating.html` (separate entry for floating window)
- Create: `src/components/ConnectionsPanel.vue`
- Modify: `src-tauri/tauri.conf.json` (add floating window config - already done)

**Step 1: Create floating.html**

```bash
cat > src-tauri/floating.html << 'EOF'
<!DOCTYPE html>
<html lang="ru">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Connections</title>
  <style>
    * {
      margin: 0;
      padding: 0;
      box-sizing: border-box;
    }
    body {
      font-family: Inter, system-ui, sans-serif;
      background: transparent;
      color: white;
      overflow: hidden;
    }
    #app {
      width: 100%;
      height: 100vh;
    }
  </style>
</head>
<body>
  <div id="app"></div>
  <script type="module" src="/src/floating-main.ts"></script>
</body>
</html>
EOF
```

**Step 2: Create floating-main.ts**

```bash
cat > src/floating-main.ts << 'EOF'
import { createApp } from 'vue'
import { listen } from '@tauri-apps/api/event'
import ConnectionsPanel from './components/ConnectionsPanel.vue'

interface ConnectionState {
  id: string
  name: string
  url: string
  status: string
  lastMessage?: string
}

const connections = new Map<string, ConnectionState>()

createApp({
  components: {
    ConnectionsPanel
  },
  data() {
    return {
      connections: new Map()
    }
  },
  template: '<ConnectionsPanel :connections="connections" />',
  mounted() {
    // Listen for connection status changes
    listen<[string, string]>('connection-status-changed', (event) => {
      const [id, status] = event.payload
      const conn = this.connections.get(id)
      if (conn) {
        conn.status = status
      }
    })

    // Listen for messages
    listen<[string, string]>('message-received', (event) => {
      const [id, message] = event.payload
      const conn = this.connections.get(id)
      if (conn) {
        conn.lastMessage = message
      }
    })
  }
}).mount('#app')
EOF
```

**Step 3: Create ConnectionsPanel.vue**

```bash
cat > src/components/ConnectionsPanel.vue << 'EOF'
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Globe } from 'lucide-vue-next'

interface ConnectionState {
  id: string
  name: string
  url: string
  status: string
  lastMessage?: string
}

const props = defineProps<{
  connections: Map<string, ConnectionState>
}>()

async function loadConnections() {
  try {
    const conns = await invoke<any[]>('get_connections')
    for (const conn of conns) {
      props.connections.set(conn.id, {
        ...conn,
        status: 'Disconnected',
        lastMessage: undefined
      })
    }
  } catch (e) {
    console.error('Failed to load connections:', e)
  }
}

onMounted(() => {
  loadConnections()
})
</script>

<template>
  <div class="connections-panel">
    <div class="panel-header">ПОДКЛЮЧЕНИЯ</div>

    <div
      v-for="[id, conn] of connections"
      :key="id"
      class="connection-card"
    >
      <div class="connection-header">
        <Globe :size="16" />
        <span class="connection-name">{{ conn.name }}</span>
        <span class="connection-status" :class="{
          'connected': conn.status === 'Connected',
          'connecting': conn.status === 'Connecting',
          'error': conn.status.startsWith('Error'),
          'disconnected': conn.status === 'Disconnected'
        }">
          {{ statusText(conn.status) }}
        </span>
      </div>
      <div v-if="conn.lastMessage" class="connection-message">
        {{ conn.lastMessage }}
      </div>
      <div v-else class="connection-message empty">
        {{ statusMessage(conn.status) }}
      </div>
    </div>

    <div v-if="connections.size === 0" class="empty-state">
      Нет подключений
    </div>
  </div>
</template>

<script setup lang="ts">
function statusText(status: string): string {
  switch (status) {
    case 'Connected': return '●'
    case 'Connecting': return '○'
    case 'Disconnected': return '○'
    default: return '!'
  }
}

function statusMessage(status: string): string {
  switch (status) {
    case 'Connected': return 'Ожидание сообщений...'
    case 'Connecting': return 'Подключение...'
    case 'Disconnected': return 'Нет соединения'
    default: return status
  }
}
</script>

<style scoped>
.connections-panel {
  padding: 0.75rem;
  background: rgba(30, 30, 30, 0.95);
  border-radius: 12px;
  max-height: 400px;
  overflow-y: auto;
}

.panel-header {
  font-size: 0.75rem;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.6);
  margin-bottom: 0.5rem;
  padding: 0 0.25rem;
}

.connection-card {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  padding: 0.5rem;
  margin-bottom: 0.5rem;
}

.connection-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.85rem;
}

.connection-name {
  flex: 1;
  font-weight: 500;
}

.connection-status {
  font-size: 0.7rem;
  padding: 0.1rem 0.4rem;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.1);
}

.connection-status.connected {
  color: #bff4d0;
  background: rgba(74, 222, 128, 0.15);
}

.connection-status.connecting {
  color: #ffd8a8;
  background: rgba(255, 193, 7, 0.15);
}

.connection-status.error {
  color: #ffb8b4;
  background: rgba(255, 111, 105, 0.15);
}

.connection-status.disconnected {
  color: rgba(255, 255, 255, 0.4);
}

.connection-message {
  font-size: 0.75rem;
  color: rgba(255, 255, 255, 0.7);
  margin-top: 0.35rem;
  padding-left: 1.25rem;
}

.connection-message.empty {
  color: rgba(255, 255, 255, 0.4);
  font-style: italic;
}

.empty-state {
  text-align: center;
  padding: 1rem;
  color: rgba(255, 255, 255, 0.4);
  font-size: 0.85rem;
}
</style>
EOF
```

**Step 4: Update tauri.conf.json for floating window**

```bash
# Update the floating window config to use the correct HTML file
# The floating window already configured in Task 1, just need to update the path
cat > src-tauri/tauri.conf.json << 'EOF'
{
  "$schema": "https://schema.tauri.app/config/2.0.0",
  "productName": "ttsbard-echo",
  "version": "0.1.0",
  "identifier": "com.ttsbard.echo",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "ttsbard-echo",
        "width": 900,
        "height": 700,
        "resizable": true,
        "decorations": true
      },
      {
        "label": "floating",
        "title": "Connections",
        "url": "floating.html",
        "width": 350,
        "height": 400,
        "resizable": false,
        "decorations": false,
        "alwaysOnTop": true,
        "skipTaskbar": true,
        "visible": false,
        "transparent": true
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": []
  }
}
EOF
```

**Step 5: Update vite.config.ts for multi-page build**

```bash
cat > vite.config.ts << 'EOF'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src')
    }
  },
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**']
    }
  },
  build: {
    rollupOptions: {
      input: {
        main: path.resolve(__dirname, 'index.html'),
        floating: path.resolve(__dirname, 'src-tauri/floating.html')
      }
    }
  }
})
EOF
```

**Step 6: Commit**

```bash
git add src-tauri/floating.html src/floating-main.ts src/components/ConnectionsPanel.vue src-tauri/tauri.conf.json vite.config.ts
git commit -m "feat: add floating panel with connection status display"
```

---

## Task 14: Add CSS Variables and Global Styles

**Files:**
- Modify: `src/style.css`

**Step 1: Update style.css with CSS variables**

```bash
cat > src/style.css << 'EOF'
:root {
  --color-text-primary: #ffffff;
  --color-text-secondary: rgba(255, 255, 255, 0.7);
  --color-text-muted: rgba(255, 255, 255, 0.5);
  --color-bg-field: rgba(255, 255, 255, 0.05);
  --color-accent: #1d8cff;
  --color-accent-strong: #0f74ff;
  --color-danger: #ff6f69;
  --font-mono: 'Consolas', 'Monaco', 'Courier New', monospace;

  font-family: Inter, system-ui, Avenir, Helvetica, Arial, sans-serif;
  line-height: 1.5;
  font-weight: 400;
  color: var(--color-text-primary);
  background-color: #111417;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  min-width: 320px;
  min-height: 100vh;
}

#app {
  width: 100%;
  height: 100vh;
}

button {
  font-family: inherit;
}

code {
  font-family: var(--font-mono);
}
EOF
```

**Step 2: Commit**

```bash
git add src/style.css
git commit -m "style: add CSS variables and global styles"
```

---

## Task 15: Final Testing and Verification

**Files:**
- Test all functionality

**Step 1: Build and test**

```bash
npm run build
cd src-tauri && cargo build
```

Expected: SUCCESS

**Step 2: Test run**

```bash
npm run tauri dev
```

Verify:
- [ ] Main window opens with sidebar
- [ ] Settings panel shows connections and appearance settings
- [ ] Can add new connection
- [ ] Can toggle floating window
- [ ] Floating window shows connection status
- [ ] Floating window appearance settings work
- [ ] Window position persists

**Step 3: Create README.md**

```bash
cat > README.md << 'EOF'
# ttsbard-echo

Tauri 2 desktop application for connecting to external HTTP/SSE servers and displaying events in a floating panel.

## Development

```bash
npm install
npm run tauri dev
```

## Building

```bash
npm run build
npm run tauri build
```

## Configuration

Settings are stored in `%APPDATA%\ttsbard-echo\`:
- `settings.json` - Connection and logging settings
- `windows.json` - Window positions and appearance
- `logs\ttsbard-echo.log` - Application logs
EOF
```

**Step 4: Final commit**

```bash
git add README.md
git commit -m "docs: add README"
```

---

## Summary

This implementation plan creates a complete Tauri 2 + Vue 3 application with:

1. **Backend (Rust)**:
   - Config management with JSON persistence
   - Event system for internal communication
   - SSE client for external server connections
   - Floating window management
   - Tauri commands for frontend API

2. **Frontend (Vue 3)**:
   - Main window with sidebar and settings
   - Floating panel for connection status
   - Real-time event updates
   - Appearance customization

3. **Features**:
   - Add/remove SSE connections
   - Real-time connection status
   - Message display in floating panel
   - Window appearance customization
   - Position persistence
   - Logging system

**Total commits:** ~15
**Estimated time:** 2-3 hours for full implementation
