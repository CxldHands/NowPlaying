use gsmtc::{ManagerEvent, SessionManager};
use std::sync::Arc;
use tokio::sync::{watch, Mutex};

use crate::state::NowPlayingState;

// ─────────────────────────────────────────────
// Configuration
// ─────────────────────────────────────────────

// Only media sources in this list are allowed to update our state.
//
// For now we're only supporting Spotify.
// Other apps such as Chrome or VLC will be ignored.
const ALLOWED_SOURCES: &[&str] = &["Spotify.exe"];

// ─────────────────────────────────────────────
// SMTC
// ─────────────────────────────────────────────

pub async fn start_smtc(
    state: Arc<Mutex<NowPlayingState>>,
    mut shutdown_rx: watch::Receiver<bool>,
) {
    let mut events = SessionManager::create()
        .await
        .expect("Failed to create SMTC session manager");

    loop {
        tokio::select! {
            Some(event) = events.recv() => {
                match event {
                    ManagerEvent::SessionCreated {
                        session_id,
                        mut rx,
                        source,
                    } => {
                        let allowed = ALLOWED_SOURCES.contains(&source.as_str());

                        println!(
                            "{} media source: {source}",
                            if allowed { "Accepted" } else { "Ignoring" }
                        );

                        let state = Arc::clone(&state);

                        tokio::spawn(async move {
                            while let Some(update) = rx.recv().await {
                                // We still consume events from ignored sessions,
                                // but they are not allowed to modify our state.
                                if !allowed {
                                    continue;
                                }

                                {
                                    let mut current = state.lock().await;

                                    match update {
                                        gsmtc::SessionUpdateEvent::Model(model) => {
                                            if let Some(media) = model.media {
                                                current.title = Some(media.title);
                                                current.artist = Some(media.artist);
                                                current.album =
                                                    media.album.map(|album| album.title);
                                            }

                                            if let Some(playback) = model.playback {
                                                current.is_playing = matches!(
                                                    playback.status,
                                                    gsmtc::PlaybackStatus::Playing
                                                );
                                            }

                                            if let Some(timeline) = model.timeline {
                                                current.position_ms = timeline.position / 10_000;
                                                current.duration_ms = timeline.end / 10_000;

                                                if current.is_playing {
                                                    current.position_anchor = Some(std::time::Instant::now());
                                                } else {
                                                    current.position_anchor = None;
                                                }
                                            }
                                        }

                                        gsmtc::SessionUpdateEvent::Media(model, image) => {
                                            if let Some(media) = model.media {
                                                current.title = Some(media.title);
                                                current.artist = Some(media.artist);
                                                current.album =
                                                    media.album.map(|album| album.title);
                                            }

                                            if let Some(image) = image {
                                                current.artwork = Some(image.data);
                                                current.artwork_type =
                                                    Some(image.content_type);
                                            }
                                        }
                                    }
                                }

                                // Debug output so we can see what the application
                                // is receiving while developing.
                                let current = state.lock().await;

                                println!("\n--- Now Playing ---");

                                if let Some(title) = &current.title {
                                    println!("Song:   {title}");
                                }

                                if let Some(artist) = &current.artist {
                                    println!("Artist: {artist}");
                                }

                                if let Some(album) = &current.album {
                                    println!("Album:  {album}");
                                }

                                println!(
                                    "Status: {}",
                                    if current.is_playing {
                                        "Playing"
                                    } else {
                                        "Paused"
                                    }
                                );

                                println!("Position: {}ms", current.current_position_ms());
                                println!("Duration: {}ms", current.duration_ms);

                                if let Some(artwork_type) = &current.artwork_type {
                                    let size = current
                                        .artwork
                                        .as_ref()
                                        .map(|data| data.len())
                                        .unwrap_or(0);

                                    println!(
                                        "Artwork: {artwork_type} ({size} bytes)"
                                    );
                                }
                            }

                            println!("Session {session_id} ended.");
                        });
                    }

                    ManagerEvent::SessionRemoved { session_id } => {
                        println!("Session removed: {session_id}");
                    }

                    ManagerEvent::CurrentSessionChanged { session_id } => {
                        println!("Current session: {session_id:?}");
                    }
                }
            }

            _ = shutdown_rx.changed() => {
                println!("Shutdown signal received by SMTC.");
                break;
            }
        }
    }

    println!("SMTC stopped.");
}