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
    pub artwork_version: u64,
    pub active_session_id: Option<String>,

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
            artwork_version: 0,
            active_session_id: None,
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

        if self.duration_ms > 0 {
            position_ms.min(self.duration_ms)
        } else {
            position_ms
        }
    }

    pub fn set_playing(&mut self, is_playing: bool) {
        if self.is_playing == is_playing {
            return;
        }

        // Preserve the interpolated position when pausing.
        if !is_playing {
            self.position_ms = self.current_position_ms();
        }

        self.is_playing = is_playing;

        // Start a fresh position anchor when resuming.
        self.position_anchor = if is_playing {
            Some(Instant::now())
        } else {
            None
        };
    }

    pub fn clear(&mut self) {
        self.title = None;
        self.artist = None;
        self.album = None;

        self.artwork = None;
        self.artwork_type = None;
        self.artwork_version += 1;
        self.active_session_id = None;

        self.position_ms = 0;
        self.duration_ms = 0;
        self.is_playing = false;
        self.position_anchor = None;
    }
}
