.PHONY: help build release windows-build windows-release linux-build linux-release check test fmt fmt-check clippy tauri-build clean check-linux-deps check-windows-deps

CARGO ?= cargo
RUSTUP ?= rustup
WINDOWS_TARGET ?= x86_64-pc-windows-gnu
WINDOWS_DEBUG_DIR := target/$(WINDOWS_TARGET)/debug
WINDOWS_RELEASE_DIR := target/$(WINDOWS_TARGET)/release
WINDOWS_DIST_DEBUG := dist/windows-debug
WINDOWS_DIST_RELEASE := dist/windows-release
UNAME_S := $(shell uname -s)
LINUX_PKG_CONFIG_DEPS := \
	atk \
	cairo \
	dbus-1 \
	gdk-3.0 \
	gdk-pixbuf-2.0 \
	gio-2.0 \
	glib-2.0 \
	gobject-2.0 \
	javascriptcoregtk-4.1 \
	libsoup-3.0 \
	pango \
	webkit2gtk-4.1

help:
	@printf '%s\n' 'Targets:'
	@printf '  %-12s %s\n' 'build' 'Build debug binary'
	@printf '  %-12s %s\n' 'release' 'Build release binary'
	@printf '  %-12s %s\n' 'windows-build' 'Build Windows debug executable'
	@printf '  %-12s %s\n' 'windows-release' 'Build Windows release executable'
	@printf '  %-12s %s\n' 'linux-build' 'Build Linux debug binary'
	@printf '  %-12s %s\n' 'linux-release' 'Build Linux release binary'
	@printf '  %-12s %s\n' 'check' 'Run cargo check'
	@printf '  %-12s %s\n' 'test' 'Run tests'
	@printf '  %-12s %s\n' 'fmt' 'Format Rust sources'
	@printf '  %-12s %s\n' 'fmt-check' 'Check Rust formatting'
	@printf '  %-12s %s\n' 'clippy' 'Run clippy with warnings denied'
	@printf '  %-12s %s\n' 'tauri-build' 'Build Tauri bundle'
	@printf '  %-12s %s\n' 'clean' 'Remove Cargo build artifacts'

ifeq ($(UNAME_S),Linux)
build: windows-build

release: windows-release
else
build:
	$(CARGO) build

release:
	$(CARGO) build --release
endif

windows-build: check-windows-deps
	$(CARGO) build --target $(WINDOWS_TARGET)
	@mkdir -p $(WINDOWS_DIST_DEBUG)
	@cp $(WINDOWS_DEBUG_DIR)/rebind.exe $(WINDOWS_DEBUG_DIR)/WebView2Loader.dll $(WINDOWS_DIST_DEBUG)/
	@[ ! -f $(WINDOWS_DEBUG_DIR)/rebind.yaml ] || cp $(WINDOWS_DEBUG_DIR)/rebind.yaml $(WINDOWS_DIST_DEBUG)/
	@printf '%s\n' "Windows app directory: $(WINDOWS_DIST_DEBUG)"
	@command -v wslpath >/dev/null && wslpath -w "$$(pwd)/$(WINDOWS_DIST_DEBUG)" || true

windows-release: check-windows-deps
	$(CARGO) build --release --target $(WINDOWS_TARGET)
	@mkdir -p $(WINDOWS_DIST_RELEASE)
	@cp $(WINDOWS_RELEASE_DIR)/rebind.exe $(WINDOWS_RELEASE_DIR)/WebView2Loader.dll $(WINDOWS_DIST_RELEASE)/
	@[ ! -f $(WINDOWS_RELEASE_DIR)/rebind.yaml ] || cp $(WINDOWS_RELEASE_DIR)/rebind.yaml $(WINDOWS_DIST_RELEASE)/
	@printf '%s\n' "Windows app directory: $(WINDOWS_DIST_RELEASE)"
	@command -v wslpath >/dev/null && wslpath -w "$$(pwd)/$(WINDOWS_DIST_RELEASE)" || true

linux-build: check-linux-deps
	$(CARGO) build

linux-release: check-linux-deps
	$(CARGO) build --release

check: check-linux-deps
	$(CARGO) check

test: check-linux-deps
	$(CARGO) test

fmt:
	$(CARGO) fmt

fmt-check:
	$(CARGO) fmt --check

clippy: check-linux-deps
	$(CARGO) clippy --all-targets -- -D warnings

tauri-build: check-linux-deps
	$(CARGO) tauri build

clean:
	$(CARGO) clean

check-linux-deps:
ifeq ($(UNAME_S),Linux)
	@command -v pkg-config >/dev/null || { \
		printf '%s\n' 'error: pkg-config is required to build Tauri on Linux.'; \
		printf '%s\n' 'Install pkg-config and dbus development headers, for example:'; \
		printf '%s\n' '  sudo apt install pkg-config libdbus-1-dev libgtk-3-dev libsoup-3.0-dev libwebkit2gtk-4.1-dev'; \
		exit 1; \
	}
	@missing=""; \
	for dep in $(LINUX_PKG_CONFIG_DEPS); do \
		pkg-config --exists "$$dep" || missing="$$missing $$dep"; \
	done; \
	if [ -n "$$missing" ]; then \
		printf '%s\n' "error: missing Linux development libraries:$$missing"; \
		printf '%s\n' 'Install them, for example:'; \
		printf '%s\n' '  sudo apt install pkg-config libdbus-1-dev libgtk-3-dev libsoup-3.0-dev libwebkit2gtk-4.1-dev'; \
		exit 1; \
	fi
endif

check-windows-deps:
	@$(RUSTUP) target list --installed | grep -qx '$(WINDOWS_TARGET)' || { \
		printf '%s\n' "error: Rust target $(WINDOWS_TARGET) is not installed."; \
		printf '%s\n' 'Install it with:'; \
		printf '%s\n' "  rustup target add $(WINDOWS_TARGET)"; \
		exit 1; \
	}
	@command -v x86_64-w64-mingw32-gcc >/dev/null || { \
		printf '%s\n' 'error: MinGW-w64 is required to build a Windows .exe from WSL/Linux.'; \
		printf '%s\n' 'Install it with:'; \
		printf '%s\n' '  sudo apt install mingw-w64'; \
		exit 1; \
	}
