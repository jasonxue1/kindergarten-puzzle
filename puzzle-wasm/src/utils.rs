use wasm_bindgen::{JsCast, JsValue};
use web_sys::Window;

use crate::models::Point;
use crate::state::State;

/// Log a message to the browser console.
pub fn log(s: &str) {
    web_sys::console::log_1(&JsValue::from_str(s));
}

/// Ensure the canvas backing store matches the CSS size and device pixel ratio
/// to prevent non-uniform stretching.
pub fn sync_canvas_size(state: &mut State) {
    let dpr = state.window.device_pixel_ratio();
    let (css_w, css_h) = if let Some(el) = state.canvas.dyn_ref::<web_sys::Element>() {
        let rect = el.get_bounding_client_rect();
        (rect.width().max(1.0), rect.height().max(1.0))
    } else {
        (
            state.canvas.client_width() as f64,
            state.canvas.client_height() as f64,
        )
    };
    let target_w = (css_w * dpr).round().clamp(1.0, 10000.0) as u32;
    let target_h = (css_h * dpr).round().clamp(1.0, 10000.0) as u32;
    if state.canvas.width() != target_w {
        state.canvas.set_width(target_w);
    }
    if state.canvas.height() != target_h {
        state.canvas.set_height(target_h);
    }
}

/// Convert a puzzle-space point to screen coordinates.
pub fn to_screen(p: Point, canvas_h: f64, scale: f64, offset: (f64, f64)) -> (f64, f64) {
    let (ox, oy) = offset;
    (p.x * scale + ox, canvas_h - (p.y * scale + oy))
}

/// Convert screen coordinates back into puzzle space.
pub fn from_screen(x: f64, y: f64, canvas_h: f64, scale: f64, offset: (f64, f64)) -> Point {
    let (ox, oy) = offset;
    Point {
        x: (x - ox) / scale,
        y: (canvas_h - y - oy) / scale,
    }
}

/// Build an absolute URL for an asset, taking into account the optional
/// `window.__BASE_URL` which is set by the host page.
pub fn asset_url(path: &str) -> String {
    let p = path.trim();
    if p.starts_with("http://") || p.starts_with("https://") || p.starts_with("data:") {
        return p.to_string();
    }
    let base = web_sys::window()
        .and_then(|w| {
            let v = js_sys::Reflect::get(&w, &JsValue::from_str("__BASE_URL")).ok()?;
            v.as_string()
        })
        .unwrap_or_else(|| "/".to_string());
    let base = if base.ends_with('/') {
        base
    } else {
        format!("{}/", base)
    };
    let p = p.trim_start_matches('/');
    format!("{}{}", base, p)
}

/// Fetch a text resource trying a list of fallback URLs in order.
pub async fn fetch_text_with_fallbacks(window: &Window, urls: &[&str]) -> Option<String> {
    for url in urls {
        let resp_value =
            match wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(url)).await {
                Ok(v) => v,
                Err(_) => continue,
            };
        let resp: web_sys::Response = match resp_value.dyn_into() {
            Ok(r) => r,
            Err(_) => continue,
        };
        if !resp.ok() {
            continue;
        }
        if let Ok(text_promise) = resp.text()
            && let Ok(text_js) = wasm_bindgen_futures::JsFuture::from(text_promise).await
            && let Some(s) = text_js.as_string()
        {
            return Some(s);
        }
    }
    None
}

/// Simple query string parser used at start-up.
pub fn get_query_param(search: &str, key: &str) -> Option<String> {
    let s = search.trim_start_matches('?');
    for pair in s.split('&') {
        let mut it = pair.splitn(2, '=');
        let k = it.next()?;
        let v = it.next().unwrap_or("");
        if k == key {
            return Some(url_decode(v));
        }
    }
    None
}

fn url_decode(s: &str) -> String {
    percent_encoding::percent_decode_str(s)
        .decode_utf8()
        .unwrap_or_else(|_| s.into())
        .to_string()
}
