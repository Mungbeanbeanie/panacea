# Current Dev Plan — Phase 11, Steps 1–11 (Solana Migration)

> Scratch planning doc only (`.josh/` is gitignored). Conforms to
> `.claude/docs/source-of-truth.md` (canonical) and `.claude/docs/plan.md` Phase 11
> (owner B).
>
> **✅ Steps 1–5 implemented and verified — Phase 11's on-chain program side is complete.**
> `anchor build` compiles clean (no errors, no warnings) with `submit_threat`/
> `commit_gene`/`suppress_gene` all present in the IDL, plus the `commit_gene`
> suppressed-gene guard (Q5, resolved: added). Deployer wallet funded via the web faucet
> (GitHub sign-in unblocked the CLI airdrop's rate limit — 2.5 SOL). **Deployed to devnet**
> via `anchor deploy`: program ID `FPn8ftGPZ5fr6rH3qtttBp2ppx7HNV4zscGhkFSv8t5x`, confirmed
> live via `solana program show` (owner `BPFLoaderUpgradeab1e...`, upgrade authority =
> deployer wallet, 189,952 bytes, 1.32327 SOL locked as rent-exemption). IDL metadata also
> initialized on-chain as a side effect of the modern `anchor deploy` flow.
>
> No client-side wiring yet (box 6+: `ledger/client.rs`/`state.rs`/`registry.rs` still on
> the Rust mock, not calling this program) — that's the next slice.

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

## Scope (`.claude/docs/plan.md` Phase 11, all 11 boxes)

| # | Phase 11 box | Status |
|---|---|---|
| 1 | Anchor workspace with Threat Registry + Genome Registry account types (PDAs) | ✅ **implemented** |
| 2 | `submit_threat` instruction | ✅ **implemented** |
| 3 | `commit_gene` instruction (Proof of Immunity) | ✅ **implemented** |
| 4 | `suppress_gene` instruction | ✅ **implemented** |
| 5 | Deploy to Solana devnet; record program ID | ✅ **implemented** (`FPn8ftGPZ5fr6rH3qtttBp2ppx7HNV4zscGhkFSv8t5x`) |
| 6 | `ledger/client.rs`: real `solana-client`/RPC transport | 🟡 staged below — **not implemented** |
| 7 | `ledger/state.rs`: commitment-level reads, drop Merkle code | 🟡 staged below — **not implemented** |
| 8 | `ledger/registry.rs`: read-through cache of on-chain PDAs | 🟡 staged below — **not implemented** |
| 9 | Devnet keypair provisioning (endpoints + validators) | 🟡 staged below — **not implemented** |
| 10 | Real IPFS pinning-service integration | 🟡 staged below — **not implemented** |
| 11 | Re-run `suppression-path-test` / `sandbox-isolation-check` against the live path | 🟡 staged below — **not implemented** |

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

---

# Steps 6–11: wiring the Rust core to the live program

Read this session: `agents/soldier.rs`, `agents/scout.rs`, `agents/mod.rs`,
`ledger/state.rs` (full), `evolution/lymph_node.rs` (confirmed untouched — Stage 3's
allergy check is off-chain and independent of the ledger), `src-tauri/Cargo.toml` (no
`solana-client`/`anchor-client`/HTTP crate present yet — all new deps for this slice).

## The cross-cutting fork this slice can't avoid (read first)

Boxes 6–8 are listed as `ledger/`-only (owner B) in plan.md, but they aren't self-contained
the way steps 1–5 were. `agents/soldier.rs`'s `Pharmacy` struct
(`{ genome: GenomeRegistry, ipfs: MockIpfsStore, state: StateLedger, link:
MockConjugationLink }`) and `resolve_and_run(threat_id, &GenomeRegistry, &MockIpfsStore,
&StateLedger, &MerkleProof, Sandbox)` are built directly against today's mock types. Once
those types become Solana-backed, `resolve_and_run`'s signature necessarily changes too —
most importantly, **the `MerkleProof` parameter disappears entirely**, because
`source-of-truth.md`'s Ledger 1 section now says reading a `finalized` Solana account
*already* carries the integrity guarantee a Merkle path used to provide. `agents/scout.rs`'s
`spawn_demo()` also constructs a `Pharmacy` directly and will need the same update.

This is real owner-A/owner-B boundary crossing, the same shape as the Gap-B issue flagged
back in the Phase 9 planning history. Flagging it up front rather than three boxes in:
**touching `agents/soldier.rs` and `agents/scout.rs` is unavoidable to finish this phase**,
not scope creep — the plan's own done-when criterion ("a Soldier fetches and verifies it
via RPC + real IPFS CID") requires it.

## New dependencies — unverified, don't guess versions at implementation time (Rule 4)

None of these are in `src-tauri/Cargo.toml` yet:

- A Solana RPC client. Two options: raw `solana-client` + `solana-sdk` (build/sign/send
  transactions by hand), or `anchor-client` (higher-level, builds instructions from the
  IDL — the natural complement to the program we already built with Anchor). **Leaning
  `anchor-client`** to avoid hand-rolling instruction/account encoding, but its exact
  version compatible with `anchor-lang 1.1.2` and the Solana "v3" crate family we saw
  during that build (`solana-pubkey v3.0.0`, `solana-transaction v3.4.0`, etc.) is
  unverified — the same lesson as `init-if-needed`: check by actually resolving it, don't
  assert a version from memory.
- An HTTP client for the IPFS pinning service — `reqwest` is the standard choice, version
  unverified against this project's edition/toolchain.
- **Recommendation: stay on a blocking/synchronous RPC client, not async.** `solana-client`
  has both a blocking `RpcClient` and a `nonblocking` (tokio) one. The whole agent core
  today is `std::thread` + `mpsc` — no `tokio`, no async anywhere in `src-tauri`. Pulling in
  the async client would ripple `async fn`/`.await` through `Pharmacy`, `resolve_and_run`,
  `Soldier::dispense`, `Scout::tick`, and `main.rs`'s setup — a much bigger refactor than
  this phase needs. The blocking client keeps the change contained to `ledger/` +
  `agents/soldier.rs`/`scout.rs`'s call sites, matching Rule 2/3. Flagging as a real
  decision, not defaulting silently, since it's a foundational choice for the rest of the
  core.

## Box 6 — `ledger/client.rs`: real RPC transport

Replaces `MockConjugationLink` (`commit_block`/`broadcast_suppressor`/`drain_suppressors`/
`threat_proof`/`genome_proof`) with a thin wrapper — tentatively `SolanaConjugationLink` —
around the blocking RPC client, exposing:

- `submit_threat(threat_id, schema_hash) -> Signature` — signs with the caller's keypair,
  sends the `submit_threat` instruction.
- `commit_gene(threat_id, gene_hash, ipfs_cid, validator_keypairs: &[Keypair]) -> Signature`
  — the multisig call. **PoC simplification, flagging explicitly:** the demo has one
  process holding all 5 Lymph Node keypair files (`keys/lymph-nodes/validator-{1..5}.json`,
  generated in the steps 1–3 slice) rather than 5 independent validator processes. This
  local process loads 3+ of them to co-sign a single transaction, simulating "quorum was
  reached" without standing up separate nodes — consistent with the PoC bar
  (`build-run-test.md`: model the flow, don't build real infrastructure for it), but a real
  design simplification worth confirming rather than assuming silently.
- `suppress_gene(threat_id, validator_keypairs: &[Keypair]) -> Signature` — same shape.

No more `commit_block`/Merkle leaves — a Solana transaction's confirmation *is* the commit;
there's no separate block to assemble.

## Box 7 — `ledger/state.rs`: commitment-level reads, drop the Merkle code

Deletes `MerkleTree`, `MerkleProof`, `Side`, `BlockHeader`, `Hash`, `StateLedger`, and their
tests wholesale — plan.md's box 7 says so explicitly ("drop the custom Merkle-path code"),
and nothing else in the codebase constructs a Merkle proof outside this file and its own
tests (confirmed by what was read this session). Replaces it with something like a
`SolanaLightClient` wrapping the RPC client, offering:

- `get_threat_entry(threat_id) -> Option<ThreatEntry>` (deserialized on-chain account)
- `get_genome_entry(threat_id) -> Option<GenomeEntry>`

Both reads at a chosen commitment level — **open question: `confirmed` or `finalized`?**
Re-checked the actual text: `source-of-truth.md` permits *either* — "at `confirmed` or
`finalized` commitment" — so this isn't a spec-compliance question at all (an earlier draft
of this doc wrongly implied `confirmed` would be an off-spec deviation; it isn't). Purely a
tradeoff: `finalized` is the stronger guarantee but slower (devnet finality trails a single
confirmation noticeably); `confirmed` is faster and better for a live demo's responsiveness.
Recommend `confirmed` for the demo, given both are equally on-spec.

This is the biggest deletion of this slice — flagging clearly rather than doing it
silently, per Rule 3.

## Box 8 — `ledger/registry.rs`: read-through cache of on-chain PDAs

`ThreatRegistry`/`GenomeRegistry`'s `HashMap`-backed storage goes away; they become thin
wrappers that call the box-7 light client on every `get()`/`report()`/`publish()`-equivalent
call rather than holding local state — "cache" here means "the read path," not a persistent
local store, though a short-lived in-memory memo (avoid two RPC round-trips for the same
`threat_id` within one Soldier dispense) is a reasonable, small addition. `MockIpfsStore`
also currently lives in this file — **proposing it moves to a new `ledger/ipfs.rs`** for
single-responsibility (`conventions.md`: keep modules focused on one concern), rather than
growing `registry.rs` further. Not in `file-structure.md` yet; flagging as a file-structure
addition to confirm, not just doing it.

## Box 9 — Keypair provisioning

- **Lymph Node validators:** already done (5 real keypairs, steps 1–3 slice). **Correction
  on funding:** re-checked the actual `commit_gene`/`suppress_gene` account structs —
  `validator_1..5` are plain `Option<Signer<'info>>` with no `#[account(mut)]`; they're
  pure co-signers that never pay rent or fees. Producing a signature costs nothing
  regardless of balance — only the designated `payer` needs SOL. So plan.md's box 9
  "funded via faucet" wording really only applies to the endpoint/Scout signer below;
  funding the 5 validator wallets would be wasted faucet requests. Nothing new needed
  here beyond what's already done.
- **Endpoint/Scout signer:** plan.md calls for "per endpoint." For this single-node demo,
  **proposing reuse of the existing deployer wallet** (`~/.config/solana/id.json`,
  `HeAubH3AUZDwztNC3BCsDSacnGSAXnd2ZpLJ3H68W6b3`, currently ~1.16 SOL) as the Scout's
  signer too, rather than generating yet another keypair that would also need its own
  faucet funding — simplest option consistent with "one demo node." Flagging as an
  assumption, not generating a new keypair silently.
- **Funding headroom:** each `submit_threat`/`commit_gene`/`suppress_gene` call is a tiny
  tx fee (~0.000005 SOL) plus, for a **new** `ThreatEntry`/`GenomeEntry` PDA, a small
  rent-exemption (a few thousandths of a SOL for these small accounts — far less than the
  program deploy's 1.32 SOL). Current ~1.16 SOL balance should comfortably cover the demo's
  handful of transactions; flagging only so a sudden "insufficient funds" isn't a surprise
  if the demo ends up creating many distinct `threat_id`s.

## Box 10 — Real IPFS pinning-service integration

- New `ledger/ipfs.rs` (see box 8), wrapping a pinning service's HTTP API (e.g.
  web3.storage or Pinata) via `reqwest`: `upload(bytes) -> Cid` (called from the
  `commit_gene` client flow) and `fetch(cid) -> bytes` (called from the Soldier's dispense
  path, replacing `MockIpfsStore::fetch`).
- **Real external blocker, not something I can resolve myself:** this needs an actual
  account + API key with whichever pinning service is chosen — creating that account is
  yours to do (I'm not signing up for a third-party service on your behalf). Same shape as
  the devnet faucet blocker earlier in this phase.
- The API key goes in `.env` (already gitignored per this session's earlier `.gitignore`
  edit — "Env / secrets" section already covers `.env`), read via `std::env::var`. Open
  question: add a small `dotenvy`-style crate to auto-load `.env` in dev, or just document
  that the operator exports the var before running? Leaning "just document it" — one fewer
  dependency, and `build-run-test.md` already has room for a setup note.
- After fetch, the Soldier compares `sha256(fetched_bytes) == entry.gene_hash` directly —
  this *is* the verification step now (see box 7's commitment-level read discussion); no
  separate proof object.

## Box 11 — Re-run the safety skills against the live path

`agents/soldier.rs`'s `pharmacy_flow_tests` and `live_dispense_tests` currently construct
the mock `Pharmacy` directly and will break once boxes 6–8 change its field types. **Open
design question, not decided here:** introduce a small trait (e.g. a `GenomeSource`
boundary `resolve_and_run` depends on) so the existing tests keep running fast and offline
against a fake implementation, with a small number of new `#[ignore]`d live-devnet tests
(mirroring the existing pattern already used for the real-process tests in `soldier.rs`/
`scout.rs`) — versus converting the existing tests to hit live devnet directly (simpler
change, but slower, costs real devnet SOL per test run, and flaky under network
conditions). **Recommend the trait approach** to keep the suite fast and CI-safe, but this
is a genuine design fork worth confirming, not assuming. A lighter-weight alternative worth
noting: skip a trait entirely and give the concrete registry types an internal
`Backing::Rpc(...) | Backing::Offline(HashMap<...>)` enum instead — no generic/dyn-dispatch
boundary threaded through `resolve_and_run`, at the cost of a branch inside each method
instead of a clean seam. Either is reasonable; flagging both rather than picking silently.

Once whichever path is chosen and implemented, re-run `suppression-path-test` and
`sandbox-isolation-check` per the box's own wording. Worth noting `sandbox-isolation-check`
specifically: boxes 6–10 never touch `evolution/sandbox.rs` or the gene-execution call
(`sandbox.run(&gene.sequence)` is unchanged — only *where the bytes come from* changes), so
that check should trivially still pass. Re-running it is compliance with the box's literal
wording, not a response to any new risk this slice introduces.

## Open questions / assumptions to confirm before "implement" (steps 6–11)

8. **RPC client choice** — `anchor-client` (recommended) vs. hand-rolled `solana-client` +
   manual instruction encoding?
9. **Blocking vs. async RPC** — stay synchronous (recommended, avoids a `tokio` refactor
   across the whole agent core) vs. adopt `nonblocking`/async now?
10. **Commitment level** — both `confirmed` and `finalized` are explicitly on-spec per
    `source-of-truth.md`'s own wording (corrected from an earlier mischaracterization in
    this doc); recommend `confirmed` for demo responsiveness, `finalized` for the stronger
    guarantee at the cost of speed — a pure tradeoff, not a compliance question.
11. **Multisig demo simplification** — OK that one process holds all 5 Lymph Node keypairs
    to self-sign quorum, rather than simulating separate validator processes?
12. **Scout signer** — reuse the existing deployer wallet, or provision a fresh keypair
    (needing its own funding) for the Scout specifically?
13. **`ledger/ipfs.rs` as a new file** — OK to split IPFS out of `registry.rs`, and to add
    it to `file-structure.md`?
14. **`.env` loading** — read `std::env::var` directly (recommended, no new dependency) vs.
    add a `dotenvy`-style crate for convenience?
15. **Test strategy (box 11)** — introduce a `GenomeSource`-style trait so existing tests
    stay offline (recommended) vs. convert directly to live-devnet integration tests?
16. **Pinning service account** — which service (web3.storage / Pinata / other), and you'll
    need to create that account and hand me the API key yourself.

## Recommendation

Steps 6–11 are a meaningfully bigger, more cross-cutting slice than 1–5: they touch
`agents/soldier.rs` and `agents/scout.rs` (owner A's files) in addition to `ledger/` (owner
B), add three new external dependencies with unverified versions, delete a whole module's
worth of Merkle code, and depend on one external account you'll need to create yourself
(the IPFS pinning service). None of that is a reason to avoid it — the phase doesn't finish
without it — but it's a bigger single "implement" than steps 1–5 was, and questions
8–16 above have real architectural consequences (especially 8–10, which shape every
subsequent box). Worth resolving those before diving in, rather than defaulting through all
of them at once.
