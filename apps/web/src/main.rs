use axum::{
    Router,
    body::Bytes,
    extract::Path,
    http::{HeaderMap, StatusCode},
    routing::post,
};
use serde_json::Value;
use tokio::net::TcpListener;

async fn handle_log(headers: HeaderMap, Path(path): Path<String>, body: Bytes) -> StatusCode {
    let api_key = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok());

    match api_key {
        Some("password") => {
            match serde_json::from_slice::<Value>(&body) {
                Ok(json) => println!("[/{}] Received log: {}", path, json),
                Err(_) => println!("[/{}] Received log (raw): {}", path, String::from_utf8_lossy(&body)),
            }
            StatusCode::OK
        }
        _ => StatusCode::UNAUTHORIZED,
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/{path}", post(handle_log));

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Web server listening on 0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
}
