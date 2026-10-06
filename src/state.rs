use std::time::Instant;

// ─────────────────────────────────────────────
// Application State
// ─────────────────────────────────────────────

#[derive(Debug)]
pub struct NowPlayingState {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub artwork: Option<Vec<u8>>,
    pub artwork_type: Option<String>,

    // Playback information
    pub position_ms: i64,
    pub duration_ms: i64,
    pub is_playing: bool,

    // The position reported by SMTC and the moment it was reported.
    pub position_anchor: Option<Instant>,
}

impl Default for NowPlayingState {
    fn default() -> Self {
        Self {
            title: None,
            artist: None,
            album: None,
            artwork: None,
            artwork_type: None,
            position_ms: 0,
            duration_ms: 0,
            is_playing: false,
            position_anchor: None,
        }
    }
}

impl NowPlayingState {
    pub fn current_position_ms(&self) -> i64 {
        if !self.is_playing {
            return self.position_ms;
        }

        let Some(anchor) = self.position_anchor else {
            return self.position_ms;
        };

        let elapsed_ms = anchor.elapsed().as_millis() as i64;

        let position_ms = self.position_ms + elapsed_ms;

        position_ms.min(self.duration_ms)
    }
}