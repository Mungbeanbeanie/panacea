//! Bio-Digital Defense — Rust core entry point.
//!
//! Registers Tauri commands and event channels, then hands off to the autonomous engine
//! (agents, evolution, ledger). See ../.claude/docs/architecture.md.

mod agents;
mod core;
mod evolution;
mod ledger;

fn main() {
    // TODO(M0): wire the Tauri builder + command/event registration here. Generate the
    // Tauri scaffold with `cargo tauri init` so versions/config are pinned from the
    // toolchain, not guessed (see ../.claude/docs/working-agreement.md, Rule 4).
}
