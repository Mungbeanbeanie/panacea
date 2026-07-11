//! Bio-Digital Defense — Rust core entry point.
//!
//! Registers Tauri commands and event channels, then hands off to the autonomous engine
//! (agents, evolution, ledger). See ../.claude/docs/architecture.md.

mod agents;
mod core;
mod dashboard;
mod evolution;
mod ledger;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Scout daemon runs detached for the app lifetime; handle dropped on purpose.
            // Spawned on its own thread (not run inline in this callback) because
            // spawn_demo() panics loudly on missing devnet setup (keypairs, network), and a
            // panic unwinding through this macOS AppKit callback aborts the whole process
            // instead of just failing the demo.
            let handle = app.handle().clone();
            std::thread::spawn(move || agents::scout::spawn_demo(handle));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
