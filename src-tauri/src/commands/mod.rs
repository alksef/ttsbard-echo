use parking_lot::RwLock;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

pub mod app;
pub mod connections;
pub mod settings;
pub mod windows;

/// Emit the umbrella `settings-changed` event so `useAppSettings` reloads.
///
/// The event name is the canonical one from `AppEvent::SettingsChanged` (the
/// single source of truth shared with `event_loop`), so a direct `app_handle`
/// emit here and the channel-routed emit in `event_loop` always agree.
pub fn emit_settings_changed(app_handle: &AppHandle) {
    let _ = app_handle.emit(
        crate::events::AppEvent::SettingsChanged.to_tauri_event(),
        (),
    );
}

/// Run a sync manager operation on a blocking thread pool.
///
/// Disk-bound mutations (e.g. `SettingsManager::set_*`, `WindowsManager::save`) must
/// not run on the tokio worker pool — they would stall the async runtime (and the SSE
/// stream) under antivirus/disk load. This helper moves the work to `spawn_blocking`.
///
/// `manager` is the `Arc<RwLock<M>>` stored on `AppState`; we take it **by value**
/// (a cheap `Arc` clone the caller makes) so the spawned task is `'static` and does not
/// borrow the Tauri `State<'_>` guard. The closure receives a shared `&M` borrow —
/// manager mutators all take `&self` and synchronise internally.
pub async fn persist_blocking<M, F, R>(manager: Arc<RwLock<M>>, op: F) -> Result<R, String>
where
    M: Send + Sync + 'static,
    F: FnOnce(&M) -> anyhow::Result<R> + Send + 'static,
    R: Send + 'static,
{
    tokio::task::spawn_blocking(move || {
        let guard = manager.read();
        op(&guard)
    })
    .await
    .map_err(|e| format!("blocking task panicked: {}", e))?
    .map_err(|e| e.to_string())
}
