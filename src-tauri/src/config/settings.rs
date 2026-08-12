use anyhow::Result;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::config::{
    constants::DEFAULT_LOG_LEVEL,
    validation::{validate_connection_id, validate_connection_name, validate_url},
};

/* ==========================================================================
Theme
========================================================================== */
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

/* ==========================================================================
Connection Config
========================================================================== */
// Newtype wrapper to mask the access_token in logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskedAccessTokens {
    #[serde(default)]
    pub access_token: Option<String>,
}

impl std::fmt::Display for MaskedAccessTokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.access_token {
            Some(_) => write!(f, "[masked]"),
            None => write!(f, "None"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub id: String,
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub access_token: Option<String>,
}

impl ConnectionConfig {
    pub fn validate(&self) -> Result<()> {
        validate_connection_id(&self.id).map_err(|e| anyhow::anyhow!(e))?;
        validate_connection_name(&self.name).map_err(|e| anyhow::anyhow!(e))?;
        validate_url(&self.url).map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    }
}

/* ==========================================================================
Logging Settings
========================================================================== */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSettings {
    pub enabled: bool,
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub module_levels: HashMap<String, String>,
}

fn default_log_level() -> String {
    DEFAULT_LOG_LEVEL.to_string()
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

/* ==========================================================================
Hotkey Settings (NEW)
========================================================================== */
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HotkeySettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub toggle_window: Option<String>,
}

/* ==========================================================================
General Settings (NEW)
========================================================================== */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    #[serde(default)]
    pub exclude_from_capture: bool,
    #[serde(default)]
    pub theme: Option<Theme>,
    #[serde(default = "default_message_clear_interval_seconds")]
    pub message_clear_interval_seconds: u32,
}

fn default_message_clear_interval_seconds() -> u32 {
    30
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            exclude_from_capture: false,
            theme: None,
            message_clear_interval_seconds: default_message_clear_interval_seconds(),
        }
    }
}

/* ==========================================================================
App Settings
========================================================================== */
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    #[serde(default)]
    pub connections: Vec<ConnectionConfig>,
    #[serde(default)]
    pub logging: LoggingSettings,
    #[serde(default)]
    pub theme: Theme,
    #[serde(default)]
    pub hotkeys: HotkeySettings,
    #[serde(default)]
    pub general: GeneralSettings,
}

impl AppSettings {
    pub fn validate(&self) -> Result<()> {
        // Validate each connection
        for connection in &self.connections {
            connection.validate()?;
        }
        Ok(())
    }

    pub fn with_defaults() -> Self {
        Self {
            connections: Vec::new(),
            logging: LoggingSettings::default(),
            theme: Theme::Dark,
            hotkeys: HotkeySettings::default(),
            general: GeneralSettings::default(),
        }
    }
}

/* ==========================================================================
Settings Manager
========================================================================== */
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
        // Validate before saving
        settings.validate()?;

        let settings_file = self.config_dir.join("settings.json");
        let content = serde_json::to_string_pretty(settings)?;
        std::fs::write(&settings_file, content)?;
        *self.cache.write() = settings.clone();
        Ok(())
    }

    /* ---------------------------------------------------------------------
    Connections
    --------------------------------------------------------------------- */
    pub fn add_connection(&self, connection: ConnectionConfig) -> Result<()> {
        // Validate the connection before adding it
        connection.validate()?;

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
        // Validate the updated connection
        updated.validate()?;

        let mut settings = self.load();
        if let Some(conn) = settings.connections.iter_mut().find(|c| c.id == id) {
            *conn = updated;
            self.save(&settings)
        } else {
            Err(anyhow::anyhow!("Connection not found: {}", id))
        }
    }

    /* ---------------------------------------------------------------------
    Theme
    --------------------------------------------------------------------- */
    pub fn set_theme(&self, theme: Theme) -> Result<()> {
        let mut settings = self.load();
        settings.theme = theme;
        self.save(&settings)
    }

    /* ---------------------------------------------------------------------
    Logging
    --------------------------------------------------------------------- */
    pub fn set_logging_enabled(&self, enabled: bool) -> Result<()> {
        let mut settings = self.load();
        settings.logging.enabled = enabled;
        self.save(&settings)
    }

    pub fn set_logging_level(&self, level: String) -> Result<()> {
        let mut settings = self.load();
        settings.logging.level = level;
        self.save(&settings)
    }

    /* ---------------------------------------------------------------------
    Hotkeys
    --------------------------------------------------------------------- */
    pub fn set_hotkey_enabled(&self, enabled: bool) -> Result<()> {
        let mut settings = self.load();
        settings.hotkeys.enabled = enabled;
        self.save(&settings)
    }

    pub fn set_toggle_window_hotkey(&self, hotkey: Option<String>) -> Result<()> {
        let mut settings = self.load();
        settings.hotkeys.toggle_window = hotkey;
        self.save(&settings)
    }

    /* ---------------------------------------------------------------------
    General
    --------------------------------------------------------------------- */
    pub fn set_exclude_from_capture(&self, exclude: bool) -> Result<()> {
        let mut settings = self.load();
        settings.general.exclude_from_capture = exclude;
        self.save(&settings)
    }

    pub fn set_message_clear_interval_seconds(&self, seconds: u32) -> Result<()> {
        if !(1..=3600).contains(&seconds) {
            return Err(anyhow::anyhow!(
                "Message clear interval must be between 1 and 3600 seconds"
            ));
        }
        let mut settings = self.load();
        settings.general.message_clear_interval_seconds = seconds;
        self.save(&settings)
    }
}
