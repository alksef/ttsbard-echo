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
        let _ = windows_manager.set_floating_visible(true);
        state.emit_event(AppEvent::FloatingVisibilityChanged(true));
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
        let _ = state.windows_manager.read().set_floating_visible(false);
        state.emit_event(AppEvent::FloatingVisibilityChanged(false));
    }

    Ok(())
}

pub fn toggle_floating_window(app_handle: &AppHandle, state: &AppState) -> tauri::Result<bool> {
    if let Some(window) = app_handle.get_webview_window("floating") {
        if window.is_visible()? {
            hide_floating_window(app_handle, state)?;
            return Ok(false);
        } else {
            show_floating_window(app_handle, state)?;
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn update_floating_appearance(app_handle: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app_handle.get_webview_window("floating") {
        window.emit("floating-appearance-update", ())?;
    }
    Ok(())
}
