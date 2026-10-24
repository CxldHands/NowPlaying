use axum::{
    Router,
    extract::{Json, Query, Request, State},
    http::{StatusCode, header::HOST},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, watch};

use crate::config::save_selected_source;
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

#[derive(Debug, Serialize)]
struct SourcesResponse {
    selected_source: String,
    available_sources: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SelectSourceRequest {
    source: String,
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
        "glass" => include_str!("../overlay/skins/glass/index.html"),
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
// Glass Skin
// ─────────────────────────────────────────────

async fn glass_style() -> impl IntoResponse {
    (
        [("Content-Type", "text/css; charset=utf-8")],
        include_str!("../overlay/skins/glass/style.css"),
    )
}

async fn glass_script() -> impl IntoResponse {
    (
        [("Content-Type", "application/javascript; charset=utf-8")],
        include_str!("../overlay/skins/glass/skin.js"),
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

async fn glass_preview_art() -> ([(&'static str, &'static str); 1], &'static [u8]) {
    (
        [("Content-Type", "image/webp")],
        include_bytes!("../overlay/assets/previews/glass.webp"),
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
// Media Sources API
// ─────────────────────────────────────────────

async fn sources(State(state): State<AppState>) -> impl IntoResponse {
    let current = state.lock().await;

    let mut available_sources: Vec<String> = current.sessions.values().cloned().collect();

    available_sources.sort();
    available_sources.dedup();

    axum::Json(SourcesResponse {
        selected_source: current.selected_source.clone(),
        available_sources,
    })
}

// ─────────────────────────────────────────────
// Media Source Selection API
// ─────────────────────────────────────────────

async fn select_source(
    State(state): State<AppState>,
    Json(payload): Json<SelectSourceRequest>,
) -> Response {
    let source = payload.source.trim().to_string();

    if source.is_empty() {
        return (StatusCode::BAD_REQUEST, "Source name cannot be empty").into_response();
    }

    let mut current = state.lock().await;

    let matching_session = current
        .sessions
        .iter()
        .find(|(_, session_source)| session_source.as_str() == source)
        .map(|(session_id, _)| session_id.clone());

    if current.selected_source == source
        && matching_session.is_some()
        && current.active_session_id == matching_session
    {
        let mut available_sources: Vec<String> = current.sessions.values().cloned().collect();

        available_sources.sort();
        available_sources.dedup();

        return axum::Json(SourcesResponse {
            selected_source: current.selected_source.clone(),
            available_sources,
        })
        .into_response();
    }

    let preference_changed = current.selected_source != source;
    current.selected_source = source.clone();

    match matching_session.as_deref() {
        Some(session_id) => {
            current.activate_session(session_id);
        }
        None => {
            current.clear();
        }
    }

    let is_available = current.active_session_id.is_some();

    let mut available_sources: Vec<String> = current.sessions.values().cloned().collect();

    available_sources.sort();
    available_sources.dedup();

    let response = SourcesResponse {
        selected_source: current.selected_source.clone(),
        available_sources,
    };

    drop(current);

    if preference_changed {
        let source_to_save = source.clone();

        match tokio::task::spawn_blocking(move || save_selected_source(&source_to_save)).await {
            Ok(Ok(())) => {
                println!("Saved media source preference: {source}");
            }
            Ok(Err(error)) => {
                eprintln!("Could not save media source preference: {error}");
            }
            Err(error) => {
                eprintln!("Configuration save task failed: {error}");
            }
        }
    }

    println!(
        "Selected media source: {source} ({})",
        if is_available {
            "available"
        } else {
            "waiting for session"
        }
    );

    axum::Json(response).into_response()
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
        // Glass skin
        .route("/skins/glass/style.css", get(glass_style))
        .route("/skins/glass/skin.js", get(glass_script))
        // Preview assets
        .route("/assets/previews/default.webp", get(default_preview_art))
        .route("/assets/previews/ambient.webp", get(ambient_preview_art))
        .route("/assets/previews/glass.webp", get(glass_preview_art))
        // Shared assets
        .route("/app.js", get(app_script))
        .route("/utils.js", get(utils))
        .route("/gsap.min.js", get(gsap))
        .route("/favicon.svg", get(favicon))
        // API
        .route("/now-playing", get(now_playing))
        .route("/sources", get(sources))
        .route("/sources/select", post(select_source))
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
