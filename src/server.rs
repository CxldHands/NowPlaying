use axum::{
    Router,
    extract::{Query, Request, State},
    http::{StatusCode, header::HOST},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, watch};

use crate::state::NowPlayingState;

// ─────────────────────────────────────────────
// Shared Types
// ─────────────────────────────────────────────

type AppState = Arc<Mutex<NowPlayingState>>;

#[derive(Debug, Deserialize)]
struct OverlayQuery {
    skin: Option<String>,
}

#[derive(Debug, Serialize)]
struct NowPlayingResponse {
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    artwork_version: u64,
    position_ms: i64,
    duration_ms: i64,
    is_playing: bool,
}

// ─────────────────────────────────────────────
// Control Page
// ─────────────────────────────────────────────

async fn control() -> impl IntoResponse {
    (
        [("Content-Type", "text/html; charset=utf-8")],
        include_str!("../overlay/control.html"),
    )
}

async fn control_style() -> impl IntoResponse {
    (
        [("Content-Type", "text/css; charset=utf-8")],
        include_str!("../overlay/control.css"),
    )
}

async fn control_script() -> impl IntoResponse {
    (
        [("Content-Type", "application/javascript; charset=utf-8")],
        include_str!("../overlay/control.js"),
    )
}

// ─────────────────────────────────────────────
// Overlay
// ─────────────────────────────────────────────

async fn overlay(Query(query): Query<OverlayQuery>) -> impl IntoResponse {
    let skin = query.skin.as_deref().unwrap_or("default");

    let html = match skin {
        "ambient" => include_str!("../overlay/skins/ambient/index.html"),
        _ => include_str!("../overlay/skins/default/index.html"),
    };

    ([("Content-Type", "text/html; charset=utf-8")], html)
}

// ─────────────────────────────────────────────
// Default Skin
// ─────────────────────────────────────────────

async fn default_style() -> impl IntoResponse {
    (
        [("Content-Type", "text/css; charset=utf-8")],
        include_str!("../overlay/skins/default/style.css"),
    )
}

async fn default_script() -> impl IntoResponse {
    (
        [("Content-Type", "application/javascript; charset=utf-8")],
        include_str!("../overlay/skins/default/skin.js"),
    )
}

// ─────────────────────────────────────────────
// Ambient Skin
// ─────────────────────────────────────────────

async fn ambient_style() -> impl IntoResponse {
    (
        [("Content-Type", "text/css; charset=utf-8")],
        include_str!("../overlay/skins/ambient/style.css"),
    )
}

async fn ambient_script() -> impl IntoResponse {
    (
        [("Content-Type", "application/javascript; charset=utf-8")],
        include_str!("../overlay/skins/ambient/skin.js"),
    )
}

// ─────────────────────────────────────────────
// Preview Assets
// ─────────────────────────────────────────────

async fn default_preview_art() -> ([(&'static str, &'static str); 1], &'static [u8]) {
    (
        [("Content-Type", "image/webp")],
        include_bytes!("../overlay/assets/previews/default.webp"),
    )
}

async fn ambient_preview_art() -> ([(&'static str, &'static str); 1], &'static [u8]) {
    (
        [("Content-Type", "image/webp")],
        include_bytes!("../overlay/assets/previews/ambient.webp"),
    )
}

// ─────────────────────────────────────────────
// Shared Assets
// ─────────────────────────────────────────────

async fn app_script() -> impl IntoResponse {
    (
        [("Content-Type", "application/javascript; charset=utf-8")],
        include_str!("../overlay/app.js"),
    )
}

async fn utils() -> impl IntoResponse {
    (
        [("Content-Type", "application/javascript; charset=utf-8")],
        include_str!("../overlay/utils.js"),
    )
}

async fn gsap() -> impl IntoResponse {
    (
        [("Content-Type", "application/javascript; charset=utf-8")],
        include_str!("../overlay/gsap.min.js"),
    )
}

async fn favicon() -> impl IntoResponse {
    (
        [("Content-Type", "image/svg+xml")],
        include_str!("../overlay/assets/favicon.svg"),
    )
}

// ─────────────────────────────────────────────
// Now Playing API
// ─────────────────────────────────────────────

async fn now_playing(State(state): State<AppState>) -> impl IntoResponse {
    let current = state.lock().await;

    let response = NowPlayingResponse {
        title: current.title.clone(),
        artist: current.artist.clone(),
        album: current.album.clone(),
        artwork_version: current.artwork_version,
        position_ms: current.current_position_ms(),
        duration_ms: current.duration_ms,
        is_playing: current.is_playing,
    };

    axum::Json(response)
}

// ─────────────────────────────────────────────
// Artwork API
// ─────────────────────────────────────────────

async fn artwork(State(state): State<AppState>) -> impl IntoResponse {
    let current = state.lock().await;

    let Some(data) = current.artwork.as_ref() else {
        return (
            StatusCode::NOT_FOUND,
            [("Content-Type", "text/plain")],
            Vec::new(),
        )
            .into_response();
    };

    let content_type = current
        .artwork_type
        .as_deref()
        .unwrap_or("image/jpeg")
        .to_string();

    (
        StatusCode::OK,
        [
            ("Content-Type", content_type.as_str()),
            ("Cache-Control", "no-cache"),
        ],
        data.clone(),
    )
        .into_response()
}

/// Reject requests addressed to unexpected hostnames.
async fn validate_host(request: Request, next: Next) -> Response {
    let host = request
        .headers()
        .get(HOST)
        .and_then(|value| value.to_str().ok())
        .map(str::to_ascii_lowercase);

    match host.as_deref() {
        Some("localhost:3000" | "127.0.0.1:3000") => next.run(request).await,
        _ => StatusCode::BAD_REQUEST.into_response(),
    }
}

// ─────────────────────────────────────────────
// Server
// ─────────────────────────────────────────────

pub async fn start_server(
    state: AppState,
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<(), std::io::Error> {
    let app = Router::new()
        // Control page
        .route("/", get(control))
        .route("/control.css", get(control_style))
        .route("/control.js", get(control_script))
        // Overlay
        .route("/overlay", get(overlay))
        // Default skin
        .route("/skins/default/style.css", get(default_style))
        .route("/skins/default/skin.js", get(default_script))
        // Ambient skin
        .route("/skins/ambient/style.css", get(ambient_style))
        .route("/skins/ambient/skin.js", get(ambient_script))
        // Preview assets
        .route("/assets/previews/default.webp", get(default_preview_art))
        .route("/assets/previews/ambient.webp", get(ambient_preview_art))
        // Shared assets
        .route("/app.js", get(app_script))
        .route("/utils.js", get(utils))
        .route("/gsap.min.js", get(gsap))
        .route("/favicon.svg", get(favicon))
        // API
        .route("/now-playing", get(now_playing))
        .route("/artwork", get(artwork))
        .with_state(state)
        .layer(middleware::from_fn(validate_host));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    println!("HTTP server running at http://localhost:3000");

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.changed().await;
            println!("HTTP server shutting down.");
        })
        .await?;

    Ok(())
}
