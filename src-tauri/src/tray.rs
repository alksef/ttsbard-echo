use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, Manager};
use tracing::info;

/// Initialize system tray with icon and menu
pub fn init_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.handle().clone();

    // Embed the generated tray icon so development and production use the same asset.
    let icon_data = include_bytes!("../icons/32x32.png");
    let decoded_image = image::load_from_memory(icon_data)?;
    let rgba_image = decoded_image.to_rgba8();
    let icon = tauri::image::Image::new_owned(rgba_image.into_raw(), 32, 32);

    // Create menu items
    let show_item = MenuItem::with_id(&app_handle, "show", "Показать", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(&app_handle, "quit", "Выход", true, None::<&str>)?;
    let menu = Menu::with_items(&app_handle, &[&show_item, &quit_item])?;

    // Build tray icon
    info!("Initializing system tray");
    let _ = TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip("ttsbard-echo")
        .menu(&menu)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button,
                button_state,
                ..
            } = event
            {
                if matches!(button, MouseButton::Left)
                    && matches!(button_state, MouseButtonState::Up)
                {
                    if let Some(window) = tray.app_handle().get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .on_menu_event(|tray, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => {
                tray.app_handle().exit(0);
            }
            _ => {}
        })
        .build(&app_handle);

    info!("System tray initialized");
    Ok(())
}

/// Try to load icon from disk (various paths for dev/prod)
fn try_load_icon_from_disk(app: &App) -> Option<Vec<u8>> {
    if let Ok(resource_dir) = app.path().resource_dir() {
        // Try resource_dir/icons/icon.ico
        let icon_path = resource_dir.join("icons").join("icon.ico");
        if icon_path.exists() {
            return std::fs::read(&icon_path).ok();
        }

        // Try parent dirs for development (resource_dir might be in target/debug)
        for parent in std::iter::successors(resource_dir.parent(), |p| p.parent()).take(3) {
            let dev_icon_path = parent.join("icons").join("icon.ico");
            if dev_icon_path.exists() {
                return std::fs::read(&dev_icon_path).ok();
            }
        }
    }
    None
}

/// Fallback icon data - simple 32x32 RGBA icon
fn get_fallback_icon() -> Vec<u8> {
    // Create a simple 32x32 icon with a circle
    let mut icon_data = vec![0u8; 32 * 32 * 4];
    for y in 0..32 {
        for x in 0..32 {
            let idx = (y * 32 + x) * 4;
            let center_x = 16.0;
            let center_y = 16.0;
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < 14.0 {
                // Blue circle with gradient
                let alpha = if dist < 12.0 {
                    255
                } else {
                    ((14.0 - dist) * 255.0) as u8
                };
                icon_data[idx] = 70; // R
                icon_data[idx + 1] = 130; // G
                icon_data[idx + 2] = 180; // B
                icon_data[idx + 3] = alpha; // A
            } else {
                icon_data[idx + 3] = 0; // Transparent
            }
        }
    }
    icon_data
}
