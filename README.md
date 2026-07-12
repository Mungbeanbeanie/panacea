# Panacea (Bio-Digital Defense)

Panacea is a decentralized, autonomous anti-virus platform modeled on biological immunology
— endpoints detect and neutralize threats locally in an isolated sandbox, then publish
verified cures to a shared blockchain ledger so the whole network gains immunity. It ships
as a Tauri desktop app: a Rust core does all the detection/remediation/consensus work, and a
React + Vite dashboard renders a read-only, live view of what the core is doing.

This is a **hackathon proof-of-concept**. The malware itself and the MicroVM/Wasm sandbox
*host* are simulated (synthetic fixtures, never real malware) — but the blockchain backbone
is real: a real Solana devnet program, real signed transactions, real multisig consensus,
real finality. Nothing about the chain is mocked.

## What is Panacea

Traditional anti-virus relies on centralized, vendor-maintained signature databases —
static, slow to propagate, and single-point-of-trust. Panacea reframes detection and
remediation the way an immune system works:

- **Scouts** are lightweight background daemons that watch process behavior (syscall
  patterns, I/O bursts, network activity) and score it — they never kill anything themselves.
- **Soldiers** are dormant, un-executed payloads ("spores") that wake only when a Scout
  reports a high-confidence threat. A Soldier isolates the threat, evolves a kill sequence
  inside a sandbox, runs it, and self-terminates ("apoptosis").
- Once a cure clears every verification stage, it's committed to a **shared on-chain
  ledger** so every other endpoint in the network inherits the immunity instantly, without
  ever having encountered the threat itself.

Every biological term in the codebase maps to a concrete engineering concept — see the
[Domain Glossary](.claude/docs/glossary.md) if a name like "epigenetic status" or
"conjugation" is unclear.

## Architecture

Four moving parts:

**1. The agent caste system** (`src-tauri/src/agents/`)
Scouts observe and score; Soldiers wake, remediate, and self-terminate. Neither ever touches
the live host outside a sandbox.

**2. A 4-stage verification pipeline** — every candidate cure clears all four before it's
published globally:

```
[STAGE 1] Trajectory Scoring        → cumulative anomaly score crosses 100 pts
[STAGE 2] Local Isolation & Fuzzing → MicroVM/Wasm sandbox, combinatorial allele search
[STAGE 3] Lymph Node Regression     → does it break any of a whitelisted app set?
[STAGE 4] Consensus & Commitment    → 3-of-5 multisig ("Proof of Immunity") → Solana devnet
```

Stage 1's state-transition matrix: Action A (+20), Action B (+50), Action C (+40); crossing
**100 cumulative points** hard-suspends the process and wakes the local Soldier.

**3. A 3-ledger design on top of one chain** (`src-tauri/src/ledger/`, `programs/`)
- **Ledger 1 — State Ledger:** Solana's own chain. Endpoints are ultra-light RPC clients
  (no local validator, no custom Merkle scheme) — they trust `confirmed`/`finalized` reads.
- **Ledger 2 — Threat Registry:** an Anchor PDA keyed by `Threat_ID`, tracking a
  `Confidence_Score` that increments as independent Scouts report the same behavioral
  signature — the "crowd-sourced" part of crowd-sourced immunity.
