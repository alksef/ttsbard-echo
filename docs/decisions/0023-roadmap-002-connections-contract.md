# Decision 0023 — Connections domain contract

Roadmap 002 owns the Connections screen in both the main and floating webviews.

## Ownership

- Persisted configuration is read and mutated through the Connections screen.
- Runtime state is backend-owned and read through `get_connection_runtime_snapshot`.
- `ConnectionRuntimeSnapshotDto` exposes only `id`, `status`, `last_message`, and
  `error_message`; access tokens never enter the snapshot.
- `connection-status-changed` and `message-received` are subscribed once per
  webview by `useConnections`, and all listeners are disposed on unmount.

## Form

Add and Edit use `ConnectionFormDialog`. The canonical fields are `name`, full
HTTP(S) `url`, and optional `access_token`; `/sse` remains the default only for
new connections. Existing non-standard URLs are preserved during editing.

## Lifecycle

Connect/disconnect/remove mutations are followed by a backend snapshot reload.
Removing a connection first aborts its active SSE task, then removes persisted
configuration. The persisted `enabled` field remains the startup policy; the
screen's power action controls runtime connect/disconnect and does not rewrite
`enabled` automatically.
