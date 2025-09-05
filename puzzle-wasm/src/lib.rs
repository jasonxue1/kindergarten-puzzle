use std::cell::RefCell;
use std::rc::Rc;

use js_sys::Array;
use png::{BitDepth, ColorType, Compression, Encoder, FilterType};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{
    Blob, CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlElement, KeyboardEvent,
    MouseEvent, Url, Window,
};

mod canvas;
mod upload;

const DEFAULT_MM2PX: f64 = 3.0;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
struct Point {
    x: f64,
    y: f64,
}

impl From<(f64, f64)> for Point {
    fn from(v: (f64, f64)) -> Self {
        Point { x: v.0, y: v.1 }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Board {
    #[serde(rename = "type")]
    type_: Option<String>,
    w: Option<f64>,
    h: Option<f64>,
    r: Option<f64>,
    cut_corner: Option<String>,
    points: Option<Vec<[f64; 2]>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Piece {
    id: Option<String>,
    #[serde(rename = "type")]
    type_: String,
    // common fields
    at: Option<[f64; 2]>,
    rotation: Option<f64>,
    anchor: Option<String>,
    flip: Option<bool>,
    // rect
    w: Option<f64>,
    h: Option<f64>,
    // equilateral_triangle
    side: Option<f64>,
    // right_triangle
    a: Option<f64>,
    b: Option<f64>,
    // regular_polygon
    n: Option<u32>,
    // circle
    d: Option<f64>,
    r: Option<f64>,
    // isosceles_trapezoid
    base_bottom: Option<f64>,
    base_top: Option<f64>,
    height: Option<f64>,
    // parallelogram
    base: Option<f64>,
    offset_top: Option<f64>,
    // polygon
    points: Option<Vec<[f64; 2]>>,

    // cached runtime fields (not serialized)
    #[serde(skip)]
    __ctr: Option<Point>,
    #[serde(skip)]
    __geom: Option<Vec<Point>>, // for hit-testing
    #[serde(skip)]
    __color_idx: Option<usize>, // stable color assignment
    #[serde(skip)]
    __label_idx: Option<usize>, // stable numeric label (0-based)
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Puzzle {
    units: Option<String>,
    board: Option<Board>,
    pieces: Vec<Piece>,
    // Optional per-puzzle notes in two languages
    note_en: Option<String>,
    note_zh: Option<String>,
}

// Counts + shapes catalog for building a default puzzle without per-piece positions
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct PartSpec {
    #[serde(rename = "type")]
    type_: String,
    count: u32,
    w: Option<f64>,
    h: Option<f64>,
    side: Option<f64>,
    a: Option<f64>,
    b: Option<f64>,
    n: Option<u32>,
    d: Option<f64>,
    r: Option<f64>,
    base_bottom: Option<f64>,
    base_top: Option<f64>,
    height: Option<f64>,
    base: Option<f64>,
    offset_top: Option<f64>,
    points: Option<Vec<[f64; 2]>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct ShapeDef {
    id: String,
    #[serde(rename = "type")]
    type_: String,
    w: Option<f64>,
    h: Option<f64>,
    side: Option<f64>,
    a: Option<f64>,
    b: Option<f64>,
    n: Option<u32>,
    d: Option<f64>,
    r: Option<f64>,
    base_bottom: Option<f64>,
    base_top: Option<f64>,
    height: Option<f64>,
    base: Option<f64>,
    offset_top: Option<f64>,
    points: Option<Vec<[f64; 2]>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct ShapesCatalog {
    shapes: Vec<ShapeDef>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct CountsSpec {
    units: Option<String>,
    board: Option<Board>,
    counts: std::collections::HashMap<String, u32>,
    shapes_file: Option<String>,
    note_en: Option<String>,
    note_zh: Option<String>,
}

struct State {
    window: Window,
    document: Document,
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    data: Puzzle,
    dragging_idx: Option<usize>,
    drag_off: (f64, f64), // screen-space offset from piece center
    // view transform
    scale: f64,         // px per mm
    offset: (f64, f64), // (ox, oy) in px from bottom-left
    // continuous rotation control (deg per second, +ccw)
    rot_vel: f64,
    // speed control for Q/E
    slow_mode: bool,     // false = fast, true = slow
    rot_speed_fast: f64, // deg per second for fast mode
    rot_speed_slow: f64, // deg per second for slow mode
    // movement constraints
    restrict_mode: bool, // L toggles: prevent overlaps with pieces/border while moving
    shift_down: bool,    // temporary constraint while Shift held
    // initial snapshot for reset
    initial_data: Puzzle,
    // UI language: "en" or "zh"
    lang: String,
}

thread_local! {
    static STATE: RefCell<Option<Rc<RefCell<State>>>> = const { RefCell::new(None) };
}

fn log(s: &str) {
    web_sys::console::log_1(&JsValue::from_str(s));
}

fn to_screen(p: Point, canvas_h: f64, scale: f64, offset: (f64, f64)) -> (f64, f64) {
    let (ox, oy) = offset;
    (p.x * scale + ox, canvas_h - (p.y * scale + oy))
}

fn from_screen(x: f64, y: f64, canvas_h: f64, scale: f64, offset: (f64, f64)) -> Point {
    let (ox, oy) = offset;
    Point {
        x: (x - ox) / scale,
        y: (canvas_h - y - oy) / scale,
    }
}

use crate::canvas::{set_fill_style, set_stroke_style};

fn rotate_point(p: Point, c: Point, ang: f64, flip: bool) -> Point {
    let mut dx = p.x - c.x;
    let dy = p.y - c.y;
    if flip {
        dx = -dx;
    }
    let (s, ca) = ang.sin_cos();
    Point {
        x: c.x + dx * ca - dy * s,
        y: c.y + dx * s + dy * ca,
    }
}

fn piece_rotation(p: &Piece) -> f64 {
    (p.rotation.unwrap_or(0.0)).to_radians()
}
fn piece_flip(p: &Piece) -> bool {
    p.flip.unwrap_or(false)
}

fn piece_geom(p: &Piece) -> (Vec<Point>, Point) {
    let rot = piece_rotation(p);
    let flip = piece_flip(p);
    let anchor = p.anchor.clone().unwrap_or_else(|| "bottomleft".to_string());
    let apply = |pts: Vec<Point>, ctr: Point| -> (Vec<Point>, Point) {
        let out = pts
            .into_iter()
            .map(|q| rotate_point(q, ctr, rot, flip))
            .collect();
        (out, ctr)
    };
    match p.type_.as_str() {
        "rect" => {
            let w = p.w.unwrap_or(0.0);
            let h = p.h.unwrap_or(0.0);
            let at = p.at.unwrap_or([0.0, 0.0]);
            let bl = if anchor == "center" {
                Point {
                    x: at[0] - w / 2.0,
                    y: at[1] - h / 2.0,
                }
            } else {
                Point { x: at[0], y: at[1] }
            };
            let tl = Point {
                x: bl.x,
                y: bl.y + h,
            };
            let tr = Point {
                x: bl.x + w,
                y: bl.y + h,
            };
            let br = Point {
                x: bl.x + w,
                y: bl.y,
            };
            let ctr = Point {
                x: bl.x + w / 2.0,
                y: bl.y + h / 2.0,
            };
            apply(vec![bl, br, tr, tl], ctr)
        }
        "equilateral_triangle" => {
            let s = p.side.unwrap_or(0.0);
            let h = s * 3.0_f64.sqrt() / 2.0;
            let at = p.at.unwrap_or([0.0, 0.0]);
            let bl = if anchor == "center" {
                Point {
                    x: at[0] - s / 2.0,
                    y: at[1] - h / 3.0,
                }
            } else {
                Point { x: at[0], y: at[1] }
            };
            let a = Point { x: bl.x, y: bl.y };
            let b = Point {
                x: bl.x + s,
                y: bl.y,
            };
            let c = Point {
                x: bl.x + s / 2.0,
                y: bl.y + h,
            };
            let ctr = Point {
                x: (a.x + b.x + c.x) / 3.0,
                y: (a.y + b.y + c.y) / 3.0,
            };
            apply(vec![a, b, c], ctr)
        }
        "right_triangle" => {
            let at = p.at.unwrap_or([0.0, 0.0]);
            let a = Point { x: at[0], y: at[1] };
            let b = Point {
                x: at[0] + p.a.unwrap_or(0.0),
                y: at[1],
            };
            let c = Point {
                x: at[0],
                y: at[1] + p.b.unwrap_or(0.0),
            };
            let ctr = Point {
                x: (a.x + b.x + c.x) / 3.0,
                y: (a.y + b.y + c.y) / 3.0,
            };
            apply(vec![a, b, c], ctr)
        }
        "regular_polygon" => {
            let n = p.n.unwrap_or(3) as i32;
            let side = p.side.unwrap_or(0.0);
            let r = side / (2.0 * (std::f64::consts::PI / n as f64).sin());
            let at = p.at.unwrap_or([0.0, 0.0]);
            let ctr = Point { x: at[0], y: at[1] };
            let base_ang = piece_rotation(p)
                + if piece_flip(p) {
                    std::f64::consts::PI
                } else {
                    0.0
                };
            let mut pts = Vec::new();
            for i in 0..n {
                let a = base_ang + (i as f64) * 2.0 * std::f64::consts::PI / (n as f64);
                pts.push(Point {
                    x: ctr.x + r * a.cos(),
                    y: ctr.y + r * a.sin(),
                });
            }
            (pts, ctr)
        }
        "circle" => {
            let r = p.d.unwrap_or_else(|| p.r.unwrap_or(0.0) * 2.0) / 2.0;
            let at = p.at.unwrap_or([0.0, 0.0]);
            let ctr = Point { x: at[0], y: at[1] };
            let k = 32;
            let mut pts = Vec::new();
            for i in 0..k {
                let a = (i as f64) * 2.0 * std::f64::consts::PI / (k as f64);
                pts.push(Point {
                    x: ctr.x + r * a.cos(),
                    y: ctr.y + r * a.sin(),
                });
            }
            (pts, ctr)
        }
        "isosceles_trapezoid" => {
            let b0 = p.base_bottom.unwrap_or(0.0);
            let b1 = p.base_top.unwrap_or(0.0);
            let h = p.height.unwrap_or(0.0);
            let at = p.at.unwrap_or([0.0, 0.0]);
            let bl = Point { x: at[0], y: at[1] };
            let br = Point {
                x: bl.x + b0,
                y: bl.y,
            };
            let tl = Point {
                x: bl.x + (b0 - b1) / 2.0,
                y: bl.y + h,
            };
            let tr = Point {
                x: tl.x + b1,
                y: tl.y,
            };
            let ctr = Point {
                x: (bl.x + br.x + tr.x + tl.x) / 4.0,
                y: (bl.y + br.y + tr.y + tl.y) / 4.0,
            };
            apply(vec![bl, br, tr, tl], ctr)
        }
        "parallelogram" => {
            let b = p.base.unwrap_or(0.0);
            let h = p.height.unwrap_or(0.0);
            let off = p.offset_top.unwrap_or(0.0);
            let at = p.at.unwrap_or([0.0, 0.0]);
            let bl = Point { x: at[0], y: at[1] };
            let br = Point {
                x: bl.x + b,
                y: bl.y,
            };
            let tl = Point {
                x: bl.x + off,
                y: bl.y + h,
            };
            let tr = Point {
                x: tl.x + b,
                y: tl.y,
            };
            let ctr = Point {
                x: (bl.x + br.x + tr.x + tl.x) / 4.0,
                y: (bl.y + br.y + tr.y + tl.y) / 4.0,
            };
            apply(vec![bl, br, tr, tl], ctr)
        }
        "polygon" => {
            let pts = p
                .points
                .clone()
                .unwrap_or_default()
                .into_iter()
                .map(|v| Point { x: v[0], y: v[1] })
                .collect::<Vec<_>>();
            let n = pts.len().max(1) as f64;
            let ctr = pts.iter().fold(Point { x: 0.0, y: 0.0 }, |acc, q| Point {
                x: acc.x + q.x,
                y: acc.y + q.y,
            });
            let ctr = Point {
                x: ctr.x / n,
                y: ctr.y / n,
            };
            apply(pts, ctr)
        }
        _ => (Vec::new(), Point { x: 0.0, y: 0.0 }),
    }
}

fn draw(state: &mut State) {
    update_viewport(state);
    let width = state.canvas.width() as f64;
    let height = state.canvas.height() as f64;
    state.ctx.clear_rect(0.0, 0.0, width, height);
    draw_board(state);

    for (i, p) in state.data.pieces.iter_mut().enumerate() {
        let (geom, ctr) = piece_geom(p);
        p.__geom = Some(geom.clone());
        p.__ctr = Some(ctr);
        let color_idx = p.__color_idx.unwrap_or(i);
        let color = puzzle_core::piece_color(color_idx);
        draw_colored_polygon(
            &state.ctx,
            height,
            &geom,
            false,
            state.scale,
            state.offset,
            &color,
        );
        // Draw center number label
        let (cx, cy) = to_screen(ctr, height, state.scale, state.offset);
        let size = (4.5 * state.scale).clamp(10.0, 28.0);
        state.ctx.set_font(&format!("bold {}px sans-serif", size));
        state.ctx.set_text_align("center");
        state.ctx.set_text_baseline("middle");
        let num = p.__label_idx.unwrap_or(i) + 1;
        // Outline for contrast
        state.ctx.set_line_width((size / 5.0).clamp(2.0, 5.0));
        set_stroke_style(&state.ctx, "#fff");
        let _ = state.ctx.stroke_text(&num.to_string(), cx, cy);
        set_fill_style(&state.ctx, "#111");
        let _ = state.ctx.fill_text(&num.to_string(), cx, cy);
    }
    update_validation_dom(state);
}

fn assign_piece_colors(p: &mut Puzzle) {
    // Assign stable numeric labels based on original input order,
    // and set colors to follow the same numbering (mod 8):
    // 红, 橙, 黄, 绿, 青, 蓝, 紫, 粉
    for (i, pc) in p.pieces.iter_mut().enumerate() {
        pc.__label_idx = Some(i);
        pc.__color_idx = Some(i);
    }
}

fn update_note_dom(state: &State) {
    let doc = &state.document;
    if let Some(el) = doc.get_element_by_id("note") {
        let el: HtmlElement = match el.dyn_into() {
            Ok(e) => e,
            Err(_) => return,
        };
        let mut txt = String::new();
        let lang = state.lang.as_str();
        if lang == "zh" {
            if let Some(n) = &state.data.note_zh {
                txt = n.clone();
            } else if let Some(n) = &state.data.note_en {
                txt = n.clone();
            }
        } else if let Some(n) = &state.data.note_en {
            txt = n.clone();
        } else if let Some(n) = &state.data.note_zh {
            txt = n.clone();
        }
        el.set_inner_text(&txt);
    }
}

fn update_status_dom(state: &State) {
    if let Some(el) = state.document.get_element_by_id("status")
        && let Ok(el) = el.dyn_into::<HtmlElement>()
    {
        let lock_en = if state.shift_down {
            "Lock: Temporary"
        } else if state.restrict_mode {
            "Lock: Locked"
        } else {
            "Lock: Unlocked"
        };
        let lock_zh = if state.shift_down {
            "锁定：临时锁定"
        } else if state.restrict_mode {
            "锁定：已锁定"
        } else {
            "锁定：未锁定"
        };
        let speed_en = if state.slow_mode {
            "Speed: Slow"
        } else {
            "Speed: Fast"
        };
        let speed_zh = if state.slow_mode {
            "速度：慢"
        } else {
            "速度：快"
        };
        let txt = if state.lang == "zh" {
            format!("{}  |  {}", lock_zh, speed_zh)
        } else {
            format!("{}  |  {}", lock_en, speed_en)
        };
        el.set_inner_text(&txt);
    }
}

fn update_validation_dom(state: &State) {
    let doc = &state.document;
    let el = match doc.get_element_by_id("validationContent") {
        Some(e) => match e.dyn_into::<HtmlElement>() {
            Ok(v) => v,
            Err(_) => return,
        },
        None => return,
    };

    // Gather board geometry
    let board_geom = state.data.board.as_ref().and_then(board_to_geom);

    // Gather piece geoms with a stable label index (independent of current array order)
    // Use per-piece __label_idx if present; otherwise fall back to loop index.
    let mut geoms: Vec<(usize, Vec<Point>)> = Vec::new();
    for (i, p) in state.data.pieces.iter().enumerate() {
        let label_idx = p.__label_idx.unwrap_or(i);
        if let Some(g) = &p.__geom {
            geoms.push((label_idx, g.clone()));
        } else {
            let (g, _c) = piece_geom(p);
            geoms.push((label_idx, g));
        }
    }

    let mut errors_en: Vec<String> = Vec::new();
    let mut errors_zh: Vec<String> = Vec::new();

    // 1) Piece-piece overlaps
    for a in 0..geoms.len() {
        for b in (a + 1)..geoms.len() {
            if polygons_intersect(&geoms[a].1, &geoms[b].1) {
                let la = geoms[a].0 + 1;
                let lb = geoms[b].0 + 1;
                errors_en.push(format!("Piece {} overlaps piece {}", la, lb));
                errors_zh.push(format!("拼图 {} 与拼图 {} 重叠", la, lb));
            }
        }
    }

    if let Some(bg) = &board_geom {
        let bn = bg.len();
        // helpers
        let edges_cross = |poly: &Vec<Point>| -> bool {
            let an = poly.len();
            for i in 0..an {
                let a1 = poly[i];
                let a2 = poly[(i + 1) % an];
                for j in 0..bn {
                    let b1 = bg[j];
                    let b2 = bg[(j + 1) % bn];
                    if segments_intersect(a1, a2, b1, b2) {
                        return true;
                    }
                }
            }
            false
        };
        let fully_inside =
            |poly: &Vec<Point>| -> bool { poly.iter().all(|p| poly_contains_point(bg, *p)) };

        for (label_idx, pg) in &geoms {
            let num = label_idx + 1;
            if edges_cross(pg) {
                errors_en.push(format!("Piece {} overlaps the border", num));
                errors_zh.push(format!("拼图 {} 与边框重叠", num));
            } else if !fully_inside(pg) {
                errors_en.push(format!("Piece {} is outside the border", num));
                errors_zh.push(format!("拼图 {} 在边框外", num));
            }
        }
    }

    if state.lang == "zh" {
        if errors_zh.is_empty() {
            el.set_inner_html("<div style=\"opacity:.7\">成功</div>");
        } else {
            let mut html = String::new();
            html.push_str("<ul style=\"margin:0;padding-left:18px\">");
            for e in errors_zh {
                html.push_str(&format!("<li>{}</li>", e));
            }
            html.push_str("</ul>");
            el.set_inner_html(&html);
        }
    } else if errors_en.is_empty() {
        el.set_inner_html("<div style=\"opacity:.7\">Success</div>");
    } else {
        let mut html = String::new();
        html.push_str("<ul style=\"margin:0;padding-left:18px\">");
        for e in errors_en {
            html.push_str(&format!("<li>{}</li>", e));
        }
        html.push_str("</ul>");
        el.set_inner_html(&html);
    }
}

fn event_canvas_coords(e: &MouseEvent, cv: &HtmlCanvasElement) -> (f64, f64) {
    // Convert client coordinates into canvas internal pixel coordinates
    // so hit testing works even if CSS scales the canvas element.
    // Fallback to offset if element cast fails.
    if let Some(el) = cv.dyn_ref::<web_sys::Element>() {
        let rect = el.get_bounding_client_rect();
        let x = (e.client_x() as f64 - rect.left()) * (cv.width() as f64) / rect.width().max(1.0);
        let y = (e.client_y() as f64 - rect.top()) * (cv.height() as f64) / rect.height().max(1.0);
        (x, y)
    } else {
        (e.offset_x() as f64, e.offset_y() as f64)
    }
}

fn draw_colored_polygon(
    ctx: &CanvasRenderingContext2d,
    canvas_h: f64,
    pts: &[Point],
    for_hit: bool,
    scale: f64,
    offset: (f64, f64),
    color: &str,
) {
    if pts.is_empty() {
        return;
    }
    ctx.begin_path();
    let (sx, sy) = to_screen(pts[0], canvas_h, scale, offset);
    ctx.move_to(sx, sy);
    for p in &pts[1..] {
        let (x, y) = to_screen(*p, canvas_h, scale, offset);
        ctx.line_to(x, y);
    }
    ctx.close_path();
    ctx.set_line_width(if for_hit { 10.0 } else { 1.6 });
    if !for_hit {
        set_fill_style(ctx, color);
        ctx.fill();
        set_stroke_style(ctx, "#333");
        ctx.stroke();
    } else {
        set_fill_style(ctx, "#000");
        set_stroke_style(ctx, "#000");
        ctx.fill();
        ctx.stroke();
    }
}

fn board_to_geom(board: &Board) -> Option<Vec<Point>> {
    match board.type_.as_deref() {
        Some("rect_with_quarter_round_cut") => {
            let w = board.w.unwrap_or(0.0);
            let h = board.h.unwrap_or(0.0);
            let r = board.r.unwrap_or(0.0);
            let corner = board
                .cut_corner
                .clone()
                .unwrap_or_else(|| "topright".to_string());
            if corner == "topright" {
                let cx = w - r;
                let cy = h - r;
                let n = 24;
                let mut pts = vec![
                    Point { x: 0.0, y: 0.0 },
                    Point { x: w, y: 0.0 },
                    Point { x: w, y: h - r },
                ];
                for i in 0..=n {
                    let a = 0.0 + std::f64::consts::FRAC_PI_2 * (i as f64) / (n as f64);
                    pts.push(Point {
                        x: cx + r * a.cos(),
                        y: cy + r * a.sin(),
                    });
                }
                pts.push(Point { x: 0.0, y: h });
                Some(pts)
            } else {
                Some(vec![
                    Point { x: 0.0, y: 0.0 },
                    Point { x: w, y: 0.0 },
                    Point { x: w, y: h },
                    Point { x: 0.0, y: h },
                ])
            }
        }
        Some("polygon") => {
            let pts = board
                .points
                .clone()
                .unwrap_or_default()
                .into_iter()
                .map(|v| Point { x: v[0], y: v[1] })
                .collect::<Vec<_>>();
            if pts.is_empty() { None } else { Some(pts) }
        }
        _ => None,
    }
}

fn draw_board(state: &mut State) {
    if let Some(b) = &state.data.board
        && let Some(geom) = board_to_geom(b)
    {
        state.ctx.set_line_width(2.4);
        set_stroke_style(&state.ctx, "#222");
        draw_colored_polygon(
            &state.ctx,
            state.canvas.height() as f64,
            &geom,
            false,
            state.scale,
            state.offset,
            "#ffffff",
        );
    }
}

fn point_in_polygon(
    pt: (f64, f64),
    poly: &[Point],
    canvas_h: f64,
    scale: f64,
    offset: (f64, f64),
) -> bool {
    // Use geometry space for tests, convert screen point to geometry first
    let gp = from_screen(pt.0, pt.1, canvas_h, scale, offset);
    let (x, y) = (gp.x, gp.y);
    let mut inside = false;
    let n = poly.len();
    let mut j = n - 1;
    for i in 0..n {
        let xi = poly[i].x;
        let yi = poly[i].y;
        let xj = poly[j].x;
        let yj = poly[j].y;
        let intersect =
            ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi + 1e-12) + xi);
        if intersect {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn poly_contains_point(poly: &[Point], p: Point) -> bool {
    let (x, y) = (p.x, p.y);
    let mut inside = false;
    let n = poly.len();
    let mut j = n - 1;
    for i in 0..n {
        let xi = poly[i].x;
        let yi = poly[i].y;
        let xj = poly[j].x;
        let yj = poly[j].y;
        let intersect =
            ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi + 1e-12) + xi);
        if intersect {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn segments_intersect(a1: Point, a2: Point, b1: Point, b2: Point) -> bool {
    fn cross(a: Point, b: Point, c: Point) -> f64 {
        (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
    }
    let d1 = cross(a1, a2, b1);
    let d2 = cross(a1, a2, b2);
    let d3 = cross(b1, b2, a1);
    let d4 = cross(b1, b2, a2);
    if ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
        && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
    {
        return true;
    }
    false
}

fn polygons_intersect(a: &[Point], b: &[Point]) -> bool {
    if a.is_empty() || b.is_empty() {
        return false;
    }
    let an = a.len();
    let bn = b.len();
    for i in 0..an {
        let a1 = a[i];
        let a2 = a[(i + 1) % an];
        for j in 0..bn {
            let b1 = b[j];
            let b2 = b[(j + 1) % bn];
            if segments_intersect(a1, a2, b1, b2) {
                return true;
            }
        }
    }
    if poly_contains_point(a, b[0]) || poly_contains_point(b, a[0]) {
        return true;
    }
    false
}

// color helper moved to puzzle-core

// (removed unused SVG helpers that triggered dead-code lints)

fn save_text_as_file(document: &Document, filename: &str, text: &str) -> Result<(), JsValue> {
    let array = Array::new();
    array.push(&JsValue::from_str(text));
    let blob = Blob::new_with_str_sequence(&array)?;
    let url = Url::create_object_url_with_blob(&blob)?;
    let a = document.create_element("a")?.dyn_into::<HtmlElement>()?;
    a.set_attribute("href", &url)?;
    a.set_attribute("download", filename)?;
    a.click();
    Url::revoke_object_url(&url)?;
    Ok(())
}

fn attach_ui(state: Rc<RefCell<State>>) -> Result<(), JsValue> {
    let doc = state.borrow().document.clone();
    // File input
    upload::attach_file_input(state.clone())?;

    // Reset button (restore to initial state)
    if let Some(btn) = doc.get_element_by_id("resetPuzzle") {
        let btn: HtmlElement = btn.dyn_into().unwrap();
        let st = state.clone();
        let onclick = Closure::<dyn FnMut()>::wrap(Box::new(move || {
            let mut s = st.borrow_mut();
            s.data = s.initial_data.clone();
            s.dragging_idx = None;
            s.rot_vel = 0.0;
            s.slow_mode = false;
            s.restrict_mode = false;
            s.shift_down = false;
            s.scale = DEFAULT_MM2PX;
            s.offset = (0.0, 0.0);
            update_status_dom(&s);
            draw(&mut s);
        }));
        btn.set_onclick(Some(onclick.as_ref().unchecked_ref()));
        onclick.forget();
    }

    // Export PNG (blueprint; deterministic)
    if let Some(btn) = doc.get_element_by_id("exportPng") {
        let btn: HtmlElement = btn.dyn_into().unwrap();
        let st = state.clone();
        let onclick = Closure::<dyn FnMut()>::wrap(Box::new(move || {
            let _ = export_png_blueprint(&st.borrow());
        }));
        btn.set_onclick(Some(onclick.as_ref().unchecked_ref()));
        onclick.forget();
    }

    // Language selector
    if let Some(sel) = doc.get_element_by_id("langSel") {
        let sel: HtmlElement = sel.dyn_into().unwrap();
        let st = state.clone();
        let onchange = Closure::<dyn FnMut()>::wrap(Box::new(move || {
            let mut s = st.borrow_mut();
            if let Some(input) = s.document.get_element_by_id("langSel")
                && let Ok(sel) = input.dyn_into::<web_sys::HtmlSelectElement>()
            {
                let v = sel.value();
                s.lang = if v.to_lowercase().starts_with("zh") {
                    "zh".to_string()
                } else {
                    "en".to_string()
                };
                update_note_dom(&s);
                update_status_dom(&s);
                update_validation_dom(&s);
            }
        }));
        sel.set_onchange(Some(onchange.as_ref().unchecked_ref()));
        onchange.forget();
    }

    // Speed controls (fast/slow) with slider + number, kept in sync
    {
        let st = state.clone();
        // Initialize and wire fast speed controls
        if let (Some(sl), Some(nb)) = (
            doc.get_element_by_id("fastSpeedSlider"),
            doc.get_element_by_id("fastSpeedNumber"),
        ) && let (Ok(sl), Ok(nb)) = (
            sl.dyn_into::<web_sys::HtmlInputElement>(),
            nb.dyn_into::<web_sys::HtmlInputElement>(),
        ) {
            // Set initial values from state
            let val = st.borrow().rot_speed_fast.round().clamp(1.0, 180.0) as i32;
            sl.set_value(&val.to_string());
            nb.set_value(&val.to_string());

            // Slider -> Number + State
            let st1 = st.clone();
            let nb1 = nb.clone();
            let sl_read = sl.clone();
            let oninput = Closure::<dyn FnMut()>::wrap(Box::new(move || {
                let mut s = st1.borrow_mut();
                if let Ok(v) = sl_read.value().parse::<i32>() {
                    let v = v.clamp(1, 180) as f64;
                    nb1.set_value(&((v as i32).to_string()));
                    s.rot_speed_fast = v;
                    if s.rot_vel != 0.0 && !s.slow_mode {
                        let dir = if s.rot_vel > 0.0 { 1.0 } else { -1.0 };
                        s.rot_vel = dir * s.rot_speed_fast;
                    }
                }
            }));
            sl.set_oninput(Some(oninput.as_ref().unchecked_ref()));
            oninput.forget();

            // Number -> Slider + State
            let st2 = st.clone();
            let sl2 = sl.clone();
            let nb_read = nb.clone();
            let oninput2 = Closure::<dyn FnMut()>::wrap(Box::new(move || {
                let mut s = st2.borrow_mut();
                if let Ok(mut v) = nb_read.value().parse::<i32>() {
                    v = v.clamp(1, 180);
                    nb_read.set_value(&v.to_string());
                    sl2.set_value(&v.to_string());
                    s.rot_speed_fast = v as f64;
                    if s.rot_vel != 0.0 && !s.slow_mode {
                        let dir = if s.rot_vel > 0.0 { 1.0 } else { -1.0 };
                        s.rot_vel = dir * s.rot_speed_fast;
                    }
                }
            }));
            nb.set_oninput(Some(oninput2.as_ref().unchecked_ref()));
            oninput2.forget();
        }

        // Initialize and wire slow speed controls
        if let (Some(sl), Some(nb)) = (
            doc.get_element_by_id("slowSpeedSlider"),
            doc.get_element_by_id("slowSpeedNumber"),
        ) && let (Ok(sl), Ok(nb)) = (
            sl.dyn_into::<web_sys::HtmlInputElement>(),
            nb.dyn_into::<web_sys::HtmlInputElement>(),
        ) {
            // Set initial values from state
            let val = st.borrow().rot_speed_slow.round().clamp(1.0, 180.0) as i32;
            sl.set_value(&val.to_string());
            nb.set_value(&val.to_string());

            // Slider -> Number + State
            let st1 = st.clone();
            let nb1 = nb.clone();
            let sl_read = sl.clone();
            let oninput = Closure::<dyn FnMut()>::wrap(Box::new(move || {
                let mut s = st1.borrow_mut();
                if let Ok(v) = sl_read.value().parse::<i32>() {
                    let v = v.clamp(1, 180) as f64;
                    nb1.set_value(&((v as i32).to_string()));
                    s.rot_speed_slow = v;
                    if s.rot_vel != 0.0 && s.slow_mode {
                        let dir = if s.rot_vel > 0.0 { 1.0 } else { -1.0 };
                        s.rot_vel = dir * s.rot_speed_slow;
                    }
                }
            }));
            sl.set_oninput(Some(oninput.as_ref().unchecked_ref()));
            oninput.forget();

            // Number -> Slider + State
            let st2 = st.clone();
            let sl2 = sl.clone();
            let nb_read = nb.clone();
            let oninput2 = Closure::<dyn FnMut()>::wrap(Box::new(move || {
                let mut s = st2.borrow_mut();
                if let Ok(mut v) = nb_read.value().parse::<i32>() {
                    v = v.clamp(1, 180);
                    nb_read.set_value(&v.to_string());
                    sl2.set_value(&v.to_string());
                    s.rot_speed_slow = v as f64;
                    if s.rot_vel != 0.0 && s.slow_mode {
                        let dir = if s.rot_vel > 0.0 { 1.0 } else { -1.0 };
                        s.rot_vel = dir * s.rot_speed_slow;
                    }
                }
            }));
            nb.set_oninput(Some(oninput2.as_ref().unchecked_ref()));
            oninput2.forget();
        }
    }

    // Save JSON
    if let Some(btn) = doc.get_element_by_id("saveJson") {
        let btn: HtmlElement = btn.dyn_into().unwrap();
        let st = state.clone();
        let onclick = Closure::<dyn FnMut()>::wrap(Box::new(move || {
            let s = serde_json::to_string_pretty(&st.borrow().data).unwrap_or("{}".to_string());
            let _ = save_text_as_file(&st.borrow().document, "puzzle.json", &s);
        }));
        btn.set_onclick(Some(onclick.as_ref().unchecked_ref()));
        onclick.forget();
    }

    // Mouse events
    {
        let st = state.clone();
        let mousedown = Closure::<dyn FnMut(MouseEvent)>::wrap(Box::new(move |e: MouseEvent| {
            let mut s = st.borrow_mut();
            let pt = event_canvas_coords(&e, &s.canvas);
            let h = s.canvas.height() as f64;
            // find topmost piece under cursor
            for i in (0..s.data.pieces.len()).rev() {
                if let Some(ref geom) = s.data.pieces[i].__geom
                    && point_in_polygon(pt, geom, h, s.scale, s.offset)
                {
                    s.dragging_idx = Some(i);
                    let ctr = s.data.pieces[i].__ctr.unwrap_or(Point { x: 0.0, y: 0.0 });
                    let (sx, sy) = to_screen(ctr, h, s.scale, s.offset);
                    s.drag_off = (pt.0 - sx, pt.1 - sy);
                    // bring to top
                    let it = s.data.pieces.remove(i);
                    s.data.pieces.push(it);
                    s.dragging_idx = Some(s.data.pieces.len() - 1);
                    break;
                }
            }
        }));
        state
            .borrow()
            .canvas
            .add_event_listener_with_callback("mousedown", mousedown.as_ref().unchecked_ref())?;
        mousedown.forget();
    }
    {
        let st = state.clone();
        let mousemove = Closure::<dyn FnMut(MouseEvent)>::wrap(Box::new(move |e: MouseEvent| {
            let mut s = st.borrow_mut();
            if let Some(idx) = s.dragging_idx {
                let h = s.canvas.height() as f64;
                let raw = event_canvas_coords(&e, &s.canvas);
                let pt = (raw.0 - s.drag_off.0, raw.1 - s.drag_off.1);
                let gp = from_screen(pt.0, pt.1, h, s.scale, s.offset);
                // move by center
                if let Some(ctr) = s.data.pieces[idx].__ctr {
                    let dx = gp.x - ctr.x;
                    let dy = gp.y - ctr.y;
                    // propose new position and validate collisions and board
                    let mut pclone = s.data.pieces[idx].clone();
                    if let Some(mut at) = pclone.at {
                        at[0] += dx;
                        at[1] += dy;
                        pclone.at = Some(at);
                    } else if pclone.points.is_some() {
                        let pts = pclone.points.clone().unwrap();
                        let moved = pts
                            .into_iter()
                            .map(|v| [v[0] + dx, v[1] + dy])
                            .collect::<Vec<_>>();
                        pclone.points = Some(moved);
                    } else {
                        pclone.at = Some([dx, dy]);
                    }
                    let (cand_geom, _c2) = piece_geom(&pclone);
                    let constraints_active = s.restrict_mode || s.shift_down;
                    let mut board_ok = true;
                    if constraints_active
                        && let Some(b) = &s.data.board
                        && let Some(board_geom) = board_to_geom(b)
                    {
                        // allow fully inside or fully outside; disallow only when crossing edges
                        let an = cand_geom.len();
                        let bn = board_geom.len();
                        let mut edges_cross = false;
                        'outer: for i in 0..an {
                            let a1 = cand_geom[i];
                            let a2 = cand_geom[(i + 1) % an];
                            for j in 0..bn {
                                let b1 = board_geom[j];
                                let b2 = board_geom[(j + 1) % bn];
                                if segments_intersect(a1, a2, b1, b2) {
                                    edges_cross = true;
                                    break 'outer;
                                }
                            }
                        }
                        board_ok = !edges_cross;
                    }
                    let mut pieces_ok = true;
                    if constraints_active && board_ok {
                        for j in 0..s.data.pieces.len() {
                            if j == idx {
                                continue;
                            }
                            if let Some(ref og) = s.data.pieces[j].__geom
                                && polygons_intersect(&cand_geom, og)
                            {
                                pieces_ok = false;
                                break;
                            }
                        }
                    }
                    if !constraints_active || (board_ok && pieces_ok) {
                        let p = &mut s.data.pieces[idx];
                        if let Some(mut at) = p.at {
                            at[0] += dx;
                            at[1] += dy;
                            p.at = Some(at);
                        } else if p.points.is_some() {
                            let pts = p.points.clone().unwrap();
                            let moved = pts
                                .into_iter()
                                .map(|v| [v[0] + dx, v[1] + dy])
                                .collect::<Vec<_>>();
                            p.points = Some(moved);
                        } else {
                            p.at = Some([dx, dy]);
                        }
                    }
                }
                draw(&mut s);
            }
        }));
        state
            .borrow()
            .canvas
            .add_event_listener_with_callback("mousemove", mousemove.as_ref().unchecked_ref())?;
        mousemove.forget();
    }
    {
        let st = state.clone();
        let mouseup = Closure::<dyn FnMut(MouseEvent)>::wrap(Box::new(move |_e: MouseEvent| {
            st.borrow_mut().dragging_idx = None;
        }));
        state
            .borrow()
            .window
            .add_event_listener_with_callback("mouseup", mouseup.as_ref().unchecked_ref())?;
        mouseup.forget();
    }

    // Keyboard
    {
        let st = state.clone();
        let keydown =
            Closure::<dyn FnMut(KeyboardEvent)>::wrap(Box::new(move |e: KeyboardEvent| {
                let key = e.key().to_lowercase();
                let mut s = st.borrow_mut();
                if s.data.pieces.is_empty() {
                    return;
                }
                let idx = s.data.pieces.len() - 1;
                let p = &mut s.data.pieces[idx];
                match key.as_str() {
                    // q counter-clockwise (3→12→9→6), e clockwise; speed depends on mode
                    "q" => {
                        let speed = if s.slow_mode {
                            s.rot_speed_slow
                        } else {
                            s.rot_speed_fast
                        };
                        s.rot_vel = speed;
                    }
                    "e" => {
                        let speed = if s.slow_mode {
                            s.rot_speed_slow
                        } else {
                            s.rot_speed_fast
                        };
                        s.rot_vel = -speed;
                    }
                    // toggle slow/fast mode
                    "s" => {
                        s.slow_mode = !s.slow_mode;
                        let new_speed = if s.slow_mode {
                            s.rot_speed_slow
                        } else {
                            s.rot_speed_fast
                        };
                        if s.rot_vel != 0.0 {
                            let dir = if s.rot_vel > 0.0 { 1.0 } else { -1.0 };
                            s.rot_vel = dir * new_speed;
                        }
                        log(if s.slow_mode {
                            "Switched to slow mode"
                        } else {
                            "Switched to fast mode"
                        });
                        update_status_dom(&s);
                    }
                    "f" => {
                        p.flip = Some(!p.flip.unwrap_or(false));
                        draw(&mut s);
                    }
                    // toggle restrict movement mode
                    "l" => {
                        s.restrict_mode = !s.restrict_mode;
                        log(if s.restrict_mode {
                            "Restriction: ON (no overlaps with pieces/border)"
                        } else {
                            "Restriction: OFF"
                        });
                        update_status_dom(&s);
                    }
                    // track Shift press for temporary constraint
                    "shift" => {
                        s.shift_down = true;
                        update_status_dom(&s);
                    }
                    _ => {}
                }
            }));
        state
            .borrow()
            .window
            .add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref())?;
        keydown.forget();
    }
    // keyup to stop continuous rotation
    {
        let st = state.clone();
        let keyup = Closure::<dyn FnMut(KeyboardEvent)>::wrap(Box::new(move |e: KeyboardEvent| {
            let key = e.key().to_lowercase();
            let mut s = st.borrow_mut();
            if key == "q" || key == "e" {
                s.rot_vel = 0.0;
            }
            if key == "shift" {
                s.shift_down = false;
                update_status_dom(&s);
            }
        }));
        state
            .borrow()
            .window
            .add_event_listener_with_callback("keyup", keyup.as_ref().unchecked_ref())?;
        keyup.forget();
    }

    // Mouse wheel rotation removed by request

    Ok(())
}

fn export_png_blueprint(state: &State) -> Result<(), JsValue> {
    let px_per_mm = 4.0; // export resolution
    // Set language for labels
    blueprint_core::set_language(&state.lang);

    // Build a PuzzleSpec (pieces-only), ignoring current poses to match CLI blueprint semantics
    let board = state.data.board.clone().map(|b| blueprint_core::Board {
        type_: b.type_,
        w: b.w,
        h: b.h,
        r: b.r,
        cut_corner: b.cut_corner,
        points: b.points,
        label: None,
        label_lines: None,
    });
    let pieces = state
        .data
        .pieces
        .iter()
        .map(|p| blueprint_core::Piece {
            id: p.id.clone(),
            type_: p.type_.clone(),
            at: Some([0.0, 0.0]),
            rotation: Some(0.0),
            anchor: Some("bottomleft".to_string()),
            flip: Some(false),
            w: p.w,
            h: p.h,
            side: p.side,
            a: p.a,
            b: p.b,
            n: p.n,
            d: p.d,
            r: p.r,
            base_bottom: p.base_bottom,
            base_top: p.base_top,
            height: p.height,
            base: p.base,
            offset_top: p.offset_top,
            points: p.points.clone(),
        })
        .collect::<Vec<_>>();
    let spec = blueprint_core::PuzzleSpec {
        units: state.data.units.clone(),
        board,
        pieces: Some(pieces),
        parts: None,
        counts: None,
        shapes_file: None,
    };

    let (svg, w_px, h_px) = blueprint_core::build_blueprint_svg(&spec, px_per_mm, None);

    // Render SVG to RGBA using embedded font
    let mut opt = usvg::Options::default();
    let mut fontdb = usvg::fontdb::Database::new();
    fontdb.load_font_data(fonts::FONT_BYTES.to_vec());
    let family_name = {
        let mut it = fontdb.faces();
        if let Some(face) = it.next() {
            face.families.first().map(|(n, _)| n.clone())
        } else {
            None
        }
    };
    if let Some(name) = family_name {
        fontdb.set_sans_serif_family(name);
    }
    opt.fontdb = std::sync::Arc::new(fontdb);
    let tree = usvg::Tree::from_str(&svg, &opt)
        .map_err(|e| JsValue::from_str(&format!("SVG parse error: {e:?}")))?;
    let mut pixmap =
        tiny_skia::Pixmap::new(w_px, h_px).ok_or(JsValue::from_str("pixmap alloc failed"))?;
    let mut pm = pixmap.as_mut();
    resvg::render(&tree, tiny_skia::Transform::identity(), &mut pm);

    // Deterministic PNG encoding into memory
    let bytes = encode_png_deterministic_to_vec(&pixmap)
        .map_err(|e| JsValue::from_str(&format!("encode: {e}")))?;

    // Create Blob and trigger download
    let document = state.document.clone();
    let array = js_sys::Array::new();
    let u8 = js_sys::Uint8Array::from(bytes.as_slice());
    array.push(&u8);
    let blob = Blob::new_with_u8_array_sequence(&array)?;
    let url = Url::create_object_url_with_blob(&blob)?;
    let a = document.create_element("a")?.dyn_into::<HtmlElement>()?;
    a.set_attribute("href", &url)?;
    a.set_attribute("download", "puzzle_blueprint.png")?;
    a.click();
    Url::revoke_object_url(&url)?;
    Ok(())
}

fn encode_png_deterministic_to_vec(
    pixmap: &tiny_skia::Pixmap,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut buf = Vec::new();
    let w = pixmap.width();
    let h = pixmap.height();
    let mut enc = Encoder::new(&mut buf, w, h);
    enc.set_color(ColorType::Rgba);
    enc.set_depth(BitDepth::Eight);
    enc.set_filter(FilterType::NoFilter);
    enc.set_compression(Compression::Default);
    {
        let mut writer = enc.write_header()?;
        writer.write_image_data(pixmap.data())?;
    }
    Ok(buf)
}

fn init_canvas(
    document: &Document,
) -> Result<(HtmlCanvasElement, CanvasRenderingContext2d), JsValue> {
    let cv = document
        .get_element_by_id("cv")
        .ok_or_else(|| JsValue::from_str("canvas #cv not found"))?
        .dyn_into::<HtmlCanvasElement>()?;
    let ctx = cv
        .get_context("2d")?
        .ok_or_else(|| JsValue::from_str("2D context not available"))?
        .dyn_into::<CanvasRenderingContext2d>()?;
    Ok((cv, ctx))
}

fn start_animation(state: Rc<RefCell<State>>) {
    type RafClosure = Closure<dyn FnMut(f64)>;
    let f: Rc<RefCell<Option<RafClosure>>> = Rc::new(RefCell::new(None));
    let g = f.clone();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move |_ts: f64| {
        {
            let mut s = state.borrow_mut();
            let vel = s.rot_vel;
            if vel.abs() > 0.0 {
                if !s.data.pieces.is_empty() {
                    let idx = s.dragging_idx.unwrap_or_else(|| s.data.pieces.len() - 1);
                    let p = &mut s.data.pieces[idx];
                    p.rotation = Some(p.rotation.unwrap_or(0.0) + vel / 60.0);
                }
                draw(&mut s);
            }
        }
        let _ = web_sys::window()
            .unwrap()
            .request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref());
    }) as Box<dyn FnMut(f64)>));
    let _ = web_sys::window()
        .unwrap()
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref());
}

fn default_puzzle() -> Puzzle {
    // Build from counts + shapes so we don't rely on a positioned piece file
    let counts_txt = include_str!("../../puzzle/k11.json");
    let shapes_txt = include_str!("../../shapes.json");
    if let (Ok(spec), Ok(catalog)) = (
        serde_json::from_str::<CountsSpec>(counts_txt),
        serde_json::from_str::<ShapesCatalog>(shapes_txt),
    ) {
        build_puzzle_from_counts(&spec, &catalog)
    } else {
        Puzzle {
            units: Some("mm".to_string()),
            board: None,
            pieces: Vec::new(),
            note_en: None,
            note_zh: None,
        }
    }
}

fn build_puzzle_from_counts(spec: &CountsSpec, catalog: &ShapesCatalog) -> Puzzle {
    use std::collections::HashMap;
    let mut by_id: HashMap<&str, &ShapeDef> = HashMap::new();
    for s in &catalog.shapes {
        by_id.insert(s.id.as_str(), s);
    }
    let mut pieces: Vec<Piece> = Vec::new();
    for (id, ct) in &spec.counts {
        if let Some(sd) = by_id.get(id.as_str()) {
            for _ in 0..*ct {
                let p = Piece {
                    type_: sd.type_.clone(),
                    w: sd.w,
                    h: sd.h,
                    side: sd.side,
                    a: sd.a,
                    b: sd.b,
                    n: sd.n,
                    d: sd.d,
                    r: sd.r,
                    base_bottom: sd.base_bottom,
                    base_top: sd.base_top,
                    height: sd.height,
                    base: sd.base,
                    offset_top: sd.offset_top,
                    points: sd.points.clone(),
                    ..Default::default()
                };
                // For initial layout: arrange in rows inside board or in a grid starting at (10,10)
                pieces.push(p);
            }
        }
    }

    // Simple initial placement: grid with 10mm margin and 5mm gap
    let margin = 10.0;
    let gap = 5.0;
    let (bw, _bh) = spec
        .board
        .as_ref()
        .map(|b| (b.w.unwrap_or(200.0), b.h.unwrap_or(200.0)))
        .unwrap_or((200.0, 200.0));
    let mut x = margin;
    let mut y = margin;
    let maxw = bw - margin;
    let mut row_h = 0.0;
    for p in &mut pieces {
        let (geom, _ctr) = piece_geom(p);
        let bb = bounds_of_points(&geom);
        let w = bb.2 - bb.0;
        let h = bb.3 - bb.1;
        if x + w > maxw {
            x = margin;
            y += row_h + gap;
            row_h = 0.0;
        }
        // Anchor bottomleft by default; circles and regular polygons look better centered
        match p.type_.as_str() {
            "circle" | "regular_polygon" => {
                p.anchor = Some("center".to_string());
                p.at = Some([x + w / 2.0, y + h / 2.0]);
            }
            _ => {
                p.anchor = Some("bottomleft".to_string());
                p.at = Some([x, y]);
            }
        }
        x += w + gap;
        if h > row_h {
            row_h = h;
        }
    }

    Puzzle {
        units: spec.units.clone().or(Some("mm".to_string())),
        board: spec.board.clone(),
        pieces,
        note_en: spec.note_en.clone(),
        note_zh: spec.note_zh.clone(),
    }
}

fn bounds_of_points(pts: &[Point]) -> (f64, f64, f64, f64) {
    let mut minx = f64::INFINITY;
    let mut miny = f64::INFINITY;
    let mut maxx = f64::NEG_INFINITY;
    let mut maxy = f64::NEG_INFINITY;
    for p in pts {
        minx = minx.min(p.x);
        miny = miny.min(p.y);
        maxx = maxx.max(p.x);
        maxy = maxy.max(p.y);
    }
    (minx, miny, maxx, maxy)
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // console_error_panic_hook is optional; avoid extra dep here.
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let (canvas, ctx) = init_canvas(&document)?;

    let data = default_puzzle();
    // If URL param p is set, we try to fetch puzzles/<p>.json; otherwise use default
    if let Ok(search) = window.location().search()
        && let Some(p) = get_query_param(&search, "p")
    {
        // Try to fetch; fire-and-forget; fallback to default already loaded
        let win = window.clone();
        let doc = document.clone();
        let cv = canvas.clone();
        let ctx2 = ctx.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(err) = fetch_and_load_puzzle(win, doc, cv, ctx2, &p).await {
                log(&format!("Failed to load puzzle '{}': {:?}", p, err));
            }
        });
    }

    let state = Rc::new(RefCell::new(State {
        window,
        document,
        canvas,
        ctx,
        data,
        dragging_idx: None,
        drag_off: (0.0, 0.0),
        scale: DEFAULT_MM2PX,
        offset: (0.0, 0.0),
        rot_vel: 0.0,
        slow_mode: false,
        rot_speed_fast: 180.0,
        rot_speed_slow: 30.0,
        restrict_mode: false,
        shift_down: false,
        initial_data: Puzzle {
            units: None,
            board: None,
            pieces: Vec::new(),
            note_en: None,
            note_zh: None,
        },
        lang: "en".to_string(),
    }));

    STATE.with(|st| st.replace(Some(state.clone())));
    // Assign stable colors before first draw
    STATE.with(|st| {
        if let Some(st_rc) = st.borrow().as_ref() {
            let mut s = st_rc.borrow_mut();
            assign_piece_colors(&mut s.data);
            s.initial_data = s.data.clone();
            update_note_dom(&s);
            update_status_dom(&s);
        }
    });
    attach_ui(state.clone())?;
    start_animation(state.clone());
    draw(&mut state.borrow_mut());
    Ok(())
}

async fn fetch_and_load_puzzle(
    window: Window,
    document: Document,
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    name: &str,
) -> Result<(), JsValue> {
    let text = fetch_text_with_fallbacks(
        &window,
        &[
            &asset_url(&format!("puzzle/{}.json", name)),
            &format!("/puzzle/{}.json", name),
            &format!("puzzle/{}.json", name),
        ],
    )
    .await
    .unwrap_or_default();
    // Try parse as full Puzzle; fall back to counts+shapes
    let puzzle: Puzzle = if let Ok(p) = serde_json::from_str::<Puzzle>(&text) {
        p
    } else if let Ok(spec) = serde_json::from_str::<CountsSpec>(&text) {
        // Fetch shapes file if provided; else fallback to bundled shapes
        let shapes_text = if let Some(sf) = spec.shapes_file.clone() {
            fetch_text_with_fallbacks(&window, &[&asset_url(&sf), &sf])
                .await
                .unwrap_or_default()
        } else {
            include_str!("../../shapes.json").to_string()
        };
        let catalog = serde_json::from_str::<ShapesCatalog>(&shapes_text)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        build_puzzle_from_counts(&spec, &catalog)
    } else {
        return Err(JsValue::from_str("Unrecognized puzzle JSON format"));
    };

    STATE.with(|st| {
        if let Some(st_rc) = st.borrow().as_ref() {
            let mut s = st_rc.borrow_mut();
            s.data = puzzle;
            assign_piece_colors(&mut s.data);
            s.initial_data = s.data.clone();
            update_note_dom(&s);
            update_status_dom(&s);
            s.window = window.clone();
            s.document = document.clone();
            s.canvas = canvas.clone();
            s.ctx = ctx.clone();
            draw(&mut s);
        }
    });
    Ok(())
}

fn asset_url(path: &str) -> String {
    // Use a base prefix provided by the host page via window.__BASE_URL if present,
    // else default to "/". If the input is an absolute URL or already starts with
    // the base, return as-is.
    let p = path.trim();
    if p.starts_with("http://") || p.starts_with("https://") || p.starts_with("data:") {
        return p.to_string();
    }
    // Read base from window
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

async fn fetch_text_with_fallbacks(window: &Window, urls: &[&str]) -> Option<String> {
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

fn get_query_param(search: &str, key: &str) -> Option<String> {
    // naive parser for ?a=b&c=d
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
    // fallback if decode_uri_component not used; replace + with space and percent-decode best-effort
    let s = s.replace('+', " ");
    percent_encoding::percent_decode_str(&s)
        .decode_utf8_lossy()
        .to_string()
}

fn update_viewport(state: &mut State) {
    let canvas_w = state.canvas.width() as f64;
    let canvas_h = state.canvas.height() as f64;
    // Determine content bounds in mm
    let mut minx = f64::INFINITY;
    let mut miny = f64::INFINITY;
    let mut maxx = f64::NEG_INFINITY;
    let mut maxy = f64::NEG_INFINITY;

    if let Some(b) = &state.data.board
        && let Some(geom) = board_to_geom(b)
    {
        for p in geom {
            minx = minx.min(p.x);
            maxx = maxx.max(p.x);
            miny = miny.min(p.y);
            maxy = maxy.max(p.y);
        }
    }
    // Always include pieces in the bounds so off-board pieces remain visible
    for p in &state.data.pieces {
        let (geom, _ctr) = piece_geom(p);
        for q in geom {
            minx = minx.min(q.x);
            maxx = maxx.max(q.x);
            miny = miny.min(q.y);
            maxy = maxy.max(q.y);
        }
    }
    let have_bounds = maxx.is_finite() && maxy.is_finite();

    let (w_mm, h_mm) = if have_bounds {
        ((maxx - minx).max(1.0), (maxy - miny).max(1.0))
    } else {
        (canvas_w / DEFAULT_MM2PX, canvas_h / DEFAULT_MM2PX)
    };

    let margin = 20.0; // px
    let scale_x = (canvas_w - 2.0 * margin) / w_mm;
    let scale_y = (canvas_h - 2.0 * margin) / h_mm;
    let scale = scale_x.min(scale_y).max(0.1);
    let content_w_px = w_mm * scale;
    let content_h_px = h_mm * scale;
    let ox = (canvas_w - content_w_px) / 2.0 - minx * scale;
    let oy = (canvas_h - content_h_px) / 2.0 - miny * scale;

    state.scale = scale;
    state.offset = (ox, oy);
}