- **Ledger 3 — Genome Registry:** an Anchor PDA mapping `Threat_ID → Wasm_Gene_Hash`, with
  the compiled gene bytes (`gene_seq`) stored directly in the account (no off-chain blob
  store — see [Key Decisions](#key-decisions-made)) and an `Epigenetic_Status` flag.

**4. The kill-switch.** If a published cure turns out to break something legitimate, the
same 3-of-5 multisig authority that approved it can flip `Epigenetic_Status` to `1` via a
real signed transaction. Every Soldier checks that flag *before* fetching or running a gene
— a bad cure stops network-wide within seconds, no chain fork required.

The [Source of Truth](.claude/docs/source-of-truth.md) is the canonical, detailed spec for
all of the above (exact thresholds, ledger fields, instruction semantics) — read it before
changing anything that touches a stage, ledger, or agent.

## Quick Setup

Prerequisites: Rust toolchain, Node.js, and a build toolchain for the C++ test specimens
(`c++`/`g++`). Confirm exact versions against `src-tauri/Cargo.toml` / `landing/package.json`
rather than assuming.

```bash
# 1. Frontend deps
npm install --prefix landing

# 2. Build the benign detector test specimens (real processes the Scout watches)
make -C fake_viruses

# 3. One-time devnet keypair setup (no real funds — devnet faucet only)
solana-keygen new -o ~/.config/solana/id.json
solana airdrop 2 --url devnet

# 4. Run the desktop app in dev mode (hot-reloading UI)
make dev
# — or, if you don't have `make` (e.g. plain Windows shells don't ship it):
npx --prefix landing tauri dev    # run from the repo root, not landing/
```

`make help` lists the other targets (`build`, `check`, `test`, `fmt`, `lint`, `down`, `clean`).
Full command reference: [Build, Run, Test](.claude/docs/build-run-test.md).

**Windows note:** the app builds and opens on Windows (that's what the `windows-latest` CI
leg checks), but the *live* demo won't catch anything there yet — process suspension and the
real behavioral detectors (`agents/scout.rs`) are `#[cfg(unix)]`, with Windows stubbed to
always report "nothing detected." Use WSL2 if you need to see the live catch-and-cure flow.
This is a known, deliberately deprioritized gap (see the Plan's "Beyond the Demo" list) —
platform breadth, not core function.

**Definition of done for this repo:** it compiles and the demo works. Clippy-zero-warnings
and exhaustive test coverage are nice-to-have, not gates.

## Key decisions made

- **The chain is real; only the malware and the sandbox *host* are simulated.** Solana
  devnet, a deployed Anchor program, real signed transactions, and real 3-of-5 multisig
  finality all work end-to-end. What's fake is scoped tightly on purpose: the "virus" is a
  synthetic fixture and the MicroVM host the sandbox isolates against is mocked — never the
  ledger, consensus, or on-chain program.
- **Detection is real, not scripted, for two of three action types.** `agents/scout.rs`
  spawns actual `fake_viruses/virus`/`disease` processes and detects their real behavior via
  `lsof` (fd-count bursts, live network connections) — not a cooperative script that
  announces itself. The third action (`HiddenChildFromTemp`) has no real detector yet and is
  deliberately left unfaked rather than stubbed dishonestly.
- **The sandbox's isolation boundary is real; its target-vulnerability model is not.**
  `evolution/sandbox.rs` runs actual Wasm bytecode through a real `wasmi` runtime, and that
  module provably has *zero* host imports — it cannot touch the live host no matter what
  runs inside it (see `sandbox-isolation-check`). What's still synthetic is which allele
  combinations "win": that verdict is fixed physics compiled into the module (e.g.
  `Allele04`+`Allele12` always crashes the mock target), not derived from actually analyzing
  a real target's real memory.
- **Gene bytes live on-chain; there is no off-chain blob store.** An early design used IPFS
  for gene payloads (`Pinata`, a CID, a fetch-and-verify step). That was deleted: a gene is a
  `Vec<Allele>` over a 3-variant enum — a handful of bytes, smaller than the 64-char CID that
  would have pointed at it. Simpler, one fewer network dependency, one fewer thing that can
  fail out-of-band.
- **`anchor-client` over hand-rolled `solana-client`/`solana-sdk`.** It builds instructions
  from the program's IDL instead of hand-encoding instruction data, which is less code to get
  wrong when the program's account layout changes.
- **Proof of Immunity is a real multisig attestation, not a ZK-proof.** It's an app-level
  gate the Anchor program enforces on top of Solana's own block-level consensus — not a
  replacement for it, and no raw host telemetry or malware payload ever goes into instruction
  data regardless of the gate's mechanism.
- **The frontend holds zero security logic.** `landing/` is strictly observability — it
  renders state streamed from the Rust core (`ecosystem`/`ledger`/`strains`/`stats` Tauri
  events) and never makes a detection, remediation, or consensus decision itself. If a task
  ever seems to need logic in the UI, that's a signal it belongs in Rust instead.
- **Tauri's capability/ACL system is deny-by-default and must be explicitly granted.**
  `src-tauri/capabilities/default.json` grants the main window `core:default` (which
  includes event listen/emit) — without it, every `@tauri-apps/api` call from the frontend
  is silently rejected by the ACL, which is a real gap this project hit (see the git history
  around `useTauriEvents.js` and `capabilities/`).

## Other things worth knowing

- **Directory ownership** (for the 3-person team) maps cleanly to the module tree: agents +
  core → owner A; evolution + ledger + the Anchor program → owner B; `landing/` → owner C.
  See [Collaboration](.claude/docs/collaboration.md) for the branching model and merge
  hotspots (`main.rs`, the three manifest files, `tauri.conf.json`).
- **Safety boundaries that are non-negotiable even in a PoC:** alleles/genes execute only
  inside the sandbox, never against the live host; the suppression path (`Epigenetic_Status`)
  is checked *before* any gene fetch/exec, not after; every on-chain read is at `confirmed`/
  `finalized` commitment; devnet keypairs are treated with real key hygiene and never
  committed. Full detail in [Security & Safety](.claude/docs/security.md).
- **The demo is not idempotent across restarts against the same deployed program.** Threat
  IDs are deterministic hashes of a fixed behavior sequence, and gene suppression is
  permanent by design (no un-suppress path). So only the *first* run against a given program
  deployment shows the full "cure dispensed → then suppressed" story; later restarts replay
  against an already-suppressed entry. Redeploy to a fresh program ID to reset the story.
- **The full phased build roadmap**, including what's explicitly still simulated and the
  ranked list of what a production system would need next, lives in
  [Plan](.claude/docs/plan.md) — see especially "Beyond the Demo" for the honest gap list
  (real sandbox host, real regression corpus, multi-endpoint reality, Windows support, etc.).
- **Advisory reviewers and check skills** live in `.claude/agents/` and `.claude/skills/` —
  `sandbox-isolation-check`, `suppression-path-test`, and `source-of-truth-check` are worth
  running by hand after touching the sandbox, the kill-switch, or any stage/ledger/agent,
  even outside of an AI-assisted workflow.
