# Architecture (orientation)

A quick map of the system. **Authoritative detail lives in
[source-of-truth.md](source-of-truth.md) — read it before implementing any stage, ledger,
or agent.** This page just orients you; the [Glossary](glossary.md) bridges each biological
term to its engineering meaning.

The system has four moving parts:

- **Agent caste system** (`src-tauri/src/agents/`) — **Scouts** observe and score process
  behavior (they never kill); **Soldiers** are dormant on-disk spores that wake, remediate
  inside a sandbox, and undergo apoptosis.
- **The 4-stage verification pipeline** — trajectory scoring (Stage 1) → local isolation &
  evolutionary fuzzing (Stage 2) → Lymph Node allergy check (Stage 3) → consensus & ledger
  commitment (Stage 4). A candidate cure must clear all four before global publication.
- **The 3-ledger design** (`src-tauri/src/ledger/`) — State Ledger (Solana's own chain),
  Threat Registry, and Genome Registry (the latter two as Anchor program accounts on
  Solana devnet), so endpoints run as ultra-light RPC clients, not validators.
- **The kill-switch** — Epigenetic Suppression flips a gene's `Epigenetic_Status` to 1 via
  a real signed Solana transaction, so Soldiers stop executing a harmful cure within
  seconds, without forking the chain.

For a hackathon PoC, model each stage faithfully and show it working end-to-end on a real
Solana devnet backbone — chain, consensus finality, and the on-chain program (gene bytes
included — no off-chain gene storage) are real; only the malware/virus and the MicroVM/Wasm
sandbox host stay simulated. See [Security & Safety](security.md).
