# Current Dev Plan — Phase 1, Steps 2–7 (rest of Phase 1)

> Scratch planning doc only (`.josh/` is gitignored). Nothing here is implemented until the
> user says **"implement"**. Conforms to `.claude/docs/source-of-truth.md` (canonical) and
> the Working Agreement. Owner: **A** (`agents/`, `core/`).

## Scope

The remaining Phase-1 boxes (`.claude/docs/plan.md:70–75`); Step 1 (the daemon loop) is done:

- [ ] **2 — Behavioral tracing** (syscall anomalies, memory-boundary violations, I/O bursts)
- [ ] **3 — State-transition matrix** (Action A +20, B +50, C +40)
- [ ] **4 — Per-PID cumulative trajectory score**
- [ ] **5 — 100-pt threshold → hard interrupt: suspend all threads of the target PID**
- [ ] **6 — Signal the local Soldier spore on threshold cross**
- [ ] **7 — Emit `Behavioral_Schema` (syscall/port sequence) for the Threat Registry**

**Done when (whole phase):** a scripted "bad" process crosses 100 pts, is suspended, and the
spore is signaled.

These six steps are one cohesive change: the Step-1 inert loop becomes a stateful Stage-1
scorer. I'll plan them together but keep each box's logic in its own small function so the
diff maps to the checklist.

## Baseline (verified this session)

- `agents/scout.rs` has `spawn()` running `loop { tick(); sleep(500ms) }`; `tick()` is a
  no-op. Wired from `main.rs` `.setup(...)`.
- `core`: `Pid = u32`, `AnomalyScore(u32)` (derives `Ord`), `ANOMALY_THRESHOLD =
  AnomalyScore(100)`, `ThreatId([u8;32])`, `GeneHandle([u8;32])`.
- `soldier.rs`, `ledger/registry.rs` are doc-comment stubs (Phases 3 & 2, owners A & B).
- `Cargo.toml`: `tauri 2.11.5`, `serde`, `serde_json`. **No hashing or process-signal crate.**
- `cargo check` green (5 pre-existing `dead_code` warnings on `core` types — those go away as
  Phase 1 starts consuming them).

---

## Key decisions (flagged, Rule 1 / Rule 4 / security.md) — confirm at implement

### D1 — How the Scout observes: scripted feed, not real syscall tracing
Real cross-platform syscall/ESF/eBPF tracing is the classic "heavy piece" to mock
(CLAUDE.md), and the Source-of-Truth actions are Windows-specific (`vssadmin.exe` / Volume
Shadow Copy) — not observable on this macOS dev box. **Plan:** a `BehaviorSource` trait with
a PoC `ScriptedSource` that emits a preset Action A→B→C sequence for a target PID. The
Scout's scoring/threshold/suspension logic is **real**; only the sensor is mocked, behind a
clean seam a real tracer swaps into later. *This is faithful modeling, not faking.*

### D2 — Suspension is REAL on a process we spawn (recommended)
For a convincing demo, `ScriptedSource` spawns a benign helper (e.g. `sleep 600`), targets
its **real PID**, and on threshold the Scout genuinely `SIGSTOP`s it (observable as a stopped
process). Behavioral events are scripted; the freeze is real.
- **Safety guardrails (security.md — suspension is a real side effect):** refuse to suspend
  our own PID or any PID below a floor (`PID_FLOOR`), so the demo can never freeze a
  system/critical process. Only PIDs the Scout is actively scoring can be suspended.
- **Alternative:** fully in-memory simulation (suspension = a state flag). Less compelling;
  recommend the real path with guards.

### D3 — New dependencies (versions from the registry at implement time, Rule 4)
- **`sha2`** (all platforms) — `Threat_ID` = SHA-256 of the behavioral vector → `[u8;32]`,
  matching `ThreatId`.
- **`nix`** (feature `signal`) — `SIGSTOP` for the hard interrupt. **Unix-only**, so it goes
  under `[target.'cfg(unix)'.dependencies]` and `suspend()` is `#[cfg(unix)]`-gated with a
  `#[cfg(not(unix))]` stub. ⚠️ **This keeps the 3-OS CI matrix green** — an unconditional
  `nix` dep would break the Windows job. PoC targets macOS/Linux; Windows suspension
  (`NtSuspendProcess`) is out of scope, stubbed.

### D4 — Mock boundaries to Phase 2 & Phase 3 (don't edit owners B's files)
The Scout emits *outward* to the Threat Registry (Ledger 2, Phase 2, owner B) and the Soldier
(Phase 3, owner A). Per the decouple strategy, the Scout pushes to **`std::sync::mpsc`
channels**; the PoC wires stub consumers that log. Phase 2/3 replace the consumers later.
This means **Phase 1 does not touch `ledger/`** (Rule 3 / ownership).

---

## Per-step design & sketches (not yet applied)

