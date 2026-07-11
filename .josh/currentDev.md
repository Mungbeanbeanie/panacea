# Current Dev Plan — Phase 11, Steps 1–5 (Solana Migration: Anchor Program Foundations)

> Scratch planning doc only (`.josh/` is gitignored). Conforms to
> `.claude/docs/source-of-truth.md` (canonical) and `.claude/docs/plan.md` Phase 11
> (owner B).
>
> **✅ Implemented and verified this session.** `anchor build` compiles clean (no errors,
> no warnings); `target/deploy/bio_digital_defense.so` produced; IDL lists both
> `submit_threat`/`commit_gene` instructions and both `ThreatEntry`/`GenomeEntry` accounts;
> `anchor keys list` matches `declare_id!`/`Anchor.toml`. Not yet deployed to devnet
> (box 5) and no client-side wiring (box 6+) — that's the next slice.

## What changed from the staged plan during implementation

- `programs/bio_digital_defense/Cargo.toml` needed
  `anchor-lang = { version = "1.1.2", features = ["init-if-needed"] }` — the plain
  `"1.1.2"` version string alone fails to compile (`init_if_needed` requires the feature
  flag explicitly; its absence produced a cascade of ~30 downstream `Bumps`/`Accounts`
  trait errors that all trace back to this one root cause).
- 5 real devnet keypairs generated at `keys/lymph-nodes/validator-{1..5}.json` (gitignored)
  for the Lymph Node validators; their pubkeys are hardcoded in `constants.rs` per the
  reviewed plan. The program's own keypair is at `target/deploy/bio_digital_defense-keypair.json`
  (also gitignored via the existing `target/` rule), pubkey `FPn8ftGPZ5fr6rH3qtttBp2ppx7HNV4zscGhkFSv8t5x`,
  matching `declare_id!` and `Anchor.toml`'s `[programs.devnet]` entry.
- Root `Anchor.toml` and `Cargo.toml` (workspace, `members = ["programs/*"]`) added —
  confirmed via `git status` that `src-tauri/`'s independent `Cargo.toml` isn't absorbed
  into this workspace.

## Scope (`.claude/docs/plan.md` Phase 11, boxes 1–5)

| # | Phase 11 box | This doc covers it? |
|---|---|---|
| 1 | Anchor workspace (`programs/bio_digital_defense/`) with Threat Registry + Genome Registry account types (PDAs) | ✅ **implemented** |
| 2 | `submit_threat` instruction: create/update a Threat Registry PDA, increment `Confidence_Score` | ✅ **implemented** |
| 3 | `commit_gene` instruction: 3-of-5 multisig-gated write to a Genome Registry PDA (Proof of Immunity) | ✅ **implemented** |
| 4 | `suppress_gene` instruction: same multisig authority flips `Epigenetic_Status` to 1 | 🟡 staged below — **not implemented** |
| 5 | Deploy the program to Solana devnet; record the program ID in `Anchor.toml` | 🟡 staged below — **not implemented** |
| 6–11 | `ledger/` rewiring, endpoint keypair provisioning, real IPFS, test re-run | ❌ out of scope — later slices |

This is purely the **on-chain program side** (new `programs/` crate). It does **not** touch
`src-tauri/src/ledger/{client,state,registry,consensus}.rs` — those stay on their current
mock implementation until a later slice rewires them to call this program over RPC. No
collision with owner A's `agents/`.

## Toolchain status (updated — installed this session)

`solana-cli 3.1.10` (Agave), `anchor-cli 1.1.2`, and `avm 1.1.2` are now installed.
`avm use latest` auto-selected Solana `3.1.10` as the version paired with Anchor `1.1.2`
(downgraded from the `stable` 4.1.1 installed earlier — this is Anchor's own compatibility
resolution, not a manual choice). **Still unverified:** which `anchor-lang` crate version
in `programs/bio_digital_defense/Cargo.toml` actually pairs with CLI `1.1.2` — don't guess
this from memory at implementation time (Rule 4); check `anchor --version` / the Anchor
release notes for the matching crate version before pinning it.

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
- **No third account for the validator set.** Reconsidered from the last draft: a
  `LymphNodeConfig` account would make the 5 validators/threshold *runtime-mutable*
  state, which nothing else in this codebase does for a threshold (`ANOMALY_THRESHOLD`,
  `MOBILIZATION_THRESHOLD`, `POI_QUORUM` are all `pub const`s next to the logic that checks
  them). Instead: `pub const LYMPH_NODE_VALIDATORS: [Pubkey; 5] = [...]` and
  `pub const POI_QUORUM: usize = 3;` as consts in `lib.rs` — same multisig guarantee, one
  fewer account type, no init instruction, matches convention. Needs the 5 real devnet
  keypairs generated first so their pubkeys can be hardcoded (a `solana-keygen new` step,
  not on-chain work).

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

