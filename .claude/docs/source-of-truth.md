# Source of Truth — The Definitive Architectural Blueprint

**This file is canonical.** If any doc, code, plan, or agent output contradicts it, this
file wins. When the architecture itself changes, change it *here first*, then update the
code and docs to match. Before implementing any stage, ledger, or agent, read the relevant
section here and conform to it. Run the [`source-of-truth-check`](../skills/source-of-truth-check/SKILL.md)
skill before finishing a task that touches a stage, ledger, or agent.

The biological terms are a naming convention — the [Glossary](glossary.md) maps each to its
engineering meaning. Reason with the engineering meaning.

## Core Paradigm

An autonomous, decentralized, zero-trust anti-virus ecosystem modeled on biological
immunology. It shifts cybersecurity from static, vendor-dependent signature patching to a
real-time, crowd-sourced Bio-Digital Defense System powered by a real blockchain backbone —
**Solana** (devnet for the PoC). The threats themselves stay simulated (synthetic fixtures,
never real malware); the chain, consensus finality, on-chain program, and gene storage are
real, working end to end the way they would in a shippable product.

## 1. The Autonomous Agent Caste System

- **Scout Agents (Sensors).** Continuous, ultra-lightweight background daemons. They do
  **not** kill malware. They monitor low-level behavioral trajectories: syscall anomalies,
  memory-space boundary violations, I/O bursts.
- **Soldier Agents (Executioners).** Heavy, sophisticated security payloads that remain
  completely dormant as un-executed disk **spores** to preserve host resources. They only
  awaken when summoned by a high-confidence threat notification. A Soldier isolates the
  threat, evolves an exploit to kill it, executes it, and undergoes **apoptosis**
  (programmed deletion or re-serialization back into a passive spore).

## 2. The Multi-Stage Verification Pipeline

When an unknown process executes on an endpoint, it must pass through a strict,
deterministic 4-stage pipeline before a global cure is minted.

```
[STAGE 1: Trajectory Scoring] ──► Crossing Anomaly Threshold (100+ pts)
           │
           ▼
[STAGE 2: Local Isolation & Fuzzing] ──► MicroVM Sandbox / Allele Brute-Force
           │
           ▼
[STAGE 3: Lymph Node Regression Matrix] ──► Top 5,000 Apps Multi-OS Testing
           │
           ▼
[STAGE 4: Consensus & Ledger Commitment] ──► 3-of-5 Multisig (PoI) → Solana Devnet
```

### Stage 1 — Behavioral Trajectory Scoring (Scouts)

Instead of searching for file hashes, Scouts track **sequences** of OS instructions. The
system uses a state-transition matrix where sequential actions accumulate anomaly weight:

- **Action A** — spawns a hidden child process from a Temp directory: **+20 pts**.
- **Action B** — enumerates network adapters while attempting to modify the Volume Shadow
  Copy service (`vssadmin.exe`): **+50 pts**.
- **Action C** — rapidly loops through file handles attempting to read/write with high
  entropy: **+40 pts**.

**Threshold:** once a process trajectory exceeds a cumulative score of **100 points**, a
hard system interrupt fires. The Scout suspends all threads of the target process ID (PID)
and signals the local Soldier spore.

### Stage 2 — Local Isolation & Evolutionary Fuzzing (Soldiers)

The Soldier wakes, clones the frozen process's memory space, and replicates it inside a
localized **MicroVM Sandbox** (a Firecracker or lightweight WebAssembly container). It runs
a combinatorial fuzzing matrix using pre-compiled structural **alleles** (safe exploit
primitives), systematically trying combinations — e.g. `Allele_04` (Thread-Context Exit
Token Injection) + `Allele_12` (IPC Pipe Buffer Overflow). Iteration continues until a
combination reliably forces the target process to abort or crash **without destabilizing
the mock host OS**. The successful sequence is compiled into a lightweight **Wasm Gene
Payload**.

### Stage 3 — The Lymph Node Regression Matrix (Allergy Check)

Before the local Wasm Gene can be written to the global ledger, it must be verified as
**non-autoimmune**. The local node submits the mutation to decentralized, privileged
network validators — **Lymph Nodes**. Lymph Nodes run automated server arrays containing
identical sandboxed environments of standard OS bases (Windows/Linux) preloaded with the
**Top 5,000 most common software applications** (Chrome, VS Code, Slack, system daemons).
The proposed Wasm mutation runs inside this crowded environment. If it terminates the
target behavior but causes a single whitelisted application to crash or leak memory, an
**Allergy Flag** is raised and the gene is dropped.

### Stage 4 — Consensus & Ledger Commitment

