extern crate wasm_bindgen;

use apollo_query_planner::QueryPlanner;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = getQueryPlan)]
pub fn get_query_plan(schema: &str, query: &str) -> JsValue {
    let query_planner = QueryPlanner::new(schema);
    JsValue::from_serde(&query_planner.plan(query).unwrap()).unwrap()
}

// This is what we really want, however we can't use wasm-bindgen
// with things that have lifetimes.
// Adding a #[wasm-bindgen] to the QueryPlanner struct and
// impl yields a compile error that says exactly that.
// This restriction is arguably not that important when caching
// comes into play, as we'll only be building a query plan once
// in rust. This means it's not so important that the QueryPlanner
// instance is handed over to JS - instead we can just return the
// serialized query plan as we do up above.

// #[wasm_bindgen(js_name = getQueryPlanner)]
// pub fn get_query_planner(schema: &str) -> JsValue {
//     let query_planner = QueryPlanner::new(schema);
//     JsValue::from_serde(&query_planner).unwrap()
// }