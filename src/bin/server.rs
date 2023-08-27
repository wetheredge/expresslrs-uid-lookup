use std::fs;
use std::sync::{Arc, OnceLock};

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use axum::routing::get;
use elrs_rainbow_table::Table;
use serde_json::json;

#[derive(Debug, Clone)]
enum RawData {
    Words(Vec<u8>),
    Table(Vec<u8>),
}

static RAW_DATA: OnceLock<RawData> = OnceLock::new();

#[tokio::main]
async fn main() {
    let data = RAW_DATA.get_or_init(|| {
        if std::path::Path::new(elrs_rainbow_table::TABLE).exists() {
            RawData::Table(fs::read(elrs_rainbow_table::TABLE).unwrap())
        } else {
            RawData::Words(elrs_rainbow_table::fetch_words().unwrap())
        }
    });
    let table = match data {
        RawData::Words(words) => Table::from_words(words).unwrap(),
        RawData::Table(table) => Table::parse(table),
    };
    println!("Loaded {} entries", table.len());

    let app = axum::Router::new()
        .route("/:uid", get(find))
        .with_state(Arc::new(table));

    println!("Running on 0.0.0.0:3000");

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

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

        Json(json!({ "found": true, "bindingPhrase": binding_phrase }))
    } else {
        Json(json!({ "found": false }))
    };

    (StatusCode::OK, response)
}
