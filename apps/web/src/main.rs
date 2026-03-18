use axum::{
    Router,
    body::Bytes,
    http::{HeaderMap, StatusCode},
    routing::post,
};
use serde_json::Value;
use tokio::net::TcpListener;

async fn handle_log(path: &str, headers: HeaderMap, body: Bytes) -> StatusCode {
    let api_key = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok());

    match api_key {
        Some("password") => {
            match serde_json::from_slice::<Value>(&body) {
                Ok(json) => println!("[{}] Received log: {}", path, json),
                Err(_) => println!("[{}] Received log (raw): {}", path, String::from_utf8_lossy(&body)),
            }
            StatusCode::OK
        }
        _ => StatusCode::UNAUTHORIZED,
    }
}

async fn logs(headers: HeaderMap, body: Bytes) -> StatusCode {
    handle_log("/logs", headers, body).await
}

async fn log1(headers: HeaderMap, body: Bytes) -> StatusCode {
    handle_log("/log1", headers, body).await
}

async fn log2(headers: HeaderMap, body: Bytes) -> StatusCode {
    handle_log("/log2", headers, body).await
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/logs", post(logs))
        .route("/log1", post(log1))
        .route("/log2", post(log2));

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Web server listening on 0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
}
