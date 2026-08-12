# Testing

Run the repository checks from PowerShell:

```powershell
npm test
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

`check:ipc` verifies frontend invokes against registered Tauri commands. `check:settings` verifies the Rust/TypeScript settings fields and appearance conversion helpers. Windows WebView behavior remains a manual smoke-test surface.