### Shared types → `core/mod.rs` (Step 2 & 7 vocabulary)
`core` is the shared home so Ledger 2 (owner B) can adopt `BehavioralSchema` later without
reaching into `agents/`. ⚠️ `core/mod.rs` is a merge hotspot — append, grouped.

```rust
/// One anomalous behavioral action observed for a process in Stage 1.
///
/// The PoC vocabulary standing in for real syscall/network/I-O traces; each variant maps to
/// a Source-of-Truth Stage-1 action (weights in `scout::weight`). Fieldless so it hashes to a
/// stable byte in the Threat_ID digest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    /// Spawns a hidden child process from a Temp directory (Source of Truth: Action A).
    HiddenChildFromTemp = 0,
    /// Enumerates network adapters while tampering with the Volume Shadow Copy service
    /// (Source of Truth: Action B).
    NetEnumWithVssTamper = 1,
    /// Rapidly loops file handles reading/writing high-entropy data (Source of Truth: Action C).
    HighEntropyFileLoop = 2,
}

/// `Behavioral_Schema` — the ordered action sequence flagged for one process (Ledger 2's
/// behavioral vector, i.e. the "sequence of syscalls and network ports"). Hashing it yields
/// the [`ThreatId`]; two Scouts observing the same sequence agree on the id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BehavioralSchema {
    pub actions: Vec<Action>,
}
```

*Note:* the three tracing **categories** in Step 2 (syscall / memory-boundary / I-O) are the
observation classes the source models; A/B/C are the concrete demo instances that carry the
exact spec weights. I deliberately do **not** invent a 4th weighted action (Rule 4 — spec
lists only three).

### Agent lifecycle types → `agents/mod.rs` (Step 6 & 7 boundaries)
```rust
use crate::core::{BehavioralSchema, Pid, ThreatId};

/// Scout → Soldier wake notification (Phase 3 consumes this; mocked until then).
#[derive(Debug, Clone)]
pub struct WakeSignal { pub threat_id: ThreatId, pub pid: Pid }

/// Scout → Threat Registry (Ledger 2) submission (Phase 2 consumes this; mocked until then).
#[derive(Debug, Clone)]
pub struct ThreatReport { pub threat_id: ThreatId, pub schema: BehavioralSchema }
```

### The scorer → `agents/scout.rs` (Steps 2–7 logic)
Replaces the inert `tick()` with a stateful `Scout`. Each checklist box = one small piece:

```rust
use super::{ThreatReport, WakeSignal};
use crate::core::{Action, AnomalyScore, BehavioralSchema, Pid, ThreatId, ANOMALY_THRESHOLD};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::mpsc::Sender;

/// Lowest PID the Scout will ever suspend — guards system/critical processes (security.md).
const PID_FLOOR: Pid = 1000;

/// Step 3 — weight one action contributes to a trajectory (Source of Truth, Stage 1).
fn weight(action: Action) -> AnomalyScore {
    match action {
        Action::HiddenChildFromTemp => AnomalyScore(20),
        Action::NetEnumWithVssTamper => AnomalyScore(50),
        Action::HighEntropyFileLoop => AnomalyScore(40),
    }
}

/// One process's accumulating Stage-1 trajectory (Step 4 state).
struct Trajectory { score: AnomalyScore, actions: Vec<Action>, fired: bool }

/// Step 2 — a single observation: process `pid` performed `action`.
pub struct Observation { pub pid: Pid, pub action: Action }

/// Step 2 — source of behavioral observations. PoC = scripted feed; a real tracer implements
/// the same trait later.
pub trait BehaviorSource: Send {
    fn poll(&mut self) -> Vec<Observation>;
}

/// The Stage-1 scoring daemon. Owns per-PID trajectories and the outbound mock channels.
pub struct Scout<S: BehaviorSource> {
    source: S,
    scores: HashMap<Pid, Trajectory>,
    wake_tx: Sender<WakeSignal>,      // Step 6 → Soldier (mock)
    threat_tx: Sender<ThreatReport>,  // Step 7 → Ledger 2 (mock)
}

impl<S: BehaviorSource + 'static> Scout<S> {
    pub fn new(source: S, wake_tx: Sender<WakeSignal>, threat_tx: Sender<ThreatReport>) -> Self { /* … */ }

    /// Consume self, run the ultra-light loop on a background thread.
    pub fn spawn(mut self) -> JoinHandle<()> {
        thread::spawn(move || loop { self.tick(); thread::sleep(TICK_INTERVAL); })
    }

    fn tick(&mut self) {
        for Observation { pid, action } in self.source.poll() {
            let t = self.scores.entry(pid).or_insert_with(Trajectory::new);
            if t.fired { continue; }                 // suspend once, don't re-fire
            t.score = AnomalyScore(t.score.0 + weight(action).0);  // Step 3+4
            t.actions.push(action);
            if t.score >= ANOMALY_THRESHOLD {         // Step 5 trigger (Ord on AnomalyScore)
                t.fired = true;
                let schema = BehavioralSchema { actions: t.actions.clone() };
                let id = threat_id(&schema);
                suspend(pid);                                              // Step 5
                let _ = self.wake_tx.send(WakeSignal { threat_id: id, pid });          // Step 6
                let _ = self.threat_tx.send(ThreatReport { threat_id: id, schema });   // Step 7
            }
        }
    }
}

/// Step 7 — Threat_ID = SHA-256 over the ordered action bytes (deterministic; no serde-format
/// dependency, so any node computes the same id from the same sequence).
fn threat_id(schema: &BehavioralSchema) -> ThreatId {
    let mut h = Sha256::new();
    for a in &schema.actions { h.update([*a as u8]); }
    let out = h.finalize();
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&out);   // avoids relying on GenericArray→[u8;32] Into (verify sha2 API)
    ThreatId(bytes)
}

/// Step 5 — suspend all threads of `pid` (Stage-1 hard interrupt). Guards self/system PIDs.
#[cfg(unix)]
fn suspend(pid: Pid) {
    if pid == std::process::id() || pid < PID_FLOOR { return; }
    use nix::sys::signal::{kill, Signal};
    let _ = kill(nix::unistd::Pid::from_raw(pid as i32), Signal::SIGSTOP);
}
#[cfg(not(unix))]
fn suspend(_pid: Pid) { /* PoC targets Unix; Windows suspension out of scope */ }
```

