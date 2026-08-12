# 0022 — Roadmap 001 shell and floating visibility contract

- **Status:** accepted
- **Scope:** roadmap 001

## Close behavior

The main window keeps the existing hide-to-tray behavior. The titlebar close
button calls the native close operation; the Rust `CloseRequested` handler
prevents destruction and hides the `main` window. Explicit process exit remains
available only through the tray menu.

## IPC contract

- `get_floating_visibility` → `boolean`: reads the actual visibility of the
  `floating` webview window.
- `toggle_floating_window` → `{ visible: boolean }`: applies one show/hide
  operation and returns the resulting state. The titlebar disables the control
  while this command is pending.

## Event contract

`floating-visibility-changed` carries `{ visible: boolean }` after every backend
show/hide operation, including operations initiated outside the titlebar. The
titlebar listens to this event and uses a fresh visibility snapshot after an
invocation error.
