use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::state::NowPlayingState;

// ─────────────────────────────────────────────
// Playback Clock
// ─────────────────────────────────────────────
//
// SMTC gives us timeline updates when something changes,
// but it doesn't continuously send the current position.
//
// While playback is active, we therefore advance our
// local position once per second.

pub fn start_playback_clock(state: Arc<Mutex<NowPlayingState>>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;

            let mut current = state.lock().await;

            if current.is_playing && current.duration > 0 {
                current.position += 1;

                if current.position > current.duration {
                    current.position = current.duration;
                }
            }
        }
    });
}
