use std::cell::RefCell;
use std::rc::Rc;

use js_sys::Array;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    Blob, CanvasRenderingContext2d, Document, Event, FileReader, HtmlCanvasElement, HtmlElement,
    HtmlInputElement, KeyboardEvent, MouseEvent, Url, Window,
};

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
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Puzzle {
    units: Option<String>,
    board: Option<Board>,
    pieces: Vec<Piece>,
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
    // continuous rotation control (deg per second, +cw)
    rot_vel: f64,
}

thread_local! {
    static STATE: RefCell<Option<Rc<RefCell<State>>>> = RefCell::new(None);
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

fn rotate_point(p: Point, c: Point, ang: f64, flip: bool) -> Point {
    let mut dx = p.x - c.x;
    let mut dy = p.y - c.y;
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
        let color = piece_color(i);
        draw_colored_polygon(
            &state.ctx,
            height,
            &geom,
            false,
            state.scale,
            state.offset,
            &color,
        );
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
    let _ = ctx.move_to(sx, sy);
    for p in &pts[1..] {
        let (x, y) = to_screen(*p, canvas_h, scale, offset);
        let _ = ctx.line_to(x, y);
    }
    let _ = ctx.close_path();
    ctx.set_line_width(if for_hit { 10.0 } else { 1.6 });
    if !for_hit {
        ctx.set_fill_style(&JsValue::from_str(color));
        let _ = ctx.fill();
        ctx.set_stroke_style(&JsValue::from_str("#333"));
        let _ = ctx.stroke();
    } else {
        ctx.set_fill_style(&JsValue::from_str("#000"));
        ctx.set_stroke_style(&JsValue::from_str("#000"));
        let _ = ctx.fill();
        let _ = ctx.stroke();
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
            if pts.is_empty() {
                None
            } else {
                Some(pts)
            }
        }
        _ => None,
    }
}

