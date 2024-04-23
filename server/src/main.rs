use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Json};
use axum::routing::get;
use elrs_uid_lookup::Table;
use serde_json::json;

async fn index() -> impl IntoResponse {
    Html(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<title>ExpressLRS UID Lookup</title>
</head>
<body>
<h1>ExpressLRS UID Lookup</h1>
<p>Attempts to find an <a href="https://expresslrs.org">ExpressLRS</a> binding phrase for a given uid.</p>
<p>Example: <a href="https://elrs-uid.shuttleapp.rs/65,245,33,230,58,226">https://elrs-uid.shuttleapp.rs/65,245,33,230,58,226</a></p>
<p>See <a href="https://github.com/wetheredge/expresslrs-uid-lookup#readme">the project README</a> for details.</p>
</body>
</html>"#,
    )
}

async fn find(State(table): State<Arc<Table<'_>>>, Path(uid): Path<String>) -> impl IntoResponse {
    let Some(uid) = elrs_uid_lookup::parse_uid(&uid) else {
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
    let table = if std::path::Path::new(elrs_uid_lookup::TABLE).exists() {
        let table = std::fs::read(elrs_uid_lookup::TABLE).unwrap();
        Table::parse(table.leak())
    } else {
        let words = elrs_uid_lookup::fetch_words().unwrap();
        Table::from_words(words.leak()).unwrap()
    };
    println!("Loaded {} entries", table.len());

    let router = axum::Router::new()
        .route("/:uid", get(find))
        .route("/", get(index))
        .with_state(Arc::new(table));

    Ok(router.into())
}
