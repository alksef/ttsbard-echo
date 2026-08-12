use tauri::{AppHandle, Emitter, State};
use tracing::info;

use crate::state::AppState;

/// Quit the application
#[tauri::command]
pub async fn quit_app(app_handle: AppHandle) -> Result<(), String> {
    info!("Quit requested - initiating graceful shutdown");

    let _ = app_handle.emit("app-exit", ());
    app_handle.exit(0);
    Ok(())
}

/// Check if backend is ready (settings loaded, initialization complete)
#[tauri::command]
pub fn is_backend_ready(app_state: State<'_, AppState>) -> bool {
    app_state
        .backend_ready
        .load(std::sync::atomic::Ordering::SeqCst)
}

/// Confirm that backend is ready for operation
#[tauri::command]
pub fn confirm_backend_ready(app_state: State<'_, AppState>) -> Result<(), String> {
    // Mark the backend as ready
    app_state
        .backend_ready
        .store(true, std::sync::atomic::Ordering::SeqCst);

    // Emit event to frontend that backend is ready
    let _ = app_state.app_handle.emit("backend-ready", ());

    Ok(())
}
