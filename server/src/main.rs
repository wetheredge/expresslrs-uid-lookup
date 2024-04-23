use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use axum::routing::get;
use elrs_rainbow_table::Table;
use serde_json::json;

async fn find(State(table): State<Arc<Table<'_>>>, Path(uid): Path<String>) -> impl IntoResponse {
    let Some(uid) = elrs_rainbow_table::parse_uid(&uid) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Malformed uid" })),
        );
    };

    let response = if let Some(binding_phrase) = table.find(uid) {
        let Ok(binding_phrase) = std::str::from_utf8(binding_phrase) else {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Server error" })),
            );
        };

        json!({ "found": true, "bindingPhrase": binding_phrase })
    } else {
        json!({ "found": false })
    };

    (StatusCode::OK, Json(response))
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let table = if std::path::Path::new(elrs_rainbow_table::TABLE).exists() {
        let table = std::fs::read(elrs_rainbow_table::TABLE).unwrap();
        Table::parse(table.leak())
    } else {
        let words = elrs_rainbow_table::fetch_words().unwrap();
        Table::from_words(words.leak()).unwrap()
    };
    println!("Loaded {} entries", table.len());

    let router = axum::Router::new()
        .route("/:uid", get(find))
        .with_state(Arc::new(table));

    Ok(router.into())
}
