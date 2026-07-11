//! Bio-Digital Defense — Rust core entry point.
//!
//! Registers Tauri commands and event channels, then hands off to the autonomous engine
//! (agents, evolution, ledger). See ../.claude/docs/architecture.md.

mod agents;
mod core;
mod evolution;
mod ledger;

fn main() {
    tauri::Builder::default()
        .setup(|_app| {
            // Scout daemon runs detached for the app lifetime; handle dropped on purpose.
            agents::scout::spawn_demo();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