### Demo harness + launch (Step D2 wiring)
To keep `main.rs` (hotspot) a one-liner, encapsulate the PoC wiring in `scout::spawn_demo()`:
it spawns the helper target, builds the `ScriptedSource`, creates the two channels, starts
stub consumers that `println!` what the Scout emits, and returns the `JoinHandle`.

```rust
// scout.rs
/// PoC harness: scripted A→B→C feed against a real spawned helper PID, with stub Soldier /
/// Ledger-2 consumers. The demo entry point Step 1's `spawn()` becomes.
pub fn spawn_demo() -> JoinHandle<()> { /* channels + stub threads + ScriptedSource + Scout::new().spawn() */ }
```

`main.rs` `.setup` changes one line: `agents::scout::spawn()` → `agents::scout::spawn_demo()`.

---

## Explicitly out of scope (Rule 3)
- **Threat Registry storage / correlation / Confidence_Score** — Phase 2 (owner B). We only
  *emit* a `ThreatReport`.
- **Soldier wake/clone/apoptosis** — Phase 3. We only *send* a `WakeSignal`.
- **Dashboard events** — Phase 10. The Scout emits nothing to the UI here.
- **`IPFS_URI`, `Epigenetic_Status`, genes, sandbox** — later phases.

## Source-of-truth conformance (pre-check)
- **Weights exactly** +20 / +50 / +40; **threshold 100** (from `core`, untouched). ✅
- **Caste boundary:** Scout observes, scores, **suspends** (`SIGSTOP` = suspend, not kill),
  and signals — it does **not** terminate or remediate. Matches "suspends all threads of the
  target PID." ✅
- **Threat_ID** = cryptographic hash of the behavioral vector (SHA-256 of the schema). ✅
- **Behavioral_Schema** = ordered syscall/action sequence. ✅
- **Wake signal carries `Threat_ID`** — matches Phase 3 ("wake on a Threat_ID"). ✅
- Run the **`source-of-truth-check`** skill before done.

## Definition of done for this batch
- Scout accumulates per-PID scores from scripted A/B/C, crosses 100, **suspends the real
  target PID once**, sends a `WakeSignal` and a `ThreatReport` (both observed via stub logs).
- New deps (`sha2`; `nix` unix-only) added from the registry; `cargo check` green on the
  Unix host; Windows path compiles via the stub.
- `cargo fmt` applied; a unit test covering scoring → threshold → single-fire → emissions
  (suspension exercised in the demo, not the unit test, to avoid `SIGSTOP` in CI).
- Honest report: was the app actually run (`tauri dev`) and the freeze observed, or only
  `cargo check`?

## Open questions for the user (answer at "implement")
- **Q1 — suspension:** real `SIGSTOP` on a spawned helper with PID guards (recommended), or
  fully simulated?
- **Q2 — dependencies:** OK to add `sha2` and (unix-only) `nix`? Prefer `blake3` over `sha2`,
  or a different signal crate (`libc`) over `nix`?
- **Q3 — type homes:** `Action` + `BehavioralSchema` in `core/mod.rs` (recommended, shared
  with Ledger 2), or keep them in `agents/` for now?
- **Q4 — test depth:** unit test for the scoring/threshold/emit path is enough, or do you
  want an end-to-end test that spawns a helper and asserts it actually reaches stopped state?
