# Collaboration — team of 3, parallel branches

Goal: three people build features at once without stepping on each other at merge time.
The modular file layout does most of the work; this doc names the conventions and the
few collision points to watch.

## Branching model

- Short-lived **feature branches off `main`**, one feature per branch.
- Small, frequent PRs — merge while the branch is still a few files wide.
- **Rebase on `main` before opening/merging** a PR so conflicts surface on your branch,
  not in the shared history.
- Land the M0 scaffolding (see [Plan](plan.md)) together *first*, before branches diverge.

## Directory ownership

The module tree maps cleanly to people, so branches rarely touch the same files. A
sensible default split (swap by feature as needed):

| Owner | Directories | Roughly |
|---|---|---|
| **A** | `src-tauri/src/agents/`, `src-tauri/src/core/` | Scouts, Soldiers, shared types |
| **B** | `src-tauri/src/evolution/`, `src-tauri/src/ledger/`, `programs/` | Sandbox/fuzzing, the three ledgers, the Anchor on-chain program |
| **C** | `landing/` | The observability dashboard |

Cross-module access goes through each domain's `mod.rs` facade — so B can rely on A's
`agents` surface without editing A's files.

## Merge hotspots (a few files everyone eventually touches)

When the code exists, these are the shared-edit points. Coordinate here specifically:

- **`src-tauri/src/main.rs`** — Tauri command + event registration. Keep registrations
  grouped and append new ones at the end of the list so diffs don't overlap.
- **`src-tauri/Cargo.toml`**, **`landing/package.json`**, and **`Anchor.toml`**/`programs/*/Cargo.toml` —
  dependency and deployment-config lists. Add your crate/package on its own line; merge
  these PRs promptly. `Anchor.toml`'s program ID changes on every fresh devnet deploy —
  coordinate before redeploying so you don't invalidate a teammate's cached ID.
- **`src-tauri/tauri.conf.json`** — windows and capabilities. Narrow, grouped edits.

Rule of thumb: a PR that only touches one owner's directory is safe to review at leisure;
a PR that touches a hotspot should be merged fast before it drifts.

## Status lives outside the repo

Track who's doing what in **GitHub issues / a project board**, not in the repo docs.
`CLAUDE.md` and [plan.md](plan.md) are the *stable* roadmap — editing them constantly
turns them into merge-churn. Update them only when the plan itself changes.
