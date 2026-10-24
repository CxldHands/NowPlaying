use std::{collections::HashMap, time::Instant};

// ─────────────────────────────────────────────
// Session Snapshot
// ─────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct SessionSnapshot {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub artwork: Option<Vec<u8>>,
    pub artwork_type: Option<String>,
    pub position_ms: i64,
    pub duration_ms: i64,
    pub is_playing: bool,
    pub position_anchor: Option<Instant>,
}

impl SessionSnapshot {
    pub fn current_position_ms(&self) -> i64 {
        if !self.is_playing {
            return self.position_ms;
        }

        let Some(anchor) = self.position_anchor else {
            return self.position_ms;
        };

        let elapsed_ms = anchor.elapsed().as_millis() as i64;
        let position_ms = self.position_ms.saturating_add(elapsed_ms);

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

        if !is_playing {
            self.position_ms = self.current_position_ms();
        }

        self.is_playing = is_playing;
        self.position_anchor = if is_playing {
            Some(Instant::now())
        } else {
            None
        };
    }
}

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

    // Media source management
    pub selected_source: String,
    pub sessions: HashMap<String, String>,
    pub session_snapshots: HashMap<String, SessionSnapshot>,

    // Playback information
    pub position_ms: i64,
    pub duration_ms: i64,
    pub is_playing: bool,
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
            selected_source: "Spotify.exe".to_string(),
            sessions: HashMap::new(),
            session_snapshots: HashMap::new(),
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

    // ─────────────────────────────────────────────
    // Active Playback State
    // ─────────────────────────────────────────────

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

    pub fn activate_session(&mut self, session_id: &str) {
        self.clear();
        self.active_session_id = Some(session_id.to_string());

        if let Some(snapshot) = self.session_snapshots.get(session_id).cloned() {
            self.title = snapshot.title;
            self.artist = snapshot.artist;
            self.album = snapshot.album;
            self.artwork = snapshot.artwork;
            self.artwork_type = snapshot.artwork_type;
            self.position_ms = snapshot.position_ms;
            self.duration_ms = snapshot.duration_ms;
            self.is_playing = snapshot.is_playing;
            self.position_anchor = snapshot.position_anchor;

            self.artwork_version += 1;
        }
    }
}
