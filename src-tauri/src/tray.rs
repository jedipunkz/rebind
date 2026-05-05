use tauri::{
    AppHandle, Manager,
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
};

use crate::app::AppState;

const MENU_ENABLE: &str = "enable";
const MENU_RELOAD: &str = "reload";
const MENU_OPEN_CONFIG: &str = "open_config";
const MENU_QUIT: &str = "quit";

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let state = app.state::<std::sync::Arc<AppState>>();
    let enable = CheckMenuItem::with_id(
        app,
        MENU_ENABLE,
        "Enabled",
        true,
        state.is_enabled(),
        None::<&str>,
    )?;
    let reload = MenuItem::with_id(app, MENU_RELOAD, "Reload config", true, None::<&str>)?;
    let open_config = MenuItem::with_id(app, MENU_OPEN_CONFIG, "Open config", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, MENU_QUIT, "Quit", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&enable, &reload, &open_config, &separator, &quit])?;

    TrayIconBuilder::with_id("rebind")
        .tooltip("Rebind")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .build(app)?;

    Ok(())
}

pub fn handle_menu_event(app: &AppHandle, id: &str) {
    let state = app.state::<std::sync::Arc<AppState>>();

    match id {
        MENU_ENABLE => {
            state.set_enabled(!state.is_enabled());
        }
        MENU_RELOAD => {
            if let Err(error) = state.reload_config() {
                state.set_error(error.to_string());
                tracing::warn!("config reload failed: {error}");
            }
        }
        MENU_OPEN_CONFIG => {
            if let Err(error) = state.open_config() {
                state.set_error(error);
            }
        }
        MENU_QUIT => {
            app.exit(0);
        }
        _ => {}
    }
}
