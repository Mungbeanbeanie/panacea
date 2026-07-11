# Current Dev Plan — Phase 11, Steps 1–3 (Solana Migration: Anchor Program Foundations)

> Scratch planning doc only (`.josh/` is gitignored). **Nothing here is implemented until
> the user says "implement."** Conforms to `.claude/docs/source-of-truth.md` (canonical)
> and `.claude/docs/plan.md` Phase 11 (owner B).

## Scope (`.claude/docs/plan.md` Phase 11, boxes 1–3 only)

| # | Phase 11 box | This doc covers it? |
|---|---|---|
| 1 | Anchor workspace (`programs/bio_digital_defense/`) with Threat Registry + Genome Registry account types (PDAs) | ✅ staged below |
| 2 | `submit_threat` instruction: create/update a Threat Registry PDA, increment `Confidence_Score` | ✅ staged below |
| 3 | `commit_gene` instruction: 3-of-5 multisig-gated write to a Genome Registry PDA (Proof of Immunity) | ✅ staged below |
| 4 | `suppress_gene` instruction | ❌ out of scope — next slice |
| 5–11 | Devnet deploy, `ledger/` rewiring, keypair provisioning, real IPFS, test re-run | ❌ out of scope — later slices |

This is purely the **on-chain program side** (new `programs/` crate). It does **not** touch
`src-tauri/src/ledger/{client,state,registry,consensus}.rs` — those stay on their current
mock implementation until a later slice rewires them to call this program over RPC. No
collision with owner A's `agents/`.

## Prerequisite gap (checked this session)

Neither the Solana CLI nor Anchor CLI nor `avm` is installed on this machine (`solana`,
`anchor`, `avm` all resolve to "command not found"; only `cargo`/`rustc` 1.95.0 are
present). Nothing here can actually build or deploy until those are installed. Flagging
this now so it isn't a surprise when "implement" is said — this is a one-time setup step,
not part of the 3 code items above, but blocks them.

## What already exists (the mock this replaces/extends — read this session)

- **`src-tauri/src/ledger/registry.rs`** — `ThreatRegistry` (`report()`/`get()`,
  `MOBILIZATION_THRESHOLD = 2`) and `GenomeRegistry` (`publish()`/`get()`/`suppress()`),
  both in-memory `HashMap<ThreatId, _>`. `ThreatId`/`GeneHandle` are `[u8; 32]`
  (`src-tauri/src/core/mod.rs`) — Sha256 digests, so they drop into Anchor account seeds
  or fields with no conversion.
- **`src-tauri/src/ledger/consensus.rs`** — the current *mock* Stage 4: `ImmunityProof`
  (commitment hash, no real ZK), `POI_QUORUM = 2`, a hardcoded 3-`Validator` set,
  `run_poi_consensus()`, `commit_gene()` (free fn, writes to `GenomeRegistry` + folds it
  into a mock `MockConjugationLink` block). This free function is what the Anchor
  `commit_gene` *instruction* below actually replaces — same name, different layer (Rust
  core fn → on-chain instruction).
- **`src-tauri/src/ledger/client.rs`** — `MockConjugationLink`: P2P stand-in with
  `commit_block`, `broadcast_suppressor`/`drain_suppressors`, `threat_proof`/`genome_proof`.
  Superseded by real Solana RPC calls in a later slice (Phase 11 box 6), not this one.

None of this is deleted or edited by steps 1–3 — it's the reference for what the new
program's behavior needs to match.

## Step 1 — Anchor workspace & account types

New files, no existing file touched:

```
Anchor.toml                              # [provider] cluster = "devnet"; program id (placeholder until first deploy)
programs/
└── bio_digital_defense/
    ├── Cargo.toml                       # anchor-lang dep; crate-type ["cdylib", "lib"]
    └── src/lib.rs                       # declare_id!, account structs, instructions
```

Account types (PDAs), mirroring `registry.rs`'s row shapes:

- **`ThreatEntry`** — seeds `[b"threat", threat_id]`
  - `threat_id: [u8; 32]`
  - `behavioral_schema_hash: [u8; 32]` — **assumption, flagged below**: hash of the
    schema, not the raw `Vec<Action>` sequence, to keep the account fixed-size and
    rent-cheap.
  - `confidence_score: u32`
  - `bump: u8`
