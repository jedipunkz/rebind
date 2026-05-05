#![cfg_attr(not(windows), allow(dead_code))]

mod app;
mod config;
mod foreground;
mod hook;
mod input;
mod tray;

pub fn run() {
    app::run();
}
