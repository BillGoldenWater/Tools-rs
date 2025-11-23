use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

pub mod navigation_compass_solver;
pub mod aeroplane_chess_solver {
    pub use aeroplane_chess_solver::*;
}

#[wasm_bindgen]
pub fn wasm_init() {
    console_error_panic_hook::set_once();

    let log_level = if cfg!(debug_assertions) {
        log::Level::Debug
    } else {
        log::Level::Info
    };
    let result = console_log::init_with_level(log_level);
    if result.is_err() {
        web_sys::console::warn_1(&JsValue::from_str(
            "failed to set logger",
        ))
    }
}
