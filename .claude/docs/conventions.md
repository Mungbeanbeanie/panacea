# Coding Conventions

Guidelines, not merge blockers (this is a PoC). Follow them by default; they keep the code
readable and the branches mergeable.

## Rust (core)

- Prefer `Result<T, E>` with typed, descriptive errors (e.g. `thiserror`) over
  `unwrap()`/`expect()` in non-test code, especially in shared paths. Reserve panics for
  truly unrecoverable states.
- Keep functions small and single-purpose; keep modules focused on one caste/stage/ledger
  concern. Cross-module access goes through the domain `mod.rs` facade.
- Document every public item with `///` doc comments: what it does, its inputs/outputs, its
  failure modes, and any invariant it upholds.
- Make concurrency and shared state explicit. Any process suspension, memory cloning, or
  sandbox handle has a clear owner and a defined teardown path.
- Run `cargo fmt`; don't hand-format. Never reformat code outside your task's scope.

## React / JSX (dashboard)

- Function components and hooks only. One component per file; keep components presentational
  and push data logic into hooks (`useTauriEvents.js` is the pattern to follow).
- The UI holds no security logic and no source of truth — it mirrors Rust state.
- Document each component with a short header comment: what it visualizes and which Rust
  event stream or command feeds it.
- Handle the "no data yet / stream dropped" state on every live view; never assume events
  have already arrived.

## Documentation hygiene (everywhere)

- Comments and docs describe **current** behavior and intent, in the present tense.
- **No historical narration.** Ban "now uses", "no longer", "changed to", "previously",
  "updated to", "as before". If a reader needs history, they read Git.
- Delete dead code and stale comments rather than annotating them as old.
- Name things by their real function first; the biological metaphor is the flavor, the
  [Glossary](glossary.md) is the bridge.
