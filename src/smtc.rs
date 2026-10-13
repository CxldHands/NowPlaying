use gsmtc::{ManagerEvent, SessionManager};
use std::sync::Arc;
use tokio::sync::{Mutex, watch};

use crate::state::NowPlayingState;

// ─────────────────────────────────────────────
// Configuration
// ─────────────────────────────────────────────

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
                        let session_key = session_id.to_string();
                        let allowed = ALLOWED_SOURCES.contains(&source.as_str());

                        println!(
                            "{} media source: {source}",
                            if allowed { "Accepted" } else { "Ignoring" }
                        );

                        // Register the newly accepted session.
                        if allowed {
                            let mut current = state.lock().await;

                            current.clear();
                            current.active_session_id = Some(session_key.clone());
                        }

                        let state = Arc::clone(&state);

                        tokio::spawn(async move {
                            while let Some(update) = rx.recv().await {
                                if !allowed {
                                    continue;
                                }

                                let mut current = state.lock().await;

                                // Ignore updates from sessions that no longer
                                // own the displayed playback state.
                                if current.active_session_id.as_deref()
                                    != Some(session_key.as_str())
                                {
                                    continue;
                                }

                                match update {
                                    gsmtc::SessionUpdateEvent::Model(model) => {
                                        if let Some(media) = model.media {
                                            current.title = Some(media.title);
                                            current.artist = Some(media.artist);
                                            current.album = media.album.map(|album| album.title);
                                        }

                                        if let Some(playback) = model.playback {
                                            let is_playing = matches!(
                                                playback.status,
                                                gsmtc::PlaybackStatus::Playing
                                            );

                                            current.set_playing(is_playing);
                                        }

                                        if let Some(timeline) = model.timeline {
                                            // Convert 100-nanosecond ticks to milliseconds.
                                            current.position_ms = timeline.position.saturating_sub(timeline.start).max(0) / 10_000;
                                            current.duration_ms = timeline.end.saturating_sub(timeline.start).max(0) / 10_000;

                                            current.position_anchor = if current.is_playing {
                                                Some(std::time::Instant::now())
                                            } else {
                                                None
                                            };
                                        }
                                    }

                                    gsmtc::SessionUpdateEvent::Media(model, image) => {
                                        if let Some(media) = model.media {
                                            current.title = Some(media.title);
                                            current.artist = Some(media.artist);
                                            current.album = media.album.map(|album| album.title);
                                        }

                                        match image {
                                            Some(image) => {
                                                current.artwork = Some(image.data);
                                                current.artwork_type = Some(image.content_type);
                                            }
                                            None => {
                                                current.artwork = None;
                                                current.artwork_type = None;
                                            }
                                        }

                                        current.artwork_version += 1;
                                    }
                                }
                            }

                            // Clear state only if this session still owns it.
                            let mut current = state.lock().await;

                            if current.active_session_id.as_deref()
                                == Some(session_key.as_str())
                            {
                                current.clear();
                            }

                            println!("Session {session_key} ended.");
                        });
                    }

                    ManagerEvent::SessionRemoved { session_id } => {
                        let session_key = session_id.to_string();
                        let mut current = state.lock().await;

                        if current.active_session_id.as_deref()
                            == Some(session_key.as_str())
                        {
                            current.clear();
                        }

                        println!("Session removed: {session_key}");
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
