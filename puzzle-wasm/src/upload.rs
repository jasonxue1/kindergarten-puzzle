use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Event, FileReader, HtmlInputElement, Window};

use crate::{
    CountsSpec, Puzzle, ShapesCatalog, State, asset_url, assign_piece_colors,
    build_puzzle_from_counts, draw, log, update_note_dom, update_status_dom,
};

// Shared loader for puzzle JSON text (counts format or full puzzle)
pub async fn load_puzzle_from_text(state: Rc<RefCell<State>>, text: String) {
    if text.is_empty() {
        log("Selected file is empty or unreadable");
        return;
    }

    // Try parse as counts+shapes first; fall back to full Puzzle
    if let Ok(spec) = serde_json::from_str::<CountsSpec>(&text) {
        // Fetch shapes file if provided; else try server shapes.json, fallback to bundled
        let st_clone = state.clone();
        let win: Window = state.borrow().window.clone();
        let shapes_text = if let Some(sf) = spec.shapes_file.clone() {
            match wasm_bindgen_futures::JsFuture::from(win.fetch_with_str(&sf)).await {
                Ok(resp_value) => match resp_value.dyn_into::<web_sys::Response>() {
                    Ok(resp) => {
                        match wasm_bindgen_futures::JsFuture::from(resp.text().unwrap()).await {
                            Ok(t) => t.as_string().unwrap_or_default(),
                            Err(_) => include_str!("../../shapes.json").to_string(),
                        }
                    }
                    Err(_) => include_str!("../../shapes.json").to_string(),
                },
                Err(_) => include_str!("../../shapes.json").to_string(),
            }
        } else {
            // try base-prefixed then root, else fallback
            // asset_url uses window.__BASE_URL for deployments under subpaths
            let urls = [asset_url("shapes.json"), "/shapes.json".to_string()];
            let mut txt: Option<String> = None;
            for u in &urls {
                if let Ok(v) = wasm_bindgen_futures::JsFuture::from(win.fetch_with_str(u)).await {
                    if let Ok(resp) = v.dyn_into::<web_sys::Response>() {
                        if resp.ok() {
                            if let Ok(p) = resp.text() {
                                if let Ok(t) = wasm_bindgen_futures::JsFuture::from(p).await {
                                    txt = t.as_string();
                                }
                            }
                        }
                    }
                }
                if txt.is_some() {
                    break;
                }
            }
            txt.unwrap_or_else(|| include_str!("../../shapes.json").to_string())
        };
        match serde_json::from_str::<ShapesCatalog>(&shapes_text) {
            Ok(catalog) => {
                let p = build_puzzle_from_counts(&spec, &catalog);
                let mut s = st_clone.borrow_mut();
                s.data = p;
                s.shapes_catalog = Some(catalog);
                assign_piece_colors(&mut s.data);
                s.initial_data = s.data.clone();
                update_note_dom(&s);
                update_status_dom(&s);
                draw(&mut s);
            }
            Err(e) => {
                log(&format!("Failed to parse shapes catalog: {}", e));
            }
        }
    } else if let Ok(p) = serde_json::from_str::<Puzzle>(&text) {
        let mut s = state.borrow_mut();
        s.data = p;
        assign_piece_colors(&mut s.data);
        s.initial_data = s.data.clone();
        update_note_dom(&s);
        update_status_dom(&s);
        draw(&mut s);
    } else {
        log("Unrecognized puzzle JSON format");
        let _ = state
            .borrow()
            .window
            .alert_with_message("Unrecognized puzzle JSON format.");
    }
}

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
                let st_clone = st2.clone();
                wasm_bindgen_futures::spawn_local(load_puzzle_from_text(st_clone, text));
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
