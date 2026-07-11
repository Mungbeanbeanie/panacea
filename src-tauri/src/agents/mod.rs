//! Agent caste facade: traits and shared lifecycle types.
//!
//! Scouts observe and score; Soldiers wake, remediate, and undergo apoptosis.
//! Cross-module callers reach the agents through this facade, not sibling internals.

pub mod scout;
pub mod soldier;