- **Accounts:** `genome_entry` (`init_if_needed` PDA), `payer`, `system_program`, plus each
  Lymph Node validator as an optional `Signer` account (5 named optional `Signer<'info>`
  fields, checked against the hardcoded `LYMPH_NODE_VALIDATORS` consts — no config account
  to read).
- **Args:** `threat_id: [u8; 32]`, `gene_hash: [u8; 32]`, `ipfs_cid: String`.
- **Logic:** count how many of the 5 hardcoded validator pubkeys are present *and* marked
  `is_signer` on the transaction; `require!(count >= POI_QUORUM, ErrorCode::InsufficientQuorum)`.
  If satisfied, write/overwrite the `GenomeEntry` PDA with `epigenetic_status = 0`
  (Active) — this is the on-chain replacement for `consensus::commit_gene()` +
  `GenomeRegistry::publish()` combined.
- Client-side implication (not built in this step, just noted): the node assembling the
  transaction needs signatures from 3+ separate Lymph Node keypairs before submitting —
  i.e. an off-chain step to collect co-signatures first. That collection flow is not part
  of steps 1–3; it's implied client work for a later slice.
- **Known forward gap (step 4's territory, not fixed here):** `init_if_needed` means a
  fresh `commit_gene` call would silently overwrite an already-`Suppressed` entry back to
  `Active`, since suppression tracking doesn't exist until `suppress_gene` (box 4). Not a
  regression within steps 1–3's scope, but worth remembering when box 4 is staged so it
  isn't forgotten.

## Open questions / assumptions to confirm before "implement"

1. ~~`behavioral_schema_hash` vs. raw schema~~ — **resolved by implementation**: shipped
   storing the hash only (rent economics); raw schema stays off-chain. You didn't object
   before saying "implement," so treating this as accepted — `source-of-truth.md` was
   updated to match. Revisit only if that's wrong.
2. ~~`LymphNodeConfig` bundled into step 1~~ — **resolved**: dropped in favor of hardcoded
   `pub const` validator pubkeys, matching this codebase's existing threshold pattern. Only
   2 account types now, matching plan.md's box 1 wording exactly.
3. ~~Named optional `Signer` fields (5) vs. `remaining_accounts` loop~~ — **resolved by
   implementation**: shipped with 5 named fields. Same "no objection before implement"
   basis as #1.
4. ~~Mobilization event on `submit_threat`~~ — **resolved by implementation**: shipped as an
   Anchor `event!` emission, firing once at the crossing rather than on every repeat
   sighting past threshold (a deliberate, noted deviation from the mock's `>=` check, which
   re-fires every time). Same basis as #1.

## Step 4 — `suppress_gene` instruction

Mirrors `ledger::registry::GenomeRegistry::suppress()` /
`ledger::registry::SuppressorToken` / `ledger::client::MockConjugationLink::broadcast_suppressor()`
in the Rust mock — the on-chain replacement for that broadcast-and-apply pair, collapsed
into a single instruction (no separate "broadcast then drain" step needed once suppression
*is* the transaction itself).

- **New file:** `programs/bio_digital_defense/src/instructions/suppress_gene.rs`, wired into
  `instructions.rs` and `lib.rs` the same way as the other two.
- **Accounts:** `genome_entry` — **plain `mut`, not `init_if_needed`**. Suppressing a gene
  that doesn't exist is meaningless, so the account must already exist; Anchor's normal
  deserialization already errors (`AccountNotInitialized`) if it doesn't, no extra check
  needed. Plus the same 5 named optional `Signer<'info>` Lymph Node validator fields as
  `commit_gene`.
- **Args:** `threat_id: [u8; 32]` (for the `seeds` constraint).
- **Logic:** same quorum count as `commit_gene` — reusing `LYMPH_NODE_VALIDATORS`/
  `POI_QUORUM`, `require!(signed_count >= POI_QUORUM, ErrorCode::InsufficientQuorum)` — then
  `entry.epigenetic_status = 1`. This is the "same multisig authority" plan.md's box 4
  wording calls for; no new validator set or config needed since none was ever introduced
  in step 1.

**Resolves the forward gap flagged when `commit_gene` was implemented:** with
`suppress_gene` now designed, the earlier note (`commit_gene`'s `init_if_needed` would
silently reactivate an already-suppressed gene) becomes a real, fixable interaction, not
just a future concern. **Recommendation:** add a guard to the *already-implemented*
`commit_gene` — `require!(entry.epigenetic_status != 1, ErrorCode::GeneSuppressed)` before
overwriting an existing entry — so a fresh `commit_gene` call can't silently undo a
suppression. This touches code from steps 1–3, not just new step-4 code, so flagging it
explicitly rather than bundling it in silently: **confirm you want this guard added** when
this gets implemented; without it, `suppress_gene` "works" but has no lasting effect against
a repeated `commit_gene`.

## Step 5 — Deploy to Solana devnet

- **Sequencing:** deploy *after* step 4 lands, not before — `anchor deploy` should ship the
  program with `suppress_gene` already included, so this is one deploy rather than a deploy
  now plus an `anchor upgrade` later once step 4 exists. plan.md's box order (4 then 5)
  already implies this; stating it so it isn't done out of order by mistake.
- **Blocking gap found this session:** no default wallet keypair exists yet at
  `~/.config/solana/id.json` (the path `Anchor.toml`'s `[provider] wallet` points at) —
  `anchor deploy` needs one to pay for the deployment. A `solana-keygen new` for this
  specific path is a prerequisite, distinct from the 5 Lymph Node keypairs (those sign
  attestations; this one is the deployer/payer).
- **Second gap:** the global `solana config get` currently reports `RPC URL:
  https://api.mainnet-beta.solana.com` — mainnet, not devnet. `Anchor.toml`'s
  `[provider] cluster = "devnet"` governs `anchor deploy` itself, but any plain `solana`
  CLI command (airdrop, `program show`, balance checks) run without an explicit `--url
  devnet` would silently hit mainnet instead. Recommend `solana config set --url devnet`
  before doing anything else, to remove that footgun for the rest of this phase.
- **Funding:** `target/deploy/bio_digital_defense.so` is **178,776 bytes**. Solana program
  deployment rent-exemption cost scales with binary size (roughly on the order of 1+ SOL
  for a program this size, using the upgradeable BPF loader's buffer + program accounts) —
  a single `solana airdrop` on devnet is rate-limited (typically ~1–2 SOL per request), so
  this may need more than one airdrop call, or hitting a devnet faucet website if the CLI
  faucet is rate-limited that day. Flagging so it isn't a surprise mid-deploy.
- **Command:** `anchor deploy` (cluster already set via `Anchor.toml`, so no `--provider.cluster`
  flag needed unless overriding).
- **After deploy:** the program ID doesn't change (Anchor deploys *to* the address in the
  already-generated `target/deploy/bio_digital_defense-keypair.json`,
  `FPn8ftGPZ5fr6rH3qtttBp2ppx7HNV4zscGhkFSv8t5x`) — `Anchor.toml`'s `[programs.devnet]` entry
  is already correct and needs no update. Verify with
  `solana program show FPn8ftGPZ5fr6rH3qtttBp2ppx7HNV4zscGhkFSv8t5x --url devnet`.
- **Not included in step 5** (per plan.md's box wording): publishing the IDL on-chain
  (`anchor idl init`) isn't required for the demo to work, just for other clients to
  fetch the IDL directly from the chain — noting it as an optional extra, not proposing it
  unless asked.
- **Note on action weight:** unlike steps 1–4 (local files only), this step performs a real,
  visible action on a public network — the program becomes live and queryable by anyone on
  devnet, and stays there. Low stakes (devnet, no real funds) but not purely local/reversible
  the way file edits are, so flagging it here rather than treating it as routine.

## Open questions / assumptions to confirm before "implement" (steps 4–5)

5. **`commit_gene` guard against re-activating a suppressed gene** (see Step 4 above) — do
   you want this added to the already-implemented `commit_gene`, or left as-is for now?
6. **Wallet + devnet config setup** — OK to run `solana-keygen new` for the default wallet
   path and `solana config set --url devnet` as part of implementing step 5?
7. **Airdrop funding** — OK to run `solana airdrop` (possibly more than once) against
   devnet to fund the deploy, and to proceed with the actual `anchor deploy` to a public
   devnet once funded?

## Recommendation

Steps 1–3 are done. Steps 4–5 as staged above are still self-contained (step 4 is another
new file in `programs/`; step 5 is tooling/config + a deploy command, no source changes) —
no merge collision with owner A. The one piece of already-implemented code this touches is
`commit_gene`'s missing suppressed-gene guard (Q5) — flagged, not applied, pending
confirmation.
