use crate::config::{
    constants::{DEFAULT_FLOATING_BG_COLOR, DEFAULT_FLOATING_OPACITY},
    validation::{validate_hex_color, validate_opacity},
};
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
    #[serde(default)]
    pub use_custom_color: bool,
    #[serde(default)]
    pub visible: bool,
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
            use_custom_color: false,
            visible: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WindowsSettings {
    #[serde(default)]
    pub main: WindowPosition,
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
        let mut cache = self.cache.write();
        let windows_file = self.config_dir.join("windows.json");
        let content = serde_json::to_string_pretty(settings)?;
        std::fs::write(&windows_file, content)?;
        *cache = settings.clone();
        Ok(())
    }

    /// Update the cached snapshot and its persisted representation as one
    /// transaction. Holding the cache write lock across the read/modify/write
    /// cycle prevents concurrent position, visibility, and appearance saves
    /// from restoring an older snapshot over a newer value.
    fn update<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut WindowsSettings),
    {
        let mut cache = self.cache.write();
        let mut settings = cache.clone();
        updater(&mut settings);

        let windows_file = self.config_dir.join("windows.json");
        let content = serde_json::to_string_pretty(&settings)?;
        std::fs::write(&windows_file, content)?;
        *cache = settings;
        Ok(())
    }

    pub fn get_floating_opacity(&self) -> u8 {
        self.cache.read().floating.opacity
    }

    pub fn set_floating_opacity(&self, value: u8) -> Result<()> {
        self.update(|settings| settings.floating.opacity = validate_opacity(value))
    }

    pub fn get_floating_bg_color(&self) -> String {
        self.cache.read().floating.bg_color.clone()
    }

    pub fn set_floating_bg_color(&self, color: String) -> Result<()> {
        let color = validate_hex_color(&color).map_err(|error| anyhow::anyhow!(error))?;
        self.update(|settings| settings.floating.bg_color = color)
    }

    pub fn set_floating_use_custom_color(&self, enabled: bool) -> Result<()> {
        self.update(|settings| settings.floating.use_custom_color = enabled)
    }

    pub fn get_floating_clickthrough(&self) -> bool {
        self.cache.read().floating.clickthrough
    }

    pub fn set_floating_clickthrough(&self, enabled: bool) -> Result<()> {
        self.update(|settings| settings.floating.clickthrough = enabled)
    }

    pub fn get_floating_position(&self) -> (Option<i32>, Option<i32>) {
        let pos = &self.cache.read().floating.position;
        (pos.x, pos.y)
    }

    pub fn set_main_position(&self, x: Option<i32>, y: Option<i32>) -> Result<()> {
        self.update(|settings| {
            settings.main.x = x;
            settings.main.y = y;
        })
    }

    pub fn set_floating_position(&self, x: Option<i32>, y: Option<i32>) -> Result<()> {
        self.update(|settings| {
            settings.floating.position.x = x;
            settings.floating.position.y = y;
        })
    }

    pub fn set_floating_visible(&self, visible: bool) -> Result<()> {
        self.update(|settings| settings.floating.visible = visible)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::Barrier, thread, time::SystemTime};

    fn test_manager() -> (Arc<WindowsManager>, PathBuf) {
        let unique = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let config_dir = std::env::temp_dir().join(format!(
            "ttsbard-echo-windows-{}-{unique}",
            std::process::id()
        ));
        std::fs::create_dir_all(&config_dir).unwrap();
        let initial = WindowsSettings::default();
        std::fs::write(
            config_dir.join("windows.json"),
            serde_json::to_string_pretty(&initial).unwrap(),
        )
        .unwrap();

        let manager = Arc::new(WindowsManager {
            config_dir: config_dir.clone(),
            cache: Arc::new(RwLock::new(initial)),
        });
        (manager, config_dir)
    }

    #[test]
    fn concurrent_updates_do_not_lose_unrelated_fields() {
        let (manager, config_dir) = test_manager();
        let barrier = Arc::new(Barrier::new(3));

        let appearance_manager = manager.clone();
        let appearance_barrier = barrier.clone();
        let appearance = thread::spawn(move || {
            appearance_barrier.wait();
            appearance_manager
                .set_floating_bg_color("#12AB34".into())
                .unwrap();
            appearance_manager
                .set_floating_use_custom_color(true)
                .unwrap();
        });

        let position_manager = manager.clone();
        let position_barrier = barrier.clone();
        let position = thread::spawn(move || {
            position_barrier.wait();
            position_manager
                .set_floating_position(Some(321), Some(654))
                .unwrap();
        });

        barrier.wait();
        appearance.join().unwrap();
        position.join().unwrap();

        let settings = manager.load();
        assert_eq!(settings.floating.bg_color, "#12AB34");
        assert!(settings.floating.use_custom_color);
        assert_eq!(settings.floating.position.x, Some(321));
        assert_eq!(settings.floating.position.y, Some(654));

        let persisted: WindowsSettings = serde_json::from_str(
            &std::fs::read_to_string(config_dir.join("windows.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(persisted.floating.bg_color, "#12AB34");
        assert!(persisted.floating.use_custom_color);
        assert_eq!(persisted.floating.position.x, Some(321));
        assert_eq!(persisted.floating.position.y, Some(654));

        std::fs::remove_dir_all(config_dir).unwrap();
    }
}
