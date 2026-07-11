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
real-time, crowd-sourced Bio-Digital Defense System powered by a unified blockchain
backbone.

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
[STAGE 4: Consensus & Ledger Commitment] ──► ZK-Proof Validated via PoI
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

If the mutation passes the regression matrix, the local node generates a **Zero-Knowledge
Proof (ZK-Proof)** confirming that the gene neutralizes the specific threat signature and
passed the allergy check — without exposing raw host details or the malware payload itself.
Network validators verify the proof via **Proof of Immunity (PoI)** consensus and commit it
to the blockchain.

## 3. The 3-Ledger Blockchain Architecture

The blockchain is three highly specialized, interacting state tables running on top of a
single, unified ledger layer. This separates targeting telemetry from execution payload,
letting local endpoints behave as **ultra-light clients**.

### Ledger 1 — The State Ledger (Cryptographic Core)

- **Role:** the absolute source of truth and security backbone.
- **Data:** chronological block headers, transaction timestamps, validator signatures, and
  **Merkle Roots** of both the Threat Registry and the Genome Registry.
- **Mechanism:** local endpoints download only this ledger. To verify that a threat or a
  cure is legitimate, they don't read a giant linked list — they request a tiny
  cryptographic path from the network and verify it against the local Merkle Root. This
  mitigates blockchain data bloat on the user's PC.

### Ledger 2 — The Threat Registry (Scout Database)

- **Role:** the collective memory tracking what malware looks like across the world.
- **Data:** `Threat_ID` (cryptographic hash of the behavioral vector); `Behavioral_Schema`
  (the specific sequence of syscalls and network ports flagged in Stage 1);
  `Confidence_Score` (an integer counting how many independent Scouts globally have seen
  this trajectory).
- **Mechanism:** when a Scout on Machine A logs a new vector, it appends to this ledger.
  When Machine B's Scout sees a matching trajectory, the ledger correlates them, increments
  the confidence score, and triggers a network-wide mobilization command once the threshold
  is crossed.

### Ledger 3 — The Genome Registry (Soldier Instructions)

- **Role:** the global pharmacy containing the cryptographic cures.
- **Data:** mapping `Threat_ID → Wasm_Gene_Hash`; `IPFS_URI` (the distributed file system
  address where the compiled, sandboxed Wasm exploit binary actually sits);
  `Epigenetic_Status` (a binary flag: 0 = active expression, 1 = suppressed).
- **Mechanism:** Soldier agents never scan this registry passively. When a local Scout
  alerts a Soldier to a specific `Threat_ID` matching Ledger 2, the Soldier queries Ledger 3
  for that specific row. It verifies the `Wasm_Gene_Hash` against the State Ledger's Merkle
  root, downloads the featherweight bytecode from IPFS, runs it to kill the virus, and then
  undergoes apoptosis.

## Safeguard — Epigenetic Suppression Tokens

If an allergy slips past Stage 3 and starts breaking a legitimate program in the wild,
consensus nodes do **not** hard-fork the blockchain. They broadcast an **Epigenetic
Suppressor Token** to Ledger 3, updating the `Epigenetic_Status` flag of that specific Gene
ID to **1**. Local Soldiers reading this table instantly stop executing that cure,
neutralizing the global allergy in seconds. This path is first-class: it stays fast and
simple, and a Soldier must check `Epigenetic_Status` *before* fetching or running a gene.

---

Related: [Security & Safety](security.md) · [Build Roadmap](plan.md) · [Glossary](glossary.md)
