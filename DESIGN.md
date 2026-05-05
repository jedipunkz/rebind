# Rebind Design

## Goal

Rebind is a Windows resident tray application that provides basic Emacs-like keybindings in ordinary Windows applications.

The application is built with Tauri and Rust. Tauri owns the desktop application lifecycle, tray icon, and small control UI. Rust owns configuration loading, foreground application detection, keyboard hook handling, and synthetic keyboard input.

Initial target platform is Windows only.

## Non-Goals

- Full Emacs emulation.
- Per-application complex scripting.
- IME-aware text editing semantics beyond what Windows target applications already support.
- Kernel driver or service installation.
- Cross-platform support in the first version.

## User Experience

- The app starts without showing a main window.
- A tray icon stays resident while key rebinding is enabled.
- The tray menu exposes:
  - Enable / Disable
  - Reload config
  - Open config file
  - Quit
- Configuration is read from `rebind.yaml` in the same directory as the executable.
- If `rebind.yaml` does not exist, the app creates a default one on startup.
- Config reload failures do not stop the running hook. The previous valid config remains active, and the tray tooltip / menu status reports the error.

## Technology

- Tauri v2 for the Windows desktop shell and tray integration.
- Rust for all platform logic.
- Windows APIs:
  - `SetWindowsHookExW` with `WH_KEYBOARD_LL` for global keyboard interception.
  - `CallNextHookEx` for unhandled events.
  - `SendInput` for synthetic key events.
  - `GetForegroundWindow`, `GetWindowThreadProcessId`, `OpenProcess`, and `QueryFullProcessImageNameW` for foreground process detection.
- Suggested crates:
  - `tauri` with the `tray-icon` feature.
  - `windows` for Win32 API bindings.
  - `serde` and `serde_yaml` for config parsing.
  - `thiserror` for application errors.
  - `tracing` and `tracing-subscriber` for diagnostics.

Tauri's official v2 tray documentation describes Rust-side tray support behind the `tray-icon` feature: https://v2.tauri.app/learn/system-tray/

## High-Level Architecture

```text
┌────────────────────┐
│ Tauri App          │
│ - lifecycle        │
│ - tray menu        │
│ - config commands  │
└─────────┬──────────┘
          │ manages
┌─────────▼──────────┐
│ AppState           │
│ - enabled flag     │
│ - active config    │
│ - last error       │
└─────────┬──────────┘
          │ read-only snapshots
┌─────────▼──────────┐
│ Keyboard Hook      │
│ - WH_KEYBOARD_LL   │
│ - event matching   │
│ - event suppression│
└─────────┬──────────┘
          │ emits
┌─────────▼──────────┐
│ Input Synthesizer  │
│ - SendInput        │
│ - recursion guard  │
└────────────────────┘
```

## Configuration

The config file path is resolved as:

```text
<directory containing rebind.exe>\rebind.yaml
```

Example:

```yaml
version: 1
enabled: true

ignore_app:
  - Code.exe
  - WindowsTerminal.exe
  - emacs.exe

bindings:
  ctrl-a: home
  ctrl-e: end
  ctrl-b: left
  ctrl-f: right
  ctrl-p: up
  ctrl-n: down
  ctrl-h: backspace
  ctrl-d: delete
  ctrl-k:
    sequence:
      - shift-end
      - ctrl-x
  ctrl-w: ctrl-x
  ctrl-y: paste
  ctrl-g: escape
```

### Config Schema

```rust
struct Config {
    version: u32,
    enabled: bool,
    ignore_app: Vec<String>,
    bindings: BTreeMap<String, Action>,
}

enum Action {
    Key(KeyChord),
    Sequence(Vec<KeyChord>),
}
```

Key chord strings are normalized to lower-case internally.

`ignore_app` entries are executable file names, not full paths. Matching is case-insensitive on Windows.

Examples:

- `notepad.exe`
- `Code.exe`
- `WindowsTerminal.exe`

## Default Bindings

| Emacs key | Windows action | Notes |
| --- | --- | --- |
| `ctrl-a` | `home` | beginning of line |
| `ctrl-e` | `end` | end of line |
| `ctrl-b` | `left` | backward char |
| `ctrl-f` | `right` | forward char |
| `ctrl-p` | `up` | previous line |
| `ctrl-n` | `down` | next line |
| `ctrl-h` | `backspace` | delete backward |
| `ctrl-d` | `delete` | delete forward |
| `ctrl-k` | `shift-end`, `ctrl-x` | kill to end of line by selecting and cutting |
| `ctrl-w` | `ctrl-x` | cut current selection |
| `ctrl-y` | `paste` | yank from clipboard |
| `ctrl-g` | `escape` | cancel |

