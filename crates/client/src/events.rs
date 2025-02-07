use js_sys::Date;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::Element;
use yew::prelude::*;

use super::{clear_results, escape, open};
use crate::components::ResultListData;
use crate::constants;

pub fn handle_global_key_down(
    event: &Event,
    node_ref: UseStateHandle<NodeRef>,
    lens: UseStateHandle<Vec<String>>,
    query: UseStateHandle<String>,
    search_results: UseStateHandle<Vec<ResultListData>>,
    selected_idx: UseStateHandle<usize>,
) {
    let event = event.dyn_ref::<web_sys::KeyboardEvent>().unwrap_throw();
    // Search result navigation
    if event.key() == "ArrowDown" {
        event.stop_propagation();
        let max_len = if search_results.is_empty() {
            0
        } else {
            search_results.len() - 1
        };
        selected_idx.set((*selected_idx + 1).min(max_len));
    } else if event.key() == "ArrowUp" {
        event.stop_propagation();
        let new_idx = (*selected_idx).max(1) - 1;
        selected_idx.set(new_idx);
    } else if event.key() == "Enter" {
        let selected: &ResultListData = (*search_results).get(*selected_idx).unwrap();
        if let Some(url) = selected.url.clone() {
            spawn_local(async move {
                open(url).await.unwrap();
            });
        // Otherwise we're dealing w/ a lens, add to lens vec
        } else {
            // Add lens to list
            let mut new_lens = lens.to_vec();
            new_lens.push(selected.title.to_string());
            lens.set(new_lens);
            // Clear query string
            query.set("".to_string());
            // Clear results list
            let el = node_ref.cast::<Element>().unwrap();
            clear_results(search_results, el);
        }
    } else if event.key() == "Escape" {
        spawn_local(async move {
            escape().await.unwrap();
        });
    } else if event.key() == "Backspace" {
        event.stop_propagation();
        if query.is_empty() && !lens.is_empty() {
            log::info!("updating lenses");
            let all_but_last = lens[0..lens.len() - 1].to_vec();
            lens.set(all_but_last);
        }

        if query.len() < crate::constants::MIN_CHARS {
            // Clear results list
            let el = node_ref.cast::<Element>().unwrap();
            clear_results(search_results, el);
        }
    }
}

pub fn handle_query_change(
    query: &str,
    query_debounce: UseStateHandle<f64>,
    node_ref: UseStateHandle<NodeRef>,
    lens: UseStateHandle<Vec<String>>,
    search_results: UseStateHandle<Vec<ResultListData>>,
    selected_idx: UseStateHandle<usize>,
) {
    // Was the last char typed > 1 sec ago?
    let is_debounced = *query_debounce >= constants::DEBOUNCE_TIME_MS;

    if is_debounced && query.len() >= constants::MIN_CHARS {
        let el = node_ref.cast::<Element>().unwrap();
        if query.starts_with(constants::LENS_SEARCH_PREFIX) {
            // show lens search
            super::show_lens_results(search_results, el, selected_idx, query.to_string());
        } else {
            super::show_doc_results(search_results, &lens, el, selected_idx, query.to_string());
        }
    }

    query_debounce.set(Date::now());
}
