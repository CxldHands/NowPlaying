use axum::{Json, Router, extract::State, response::Html, routing::get};
use base64::{Engine as _, engine::general_purpose};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::NowPlayingState;

// ─────────────────────────────────────────────
// Overlay Response
// ─────────────────────────────────────────────

#[derive(Debug, serde::Serialize)]
struct NowPlayingResponse {
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    position: i64,
    duration: i64,
    is_playing: bool,
    artwork: Option<String>,
}

// ─────────────────────────────────────────────
// Overlay Routes
// ─────────────────────────────────────────────

async fn control() -> Html<&'static str> {
    Html(include_str!("../overlay/control.html"))
}

async fn control_style() -> ([(&'static str, &'static str); 1], &'static str) {
    (
        [("Content-Type", "text/css")],
        include_str!("../overlay/control.css"),
    )
}

async fn control_script() -> ([(&'static str, &'static str); 1], &'static str) {
    (
        [("Content-Type", "application/javascript")],
        include_str!("../overlay/control.js"),
    )
}

async fn overlay() -> Html<&'static str> {
    Html(include_str!("../overlay/index.html"))
}

async fn style() -> ([(&'static str, &'static str); 1], &'static str) {
    (
        [("Content-Type", "text/css")],
        include_str!("../overlay/style.css"),
    )
}

async fn script() -> ([(&'static str, &'static str); 1], &'static str) {
    (
        [("Content-Type", "application/javascript")],
        include_str!("../overlay/app.js"),
    )
}

async fn now_playing(State(state): State<Arc<Mutex<NowPlayingState>>>) -> Json<NowPlayingResponse> {
    let current = state.lock().await;

    let artwork = match (&current.artwork, &current.artwork_type) {
        (Some(data), Some(content_type)) => {
            let encoded = general_purpose::STANDARD.encode(data);
            Some(format!("data:{content_type};base64,{encoded}"))
        }
        _ => None,
    };

    Json(NowPlayingResponse {
        title: current.title.clone(),
        artist: current.artist.clone(),
        album: current.album.clone(),
        position: current.position,
        duration: current.duration,
        is_playing: current.is_playing,
        artwork,
    })
}

async fn gsap() -> ([(&'static str, &'static str); 1], &'static str) {
    (
        [("Content-Type", "application/javascript")],
        include_str!("../overlay/gsap.min.js"),
    )
}

// ─────────────────────────────────────────────
// Server
// ─────────────────────────────────────────────

pub async fn start_server(state: Arc<Mutex<NowPlayingState>>) {
    let app = Router::new()
        .route("/", get(control))
        .route("/control.css", get(control_style))
        .route("/control.js", get(control_script))
        .route("/overlay", get(overlay))
        .route("/style.css", get(style))
        .route("/app.js", get(script))
        .route("/gsap.min.js", get(gsap))
        .route("/now-playing", get(now_playing))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    // Give Axum a moment to begin accepting requests.
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Open the control page in the default browser.
    std::process::Command::new("cmd")
        .args(["/C", "start", "", "http://localhost:3000"])
        .spawn()
        .expect("Failed to open browser");

    axum::serve(listener, app).await.unwrap();
}
