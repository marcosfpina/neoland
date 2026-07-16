//! Neoland Desktop — main entrypoint.
//!
//! Prevents a console window from appearing on Windows in release builds,
//! then delegates to [`neoland_desktop::run`].

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    neoland_desktop::run();
}
