//! Neoland Web Console — Leptos WASM entrypoint.
//!
//! This is a thin wrapper that mounts the root [`neoland_web::App`] component.
//! The full application logic lives in `lib.rs` so it can be unit-tested
//! and integration-tested via `wasm-bindgen-test`.

use leptos::prelude::*;

fn main() {
    mount_to_body(neoland_web::App);
}