- **`GenomeEntry`** — seeds `[b"genome", threat_id]`
  - `threat_id: [u8; 32]`
  - `gene_hash: [u8; 32]`
  - `ipfs_cid: String` (bounded, e.g. max 64 bytes — real CIDs fit)
  - `epigenetic_status: u8` (0 = Active, 1 = Suppressed — matches `EpigeneticStatus` today)
  - `bump: u8`
- **`LymphNodeConfig`** — singleton account, not itemized in plan.md's box 1 wording but
  required to make box 3's multisig check possible: holds the 5 Lymph Node validator
  `Pubkey`s and the threshold (`3`). Initialized once via an `initialize_lymph_nodes`
  instruction (small addition, bundled into this step since box 3 can't work without it).

## Step 2 — `submit_threat` instruction

- **Accounts:** `threat_entry` (`init_if_needed` PDA), `reporter` (`Signer`, pays rent on
  first sighting), `system_program`.
- **Args:** `threat_id: [u8; 32]`, `behavioral_schema_hash: [u8; 32]`.
- **Logic:** mirrors `ThreatRegistry::report()` — if the PDA is being initialized this call,
  set `confidence_score = 1`; otherwise increment it. Reuses `MOBILIZATION_THRESHOLD = 2`
  from `registry.rs` for the "just crossed the line" check (emits an Anchor `event!` rather
  than returning a Rust enum, since there's no caller to hand a `ReportOutcome` to
  on-chain) — client code reads the emitted event or re-checks `confidence_score` after the
  tx confirms to decide whether to mobilize.
- Any funded devnet keypair can call this (every Scout-bearing endpoint signs its own
  report) — no multisig gate here, matching today's `report()` being open to any caller.

## Step 3 — `commit_gene` instruction (Proof of Immunity)

- **Accounts:** `genome_entry` (`init_if_needed` PDA), `lymph_node_config` (read, to check
  the known validator set), `payer`, `system_program`, plus each Lymph Node validator as an
  optional `Signer` account (`remaining_accounts` or 5 named optional `Signer<'info>`
  fields — leaning toward named fields for clarity at 5 validators).
- **Args:** `threat_id: [u8; 32]`, `gene_hash: [u8; 32]`, `ipfs_cid: String`.
- **Logic:** count how many of the 5 known validator pubkeys are present *and* marked
  `is_signer` on the transaction; `require!(count >= 3, ErrorCode::InsufficientQuorum)`.
  If satisfied, write/overwrite the `GenomeEntry` PDA with `epigenetic_status = 0`
  (Active) — this is the on-chain replacement for `consensus::commit_gene()` +
  `GenomeRegistry::publish()` combined.
- Client-side implication (not built in this step, just noted): the node assembling the
  transaction needs signatures from 3+ separate Lymph Node keypairs before submitting —
  i.e. an off-chain step to collect co-signatures first. That collection flow is not part
  of steps 1–3; it's implied client work for a later slice.

## Open questions / assumptions to confirm before "implement"

1. **`behavioral_schema_hash` vs. raw schema** — I'm assuming we store the hash only (rent
   economics) and keep the actual `Vec<Action>`/schema off-chain (e.g. still in the local
   `registry.rs` cache, keyed by the same `ThreatId`). If you want the full schema on-chain,
   the account gets variable-size and more expensive — say so.
2. **`LymphNodeConfig` bundled into step 1** — plan.md's box 1 wording only names Threat +
   Genome account types; I added this third account type because box 3's multisig can't be
   checked without somewhere to store the 5 validator pubkeys. Flagging in case you'd rather
   track it as its own line item.
3. **Named optional `Signer` fields (5) vs. `remaining_accounts` loop** — named fields are
   more explicit/readable in the IDL; `remaining_accounts` is more flexible if the validator
   set size ever changes. Defaulting to named fields for a fixed 3-of-5 unless you'd rather
   keep it dynamic.
4. **Mobilization event on `submit_threat`** — using an Anchor `event!` emission since
   there's no return-value equivalent to `ReportOutcome::Mobilized` on-chain. Confirm that's
   an acceptable substitute for the client-side "trigger network-wide mobilization" behavior.

## Recommendation

Steps 1–3 are self-contained new-file work in `programs/` — no edits to existing
`src-tauri` files, so no merge collision with owner A. The one real blocker is tooling
(Solana CLI / Anchor CLI / `avm` not installed) — that install is a prerequisite, not
optional, once you say "implement."