fn draw_board(state: &mut State) {
    if let Some(b) = &state.data.board {
        if let Some(geom) = board_to_geom(b) {
            state.ctx.set_line_width(2.4);
            state.ctx.set_stroke_style(&JsValue::from_str("#222"));
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

fn piece_color(i: usize) -> String {
    let h = ((i as f64) * 47.0) % 360.0;
    format!("hsl({:.0}, 65%, 75%)", h)
}

fn svg_path_from_points(pts: &[Point], canvas_h: f64, scale: f64, offset: (f64, f64)) -> String {
    if pts.is_empty() {
        return String::new();
    }
    let mut s = String::new();
    let (x0, y0) = to_screen(pts[0], canvas_h, scale, offset);
    s.push_str(&format!("M {} {}", x0, y0));
    for p in &pts[1..] {
        let (x, y) = to_screen(*p, canvas_h, scale, offset);
        s.push_str(&format!(" L {} {}", x, y));
    }
    s.push_str(" Z");
    s
}

fn to_svg(state: &State) -> String {
    let wmm = (state.canvas.width() as f64 / state.scale);
    let hmm = (state.canvas.height() as f64 / state.scale);
    let mut s = String::new();
    s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    s.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{:.2}mm\" height=\"{:.2}mm\" viewBox=\"0 0 {} {}\">\n",
        wmm,
        hmm,
        state.canvas.width(),
        state.canvas.height()
    ));
    if let Some(b) = &state.data.board {
        if let Some(geom) = board_to_geom(b) {
            let d = svg_path_from_points(
                &geom,
                state.canvas.height() as f64,
                state.scale,
                state.offset,
            );
            s.push_str(&format!(
                "<path d=\"{}\" fill=\"none\" stroke=\"#222\" stroke-width=\"2\"/>\n",
                d
            ));
        }
    }
    for p in &state.data.pieces {
        if let Some(g) = &p.__geom {
            let d =
                svg_path_from_points(g, state.canvas.height() as f64, state.scale, state.offset);
            s.push_str(&format!(
                "<path d=\"{}\" fill=\"#ffffff\" stroke=\"#333\" stroke-width=\"1.5\"/>\n",
                d
            ));
        }
    }
    s.push_str("</svg>");
    s
}

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
    if let Some(input) = doc.get_element_by_id("file") {
        let input: HtmlInputElement = input.dyn_into().unwrap();
        let st = state.clone();
        // Clone references that will be moved into closures
        let input_for_closure = input.clone();
        let onchange = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |_e: Event| {
            let files = input_for_closure.files();
            if files.is_none() {
                return;
            }
            let files = files.unwrap();
            if files.length() == 0 {
                return;
            }
            let file = files.item(0).unwrap();
            let reader = FileReader::new().unwrap();
            let st2 = st.clone();
            // Clone the FileReader for use inside the onload closure
            let reader_for_closure = reader.clone();
            let onload = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |_ev: Event| {
                let result = reader_for_closure.result().unwrap();
                let text = result.as_string().unwrap_or_default();
                if let Ok(p) = serde_json::from_str::<Puzzle>(&text) {
                    st2.borrow_mut().data = p;
                    draw(&mut st2.borrow_mut());
                }
            }));
            reader.set_onload(Some(onload.as_ref().unchecked_ref()));
            reader.read_as_text(&file).unwrap();
            onload.forget();
        }));
        input.set_onchange(Some(onchange.as_ref().unchecked_ref()));
        onchange.forget();
    }

    // Export PNG (blueprint)
    if let Some(btn) = doc.get_element_by_id("exportPng") {
        let btn: HtmlElement = btn.dyn_into().unwrap();
        let st = state.clone();
        let onclick = Closure::<dyn FnMut()>::wrap(Box::new(move || {
            let _ = export_png_blueprint(&st.borrow());
        }));
        btn.set_onclick(Some(onclick.as_ref().unchecked_ref()));
        onclick.forget();
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
            let pt = (e.offset_x() as f64, e.offset_y() as f64);
            let h = s.canvas.height() as f64;
            // find topmost piece under cursor
            for i in (0..s.data.pieces.len()).rev() {
                if let Some(ref geom) = s.data.pieces[i].__geom {
                    if point_in_polygon(pt, geom, h, s.scale, s.offset) {
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
                let pt = (
                    (e.offset_x() as f64) - s.drag_off.0,
                    (e.offset_y() as f64) - s.drag_off.1,
                );
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
                    // allow board intersection as requested
                    let board_ok = true;
                    // Allow overlapping with other pieces (取消不能重合的限制)
                    if board_ok {
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
                    // swap: e clockwise, q counter-clockwise, continuous while held
                    "q" => {
                        s.rot_vel = -180.0;
                    }
                    "e" => {
                        s.rot_vel = 180.0;
                    }
                    "f" => {
                        p.flip = Some(!p.flip.unwrap_or(false));
                        draw(&mut s);
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
        }));
        state
            .borrow()
            .window
            .add_event_listener_with_callback("keyup", keyup.as_ref().unchecked_ref())?;
        keyup.forget();
    }

    // Mouse wheel for linear rotation while dragging
    {
        let st = state.clone();
        let wheel = Closure::<dyn FnMut(web_sys::WheelEvent)>::wrap(Box::new(
            move |e: web_sys::WheelEvent| {
                let mut s = st.borrow_mut();
                if let Some(idx) = s.dragging_idx {
                    let base = if e.shift_key() { 0.03 } else { 0.1 }; // deg per deltaY unit
                    let delta = (-e.delta_y() as f64) * base;
                    let p = &mut s.data.pieces[idx];
                    p.rotation = Some(p.rotation.unwrap_or(0.0) + delta);
                    draw(&mut s);
                    e.prevent_default();
                }
            },
        ));
        state
            .borrow()
            .canvas
            .add_event_listener_with_callback("wheel", wheel.as_ref().unchecked_ref())?;
        wheel.forget();
    }

    Ok(())
}

fn export_png_blueprint(state: &State) -> Result<(), JsValue> {
    // parameters (in mm and px)
    let pad_mm = 5.0;
    let grid_gap_mm = 8.0;
    let px_per_mm = 4.0; // export resolution

    // Collect board geometry and bounds
    let mut board_geom: Vec<Point> = Vec::new();
    let mut board_bounds: Option<(f64, f64, f64, f64)> = None;
    if let Some(b) = &state.data.board {
        if let Some(g) = board_to_geom(b) {
            let (minx, miny, maxx, maxy) = bounds_of(&g);
            board_geom = g;
            board_bounds = Some((minx, miny, maxx, maxy));
        }
    }

    // Compute piece geometries and bounds
    let mut piece_geoms: Vec<(Vec<Point>, (f64, f64, f64, f64))> = Vec::new();
    for p in &state.data.pieces {
        let (g, _c) = piece_geom(p);
        if !g.is_empty() {
            piece_geoms.push((g.clone(), bounds_of(&g)));
        }
    }

    // Layout: board on top, pieces in rows below, packed to a target row width
    let board_w_mm = board_bounds.map(|b| b.2 - b.0).unwrap_or(120.0);
    let target_row_w_mm = board_w_mm.max(120.0);

    let mut rows: Vec<Vec<usize>> = Vec::new();
    let mut cur_row: Vec<usize> = Vec::new();
    let mut cur_w = 0.0;
    for (i, (_g, (minx, _miny, maxx, _maxy))) in piece_geoms.iter().enumerate() {
        let w = (maxx - minx) + grid_gap_mm;
        if !cur_row.is_empty() && cur_w + w > target_row_w_mm {
            rows.push(cur_row);
            cur_row = Vec::new();
            cur_w = 0.0;
        }
        cur_w += w;
        cur_row.push(i);
    }
    if !cur_row.is_empty() {
        rows.push(cur_row);
    }

    // Compute overall size in mm
    let mut total_w_mm = target_row_w_mm + pad_mm * 2.0;
    let board_h_mm = board_bounds.map(|b| b.3 - b.1).unwrap_or(100.0);
    let mut total_h_mm = pad_mm + board_h_mm + pad_mm; // top pad + board + pad to rows
    for row in &rows {
        let mut row_h: f64 = 0.0;
        for &idx in row {
            let (_g, (_minx, miny, _maxx, maxy)) = &piece_geoms[idx];
            row_h = row_h.max(maxy - miny);
        }
        total_h_mm += row_h + grid_gap_mm;
    }
    total_h_mm += pad_mm; // bottom pad

    // Create offscreen canvas
    let (w_px, h_px) = (
        (total_w_mm * px_per_mm).ceil() as u32,
        (total_h_mm * px_per_mm).ceil() as u32,
    );
    let document = state.document.clone();
    let canvas: HtmlCanvasElement = document.create_element("canvas")?.dyn_into()?;
    canvas.set_width(w_px);
    canvas.set_height(h_px);
    let ctx: CanvasRenderingContext2d = canvas.get_context("2d")?.unwrap().dyn_into()?;
    ctx.set_fill_style(&JsValue::from_str("#ffffff"));
    ctx.fill_rect(0.0, 0.0, w_px as f64, h_px as f64);
    ctx.set_stroke_style(&JsValue::from_str("#333"));
    ctx.set_line_width(1.5);
    ctx.set_font("12px sans-serif");

    // Transform helpers for this export
    let canvas_h = h_px as f64;
    let scale = px_per_mm;
    let offset = (0.0, 0.0);

    // Draw board centered horizontally at top
    let mut cursor_y_mm = pad_mm;
    if !board_geom.is_empty() {
        let (minx, miny, maxx, maxy) = board_bounds.unwrap();
        let bw = maxx - minx;
        let bh = maxy - miny;
        let left_mm = ((target_row_w_mm - bw) / 2.0).max(0.0) + pad_mm;
        let geom = translate_geom(&board_geom, -minx + left_mm, -miny + cursor_y_mm);
        draw_colored_polygon(&ctx, canvas_h, &geom, false, scale, offset, "#ffffff");
        // Dimensions
        draw_dimension_mm(
            &ctx,
            canvas_h,
            scale,
            offset,
            (left_mm, cursor_y_mm + bh + 3.0),
            (left_mm + bw, cursor_y_mm + bh + 3.0),
            &format!("{:.0} mm", bw),
        );
        draw_dimension_mm(
            &ctx,
            canvas_h,
            scale,
            offset,
            (left_mm - 3.0, cursor_y_mm),
            (left_mm - 3.0, cursor_y_mm + bh),
            &format!("{:.0} mm", bh),
        );
        cursor_y_mm += bh + pad_mm;
    }

    // Draw pieces row by row
    let mut row_top = cursor_y_mm;
    for row in rows {
        let mut x_mm = pad_mm;
        let mut row_h: f64 = 0.0;
        for idx in row {
            let (geom, (minx, miny, maxx, maxy)) = &piece_geoms[idx];
            let w = maxx - minx;
            let h = maxy - miny;
            let g = translate_geom(geom, -minx + x_mm, -miny + row_top);
            draw_colored_polygon(&ctx, canvas_h, &g, false, scale, offset, "#ffffff");
            // bounding box dims
            draw_dimension_mm(
                &ctx,
                canvas_h,
                scale,
                offset,
                (x_mm, row_top + h + 2.5),
                (x_mm + w, row_top + h + 2.5),
                &format!("{:.0} mm", w),
            );
            draw_dimension_mm(
                &ctx,
                canvas_h,
                scale,
                offset,
                (x_mm - 2.5, row_top),
                (x_mm - 2.5, row_top + h),
                &format!("{:.0} mm", h),
            );
            x_mm += w + grid_gap_mm;
            row_h = row_h.max(h);
        }
        row_top += row_h + grid_gap_mm;
    }

    // Save as PNG
    let cb = Closure::<dyn FnMut(Option<Blob>)>::new({
        let document = document.clone();
        move |opt_blob: Option<Blob>| {
            if let Some(blob) = opt_blob {
                if let Ok(url) = Url::create_object_url_with_blob(&blob) {
                    if let Ok(a) = document.create_element("a") {
                        if let Ok(ae) = a.dyn_into::<HtmlElement>() {
                            let _ = ae.set_attribute("href", &url);
                            let _ = ae.set_attribute("download", "puzzle_blueprint.png");
                            ae.click();
                            let _ = Url::revoke_object_url(&url);
                        }
                    }
                }
            }
        }
    });
    canvas.to_blob(cb.as_ref().unchecked_ref());
    cb.forget();
    Ok(())
}

fn translate_geom(pts: &[Point], dx: f64, dy: f64) -> Vec<Point> {
    pts.iter()
        .map(|p| Point {
            x: p.x + dx,
            y: p.y + dy,
        })
        .collect()
}

fn bounds_of(pts: &[Point]) -> (f64, f64, f64, f64) {
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

fn draw_dimension_mm(
    ctx: &CanvasRenderingContext2d,
    canvas_h: f64,
    scale: f64,
    offset: (f64, f64),
    a_mm: (f64, f64),
    b_mm: (f64, f64),
    label: &str,
) {
    let (ax, ay) = to_screen(
        Point {
            x: a_mm.0,
            y: a_mm.1,
        },
        canvas_h,
        scale,
        offset,
    );
    let (bx, by) = to_screen(
        Point {
            x: b_mm.0,
            y: b_mm.1,
        },
        canvas_h,
        scale,
        offset,
    );
    ctx.set_stroke_style(&JsValue::from_str("#999"));
    ctx.set_fill_style(&JsValue::from_str("#333"));
    let _ = ctx.begin_path();
    let _ = ctx.move_to(ax, ay);
    let _ = ctx.line_to(bx, by);
    let _ = ctx.stroke();
    // Arrow heads
    draw_arrow_head(ctx, ax, ay, bx, by);
    draw_arrow_head(ctx, bx, by, ax, ay);
    // Label at midpoint
    let mx = (ax + bx) / 2.0;
    let my = (ay + by) / 2.0;
    let _ = ctx.fill_text(label, mx + 4.0, my - 4.0);
    ctx.set_stroke_style(&JsValue::from_str("#333"));
}

fn draw_arrow_head(ctx: &CanvasRenderingContext2d, x0: f64, y0: f64, x1: f64, y1: f64) {
    let ang = (y1 - y0).atan2(x1 - x0);
    let len = 6.0;
    let a1 = ang + std::f64::consts::PI - 0.6;
    let a2 = ang + std::f64::consts::PI + 0.6;
    let p1 = (x1 + len * a1.cos(), y1 + len * a1.sin());
    let p2 = (x1 + len * a2.cos(), y1 + len * a2.sin());
    let _ = ctx.begin_path();
    let _ = ctx.move_to(x1, y1);
    let _ = ctx.line_to(p1.0, p1.1);
    let _ = ctx.move_to(x1, y1);
    let _ = ctx.line_to(p2.0, p2.1);
    let _ = ctx.stroke();
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
    let f: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
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
                let mut p = Piece::default();
                p.type_ = sd.type_.clone();
                p.w = sd.w;
                p.h = sd.h;
                p.side = sd.side;
                p.a = sd.a;
                p.b = sd.b;
                p.n = sd.n;
                p.d = sd.d;
                p.r = sd.r;
                p.base_bottom = sd.base_bottom;
                p.base_top = sd.base_top;
                p.height = sd.height;
                p.base = sd.base;
                p.offset_top = sd.offset_top;
                p.points = sd.points.clone();
                // For initial layout: arrange in rows inside board or in a grid starting at (10,10)
                pieces.push(p);
            }
        }
    }

    // Simple initial placement: grid with 10mm margin and 5mm gap
    let margin = 10.0;
    let gap = 5.0;
    let (bw, bh) = spec
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

    let mut data = default_puzzle();
    // If URL param p is set, we try to fetch puzzles/<p>.json; otherwise use default
    if let Some(search) = window.location().search().ok() {
        if let Some(p) = get_query_param(&search, "p") {
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
    }));

    STATE.with(|st| st.replace(Some(state.clone())));
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
    let url = format!("puzzle/{}.json", name);
    let resp_value = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(&url)).await?;
    let resp: web_sys::Response = resp_value.dyn_into()?;
    let text = wasm_bindgen_futures::JsFuture::from(resp.text()?).await?;
    let text = text.as_string().unwrap_or_default();
    // Try parse as full Puzzle; fall back to counts+shapes
    let puzzle: Puzzle = if let Ok(p) = serde_json::from_str::<Puzzle>(&text) {
        p
    } else if let Ok(spec) = serde_json::from_str::<CountsSpec>(&text) {
        // Fetch shapes file if provided; else fallback to bundled shapes
        let shapes_text = if let Some(sf) = spec.shapes_file.clone() {
            let resp2 = wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(&sf)).await?;
            let resp2: web_sys::Response = resp2.dyn_into()?;
            let t2 = wasm_bindgen_futures::JsFuture::from(resp2.text()?).await?;
            t2.as_string().unwrap_or_default()
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
            s.window = window.clone();
            s.document = document.clone();
            s.canvas = canvas.clone();
            s.ctx = ctx.clone();
            draw(&mut s);
        }
    });
    Ok(())
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

    let mut have_bounds = false;
    if let Some(b) = &state.data.board {
        if let Some(geom) = board_to_geom(b) {
            for p in geom {
                minx = minx.min(p.x);
                maxx = maxx.max(p.x);
                miny = miny.min(p.y);
                maxy = maxy.max(p.y);
            }
            have_bounds = true;
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
    have_bounds = maxx.is_finite() && maxy.is_finite();

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
