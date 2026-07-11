# Build Roadmap

The stable milestone sequence for the hackathon. Ordering respects the pipeline: sensing
before remediation, isolation before publication, and the kill-switch built on the ledger
it suppresses. Keep **day-to-day status in GitHub issues/board**, not here — this file
changes only when the plan itself changes (see [Collaboration](collaboration.md)).

PoC bar throughout: **compiles + demo works** (see [Build, Run, Test](build-run-test.md)).
Mock the heavy pieces (real ZK, live P2P, Firecracker) and show the flow.

| # | Milestone | Key files | Owner | Done when |
|---|---|---|---|---|
| **M0** | **Scaffolding** — Cargo workspace, Tauri 2 init, Vite/React app, CI skeleton, the module tree as stubs | `Cargo.toml`, `tauri.conf.json`, `landing/package.json`, all `mod.rs` | all (together, first) | `npm run tauri dev` opens the window with an empty dashboard |
| **M1** | **Scout + Stage 1** — behavioral tracing, state-transition scoring matrix, 100-pt hard interrupt, suspend PID + signal spore | `agents/scout.rs`, `agents/mod.rs`, `core/mod.rs` | A | a scripted "bad" process crosses 100 pts and gets suspended |
| **M2** | **Soldier lifecycle** — spore serialize/wake/remediate/apoptosis | `agents/soldier.rs` | A | a wake signal spins up a Soldier that acts then re-serializes to spore |
| **M3** | **Evolution / Stage 2** — MicroVM/Wasm isolation, allele matrix, combinatorial fuzz, Wasm gene compile | `evolution/sandbox.rs`, `evolution/alleles.rs`, `evolution/mod.rs` | B | fuzz finds an allele combo that aborts the target inside the sandbox and emits a gene |
| **M4** | **Ledger (light client)** — State Ledger Merkle verify, Threat + Genome registries, conjugation transport | `ledger/state.rs`, `ledger/registry.rs`, `ledger/client.rs`, `ledger/mod.rs` | B | a gene hash verifies against a Merkle root via a fetched path |
| **M5** | **Stages 3 & 4** — Lymph Node regression (allergy check), PoI consensus + ZK commit *(mock ZK + consensus)* | `ledger/`, `evolution/` | A + B | a candidate gene passes the allergy check and a committed record appears |
| **M6** | **Epigenetic suppression kill-switch** — the emergency brake, built on M4 | `ledger/registry.rs`, `agents/soldier.rs` | B | flipping `Epigenetic_Status = 1` halts a cure *before* fetch/exec (keep the [`suppression-path-test`](../skills/suppression-path-test/SKILL.md) check passing) |
| **M7** | **Dashboard** — live views wired to Rust event streams | `landing/src/components/*`, `landing/src/hooks/useTauriEvents.js`, `App.jsx` | C | the three views render live state and handle the no-data state |

C works in parallel from M1 onward against mock event streams, then wires to real ones as
each Rust milestone lands.
