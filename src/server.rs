use axum::{
    Json, Router,
    extract::{Query, State},
    http::header,
    response::{Html, IntoResponse},
    routing::get,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::NowPlayingState;

// ─────────────────────────────────────────────
// Shared Types
// ─────────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
struct SkinQuery {
    skin: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct NowPlayingResponse {
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    position_ms: i64,
    duration_ms: i64,
    is_playing: bool,
    artwork_version: u64,
}

// ─────────────────────────────────────────────
// Control Page
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

// ─────────────────────────────────────────────
// Overlay
// ─────────────────────────────────────────────

async fn overlay(Query(query): Query<SkinQuery>) -> Html<&'static str> {
    match query.skin.as_deref() {
        Some("ambient") => {
            Html(include_str!("../overlay/skins/ambient/index.html"))
        }

        _ => {
            Html(include_str!("../overlay/skins/default/index.html"))
        }
    }
}

// ─────────────────────────────────────────────
// Default Skin
// ─────────────────────────────────────────────

async fn default_style() -> ([(&'static str, &'static str); 1], &'static str) {
    (
        [("Content-Type", "text/css")],
        include_str!("../overlay/skins/default/style.css"),
    )
}

async fn default_script() -> ([(&'static str, &'static str); 1], &'static str) {
    (
        [("Content-Type", "application/javascript")],
        include_str!("../overlay/skins/default/skin.js"),
    )
}

// ─────────────────────────────────────────────
// Ambient Skin
// ─────────────────────────────────────────────

async fn ambient_style() -> ([(&'static str, &'static str); 1], &'static str) {
    (
        [("Content-Type", "text/css")],
        include_str!("../overlay/skins/ambient/style.css"),
    )
}

async fn ambient_script() -> ([(&'static str, &'static str); 1], &'static str) {
    (
        [("Content-Type", "application/javascript")],
        include_str!("../overlay/skins/ambient/skin.js"),
    )
}

// ─────────────────────────────────────────────
// Shared Assets
// ─────────────────────────────────────────────

async fn app_script() -> ([(&'static str, &'static str); 1], &'static str) {
    (
        [("Content-Type", "application/javascript")],
        include_str!("../overlay/app.js"),
    )
}

async fn utils() -> ([(&'static str, &'static str); 1], &'static str) {
    (
        [("Content-Type", "application/javascript")],
        include_str!("../overlay/utils.js"),
    )
}

async fn gsap() -> ([(&'static str, &'static str); 1], &'static str) {
    (
        [("Content-Type", "application/javascript")],
        include_str!("../overlay/gsap.min.js"),
    )
}

// ─────────────────────────────────────────────
// Now Playing API
// ─────────────────────────────────────────────

async fn now_playing(
    State(state): State<Arc<Mutex<NowPlayingState>>>,
) -> Json<NowPlayingResponse> {
    let current = state.lock().await;

    Json(NowPlayingResponse {
        title: current.title.clone(),
        artist: current.artist.clone(),
        album: current.album.clone(),
        position_ms: current.current_position_ms(),
        duration_ms: current.duration_ms,
        is_playing: current.is_playing,
        artwork_version: current.artwork_version,
    })
}

async fn artwork(
    State(state): State<Arc<Mutex<NowPlayingState>>>,
) -> impl IntoResponse {
    let current = state.lock().await;

    match (&current.artwork, &current.artwork_type) {
        (Some(data), Some(content_type)) => (
            [(header::CONTENT_TYPE, content_type.clone())],
            data.clone(),
        )
            .into_response(),

        _ => axum::http::StatusCode::NOT_FOUND.into_response(),
    }
}

// ─────────────────────────────────────────────
// Server
// ─────────────────────────────────────────────

pub async fn start_server(
    state: Arc<Mutex<NowPlayingState>>,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
    let app = Router::new()
        // Control
        .route("/", get(control))
        .route("/control.css", get(control_style))
        .route("/control.js", get(control_script))
        // Overlay
        .route("/overlay", get(overlay))
        // Default Skin
        .route("/skins/default/style.css", get(default_style))
        .route("/skins/default/skin.js", get(default_script))
        // Ambient Skin
        .route("/skins/ambient/style.css", get(ambient_style))
        .route("/skins/ambient/skin.js", get(ambient_script))
        // Shared Assets
        .route("/app.js", get(app_script))
        .route("/utils.js", get(utils))
        .route("/gsap.min.js", get(gsap))
        // API
        .route("/now-playing", get(now_playing))
        .route("/artwork", get(artwork))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    // Open the control page in the default browser.
    if let Err(error) = std::process::Command::new("cmd")
        .args(["/C", "start", "", "http://localhost:3000"])
        .spawn()
    {
        eprintln!("Could not open browser: {error}");
    }

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.changed().await;
            println!("Shutdown signal received by server.");
        })
        .await
        .unwrap();
}