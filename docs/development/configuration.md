# Configuration

Settings are stored under the platform config directory in the `ttsbard-echo` application folder. Connection settings include endpoint/name/enabled state and optional access token. Window settings include main/floating positions, floating opacity, background color, and click-through.

Secrets are never part of runtime snapshots or presentation cards. Defaults and serde behavior are defined in `src-tauri/src/config/` and parity is checked by `npm run check:settings`.
