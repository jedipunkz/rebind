use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use arc_swap::ArcSwap;
use thiserror::Error;

use crate::{config, hook, tray};

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    #[error(transparent)]
    Hook(#[from] hook::HookError),
}

pub struct AppState {
    enabled: AtomicBool,
    config_path: PathBuf,
    config: ArcSwap<config::Config>,
    last_error: Mutex<Option<String>>,
}

impl AppState {
    pub fn new(config_path: PathBuf, config: config::Config) -> Self {
        let enabled = config.enabled;
        Self {
            enabled: AtomicBool::new(enabled),
            config_path,
            config: ArcSwap::from_pointee(config),
            last_error: Mutex::new(None),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    pub fn config(&self) -> Arc<config::Config> {
        self.config.load_full()
    }

    pub fn set_error(&self, error: impl Into<String>) {
        *self.last_error.lock().expect("last_error poisoned") = Some(error.into());
    }

    pub fn clear_error(&self) {
        *self.last_error.lock().expect("last_error poisoned") = None;
    }

    pub fn reload_config(&self) -> Result<(), config::ConfigError> {
        let (_, config) = config::load_from_path(self.config_path.clone())?;
        self.enabled.store(config.enabled, Ordering::Relaxed);
        self.config.store(Arc::new(config));
        self.clear_error();
        Ok(())
    }

    pub fn open_config(&self) -> Result<(), String> {
        open::that(&self.config_path).map_err(|err| err.to_string())
    }
}

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rebind=info,warn".into()),
        )
        .init();

    if let Err(error) = run_inner() {
        eprintln!("failed to start Rebind: {error}");
    }
}

fn run_inner() -> Result<(), AppError> {
    let (config_path, config) = config::load_or_create_default()?;
    let state = Arc::new(AppState::new(config_path, config));
    hook::install(state.clone())?;

    tauri::Builder::default()
        .manage(state)
        .setup(|app| {
            tray::setup(app.handle())?;
            Ok(())
        })
        .on_menu_event(|app, event| {
            tray::handle_menu_event(app, event.id().as_ref());
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    hook::uninstall();
    Ok(())
}
