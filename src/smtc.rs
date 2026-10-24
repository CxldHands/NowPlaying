use gsmtc::{ManagerEvent, SessionManager};
use std::sync::Arc;
use tokio::sync::{Mutex, watch};

use crate::state::NowPlayingState;

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

                        {
                            let mut current = state.lock().await;

                            current.sessions.insert(
                                session_key.clone(),
                                source.clone(),
                            );

                            current
                                .session_snapshots
                                .entry(session_key.clone())
                                .or_default();

                            if current.selected_source == source {
                                current.activate_session(&session_key);
                            }
                        }

                        println!(
                            "{} media source: {source}",
                            {
                                let current = state.lock().await;

                                if current.selected_source == source {
                                    "Selected"
                                } else {
                                    "Discovered"
                                }
                            }
                        );

                        let state = Arc::clone(&state);
                        let source_name = source.clone();

                        tokio::spawn(async move {
                            while let Some(update) = rx.recv().await {
                                let mut current = state.lock().await;

                                // Ignore updates from sessions that have
                                // already been removed.
                                if !current.sessions.contains_key(&session_key) {
                                    continue;
                                }

                                match update {
                                    gsmtc::SessionUpdateEvent::Model(model) => {
                                        {
                                            let snapshot = current
                                                .session_snapshots
                                                .entry(session_key.clone())
                                                .or_default();

                                            if let Some(media) = model.media {
                                                snapshot.title = Some(media.title);
                                                snapshot.artist = Some(media.artist);
                                                snapshot.album =
                                                    media.album.map(|album| album.title);
                                            }

                                            if let Some(playback) = model.playback {
                                                snapshot.is_playing = matches!(
                                                    playback.status,
                                                    gsmtc::PlaybackStatus::Playing
                                                );

                                                snapshot.position_anchor =
                                                    if snapshot.is_playing {
                                                        Some(std::time::Instant::now())
                                                    } else {
                                                        None
                                                    };
                                            }

                                            if let Some(timeline) = model.timeline {
                                                snapshot.position_ms = timeline
                                                    .position
                                                    .saturating_sub(timeline.start)
                                                    .max(0) / 10_000;

                                                snapshot.duration_ms = timeline
                                                    .end
                                                    .saturating_sub(timeline.start)
                                                    .max(0) / 10_000;

                                                snapshot.position_anchor =
                                                    if snapshot.is_playing {
                                                        Some(std::time::Instant::now())
                                                    } else {
                                                        None
                                                    };
                                            }
                                        }

                                        if current.active_session_id.as_deref()
                                            == Some(session_key.as_str())
                                        {
                                            let snapshot = current
                                                .session_snapshots
                                                .get(&session_key)
                                                .cloned()
                                                .expect("Session snapshot should exist");

                                            current.title = snapshot.title;
                                            current.artist = snapshot.artist;
                                            current.album = snapshot.album;
                                            current.position_ms = snapshot.position_ms;
                                            current.duration_ms = snapshot.duration_ms;
                                            current.is_playing = snapshot.is_playing;
                                            current.position_anchor = snapshot.position_anchor;
                                        }
                                    }

                                    gsmtc::SessionUpdateEvent::Media(model, image) => {
                                        {
                                            let snapshot = current
                                                .session_snapshots
                                                .entry(session_key.clone())
                                                .or_default();

                                            if let Some(media) = model.media {
                                                snapshot.title = Some(media.title);
                                                snapshot.artist = Some(media.artist);
                                                snapshot.album =
                                                    media.album.map(|album| album.title);
                                            }

                                            match image {
                                                Some(image) => {
                                                    snapshot.artwork = Some(image.data);
                                                    snapshot.artwork_type =
                                                        Some(image.content_type);
                                                }
                                                None => {
                                                    snapshot.artwork = None;
                                                    snapshot.artwork_type = None;
                                                }
                                            }
                                        }

                                        if current.active_session_id.as_deref()
                                            == Some(session_key.as_str())
                                        {
                                            let snapshot = current
                                                .session_snapshots
                                                .get(&session_key)
                                                .cloned()
                                                .expect("Session snapshot should exist");

                                            current.title = snapshot.title;
                                            current.artist = snapshot.artist;
                                            current.album = snapshot.album;
                                            current.artwork = snapshot.artwork;
                                            current.artwork_type = snapshot.artwork_type;
                                            current.artwork_version += 1;
                                        }
                                    }
                                }
                            }

                            // Remove this session's cached data when its
                            // receiver ends.
                            let mut current = state.lock().await;

                            current.sessions.remove(&session_key);
                            current.session_snapshots.remove(&session_key);

                            if current.active_session_id.as_deref()
                                == Some(session_key.as_str())
                            {
                                current.clear();
                            }

                            println!(
                                "Session {session_key} ended ({source_name})."
                            );
                        });
                    }

                    ManagerEvent::SessionRemoved { session_id } => {
                        let session_key = session_id.to_string();
                        let mut current = state.lock().await;

                        current.sessions.remove(&session_key);
                        current.session_snapshots.remove(&session_key);

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
