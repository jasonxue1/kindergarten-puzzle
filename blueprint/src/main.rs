use png::{BitDepth, ColorType, Compression, Encoder, FilterType};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::env;
use std::fs;

thread_local! {
    static LABEL_MAP: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
struct Point {
    x: f64,
    y: f64,
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
    // Optional board label (left column). If empty, falls back to auto label "Board W×H mm (R?)"
    label: Option<String>,
    // Optional multi-line label (preferred over label) for irregular boards
    label_lines: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Piece {
    id: Option<String>,
    #[serde(rename = "type")]
    type_: String,
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
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct PuzzleSpec {
    units: Option<String>,
    board: Option<Board>,
    // Either a concrete list of pieces for gameplay JSON, or a parts list for blueprint
    pieces: Option<Vec<Piece>>,
    parts: Option<Vec<PartSpec>>,
    // Or counts per shape id from an external catalog
    counts: Option<HashMap<String, u32>>,
    shapes_file: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct PartSpec {
    #[serde(rename = "type")]
    type_: String,
    count: u32,
    // Same params as Piece, but without pose
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
    // Human-readable label for shape (left column) when using counts+catalog
    label: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct ShapesCatalog {
    shapes: Vec<ShapeDef>,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: blueprint <puzzle.json> <output.(png|svg)> [px_per_mm] [shapes.json]");
        std::process::exit(2);
    }
    let input = &args[1];
    let output = &args[2];
    let px_per_mm: f64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(4.0);
    let shapes_path_arg = args.get(4).cloned();
    let txt = fs::read_to_string(input)?;
    let puzzle: PuzzleSpec = serde_json::from_str(&txt)?;
    // normalize units: expect mm
    if puzzle.units.as_deref() == Some("px") {
        eprintln!("warning: input units are px; treating as mm");
    }

    let (svg, w_px, h_px) = build_blueprint_svg(&puzzle, px_per_mm, shapes_path_arg.as_deref());

    // PNG only: render SVG -> RGBA and save (deterministic)
    let mut opt = usvg::Options::default();
    let mut fontdb = usvg::fontdb::Database::new();
    if fonts::FONT_BYTES.is_empty() {
        panic!(
            "Embedded font is missing. Ensure fonts crate downloaded SourceHanSansCN-Regular.otf."
        );
    }
    fontdb.load_font_data(fonts::FONT_BYTES.to_vec());
    // Map generic 'sans-serif' to the embedded font family
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
    let tree = usvg::Tree::from_str(&svg, &opt).map_err(|e| format!("SVG parse error: {e:?}"))?;
    let mut pixmap = tiny_skia::Pixmap::new(w_px, h_px).ok_or("pixmap alloc failed")?;
    let mut pm = pixmap.as_mut();
    resvg::render(&tree, tiny_skia::Transform::identity(), &mut pm);
    encode_png_deterministic(&pixmap, output)?;
    Ok(())
}

fn encode_png_deterministic(
    pixmap: &tiny_skia::Pixmap,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::create(path)?;
    let w = pixmap.width();
    let h = pixmap.height();
    let mut enc = Encoder::new(file, w, h);
    enc.set_color(ColorType::Rgba);
    enc.set_depth(BitDepth::Eight);
    enc.set_filter(FilterType::NoFilter);
    enc.set_compression(Compression::Default);
    let mut writer = enc.write_header()?;
    writer.write_image_data(pixmap.data())?;
    Ok(())
}

fn build_blueprint_svg(
    p: &PuzzleSpec,
    px_per_mm: f64,
    shapes_path: Option<&str>,
) -> (String, u32, u32) {
    // Clear and prepare label cache
    LABEL_MAP.with(|m| m.borrow_mut().clear());
    // Gather board and pieces
    let mut board_geom: Vec<Point> = Vec::new();
    let mut board_bounds: Option<(f64, f64, f64, f64)> = None;
    if let Some(b) = &p.board
        && let Some(g) = board_to_geom(b)
    {
        board_bounds = Some(bounds_of(&g));
        board_geom = g;
    }
    // Build a working list of pieces from parts, counts+catalog, or concrete pieces
    let mut flat_pieces: Vec<Piece> = Vec::new();
    if let Some(parts) = &p.parts {
        for ps in parts {
            for _ in 0..ps.count {
                let pe = Piece {
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
                };
                flat_pieces.push(pe);
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
            if let Some(lbl) = &s.label {
                label_map.insert(s.id.clone(), lbl.clone());
            }
        }
        for (id, cnt) in counts.iter() {
            if let Some(sd) = by_id.get(id) {
                for _ in 0..*cnt {
                    let pe = Piece {
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
                    };
                    flat_pieces.push(pe);
                }
            }
        }
        // Store the label_map in a side channel by encoding it into a special id "__LABEL_MAP__"
        // We can't attach it to pieces directly; we'll capture it later via closure.
        // To make it accessible, we keep it in an outer variable.
        LABEL_MAP.with(|m| {
            *m.borrow_mut() = label_map;
        });
    } else if let Some(pcs) = &p.pieces {
        flat_pieces = pcs.clone();
    }

    // Group identical specs together for labels
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
        let label = label_from_catalog_or_fallback(pc);
        let it = Item {
            geom: g.clone(),
            bounds: bounds_of(&g),
        };
        if let Some(i) = index.get(&label) {
            groups[*i].1.push(it);
        } else {
            let id = groups.len();
            groups.push((label.clone(), vec![it]));
            index.insert(label, id);
        }
    }

    // Layout in mm: table with 3 columns: [label | count | graphics]
    let pad_mm = 5.0;
    let gap_mm = 8.0;
    // Dynamically size columns based on content
    let mut max_label_chars: usize = 0;
    let mut max_count_chars: usize = 0;
    for (label, items) in &groups {
        max_label_chars = max_label_chars.max(label.chars().count());
        max_count_chars = max_count_chars.max(items.len().to_string().chars().count());
    }
    let label_w_px = (max_label_chars as f64 * 26.0).max(220.0) + 44.0; // char ~ font size px; add margin
    let count_w_px = (max_count_chars as f64 * 20.0).max(40.0) + 24.0; // digits width + padding
    let label_w_mm = label_w_px / px_per_mm;
    let count_w_mm = count_w_px / px_per_mm;
    let board_w_mm = board_bounds.map(|b| b.2 - b.0).unwrap_or(120.0);
    let board_h_mm = board_bounds.map(|b| b.3 - b.1).unwrap_or(100.0);

    let mut total_w_mm = (board_w_mm + label_w_mm + count_w_mm).max(160.0) + pad_mm * 2.0;
    let mut total_h_mm = pad_mm + board_h_mm + pad_mm; // top pad + board + pad
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
        total_w_mm = total_w_mm.max(pad_mm * 2.0 + row_w);
        total_h_mm += row_h + gap_mm;
    }
    total_h_mm += pad_mm;

    // px dims
    let w_px = (total_w_mm * px_per_mm).ceil() as u32;
    let h_px = (total_h_mm * px_per_mm).ceil() as u32;

    let mut s = String::new();
    s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    s.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" stroke=\"#333\" fill=\"none\" stroke-width=\"1.8\" stroke-linejoin=\"round\" font-family=\"sans-serif\" font-size=\"26\">\n",
        w_px, h_px, w_px, h_px
    ));
    s.push_str("<rect x=\"0\" y=\"0\" width=\"100%\" height=\"100%\" fill=\"#ffffff\"/>\n");

    // helpers
    let mm2px = |x: f64| x * px_per_mm;
    let to_px = |p: Point| (mm2px(p.x), mm2px(total_h_mm - p.y)); // y-down SVG space

    // Table separators
    let x_sep1_mm = pad_mm + label_w_mm; // between label and count
    let x_sep2_mm = x_sep1_mm + count_w_mm; // between count and graphics
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
    // Draw vertical separators (full height, inside margins)
    draw_vline(&mut s, x_sep1_mm, pad_mm, total_h_mm - pad_mm);
    draw_vline(&mut s, x_sep2_mm, pad_mm, total_h_mm - pad_mm);
    // Top horizontal line
    draw_hline(&mut s, pad_mm);

    // Draw board (first row of table: left text, right graphic)
    let mut cursor_y_mm = pad_mm;
    if !board_geom.is_empty() {
        let (minx, miny, maxx, maxy) = board_bounds.unwrap();
        let bw = maxx - minx;
        let bh = maxy - miny;
        // Graphics area starts after second separator; add a small column gap
        let col_gap_mm = 2.0;
        let gfx_left_mm = x_sep2_mm + col_gap_mm;
        let gfx_w_mm = total_w_mm - pad_mm - gfx_left_mm;
        let left_mm = gfx_left_mm + ((gfx_w_mm - bw) / 2.0).max(0.0);
        let geom = translate_geom(&board_geom, -minx + left_mm, -miny + cursor_y_mm);
        s.push_str(&path_from_points(&geom, &to_px));
        // Board label (with dimensions) in left label column
        if let Some(b) = &p.board {
            let wtxt = fmt_mm(bw);
            let htxt = fmt_mm(bh);
            let rtxt = b.r.unwrap_or(0.0);
            let lx = mm2px(pad_mm + 2.0);
            let base_y_px = mm2px(total_h_mm - (cursor_y_mm + bh / 2.0));
            let mut lines: Vec<String> = Vec::new();
            if let Some(ls) = &b.label_lines
                && !ls.is_empty()
            {
                lines = ls.clone();
            }
            if lines.is_empty() {
                if let Some(lbl) = &b.label {
                    lines.push(lbl.clone());
                } else if rtxt > 0.0 {
                    lines.push(format!("Board {}×{} mm (R{})", wtxt, htxt, fmt_mm(rtxt)));
                } else {
                    lines.push(format!("Board {}×{} mm", wtxt, htxt));
                }
            }
            let n = lines.len() as i32;
            let line_gap_px: f64 = 34.0; // line gap (px)
            for (i, txt) in lines.into_iter().enumerate() {
                let idx = i as i32;
                let dy = (idx - (n - 1) / 2) as f64 * line_gap_px;
                let ly = base_y_px + dy;
                s.push_str(&format!(
                    "<text x=\"{:.2}\" y=\"{:.2}\" fill=\"#333\" font-size=\"30\">{}</text>\n",
                    lx,
                    ly,
                    svg_escape(&txt)
                ));
            }
        }
        // Only keep text, no dimension leader/arrow graphics
        cursor_y_mm += bh + pad_mm;
        // Horizontal line after board row
        draw_hline(&mut s, cursor_y_mm);
    }

    // Draw grouped rows with labels + count + graphics
    let mut row_top = cursor_y_mm;
    for ((label, items), row_h) in groups.into_iter().zip(row_heights.into_iter()) {
        // Label column (left-aligned)
        s.push_str(&format!(
            "<text x=\"{:.2}\" y=\"{:.2}\" fill=\"#333\" font-size=\"26\">{}</text>\n",
            mm2px(pad_mm + 2.0),
            mm2px(total_h_mm - (row_top + row_h / 2.0)),
            svg_escape(&label)
        ));
        // Count column (centered)
        let cx_mm = (x_sep1_mm + x_sep2_mm) / 2.0;
        s.push_str(&format!(
            "<text x=\"{:.2}\" y=\"{:.2}\" text-anchor=\"middle\" fill=\"#333\" font-size=\"26\">{}</text>\n",
            mm2px(cx_mm),
            mm2px(total_h_mm - (row_top + row_h / 2.0)),
            items.len()
        ));
        let col_gap_mm = 2.0;
        let mut x_mm = x_sep2_mm + col_gap_mm; // start after second separator with gap
        for it in items {
            let (minx, miny, maxx, maxy) = it.bounds;
            let w = maxx - minx;
            let _h = maxy - miny;
            let g = translate_geom(&it.geom, -minx + x_mm, -miny + row_top);
            s.push_str(&path_from_points(&g, &to_px));
            x_mm += w + gap_mm;
        }
        row_top += row_h + gap_mm;
        // Horizontal line after each group row
        draw_hline(&mut s, row_top);
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

// Format millimeters:
// - Near-integers (1e-6) as integers
// - Else up to 3 decimals, trim trailing zeros
fn fmt_mm(v: f64) -> String {
    if (v - v.round()).abs() < 1e-6 {
        format!("{:.0}", v)
    } else {
        format!("{:.3}", v)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

fn label_for_piece(p: &Piece) -> String {
    let f = |v: f64| -> String { fmt_mm(v) };
    match p.type_.as_str() {
        "circle" => {
            let d = p.d.unwrap_or_else(|| p.r.unwrap_or(0.0) * 2.0);
            format!("Circle (diameter {} mm)", f(d))
        }
        "rect" => {
            let w = p.w.unwrap_or(0.0);
            let h = p.h.unwrap_or(0.0);
            if (w - h).abs() < 1e-6 {
                format!("Square (side {} mm)", f(w))
            } else {
                format!("Rectangle ({}×{} mm)", f(w), f(h))
            }
        }
        "regular_polygon" => {
            let n = p.n.unwrap_or(3);
            let side = p.side.unwrap_or(0.0);
            match n {
                5 => format!("Regular pentagon (side {} mm)", f(side)),
                6 => format!("Regular hexagon (side {} mm)", f(side)),
                _ => format!("Regular {}-gon (side {} mm)", n, f(side)),
            }
        }
        "equilateral_triangle" => format!(
            "Equilateral triangle (side {} mm)",
            f(p.side.unwrap_or(0.0))
        ),
        "right_triangle" => format!(
            "Right triangle (legs {}×{} mm)",
            f(p.a.unwrap_or(0.0)),
            f(p.b.unwrap_or(0.0))
        ),
        "isosceles_trapezoid" => format!(
            "Isosceles trapezoid (bottom {} mm, top {} mm, height {} mm)",
            f(p.base_bottom.unwrap_or(0.0)),
            f(p.base_top.unwrap_or(0.0)),
            f(p.height.unwrap_or(0.0))
        ),
        "parallelogram" => format!(
            "Parallelogram (base {} mm, top offset {} mm, height {} mm)",
            f(p.base.unwrap_or(0.0)),
            f(p.offset_top.unwrap_or(0.0)),
            f(p.height.unwrap_or(0.0))
        ),
        _ => p.type_.clone(),
    }
}

fn label_from_catalog_or_fallback(p: &Piece) -> String {
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
    label_for_piece(p)
}

fn svg_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
