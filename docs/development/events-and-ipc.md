# IPC and events

The command registry is the `invoke_handler` in `src-tauri/src/lib.rs`. Frontend callers use `@tauri-apps/api/core` and are checked by `npm run check:ipc`.

Connection mutations are followed by an authoritative configs/runtime snapshot reload. Visibility and appearance changes are emitted by the backend; listeners are owned by the relevant composable/component and cleaned up on unmount.
