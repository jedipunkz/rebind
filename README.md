# Rebind

<p align="center">
  <img src="./rebind.png" alt="UnNatural icon" width="128">
</p>

Rebind is a Windows tray application that adds Emacs-like keybindings to ordinary Windows apps.

## Installation

Download the Windows binary from the GitHub Releases page and place it in any folder. To install under `C:\Program Files\rebind`, use the following layout:

```text
C:\Program Files\rebind\
  rebind.exe
  rebind.yaml
```

Rebind reads `rebind.yaml` from the same directory as `rebind.exe`. When the application starts and `rebind.yaml` does not exist, it tries to create a default configuration file next to the executable.

`C:\Program Files` is not writable by normal user permissions, so create `rebind.yaml` in advance when installing Rebind there. For example, create this file as Administrator:

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
  ctrl-y: ctrl-v
  ctrl-g: escape
```

After placing both files, run:

```powershell
C:\Program Files\rebind\rebind.exe
```

## Usage

Run `rebind.exe`. Rebind starts without showing a main window and stays in the system tray. Use the tray menu to enable or disable key remapping, reload the config, open the config file, or quit the app.

Default keybindings:

| Key | Action |
| --- | --- |
| `ctrl-a` | Move to beginning of line |
| `ctrl-e` | Move to end of line |
| `ctrl-b` / `ctrl-f` | Move left / right |
| `ctrl-p` / `ctrl-n` | Move up / down |
| `ctrl-h` / `ctrl-d` | Backspace / Delete |
| `ctrl-k` | Cut from cursor to end of line |
| `ctrl-w` / `ctrl-y` | Cut / paste |
| `ctrl-g` | Escape |

To change the bindings, edit `rebind.yaml` next to the executable and select `Reload config` from the tray menu.

## Configuration

`rebind.yaml` must be placed in the same directory as `rebind.exe`.

- `enabled`: controls whether key rebinding is enabled at startup.
- `ignore_app`: lists executable names where Rebind should not apply bindings.
- `bindings`: maps input key chords to output key chords or sequences.

## Development Build

Use `cargo build` only when building from source for development.

```bash
cargo build
```

For a release build:

```bash
cargo build --release
```

The executable is generated at `target/release/rebind.exe`. To create an installer or bundle, install the Tauri CLI and run:

```bash
cargo tauri build
```
