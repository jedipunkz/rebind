# Rebind

![Rebind](./rebind.png)

Rebind は、Windows の通常アプリで Emacs 風のキーバインドを使うための常駐トレイアプリです。

## インストール

ビルド済みの `Rebind` を任意のフォルダに配置して起動します。初回起動時に、実行ファイルと同じフォルダへ `rebind.yaml` が自動生成されます。

ソースからビルドする場合:

```bash
cargo build --release
```

生成された実行ファイルは `target/release/rebind.exe` です。インストーラーやバンドルを作る場合は、Tauri CLI を入れてから実行します。

```bash
cargo tauri build
```

## 使い方

`rebind.exe` を起動するとメインウィンドウは表示されず、タスクトレイに常駐します。トレイメニューから有効化、無効化、設定の再読み込み、設定ファイルを開く、終了ができます。

既定では次のようなキーが使えます。

| キー | 動作 |
| --- | --- |
| `ctrl-a` | 行頭へ移動 |
| `ctrl-e` | 行末へ移動 |
| `ctrl-b` / `ctrl-f` | 左 / 右へ移動 |
| `ctrl-p` / `ctrl-n` | 上 / 下へ移動 |
| `ctrl-h` / `ctrl-d` | Backspace / Delete |
| `ctrl-k` | カーソル位置から行末まで切り取り |
| `ctrl-w` / `ctrl-y` | 切り取り / 貼り付け |
| `ctrl-g` | Escape |

設定を変える場合は、実行ファイルと同じフォルダの `rebind.yaml` を編集し、トレイメニューから `Reload config` を実行してください。

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

`ignore_app` には、Rebind を無効にしたいアプリの実行ファイル名を指定します。
