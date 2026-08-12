use crate::commands::{emit_settings_changed, persist_blocking};
use crate::config::ConnectionConfig;
use crate::connections::ConnectionManager;
use crate::events::AppEvent;
use crate::state::AppState;
use tauri::State;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConnectionRuntimeSnapshotDto {
    pub id: String,
    pub status: String,
    pub last_message: Option<String>,
    pub error_message: Option<String>,
    pub is_typing: bool,
    pub preview_text: Option<String>,
}

/// Get all connections.
#[tauri::command]
pub fn get_connections(app_state: State<'_, AppState>) -> Result<Vec<ConnectionConfig>, String> {
    Ok(app_state.settings_manager.read().load().connections)
}

/// Read the non-secret runtime state for every persisted connection.
#[tauri::command]
pub fn get_connection_runtime_snapshot(
    app_state: State<'_, AppState>,
) -> Result<Vec<ConnectionRuntimeSnapshotDto>, String> {
    let configs = app_state.settings_manager.read().load().connections;
    let runtime = app_state.connections.read();

    Ok(configs
        .into_iter()
        .map(|config| {
            let state = runtime.get(&config.id);
            let (status, error_message) = match state.map(|value| &value.status) {
                Some(crate::events::ConnectionStatus::Error(message)) => {
                    ("Error".to_string(), Some(message.clone()))
                }
                Some(value) => (value.to_string(), None),
                None => ("Disconnected".to_string(), None),
            };

            ConnectionRuntimeSnapshotDto {
                id: config.id,
                status,
                last_message: state.and_then(|value| value.last_message.clone()),
                error_message,
                is_typing: state.is_some_and(|value| value.is_typing),
                preview_text: state.and_then(|value| value.preview_text.clone()),
            }
        })
        .collect())
}

/// Add a new connection.
#[tauri::command]
pub async fn add_connection(
    config: ConnectionConfig,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let id = config.id.clone();
    persist_blocking(app_state.settings_manager.clone(), move |m| {
        m.add_connection(config)
    })
    .await?;
    app_state.emit_event(AppEvent::ConnectionAdded(id));
    app_state.emit_event(AppEvent::ConnectionsChanged);
    emit_settings_changed(&app_state.app_handle);
    Ok(())
}

/// Remove a connection by ID.
#[tauri::command]
pub async fn remove_connection(
    id: String,
    app_state: State<'_, AppState>,
    manager: State<'_, ConnectionManager>,
) -> Result<(), String> {
    manager.stop_connection(&id);
    app_state.emit_event(AppEvent::ConnectionStatusChanged(
        id.clone(),
        crate::events::ConnectionStatus::Disconnected,
    ));
    let removed_id = id.clone();
    persist_blocking(app_state.settings_manager.clone(), move |m| {
        m.remove_connection(&id)
    })
    .await?;
    app_state.emit_event(AppEvent::ConnectionRemoved(removed_id));
    app_state.emit_event(AppEvent::ConnectionsChanged);
    emit_settings_changed(&app_state.app_handle);
    Ok(())
}

/// Update a connection.
#[tauri::command]
pub async fn update_connection(
    id: String,
    config: ConnectionConfig,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    persist_blocking(app_state.settings_manager.clone(), move |m| {
        m.update_connection(&id, config)
    })
    .await?;
    app_state.emit_event(AppEvent::ConnectionsChanged);
    emit_settings_changed(&app_state.app_handle);
    Ok(())
}

/// Connect a connection by ID.
///
/// Looks the connection up in settings, spawns its SSE receive loop via
/// `ConnectionManager` (tracking the task handle), and emits `Connecting`.
/// A previously-running task for the same id is aborted first, so this is
/// also the reconnect path.
#[tauri::command]
pub async fn connect_connection(
    id: String,
    app_state: State<'_, AppState>,
    manager: State<'_, ConnectionManager>,
) -> Result<(), String> {
    // Resolve the config (validated on add/update, but re-check here too).
    let config = app_state
        .settings_manager
        .read()
        .load()
        .connections
        .into_iter()
        .find(|c| c.id == id)
        .ok_or_else(|| format!("Connection not found: {}", id))?;

    app_state.emit_event(AppEvent::ConnectionStatusChanged(
        id.clone(),
        crate::events::ConnectionStatus::Connecting,
    ));

    manager
        .start_connection(config)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Disconnect a connection by ID.
///
/// Aborts the tracked SSE receive task and emits `Disconnected` so the UI
/// reflects the stopped state immediately.
#[tauri::command]
pub async fn disconnect_connection(
    id: String,
    app_state: State<'_, AppState>,
    manager: State<'_, ConnectionManager>,
) -> Result<(), String> {
    manager.stop_connection(&id);
    app_state.emit_event(AppEvent::ConnectionStatusChanged(
        id,
        crate::events::ConnectionStatus::Disconnected,
    ));
    Ok(())
}
