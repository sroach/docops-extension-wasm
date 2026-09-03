mod common;
mod envelope;
mod types;

use wasm_bindgen::prelude::*;

/// Runs once when the wasm module is loaded. Routes Rust panics to
/// console.error with a real message + stack trace instead of an opaque
/// "unreachable executed" trap.
#[wasm_bindgen(start)]
fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Single entry point called from JavaScript, for every visualization type.
/// Kept as `generate_svg` for backwards compatibility with existing HTML —
/// dispatch on type now happens internally via envelope + types::render.
#[wasm_bindgen]
pub fn generate_svg(input: &str) -> String {
    let result = envelope::parse_envelope(input)
        .and_then(|env| types::render(&env.viz_type, env.body, &env.controls));

    result.unwrap_or_else(|e| common::svg::error_svg(&e))
}
