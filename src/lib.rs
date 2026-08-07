mod app;
mod bevy_canvas;
mod model;
mod ui;
mod usd_loader;

pub use app::{App, shell};

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
