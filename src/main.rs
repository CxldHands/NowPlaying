#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod playback;
mod server;
mod smtc;
mod state;

use playback::start_playback_clock;
use server::start_server;
use smtc::start_smtc;
use state::NowPlayingState;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    println!("🎵 NowPlaying");
    println!("---------------");

    // ─────────────────────────────────────────────
    // Shared Application State
    // ─────────────────────────────────────────────

    let state = Arc::new(Mutex::new(NowPlayingState::default()));

    // ─────────────────────────────────────────────
    // Playback Clock
    // ─────────────────────────────────────────────

    start_playback_clock(Arc::clone(&state));

    // ─────────────────────────────────────────────
    // Local HTTP Server
    // ─────────────────────────────────────────────

    tokio::spawn(start_server(Arc::clone(&state)));

    // ─────────────────────────────────────────────
    // Windows SMTC
    // ─────────────────────────────────────────────

    start_smtc(Arc::clone(&state)).await;
}
