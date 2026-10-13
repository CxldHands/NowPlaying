#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod instance;
mod server;
mod smtc;
mod state;
mod tray;

use server::start_server;
use smtc::start_smtc;
use state::NowPlayingState;
use std::sync::Arc;
use tokio::sync::{Mutex, watch};

#[tokio::main]
async fn main() {
    println!("🎵 NowPlaying");
    println!("---------------");

    let _instance_guard = match instance::acquire() {
        Some(guard) => guard,
        None => {
            println!("NowPlaying is already running.");
            return;
        }
    };

    // ─────────────────────────────────────────────
    // Shared Application State
    // ─────────────────────────────────────────────

    let state = Arc::new(Mutex::new(NowPlayingState::default()));

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // ─────────────────────────────────────────────
    // Local HTTP Server
    // ─────────────────────────────────────────────

    let server_shutdown_tx = shutdown_tx.clone();

    let server_task = tokio::spawn(start_server(Arc::clone(&state), shutdown_rx.clone()));

    tokio::spawn(async move {
        match server_task.await {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                eprintln!("HTTP server failed: {error}");
                let _ = server_shutdown_tx.send(true);
            }
            Err(error) => {
                eprintln!("HTTP server task failed: {error}");
                let _ = server_shutdown_tx.send(true);
            }
        }
    });

    // ─────────────────────────────────────────────
    // Windows SMTC
    // ─────────────────────────────────────────────

    std::thread::spawn(move || {
        tray::run_tray(shutdown_tx);
    });

    start_smtc(Arc::clone(&state), shutdown_rx).await;
}