If the mutation passes the regression matrix, each participating Lymph Node signs an
attestation (with its own persistent Solana keypair) confirming the gene neutralizes the
specific threat signature and passed the allergy check. Once a threshold of **3-of-5**
Lymph Node signatures is collected — **Proof of Immunity (PoI)** — the local node submits a
`commit_gene` instruction to the on-chain Anchor program. The program verifies the multisig
threshold on-chain and, if satisfied, writes the Genome Registry entry for that `Threat_ID`.
Solana's own network consensus (Proof of History + Proof of Stake, via devnet validators)
finalizes the transaction; PoI is an application-level gate enforced by the program, not a
replacement for or competitor to Solana's block-level consensus. PoI does not hide the
malware payload via zero-knowledge cryptography — it's a real multisig attestation, not a
ZK-proof; no raw host telemetry or the malware payload is included in the instruction data
regardless.

## 3. The 3-Ledger Blockchain Architecture

The blockchain is three highly specialized, interacting state tables running on top of a
single, unified ledger layer — **Solana**. This separates targeting telemetry from
execution payload, letting local endpoints behave as **ultra-light clients** (RPC
consumers, not validators).

### Ledger 1 — The State Ledger (Solana's Native Chain)

- **Role:** the absolute source of truth and security backbone.
- **Data:** Solana's own block/account state — chronological blocks, validator signatures,
  and account state roots. No custom chain or custom Merkle scheme is built on top of it.
- **Mechanism:** local endpoints run no validator and download no chain. They act as light
  clients by querying a Solana devnet RPC endpoint (`getAccountInfo` / `getProgramAccounts`)
  at `confirmed` or `finalized` commitment. Trusting a finalized-commitment RPC response is
  the standard Solana light-client trust model — it mitigates blockchain data bloat on the
  user's PC the same way a custom Merkle-path scheme would have, without building one.

### Ledger 2 — The Threat Registry (Scout Database)

- **Role:** the collective memory tracking what malware looks like across the world.
- **Data:** an Anchor **program account (PDA)** keyed by `Threat_ID` (cryptographic hash of
  the behavioral vector), holding a hash of the `Behavioral_Schema` (the specific sequence
  of syscalls and network ports flagged in Stage 1 — hashed rather than stored raw, to keep
  the account fixed-size and rent-cheap; the raw schema stays in the node's local cache) and
  `Confidence_Score` (an integer counting how many independent Scouts globally have seen
  this trajectory).
- **Mechanism:** when a Scout on Machine A logs a new vector, its endpoint signs and submits
  a `submit_threat` instruction with its own devnet keypair, creating the PDA if new. When
  Machine B's Scout sees a matching trajectory, its `submit_threat` call increments
  `Confidence_Score` on the same PDA. Endpoints subscribe to (or poll) the account and
  trigger a network-wide mobilization command once the threshold is crossed.

### Ledger 3 — The Genome Registry (Soldier Instructions)

- **Role:** the global pharmacy containing the cryptographic cures.
- **Data:** an Anchor program account (PDA) mapping `Threat_ID → Wasm_Gene_Hash`; `gene_seq`
  (the compiled allele sequence itself, stored directly in the account — a handful of bytes,
  smaller than an off-chain content address would be, so there's no separate blob store to
  trust or fetch from); `Epigenetic_Status` (a binary flag: 0 = active expression,
  1 = suppressed).
- **Mechanism:** Soldier agents never scan this registry passively. When a local Scout
  alerts a Soldier to a specific `Threat_ID` matching Ledger 2, the Soldier queries Ledger 3
  for that specific PDA via RPC. Reading a finalized account already carries Solana's
  integrity guarantee, so there's no separate proof step — the Soldier decodes the account's
  own `gene_seq` bytes, checks their hash against `Wasm_Gene_Hash`, runs it to kill the
  virus, and then undergoes apoptosis.

## Safeguard — Epigenetic Suppression Tokens

If an allergy slips past Stage 3 and starts breaking a legitimate program in the wild,
consensus nodes do **not** hard-fork Solana. The same 3-of-5 Lymph Node multisig authority
that gates `commit_gene` submits a `suppress_gene` instruction to Ledger 3, updating the
`Epigenetic_Status` flag of that specific Gene ID to **1**. This is a normal Solana
transaction, finalized in seconds. Local Soldiers reading this account on their next RPC
query instantly stop executing that cure, neutralizing the global allergy in seconds. This
path is first-class: it stays fast and simple, and a Soldier must check `Epigenetic_Status`
*before* fetching or running a gene.

---

Related: [Security & Safety](security.md) · [Build Roadmap](plan.md) · [Glossary](glossary.md)
