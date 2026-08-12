# Development

- [Architecture](architecture.md)
- [AI workflow](ai-workflow.md)
- [Contract checks and testing](testing.md)
- [Build](building.md)
- [IPC and events](events-and-ipc.md)
- [Configuration](configuration.md)

Frontend entrypoints are `src/main.ts` and `src/floating-main.ts`. Rust commands are registered in `src-tauri/src/lib.rs`; persisted settings are owned by the settings and windows managers.
