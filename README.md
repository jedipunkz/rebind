# Rebind

Rebind is a Windows tray resident application that provides basic Emacs-like keybindings in ordinary Windows applications.

## Installation

Download the Windows binary from the GitHub Releases page and place it under `C:\Program Files\rebind`.

Recommended layout:

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
