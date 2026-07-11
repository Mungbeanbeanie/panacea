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
- **The 3-ledger design** (`src-tauri/src/ledger/`) — State Ledger (crypto core / Merkle
  roots), Threat Registry, and Genome Registry, sharing one unified ledger layer so
  endpoints run as ultra-light clients.
- **The kill-switch** — Epigenetic Suppression flips a gene's `Epigenetic_Status` to 1 so
  Soldiers stop executing a harmful cure instantly, without forking the chain.

For a hackathon PoC, model each stage faithfully and show it working end-to-end; mock the
heavy pieces (real ZK, PoI consensus, live P2P, Firecracker, IPFS) rather than building
them for real. See [Security & Safety](security.md).
