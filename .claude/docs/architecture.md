# Architecture

Reason with the engineering meaning of every term — the [Glossary](glossary.md) is the
bridge. For a hackathon PoC, model each stage faithfully and show it working end-to-end;
mock the heavy pieces (ZK, consensus) where noted rather than building them for real.

## The agent caste system (`src-tauri/src/agents/`)

- **Scouts** (`scout.rs`) — ultra-light background daemons. They observe only; they never
  terminate anything. They trace low-level behavior (syscall anomalies, memory boundary
  violations, I/O bursts) and accumulate an anomaly score per process.
- **Soldiers** (`soldier.rs`) — heavy remediation payloads that stay dormant on disk (as
  passive "spores") to preserve host resources. A Soldier wakes only on a high-confidence
  threat signal, isolates and neutralizes the threat, then performs **apoptosis** (cleans
  itself up / re-serializes back to a dormant spore).

## The four-stage verification pipeline

An unknown process must clear all four stages before a global cure is committed.

```
[STAGE 1] Trajectory Scoring        → cumulative anomaly score crosses 100 pts
    │  (Scouts, src-tauri/src/agents/scout.rs)
    ▼
[STAGE 2] Local Isolation & Fuzzing → clone into MicroVM sandbox, brute-force alleles
    │  (Soldiers + evolution/, src-tauri/src/evolution/)
    ▼
[STAGE 3] Lymph Node Regression     → test candidate cure against top-5,000 apps
    │  (network validators; local node submits, remote validators run)
    ▼
[STAGE 4] Consensus & Commitment    → ZK-proof verified via Proof of Immunity (PoI)
       (src-tauri/src/ledger/)
```

**Stage 1 — Behavioral Trajectory Scoring.** A state-transition matrix assigns weight to
action sequences (e.g. spawning a hidden child from a temp dir, touching Volume Shadow
Copy, high-entropy read/write loops). Crossing 100 points triggers a hard interrupt: the
Scout suspends the target PID's threads and signals the local Soldier spore.

**Stage 2 — Local Isolation & Evolutionary Fuzzing.** The Soldier clones the frozen
process's memory into a MicroVM (e.g. Firecracker or a lightweight Wasm container) and
runs a combinatorial fuzz over pre-compiled **alleles** (safe exploit primitives) until
it finds a combination that reliably aborts the target without destabilizing the mock
host. The winning sequence compiles to a lightweight **Wasm gene** payload.

**Stage 3 — Lymph Node Regression Matrix (allergy check).** Before global publication,
the candidate gene runs inside validator environments preloaded with the ~5,000 most
common applications. If it neutralizes the threat but crashes or leaks a whitelisted app,
an **allergy flag** is raised and the gene is dropped.

**Stage 4 — Consensus & Ledger Commitment.** The node produces a Zero-Knowledge Proof
that the gene neutralizes the threat and passed the allergy check, without exposing raw
host details or the malware payload. Validators verify via **Proof of Immunity** consensus
and commit. *(PoC: stub/mock the ZK proof and consensus; demonstrate the flow and the
committed record.)*

## The three-ledger design (`src-tauri/src/ledger/`)

Three specialized state tables share one unified ledger layer. Endpoints run as
ultra-light clients: they hold only what they need to verify proofs.

- **Ledger 1 — State Ledger (crypto core).** Source of truth: block headers, timestamps,
  validator signatures, and Merkle roots of the Threat and Genome registries. Endpoints
  download only this; they verify any threat or cure by requesting a small Merkle path and
  checking it against the local root — no full-chain download, no data bloat.
- **Ledger 2 — Threat Registry (Scout database).** Collective memory of malware behavior:
  `Threat_ID` (hash of the behavioral vector), `Behavioral_Schema` (the flagged
  syscall/port sequence), and a global `Confidence_Score` incremented as independent
  Scouts observe the same trajectory. Crossing the confidence threshold triggers
  network-wide mobilization.
- **Ledger 3 — Genome Registry (Soldier instructions).** The global "pharmacy":
  `Threat_ID → Wasm_Gene_Hash`, an `IPFS_URI` for the compiled gene binary, and an
  `Epigenetic_Status` flag (0 = active, 1 = suppressed). Soldiers query this by `Threat_ID`
  on demand, verify the hash against the State Ledger root, fetch the bytecode from IPFS,
  execute, then undergo apoptosis.

## Safety kill-switch — Epigenetic Suppression

If an allergy slips past Stage 3 and breaks a legitimate program in the wild, consensus
nodes do **not** fork the chain. They broadcast an **Epigenetic Suppressor Token** that
flips that gene's `Epigenetic_Status` to 1. Soldiers reading the registry immediately stop
executing that cure. Treat this path as first-class: it's the emergency brake, it stays
fast and simple, and a Soldier must check status *before* fetching or running a gene. See
the [`suppression-path-test`](../skills/suppression-path-test/SKILL.md) skill and
[Security](security.md).
