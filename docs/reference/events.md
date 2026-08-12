# Events

Connection status and messages use tuple payloads `[connection_id, value]`. `typing-changed` uses an object payload `{ id: string, isTyping: boolean, previewText?: string }`. Floating visibility uses `{ visible: boolean }`. `connections-changed`, `connection-added`, `connection-removed`, `floating-appearance-changed`, `theme-changed`, and `settings-changed` are invalidation events with either an identifier or empty payload. Event names are defined in `src-tauri/src/events.rs`.
