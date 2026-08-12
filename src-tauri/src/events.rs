use serde::{Deserialize, Serialize};
use std::fmt;

/* ==========================================================================
Connection Status
========================================================================== */
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/* ==========================================================================
App Events
========================================================================== */
#[derive(Debug, Clone)]
pub enum AppEvent {
    // Theme
    ThemeChanged(String),

    // Floating Window
    FloatingAppearanceChanged,
    ClickthroughChanged(bool),
    ShowFloatingWindow,
    HideFloatingWindow,
    FloatingWindowToggled,
    FloatingVisibilityChanged(bool),
    UpdateFloatingText(String),

    // Connections
    ConnectionsChanged,
    ConnectionStatusChanged(String, ConnectionStatus),
    MessageReceived(String, String),
    MessageCleared(String),
    ConnectionAdded(String),
    ConnectionRemoved(String),
    TypingChanged(String, bool, Option<String>),

    // Settings
    SettingsChanged,
    LoggingChanged,
    AppearanceChanged,
    HotkeysChanged,
    GeneralChanged,

    // System
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
            AppEvent::FloatingWindowToggled => "floating-window-toggled",
            AppEvent::FloatingVisibilityChanged(_) => "floating-visibility-changed",
            AppEvent::UpdateFloatingText(_) => "update-floating-text",
            AppEvent::ConnectionsChanged => "connections-changed",
            AppEvent::ConnectionStatusChanged(_, _) => "connection-status-changed",
            AppEvent::MessageReceived(_, _) => "message-received",
            AppEvent::MessageCleared(_) => "message-cleared",
            AppEvent::ConnectionAdded(_) => "connection-added",
            AppEvent::ConnectionRemoved(_) => "connection-removed",
            AppEvent::TypingChanged(_, _, _) => "typing-changed",
            AppEvent::SettingsChanged => "settings-changed",
            AppEvent::LoggingChanged => "logging-changed",
            AppEvent::AppearanceChanged => "appearance-changed",
            AppEvent::HotkeysChanged => "hotkeys-changed",
            AppEvent::GeneralChanged => "general-changed",
            AppEvent::BackendReady => "backend-ready",
            AppEvent::AppQuit => "app-quit",
        }
    }
}
