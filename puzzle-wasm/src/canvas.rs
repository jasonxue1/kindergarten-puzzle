use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

// Non-deprecated helpers to set canvas styles via property assignment.
pub fn set_fill_style(ctx: &CanvasRenderingContext2d, color: &str) {
    let _ = js_sys::Reflect::set(
        ctx.as_ref(),
        &JsValue::from_str("fillStyle"),
        &JsValue::from_str(color),
    );
}

pub fn set_stroke_style(ctx: &CanvasRenderingContext2d, color: &str) {
    let _ = js_sys::Reflect::set(
        ctx.as_ref(),
        &JsValue::from_str("strokeStyle"),
        &JsValue::from_str(color),
    );
}