`ctrl-k`, `ctrl-w`, and `ctrl-y` intentionally use the system clipboard because common Windows controls do not expose a universal kill ring API.

## Keyboard Event Flow

1. Windows calls the low-level keyboard hook.
2. The hook ignores key-up events except when internal modifier state needs cleanup.
3. The hook checks whether Rebind is enabled.
4. The hook resolves the foreground process executable name.
5. If the executable name matches `ignore_app`, the event is passed through.
6. The hook normalizes the incoming physical key plus active modifiers into a chord string such as `ctrl-a`.
7. If the chord has no configured action, the event is passed through.
8. If a configured action exists:
   - Mark synthetic input guard as active.
   - Emit the configured key chord or sequence via `SendInput`.
   - Clear synthetic input guard.
   - Return a non-zero result to suppress the original event.

Synthetic events are ignored by the hook to prevent recursion.

## Foreground App Detection

The foreground executable name is cached briefly to avoid opening a process handle for every key event. The cache can be invalidated when the foreground window handle changes.

Resolution steps:

1. `GetForegroundWindow`
2. `GetWindowThreadProcessId`
3. `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)`
4. `QueryFullProcessImageNameW`
5. Extract the file name portion

If detection fails, the app should default to pass-through for that event.

## Threading Model

- Tauri runs on the main thread.
- The keyboard hook runs on a dedicated OS thread with a Windows message loop.
- Shared state is stored behind `Arc`.
- Runtime config is stored as an atomic snapshot, for example `ArcSwap<Config>` or `RwLock<Arc<Config>>`.
- The enabled flag is an `AtomicBool` for fast reads in the hook.

The hook callback must not perform slow UI work. It should only read already-loaded state, do minimal foreground process detection, and emit synthetic input.

## Error Handling

- Invalid YAML:
  - Keep the previous valid config.
  - Store the parse error in `AppState`.
  - Show an error status from the tray UI.
- Missing config:
  - Write the default config beside the executable.
  - Load it immediately.
- Hook installation failure:
  - Surface a fatal startup error.
  - Do not show the app as enabled.
- SendInput failure:
  - Log the failure.
  - Suppress only if input was actually emitted successfully.

## Security and Privacy

- The hook does not record typed text.
- The app only matches configured key chords.
- Logs must not include arbitrary key streams.
- No network access is required.
- No elevated privilege is required for normal desktop applications.

Some elevated target applications may not receive synthetic input from a non-elevated Rebind process due to Windows integrity level rules. The first version documents this limitation rather than requesting elevation.

## Project Layout

Expected initial Tauri layout:

```text
.
├── DESIGN.md
├── README.md
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── icons/
│   └── src/
│       ├── main.rs
│       ├── app.rs
│       ├── config.rs
│       ├── foreground.rs
│       ├── hook.rs
│       ├── input.rs
│       └── tray.rs
└── ui/
    └── minimal status window, optional
```

The first implementation can avoid a rich frontend and use only a hidden Tauri window plus tray menu. A small status window can be added later if needed.

## Implementation Phases

1. Scaffold a Windows-only Tauri v2 app.
2. Add tray icon and menu.
3. Add `rebind.yaml` loading, default generation, and reload command.
4. Implement foreground executable detection and `ignore_app` matching.
5. Implement keyboard chord parsing and action model.
6. Implement `SendInput` synthesis.
7. Implement `WH_KEYBOARD_LL` hook thread.
8. Wire tray enable / disable / reload actions.
9. Add focused tests for config parsing, chord normalization, and ignore matching.
10. Add manual Windows verification steps.

## Manual Verification

On Windows:

1. Start Rebind.
2. Confirm a tray icon appears and no main window is shown.
3. Open Notepad.
4. Type multiple lines.
5. Verify:
   - `ctrl-a` moves to the beginning of the line.
   - `ctrl-e` moves to the end of the line.
   - `ctrl-b` / `ctrl-f` move left / right.
   - `ctrl-p` / `ctrl-n` move up / down.
   - `ctrl-k` cuts from the cursor to end of line.
   - `ctrl-w` cuts the current selection.
6. Add `notepad.exe` to `ignore_app`.
7. Reload config from the tray.
8. Confirm Notepad receives the original Windows shortcuts instead of Rebind actions.

## Open Questions

- Should `ctrl-space` set a mark and allow a closer Emacs-like region model?
- Should the app implement a private kill ring instead of using the Windows clipboard for `ctrl-k`?
- Should config support per-application binding overrides after the first version?
- Should the app expose a log file path from the tray menu?
