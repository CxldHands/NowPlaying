#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod server;
mod smtc;
mod state;
mod tray;

use server::start_server;
use smtc::start_smtc;
use state::NowPlayingState;
use std::sync::Arc;
use tokio::sync::{watch, Mutex};

#[tokio::main]
async fn main() {
    println!("🎵 NowPlaying");
    println!("---------------");

    // ─────────────────────────────────────────────
    // Shared Application State
    // ─────────────────────────────────────────────

    let state = Arc::new(Mutex::new(NowPlayingState::default()));

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // ─────────────────────────────────────────────
    // Local HTTP Server
    // ─────────────────────────────────────────────

    tokio::spawn(start_server(Arc::clone(&state)));

    // ─────────────────────────────────────────────
    // Windows SMTC
    // ─────────────────────────────────────────────

    std::thread::spawn(move || {
        tray::run_tray(shutdown_tx);
    });

    start_smtc(Arc::clone(&state), shutdown_rx).await;
}
