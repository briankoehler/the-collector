use axum::Json;
use serde_json::{json, Value};

pub mod account;
pub mod r#match;
pub mod match_stats;

pub type JsonString = Json<Value>;

pub fn jsonify(foo: impl serde::Serialize) -> JsonString {
    Json(json!(foo))
}
