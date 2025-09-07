use std::cell::RefCell;
use std::rc::Rc;

use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement, Window};

use crate::models::{Puzzle, ShapesCatalog};

/// Global application state stored behind an `Rc<RefCell<_>>` so it can be
/// shared across the WASM callbacks.
#[derive(Clone)]
pub struct State {
    pub window: Window,
    pub document: Document,
    pub canvas: HtmlCanvasElement,
    pub ctx: CanvasRenderingContext2d,
    pub data: Puzzle,
    pub puzzle_name: String,
    pub dragging_idx: Option<usize>,
    pub drag_off: (f64, f64),
    pub scale: f64,
    pub offset: (f64, f64),
    pub rot_vel: f64,
    pub slow_mode: bool,
    pub rot_speed_fast: f64,
    pub rot_speed_slow: f64,
    pub restrict_mode: bool,
    pub shift_down: bool,
    pub initial_data: Puzzle,
    pub lang: String,
    pub shapes_catalog: Option<ShapesCatalog>,
}

/// Thread local storage for the single runtime state instance.
thread_local! {
    pub static STATE: RefCell<Option<Rc<RefCell<State>>>> = const { RefCell::new(None) };
}
