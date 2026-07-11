//! Ledger facade: the unified interface the rest of the core uses to reach the State
//! Ledger, Threat Registry, and Genome Registry. See ../../.claude/docs/architecture.md.

pub mod client;
pub mod registry;
pub mod state;
