use png::{BitDepth, ColorType, Encoder};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;

thread_local! {
    static LABEL_MAP: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

thread_local! {
    static LANGUAGE: RefCell<String> = RefCell::new("en".to_string());
}

pub fn set_language(lang: &str) {
    let l = if lang.eq_ignore_ascii_case("zh") || lang == "zh-CN" || lang == "zh_TW" {
        "zh"
    } else {
        "en"
    };
    LANGUAGE.with(|s| s.replace(l.to_string()));
}

// Inject or replace the label map used when grouping pieces by ID.
// This enables callers (e.g., browser runtimes) to supply labels without
// requiring file access to shapes.json during rendering.
pub fn set_label_map(map: &HashMap<String, String>) {
    LABEL_MAP.with(|m| {
        let mut mm = m.borrow_mut();
        mm.clear();
        for (k, v) in map.iter() {
            mm.insert(k.clone(), v.clone());
        }
    });
}

fn is_en() -> bool {
    LANGUAGE.with(|s| s.borrow().as_str() == "en")
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PolygonPoint {
    Point([f64; 2]),
    /// Corner with rounding radius: [x, y, r]
    Rounded([f64; 3]),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Board {
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub w: Option<f64>,
    pub h: Option<f64>,
    pub polygons: Option<Vec<Vec<PolygonPoint>>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Piece {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_: String,
    pub at: Option<[f64; 2]>,
    pub rotation: Option<f64>,
    pub anchor: Option<String>,
    pub flip: Option<bool>,
    pub w: Option<f64>,
    pub h: Option<f64>,
    pub side: Option<f64>,
    pub a: Option<f64>,
    pub b: Option<f64>,
    pub n: Option<u32>,
    pub d: Option<f64>,
    pub r: Option<f64>,
    pub base_bottom: Option<f64>,
    pub base_top: Option<f64>,
    pub height: Option<f64>,
    pub base: Option<f64>,
    pub offset_top: Option<f64>,
    pub points: Option<Vec<[f64; 2]>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PuzzleSpec {
    pub units: Option<String>,
    pub title: Option<String>,
    pub note: Option<String>,
    pub board: Option<Board>,
    pub pieces: Option<Vec<Piece>>,
    pub parts: Option<Vec<PartSpec>>,
    pub counts: Option<HashMap<String, u32>>,
    pub shapes_file: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PartSpec {
    #[serde(rename = "type")]
    pub type_: String,
    pub count: u32,
    pub w: Option<f64>,
    pub h: Option<f64>,
    pub side: Option<f64>,
    pub a: Option<f64>,
    pub b: Option<f64>,
    pub n: Option<u32>,
    pub d: Option<f64>,
    pub r: Option<f64>,
    pub base_bottom: Option<f64>,
    pub base_top: Option<f64>,
    pub height: Option<f64>,
    pub base: Option<f64>,
    pub offset_top: Option<f64>,
    pub points: Option<Vec<[f64; 2]>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ShapeDef {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub w: Option<f64>,
    pub h: Option<f64>,
    pub side: Option<f64>,
    pub a: Option<f64>,
    pub b: Option<f64>,
    pub n: Option<u32>,
    pub d: Option<f64>,
    pub r: Option<f64>,
    pub base_bottom: Option<f64>,
    pub base_top: Option<f64>,
    pub height: Option<f64>,
    pub base: Option<f64>,
    pub offset_top: Option<f64>,
    pub points: Option<Vec<[f64; 2]>>,
    pub label: Option<String>,
    pub label_en: Option<String>,
    pub label_zh: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ShapesCatalog {
    pub shapes: Vec<ShapeDef>,
}

fn rotate_point(p: Point, c: Point, ang: f64, flip: bool) -> Point {
    let mut dx = p.x - c.x;
    let dy = p.y - c.y;
    if flip {
        dx = -dx;
    }
    let (s, ca) = ang.to_radians().sin_cos();
    Point {
        x: c.x + dx * ca - dy * s,
        y: c.y + dx * s + dy * ca,
    }
}

// Shared PNG encoder: RGBA -> PNG bytes (deterministic for same input)
pub fn encode_rgba_to_png_bytes(
    width: u32,
    height: u32,
    rgba: &[u8],
) -> Result<Vec<u8>, png::EncodingError> {
    let mut buf = Vec::new();
    {
        let mut enc = Encoder::new(&mut buf, width, height);
        enc.set_color(ColorType::Rgba);
        enc.set_depth(BitDepth::Eight);
        {
            let mut writer = enc.write_header()?;
            writer.write_image_data(rgba)?;
        }
        // enc drops here, releasing the &mut buf borrow
    }
    Ok(buf)
}
fn piece_rotation(p: &Piece) -> f64 {
    p.rotation.unwrap_or(0.0)
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
            let base_ang = piece_rotation(p) + if piece_flip(p) { 180.0 } else { 0.0 };
            let mut pts = Vec::new();
            for i in 0..n {
                let a = (base_ang + (i as f64) * 360.0 / (n as f64)).to_radians();
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
            (pts, ctr)
        }
        _ => (Vec::new(), Point { x: 0.0, y: 0.0 }),
    }
}

fn normalize(p: Point) -> Point {
    let len = (p.x * p.x + p.y * p.y).sqrt();
    if len == 0.0 {
        Point { x: 0.0, y: 0.0 }
    } else {
        Point {
            x: p.x / len,
            y: p.y / len,
        }
    }
}

fn poly_to_points(poly: &[PolygonPoint]) -> Vec<Point> {
    let mut out: Vec<Point> = Vec::new();
    let n = poly.len();
    let mut i = 0;
    while i < n {
        match &poly[i] {
            PolygonPoint::Point([x, y]) => {
                out.push(Point { x: *x, y: *y });
                i += 1;
            }
            PolygonPoint::Rounded([x, y, r]) => {
                if out.is_empty() || i + 1 >= n {
                    i += 1;
                    continue;
                }
                let prev = *out.last().unwrap();
                let next_xy = match &poly[i + 1] {
                    PolygonPoint::Point([nx, ny]) => Point { x: *nx, y: *ny },
                    PolygonPoint::Rounded([nx, ny, _]) => Point { x: *nx, y: *ny },
                };
                let corner = Point { x: *x, y: *y };
                let radius = *r;
                let v1 = normalize(Point {
                    x: prev.x - corner.x,
                    y: prev.y - corner.y,
                });
                let v2 = normalize(Point {
                    x: next_xy.x - corner.x,
                    y: next_xy.y - corner.y,
                });
                let start = Point {
                    x: corner.x + v1.x * radius,
                    y: corner.y + v1.y * radius,
                };
                let end = Point {
                    x: corner.x + v2.x * radius,
                    y: corner.y + v2.y * radius,
                };
                out.push(start);
                let center = Point {
                    x: corner.x + (v1.x + v2.x) * radius,
                    y: corner.y + (v1.y + v2.y) * radius,
                };
                let start_ang = (start.y - center.y).atan2(start.x - center.x);
                let end_ang = (end.y - center.y).atan2(end.x - center.x);
                let steps = 24;
                for j in 1..=steps {
                    let t = j as f64 / steps as f64;
                    let ang = start_ang + (end_ang - start_ang) * t;
                    out.push(Point {
                        x: center.x + radius * ang.cos(),
                        y: center.y + radius * ang.sin(),
                    });
                }
                i += 1;
            }
        }
    }
    out
}

fn board_to_geom(board: &Board) -> Option<Vec<Vec<Point>>> {
    match board.type_.as_deref() {
        Some("rect") => {
            let w = board.w.unwrap_or(0.0);
            let h = board.h.unwrap_or(0.0);
            Some(vec![vec![
                Point { x: 0.0, y: 0.0 },
                Point { x: w, y: 0.0 },
                Point { x: w, y: h },
                Point { x: 0.0, y: h },
            ]])
        }
        Some("polygon") => {
            if let Some(polys) = &board.polygons {
                let geoms = polys
                    .iter()
                    .map(|poly| poly_to_points(poly))
                    .collect::<Vec<_>>();
                if geoms.is_empty() { None } else { Some(geoms) }
            } else {
                None
            }
        }
        _ => None,
    }
}

#[derive(Clone)]
struct Segment {
    start: Point,
    end: Point,
    radius: Option<f64>,
    center: Option<Point>,
}

fn polygon_segments(poly: &[PolygonPoint]) -> Vec<Segment> {
    let n = poly.len();
    if n == 0 {
        return Vec::new();
    }
    let mut segs: Vec<Segment> = Vec::new();
    let mut j: usize = 0;
    let mut curr = match &poly[0] {
        PolygonPoint::Point([x, y]) => Point { x: *x, y: *y },
        PolygonPoint::Rounded([x, y, _]) => Point { x: *x, y: *y },
    };
    while j < n {
        let next_idx = (j + 1) % n;
        match &poly[next_idx] {
            PolygonPoint::Point([x, y]) => {
                let next = Point { x: *x, y: *y };
                segs.push(Segment {
                    start: curr,
                    end: next,
                    radius: None,
                    center: None,
                });
                curr = next;
                j += 1;
            }
            PolygonPoint::Rounded([x, y, r]) => {
                let corner = Point { x: *x, y: *y };
                let next2_idx = (j + 2) % n;
                let next_xy = match &poly[next2_idx] {
                    PolygonPoint::Point([nx, ny]) => Point { x: *nx, y: *ny },
                    PolygonPoint::Rounded([nx, ny, _]) => Point { x: *nx, y: *ny },
                };
                let radius = *r;
                let v1 = normalize(Point {
                    x: curr.x - corner.x,
                    y: curr.y - corner.y,
                });
                let v2 = normalize(Point {
                    x: next_xy.x - corner.x,
                    y: next_xy.y - corner.y,
                });
                let start = Point {
                    x: corner.x + v1.x * radius,
                    y: corner.y + v1.y * radius,
                };
                segs.push(Segment {
                    start: curr,
                    end: start,
                    radius: None,
                    center: None,
                });
                let end = Point {
                    x: corner.x + v2.x * radius,
                    y: corner.y + v2.y * radius,
                };
                let center = Point {
                    x: corner.x + (v1.x + v2.x) * radius,
                    y: corner.y + (v1.y + v2.y) * radius,
                };
                segs.push(Segment {
                    start,
                    end,
                    radius: Some(radius),
                    center: Some(center),
                });
                curr = end;
                j += 1; // skip rounded point
            }
        }
    }
    segs
}

fn board_segments(board: &Board) -> Vec<Segment> {
    match board.type_.as_deref() {
        Some("rect") => {
            let w = board.w.unwrap_or(0.0);
            let h = board.h.unwrap_or(0.0);
            vec![
                Segment {
                    start: Point { x: 0.0, y: 0.0 },
                    end: Point { x: w, y: 0.0 },
                    radius: None,
                    center: None,
                },
                Segment {
                    start: Point { x: w, y: 0.0 },
                    end: Point { x: w, y: h },
                    radius: None,
                    center: None,
                },
                Segment {
                    start: Point { x: w, y: h },
                    end: Point { x: 0.0, y: h },
                    radius: None,
                    center: None,
                },
                Segment {
                    start: Point { x: 0.0, y: h },
                    end: Point { x: 0.0, y: 0.0 },
                    radius: None,
                    center: None,
                },
            ]
        }
        Some("polygon") => {
            if let Some(polys) = &board.polygons {
                polys.iter().flat_map(|p| polygon_segments(p)).collect()
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    }
}

fn translate_geom(pts: &[Point], dx: f64, dy: f64) -> Vec<Point> {
    pts.iter()
        .map(|p| Point {
            x: p.x + dx,
            y: p.y + dy,
        })
        .collect()
}

fn translate_geoms(geoms: &[Vec<Point>], dx: f64, dy: f64) -> Vec<Vec<Point>> {
    geoms.iter().map(|g| translate_geom(g, dx, dy)).collect()
}
fn bounds_of(pts: &[Point]) -> (f64, f64, f64, f64) {
    let (mut minx, mut miny, mut maxx, mut maxy) = (
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    );
    for p in pts {
        minx = minx.min(p.x);
        miny = miny.min(p.y);
        maxx = maxx.max(p.x);
        maxy = maxy.max(p.y);
    }
    (minx, miny, maxx, maxy)
}

fn bounds_of_all(polys: &[Vec<Point>]) -> (f64, f64, f64, f64) {
    let mut first = true;
    let mut out = (0.0, 0.0, 0.0, 0.0);
    for p in polys {
        let b = bounds_of(p);
        if first {
            out = b;
            first = false;
        } else {
            out.0 = out.0.min(b.0);
            out.1 = out.1.min(b.1);
            out.2 = out.2.max(b.2);
            out.3 = out.3.max(b.3);
        }
    }
    out
}

fn svg_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
fn label_from_catalog_only(p: &Piece) -> String {
    if let Some(id) = &p.id {
        let mut hit: Option<String> = None;
        LABEL_MAP.with(|m| {
            if let Some(lbl) = m.borrow().get(id) {
                hit = Some(lbl.clone());
            }
        });
        if let Some(s) = hit {
            return s;
        }
    }
    String::new()
}

fn group_key_for_piece(p: &Piece) -> String {
    if let Some(id) = &p.id {
        return id.clone();
    }
    // Fallback: stable signature from type and key parameters (no localization)
    match p.type_.as_str() {
        "rect" => format!("rect:w={};h={}", p.w.unwrap_or(0.0), p.h.unwrap_or(0.0)),
        "equilateral_triangle" => format!("equilateral_triangle:side={}", p.side.unwrap_or(0.0)),
        "right_triangle" => format!(
            "right_triangle:a={};b={}",
            p.a.unwrap_or(0.0),
            p.b.unwrap_or(0.0)
        ),
        "regular_polygon" => format!(
            "regular_polygon:n={};side={}",
            p.n.unwrap_or(0),
            p.side.unwrap_or(0.0)
        ),
        "circle" => format!("circle:d={};r={}", p.d.unwrap_or(0.0), p.r.unwrap_or(0.0)),
        "isosceles_trapezoid" => format!(
            "isosceles_trapezoid:bb={};bt={};h={}",
            p.base_bottom.unwrap_or(0.0),
            p.base_top.unwrap_or(0.0),
            p.height.unwrap_or(0.0)
        ),
        "parallelogram" => format!(
            "parallelogram:base={};off={};h={}",
            p.base.unwrap_or(0.0),
            p.offset_top.unwrap_or(0.0),
            p.height.unwrap_or(0.0)
        ),
        "polygon" => "polygon".to_string(),
        other => other.to_string(),
    }
}

pub fn build_blueprint_svg(
    p: &PuzzleSpec,
    px_per_mm: f64,
    shapes_path: Option<&str>,
) -> (String, u32, u32) {
    // Do not clear LABEL_MAP here; callers may have provided labels via
    // set_label_map(). When counts are provided below, we overwrite entries.
    let mut board_geom: Vec<Vec<Point>> = Vec::new();
    let mut board_bounds: Option<(f64, f64, f64, f64)> = None;
    if let Some(b) = &p.board
        && let Some(g) = board_to_geom(b)
    {
        board_bounds = Some(bounds_of_all(&g));
        board_geom = g;
    }

    let mut flat_pieces: Vec<Piece> = Vec::new();
    if let Some(parts) = &p.parts {
        for ps in parts {
            for _ in 0..ps.count {
                flat_pieces.push(Piece {
                    type_: ps.type_.clone(),
                    w: ps.w,
                    h: ps.h,
                    side: ps.side,
                    a: ps.a,
                    b: ps.b,
                    n: ps.n,
                    d: ps.d,
                    r: ps.r,
                    base_bottom: ps.base_bottom,
                    base_top: ps.base_top,
                    height: ps.height,
                    base: ps.base,
                    offset_top: ps.offset_top,
                    points: ps.points.clone(),
                    ..Default::default()
                });
            }
        }
    } else if let Some(counts) = &p.counts {
        let shapes_path = shapes_path
            .map(|s| s.to_string())
            .or_else(|| p.shapes_file.clone())
            .unwrap_or_else(|| "shapes.json".to_string());
        let txt =
            fs::read_to_string(&shapes_path).unwrap_or_else(|_| "{\"shapes\":[]}".to_string());
        let catalog: ShapesCatalog = serde_json::from_str(&txt).unwrap_or_default();
        let mut by_id: HashMap<String, &ShapeDef> = HashMap::new();
        let mut label_map: HashMap<String, String> = HashMap::new();
        for s in &catalog.shapes {
            by_id.insert(s.id.clone(), s);
            // Prefer language-specific labels; for English, ignore generic 'label' to allow fallback
            let chosen = if is_en() {
                s.label_en.clone()
            } else {
                s.label_zh.clone().or_else(|| s.label.clone())
            };
            if let Some(lbl) = chosen {
                label_map.insert(s.id.clone(), lbl);
            }
        }
        for (id, cnt) in counts.iter() {
            if let Some(sd) = by_id.get(id) {
                for _ in 0..*cnt {
                    flat_pieces.push(Piece {
                        id: Some(sd.id.clone()),
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
                    });
                }
            }
        }
        LABEL_MAP.with(|m| *m.borrow_mut() = label_map);
    } else if let Some(pcs) = &p.pieces {
        flat_pieces = pcs.clone();
    }

    #[derive(Clone)]
    struct Item {
        geom: Vec<Point>,
        bounds: (f64, f64, f64, f64),
    }
    let mut groups: Vec<(String, Vec<Item>)> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    for pc in &flat_pieces {
        let (g, _c) = piece_geom(pc);
        if g.is_empty() {
            continue;
        }
        let key = group_key_for_piece(pc);
        let label = label_from_catalog_only(pc);
        let it = Item {
            geom: g.clone(),
            bounds: bounds_of(&g),
        };
        if let Some(i) = index.get(&key) {
            groups[*i].1.push(it);
        } else {
            let id = groups.len();
            groups.push((label.clone(), vec![it]));
            index.insert(key, id);
        }
    }

    let pad_mm = 5.0;
    let gap_mm = 8.0;
    let title_h_mm = 20.0;
    let mut max_label_chars: usize = 0;
    let mut max_count_chars: usize = 0;
    for (label, items) in &groups {
        max_label_chars = max_label_chars.max(label.chars().count());
        max_count_chars = max_count_chars.max(items.len().to_string().chars().count());
    }
    let label_w_px = (max_label_chars as f64 * 26.0).max(220.0) + 44.0;
    let count_w_px = (max_count_chars as f64 * 20.0).max(40.0) + 24.0;
    let label_w_mm = label_w_px / px_per_mm;
    let count_w_mm = count_w_px / px_per_mm;
    let board_w_mm = board_bounds.map(|b| b.2 - b.0).unwrap_or(120.0);
    let board_h_mm = board_bounds.map(|b| b.3 - b.1).unwrap_or(100.0);
    let mut table_w_mm = label_w_mm + count_w_mm;
    let mut table_h_mm: f64 = 0.0;
    let mut row_heights: Vec<f64> = Vec::new();
    for (_label, items) in &groups {
        let mut row_w = label_w_mm + count_w_mm;
        let mut row_h: f64 = 0.0;
        for it in items {
            let (minx, miny, maxx, maxy) = it.bounds;
            let w = maxx - minx;
            let h = maxy - miny;
            row_w += w + gap_mm;
            row_h = row_h.max(h);
        }
        row_heights.push(row_h);
        table_w_mm = table_w_mm.max(row_w);
        table_h_mm += row_h + gap_mm;
    }
    let content_w_mm = table_w_mm.max(board_w_mm);
    let mut total_w_mm = content_w_mm + pad_mm * 2.0;
    if total_w_mm < 160.0 + pad_mm * 2.0 {
        total_w_mm = 160.0 + pad_mm * 2.0;
    }
    let board_gap_mm = 20.0;
    let note_h_mm = p.note.as_ref().map(|_| 10.0).unwrap_or(0.0);
    let note_gap_mm = if note_h_mm > 0.0 { 10.0 } else { 0.0 };

    // Layout from bottom (y = 0) upwards
    let mut cursor_mm = pad_mm; // bottom padding
    let note_y_mm = if note_h_mm > 0.0 {
        let y = cursor_mm + note_h_mm / 2.0;
        cursor_mm += note_h_mm + note_gap_mm;
        Some(y)
    } else {
        None
    };
    let board_top = cursor_mm;
    if !board_geom.is_empty() {
        cursor_mm += board_h_mm + board_gap_mm;
    }
    let table_top_mm = cursor_mm;
    cursor_mm += table_h_mm + gap_mm;
    let title_y_mm = cursor_mm + title_h_mm / 2.0;
    cursor_mm += title_h_mm + pad_mm; // top padding
    let total_h_mm = cursor_mm;

    let w_px = (total_w_mm * px_per_mm).ceil() as u32;
    let h_px = (total_h_mm * px_per_mm).ceil() as u32;
    let mut s = String::new();
    s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    s.push_str(&format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" stroke=\"#333\" fill=\"none\" stroke-width=\"1.8\" stroke-linejoin=\"round\" font-family=\"sans-serif\" font-size=\"26\">\n", w_px, h_px, w_px, h_px));
    s.push_str("<rect x=\"0\" y=\"0\" width=\"100%\" height=\"100%\" fill=\"#ffffff\"/>\n");
    s.push_str("<defs><marker id=\"arrow\" viewBox=\"0 0 10 10\" refX=\"5\" refY=\"5\" markerWidth=\"6\" markerHeight=\"6\" orient=\"auto-start-reverse\"><path d=\"M 0 0 L 10 5 L 0 10 z\" fill=\"#888\" /></marker></defs>\n");
    let mm2px = |x: f64| x * px_per_mm;
    let to_px = |p: Point| (mm2px(p.x), mm2px(total_h_mm - p.y));
    let x_sep1_mm = pad_mm + label_w_mm;
    let x_sep2_mm = x_sep1_mm + count_w_mm;
    let draw_vline = |s: &mut String, x_mm: f64, y0_mm: f64, y1_mm: f64| {
        let (x, y0) = to_px(Point { x: x_mm, y: y0_mm });
        let (_x2, y1) = to_px(Point { x: x_mm, y: y1_mm });
        s.push_str(&format!(
            "<path d=\"M {:.2} {:.2} L {:.2} {:.2}\" stroke=\"#ddd\" stroke-width=\"1\"/>\n",
            x, y0, x, y1
        ));
    };
    let draw_hline = |s: &mut String, y_mm: f64| {
        let (x0, y) = to_px(Point { x: pad_mm, y: y_mm });
        let (x1, _y) = to_px(Point {
            x: total_w_mm - pad_mm,
            y: y_mm,
        });
        s.push_str(&format!(
            "<path d=\"M {:.2} {:.2} L {:.2} {:.2}\" stroke=\"#ddd\" stroke-width=\"1\"/>\n",
            x0, y, x1, y
        ));
    };
    let table_bottom_mm =
        table_top_mm + table_h_mm - if row_heights.is_empty() { 0.0 } else { gap_mm };
    if !row_heights.is_empty() {
        draw_vline(&mut s, x_sep1_mm, table_top_mm, table_bottom_mm);
        draw_vline(&mut s, x_sep2_mm, table_top_mm, table_bottom_mm);
        draw_hline(&mut s, table_top_mm);
    }
    if let Some(t) = &p.title {
        let (tx, ty) = to_px(Point {
            x: total_w_mm / 2.0,
            y: title_y_mm,
        });
        s.push_str(&format!(
            "<text x=\"{:.2}\" y=\"{:.2}\" text-anchor=\"middle\" fill=\"#333\" font-size=\"40\">{}</text>\n",
            tx,
            ty,
            svg_escape(t)
        ));
    }
    let mut row_top = table_top_mm;
    for ((label, items), row_h) in groups.into_iter().zip(row_heights.into_iter()) {
        s.push_str(&format!(
            "<text x=\"{:.2}\" y=\"{:.2}\" fill=\"#333\" font-size=\"26\">{}</text>\n",
            mm2px(pad_mm + 2.0),
            mm2px(total_h_mm - (row_top + row_h / 2.0)),
            svg_escape(&label)
        ));
        let cx_mm = (x_sep1_mm + x_sep2_mm) / 2.0;
        s.push_str(&format!("<text x=\"{:.2}\" y=\"{:.2}\" text-anchor=\"middle\" fill=\"#333\" font-size=\"26\">{}</text>\n", mm2px(cx_mm), mm2px(total_h_mm-(row_top+row_h/2.0)), items.len()));
        let col_gap_mm = 2.0;
        let mut x_mm = x_sep2_mm + col_gap_mm;
        for it in items {
            let (minx, miny, maxx, _maxy) = it.bounds;
            let w = maxx - minx;
            let g = translate_geom(&it.geom, -minx + x_mm, -miny + row_top);
            s.push_str(&path_from_points(&g, &to_px));
            x_mm += w + gap_mm;
        }
        row_top += row_h;
        draw_hline(&mut s, row_top);
        row_top += gap_mm;
    }
    if let Some(ny) = note_y_mm {
        if let Some(txt) = &p.note {
            let (tx, ty) = to_px(Point {
                x: total_w_mm / 2.0,
                y: ny,
            });
            s.push_str(&format!(
                "<text x=\"{:.2}\" y=\"{:.2}\" text-anchor=\"middle\" fill=\"#333\" font-size=\"20\">{}</text>\n",
                tx, ty, svg_escape(txt)
            ));
        }
    }
    if !board_geom.is_empty() {
        let (minx, miny, _maxx, _maxy) = board_bounds.unwrap();
        let x_mm = (total_w_mm - board_w_mm) / 2.0;
        let g = translate_geoms(&board_geom, -minx + x_mm, -miny + board_top);
        s.push_str(&paths_from_geoms(&g, &to_px));
        if let Some(b) = &p.board {
            let segs = board_segments(b);
            for seg in segs {
                let start = Point {
                    x: seg.start.x - minx + x_mm,
                    y: seg.start.y - miny + board_top,
                };
                let end = Point {
                    x: seg.end.x - minx + x_mm,
                    y: seg.end.y - miny + board_top,
                };
                if let Some(r) = seg.radius {
                    if let Some(c) = seg.center {
                        let cp = Point {
                            x: c.x - minx + x_mm,
                            y: c.y - miny + board_top,
                        };
                        let sp = start;
                        let (cx, cy) = to_px(cp);
                        let (sx, sy) = to_px(sp);
                        s.push_str(&format!("<path d=\"M {:.2} {:.2} L {:.2} {:.2}\" stroke=\"#888\" stroke-width=\"1\" marker-end=\"url(#arrow)\"/>\n", cx, cy, sx, sy));
                        let mid = Point {
                            x: (cp.x + sp.x) / 2.0,
                            y: (cp.y + sp.y) / 2.0,
                        };
                        let (tx, ty) = to_px(Point {
                            x: mid.x + 3.0,
                            y: mid.y,
                        });
                        s.push_str(&format!("<text x=\"{:.2}\" y=\"{:.2}\" fill=\"#333\" font-size=\"20\">R{:.0}</text>\n", tx, ty, r));
                    }
                } else {
                    let dx = (end.x - start.x).abs();
                    let dy = (end.y - start.y).abs();
                    let offset = 3.0;
                    if dx > 0.0 {
                        let x1 = start.x.min(end.x);
                        let x2 = start.x.max(end.x);
                        let y = start.y.max(end.y) + offset;
                        let (sx, sy) = to_px(Point { x: x1, y });
                        let (ex, ey) = to_px(Point { x: x2, y });
                        s.push_str(&format!("<path d=\"M {:.2} {:.2} L {:.2} {:.2}\" stroke=\"#888\" stroke-width=\"1\" marker-start=\"url(#arrow)\" marker-end=\"url(#arrow)\"/>\n", sx, sy, ex, ey));
                        let mid = Point {
                            x: (x1 + x2) / 2.0,
                            y: y + 4.0,
                        };
                        let (tx, ty) = to_px(mid);
                        s.push_str(&format!("<text x=\"{:.2}\" y=\"{:.2}\" text-anchor=\"middle\" fill=\"#333\" font-size=\"20\">{:.0}</text>\n", tx, ty, dx));
                    }
                    if dy > 0.0 {
                        let y1 = start.y.min(end.y);
                        let y2 = start.y.max(end.y);
                        let x = start.x.max(end.x) + offset;
                        let (sx, sy) = to_px(Point { x, y: y1 });
                        let (ex, ey) = to_px(Point { x, y: y2 });
                        s.push_str(&format!("<path d=\"M {:.2} {:.2} L {:.2} {:.2}\" stroke=\"#888\" stroke-width=\"1\" marker-start=\"url(#arrow)\" marker-end=\"url(#arrow)\"/>\n", sx, sy, ex, ey));
                        let mid = Point {
                            x: x + 4.0,
                            y: (y1 + y2) / 2.0,
                        };
                        let (tx, ty) = to_px(mid);
                        s.push_str(&format!("<text x=\"{:.2}\" y=\"{:.2}\" fill=\"#333\" font-size=\"20\">{:.0}</text>\n", tx, ty, dy));
                    }
                }
            }
        }
    }
    s.push_str("</svg>\n");
    (s, w_px, h_px)
}

fn path_from_points<F>(pts: &[Point], to_px: &F) -> String
where
    F: Fn(Point) -> (f64, f64),
{
    if pts.is_empty() {
        return String::new();
    }
    let (x0, y0) = to_px(pts[0]);
    let mut out = format!("<path d=\"M {:.2} {:.2}", x0, y0);
    for p in &pts[1..] {
        let (x, y) = to_px(*p);
        out.push_str(&format!(" L {:.2} {:.2}", x, y));
    }
    out.push_str(" Z\"/>)\n");
    out
}

fn paths_from_geoms<F>(geoms: &[Vec<Point>], to_px: &F) -> String
where
    F: Fn(Point) -> (f64, f64),
{
    let mut out = String::new();
    for g in geoms {
        out.push_str(&path_from_points(g, to_px));
    }
    out
}
