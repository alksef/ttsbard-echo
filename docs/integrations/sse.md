# SSE connection contract

Each configured connection is an HTTP(S) endpoint consumed as an SSE stream. The optional access token is sent as the configured authentication credential by the connection client; it is persisted in settings but excluded from UI runtime snapshots.

The backend owns the lifecycle: `Disconnected` → `Connecting` → `Connected`, or `Error(reason)`. Connect/disconnect commands start or abort the managed task. Reconnection and keepalive handling are implemented by the client task; the UI receives status/message events and reloads the authoritative snapshot after mutations.

Supported application events are `connection-status-changed`, `message-received`, `typing-changed`, `connections-changed`, `connection-added`, and `connection-removed`. Payload shapes are documented in [events reference](../reference/events.md).

Only HTTP(S) URLs with a non-empty host are accepted. Do not put tokens in logs, screenshots, issue reports, or documentation.
