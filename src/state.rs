use serde::Serialize;

// ─────────────────────────────────────────────
// Application State
// ─────────────────────────────────────────────

#[derive(Debug, Default, Serialize)]
pub struct NowPlayingState {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub artwork: Option<Vec<u8>>,
    pub artwork_type: Option<String>,
    pub position: i64,
    pub duration: i64,
    pub is_playing: bool,
}
