use blueprint_core::PolygonPoint;
use serde::{Deserialize, Serialize};

/// Basic two dimensional point used for geometry operations.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl From<(f64, f64)> for Point {
    fn from(v: (f64, f64)) -> Self {
        Point { x: v.0, y: v.1 }
    }
}

/// Board configuration describing available polygons and size.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Board {
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub w: Option<f64>,
    pub h: Option<f64>,
    pub polygons: Option<Vec<Vec<PolygonPoint>>>,
}

/// Piece definition covering all supported shape variants.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Piece {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub type_: String,
    // common fields
    pub at: Option<[f64; 2]>,
    pub rotation: Option<f64>,
    pub anchor: Option<String>,
    pub flip: Option<bool>,
    // rect
    pub w: Option<f64>,
    pub h: Option<f64>,
    // equilateral_triangle
    pub side: Option<f64>,
    // right_triangle
    pub a: Option<f64>,
    pub b: Option<f64>,
    // regular_polygon
    pub n: Option<u32>,
    // circle
    pub d: Option<f64>,
    pub r: Option<f64>,
    // isosceles_trapezoid
    pub base_bottom: Option<f64>,
    pub base_top: Option<f64>,
    pub height: Option<f64>,
    // parallelogram
    pub base: Option<f64>,
    pub offset_top: Option<f64>,
    // polygon
    pub points: Option<Vec<[f64; 2]>>,
    // cached runtime fields (not serialized)
    #[serde(skip)]
    pub __ctr: Option<Point>,
    #[serde(skip)]
    pub __geom: Option<Vec<Point>>, // for hit-testing
    #[serde(skip)]
    pub __geom_pl: Option<String>, // encoded polyline for debug/interop
    #[serde(skip)]
    pub __color_idx: Option<usize>, // stable color assignment
    #[serde(skip)]
    pub __label_idx: Option<usize>, // stable numeric label (0-based)
}

/// Full puzzle specification including board, pieces and optional notes.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Puzzle {
    pub units: Option<String>,
    pub board: Option<Board>,
    #[serde(default)]
    pub pieces: Vec<Piece>,
    // Optional per-puzzle notes in two languages
    pub note_en: Option<String>,
    pub note_zh: Option<String>,
}

/// Shape metadata used when building puzzles from counts specs.
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
    // Optional human labels (bilingual)
    pub label: Option<String>,
    pub label_en: Option<String>,
    pub label_zh: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ShapesCatalog {
    pub shapes: Vec<ShapeDef>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CountsSpec {
    pub units: Option<String>,
    pub board: Option<Board>,
    pub counts: std::collections::HashMap<String, u32>,
    pub shapes_file: Option<String>,
    pub note_en: Option<String>,
    pub note_zh: Option<String>,
}
