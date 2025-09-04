use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Event, FileReader, HtmlInputElement, Window};

use crate::{draw, build_puzzle_from_counts, log, CountsSpec, Puzzle, ShapesCatalog, State};

// Wires up the file input handler for loading JSON puzzle files.
pub fn attach_file_input(state: Rc<RefCell<State>>) -> Result<(), JsValue> {
    let doc: Document = state.borrow().document.clone();
    if let Some(input) = doc.get_element_by_id("file") {
        let input: HtmlInputElement = input.dyn_into().unwrap();
        let st = state.clone();
        // Clone references that will be moved into closures
        let input_for_closure = input.clone();
        let onchange = Closure::<dyn FnMut(Event)>::wrap(Box::new(move |_e: Event| {
            let files = input_for_closure.files();
            if files.is_none() {
                log("No file list on input");
                return;
            }
            let files = files.unwrap();
            if files.length() == 0 {
                log("No file selected");
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
                if text.is_empty() {
                    log("Selected file is empty or unreadable");
                    return;
                }
                // Try parse as full Puzzle; fall back to counts+shapes
                if let Ok(p) = serde_json::from_str::<Puzzle>(&text) {
                    {
                        let mut s = st2.borrow_mut();
                        s.data = p;
                        draw(&mut s);
                    }
                } else if let Ok(spec) = serde_json::from_str::<CountsSpec>(&text) {
                    // Fetch shapes file if provided; else fallback to bundled shapes
                    let st3 = st2.clone();
                    let win: Window = st2.borrow().window.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let shapes_text = if let Some(sf) = spec.shapes_file.clone() {
                            match wasm_bindgen_futures::JsFuture::from(win.fetch_with_str(&sf)).await {
                                Ok(resp_value) => {
                                    match resp_value.dyn_into::<web_sys::Response>() {
                                        Ok(resp) => match wasm_bindgen_futures::JsFuture::from(resp.text().unwrap()).await {
                                            Ok(t) => t.as_string().unwrap_or_default(),
                                            Err(_) => include_str!("../../shapes.json").to_string(),
                                        },
                                        Err(_) => include_str!("../../shapes.json").to_string(),
                                    }
                                }
                                Err(_) => include_str!("../../shapes.json").to_string(),
                            }
                        } else {
                            include_str!("../../shapes.json").to_string()
                        };
                        match serde_json::from_str::<ShapesCatalog>(&shapes_text) {
                            Ok(catalog) => {
                                let p = build_puzzle_from_counts(&spec, &catalog);
                                let mut s = st3.borrow_mut();
                                s.data = p;
                                draw(&mut s);
                            }
                            Err(e) => {
                                log(&format!("Failed to parse shapes catalog: {e}"));
                                let _ = st3.borrow().window.alert_with_message(
                                    "无法解析 shapes.json，请检查文件格式。",
                                );
                            }
                        }
                    });
                } else {
                    log("Unrecognized puzzle JSON format");
                    let _ = st2
                        .borrow()
                        .window
                        .alert_with_message("无法识别的拼图 JSON 文件。");
                }
            }));
            reader.set_onload(Some(onload.as_ref().unchecked_ref()));
            if let Err(e) = reader.read_as_text(&file) {
                log(&format!("Failed to read file: {:?}", e));
            }
            onload.forget();
        }));
        input.set_onchange(Some(onchange.as_ref().unchecked_ref()));
        onchange.forget();
    }
    Ok(())
}

