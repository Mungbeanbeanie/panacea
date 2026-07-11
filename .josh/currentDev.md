# Current Dev Plan — Phase 3 (Soldier Spore Lifecycle)

> Scratch planning doc only (`.josh/` is gitignored). Nothing here is implemented until the
> user says **"implement"**. Conforms to `.claude/docs/source-of-truth.md` (canonical) and
> the Working Agreement. Owner: **A** (`agents/`). Depends on Phase 0; decouples from Phase 1
> via the **mock wake signal**.

## Scope

All of **Phase 3** (`.claude/docs/plan.md:93–96`):

- [ ] **1 — Soldier at rest = dormant, serialized, un-executed spore on disk**
- [ ] **2 — Wake on a high-confidence threat notification (`Threat_ID`)**
- [ ] **3 — Clone the frozen process's memory space**
- [ ] **4 — Apoptosis: programmed deletion / re-serialize back to a passive spore after acting**

**Done when:** a wake signal spins up a Soldier that acts, then re-serializes to a spore.

This phase builds the Soldier **lifecycle envelope** only. The actual remediation mechanism
(sandbox + evolutionary fuzz = Phase 4; verified gene fetch/exec = Phase 6) is a **mocked
seam** here — modeled honestly as a stub `neutralize`, not faked as a real kill.

## Baseline (verified this session)

- `agents/soldier.rs` is a doc-comment stub — no code.
- `agents::WakeSignal { threat_id: ThreatId, pid: Pid }` already exists (Phase 1) — this is
  the exact "mock wake signal" Phase 3 consumes.
- Phase 1's `scout::spawn_demo()` currently wires the Scout's `wake_tx` to a **stub consumer
  that just logs**. Phase 3 replaces that stub with the real Soldier (integration step below).
- `serde` + `serde_json` are already deps → spore (de)serialization needs **no new crate**.
  File I/O and the process snapshot use `std` / a `ps` shell-out (same pattern as Phase 1).
- `cargo check` green (1 pre-existing `GeneHandle` warning, unused until Phase 6).

---

## Key decisions (flagged, Rule 1 / Rule 4 / security.md) — confirm at implement

### D1 — Memory-space cloning is mocked (privileged operation)
Real cross-process memory cloning needs `task_for_pid`/`ptrace` (root or a signed
entitlement on macOS) — a classic "heavy piece" to mock (CLAUDE.md). **Plan:** capture a
lightweight *snapshot* of the frozen target (its PID + resident size via `ps -o rss=`) as a
`MemoryClone` stand-in. Faithful to the lifecycle (the Soldier produces a clone artifact the
Phase-4 sandbox will consume) without faking a full memory dump. Flagged, not silently faked.

### D2 — `neutralize` is a logged mock, not a real kill
The Soldier's real kill path is an evolved Wasm gene executed **inside the sandbox**
(Phase 4/6). Phase 3 must not bypass that boundary (security.md). **Plan:** `neutralize()`
logs a mock "neutralized" and does **not** kill anything. The Phase-4/6 remediation plugs in
here later.
- ⚠️ **Decision:** for a cleaner demo, should apoptosis also `SIGCONT` + terminate the
  Phase-1 helper that was left suspended (labeled explicitly as *demo cleanup*, not "the gene
  killed it")? **Recommend yes** — otherwise the scripted target stays frozen until app exit.
  It's an honest cleanup step, clearly not the real remediation mechanism.

### D3 — Spore persistence path
The dormant spore is a serialized file. **Plan:** default to
`std::env::temp_dir().join("bio-digital-defense.spore")`; the lifecycle functions take a
`&Path` so tests use their own temp file. Could later move to Tauri's app-data dir
(`AppHandle::path()`), but that adds a Tauri coupling not needed now (Rule 2). Flagged.

### D4 — Integration: rewire `spawn_demo`, or stand alone?
Phase 3 is decouple-testable against a **mock** `WakeSignal` (no Phase 1 needed). For the
full Scout→Soldier demo, replace the stub wake consumer in `scout::spawn_demo()` with
`soldier::run(wake_rx, path)`. That edits a Phase-1 demo fn, but it's **owner A's file**, so
in-bounds. **Recommend** doing the rewire so the end-to-end chain is visible; the core Soldier
API is independently unit-tested with a mock signal regardless.

### D5 — Apoptosis mode
SoT allows "programmed deletion **or** re-serialization to a passive spore." **Plan:**
re-serialize (the loop-friendly mode: the Soldier returns to dormancy ready for the next
threat, incrementing a `generation` counter to show it cycled). Deletion is the alternative;
re-serialization is the better demo. Flagged.

---

## Per-step design & sketches → `agents/soldier.rs` (not yet applied)

Zero new dependencies. Reuses `agents::WakeSignal` and `core::{ThreatId, Pid}`.

```rust
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::thread::{self, JoinHandle};

use serde::{Deserialize, Serialize};

use super::WakeSignal;
use crate::core::Pid;

/// Step 1 — a Soldier at rest: dormant, serialized, un-executed state on disk.
///
/// The "payload" is modeled as serialized lifecycle state (a generation counter), not literal
/// executable bytes — the Rust code lives in the binary; the spore models the *dormant,
/// resource-free* form the Soldier collapses back to between threats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spore {
    /// How many times this spore has woken and re-serialized (0 = never fired).
    generation: u64,
}

impl Spore {
    /// A never-woken spore.
    fn dormant() -> Self { Self { generation: 0 } }

    /// Load the spore from disk, or start dormant if none exists yet.
    fn load_or_dormant(path: &Path) -> Self {
        std::fs::read(path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_else(Self::dormant)
    }

    /// Serialize the passive spore back to disk (apoptosis end-state).
    fn save(&self, path: &Path) -> std::io::Result<()> {
        std::fs::write(path, serde_json::to_vec(self)?)
    }
}

/// Step 3 — a mock snapshot of the frozen target's memory space (real cloning is privileged).
/// The artifact the Phase-4 sandbox will later replicate and fuzz against.
#[derive(Debug, Clone)]
pub struct MemoryClone {
    pid: Pid,
    rss_kib: u64,
}

/// An awakened Soldier: transient, holds the threat it's acting on and its memory clone.
pub struct Soldier {
    pid: Pid,
    clone: MemoryClone,
    generation: u64,
}

impl Soldier {
    /// Step 2 — wake from a dormant spore on a `Threat_ID` notification, cloning the target.
    fn wake(spore: Spore, signal: &WakeSignal) -> Self {
        let clone = clone_memory(signal.pid);
        Soldier { pid: signal.pid, clone, generation: spore.generation }
    }

    /// Remediation seam. Phase 4 (sandbox+fuzz) / Phase 6 (verified gene) plug in here; the
    /// Epigenetic_Status check (Phase 9) goes *before* any gene fetch/exec at that point.
    /// Mocked now: logs only, kills nothing.
    fn neutralize(&self) {
        println!("[soldier] neutralize (mock) pid {} rss {} KiB", self.clone.pid, self.clone.rss_kib);
    }

    /// Step 4 — apoptosis: re-serialize to a passive spore (generation+1), then drop self.
    fn apoptosis(self, path: &Path) -> std::io::Result<Spore> {
        let next = Spore { generation: self.generation + 1 };
        next.save(path)?;
        Ok(next)
    }
}

/// Step 3 — clone the frozen process's memory space (mock: PID + resident size via `ps`).
fn clone_memory(pid: Pid) -> MemoryClone {
    let rss_kib = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &pid.to_string()])
        .output()
        .ok()
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<u64>().ok())
        .unwrap_or(0);
    MemoryClone { pid, rss_kib }
}

/// Full lifecycle for one wake: load spore → wake+clone → act → apoptosis (re-serialize).
pub fn handle_wake(signal: &WakeSignal, spore_path: &Path) -> std::io::Result<Spore> {
    let spore = Spore::load_or_dormant(spore_path);
    let soldier = Soldier::wake(spore, signal);
    soldier.neutralize();
    soldier.apoptosis(spore_path)
}

/// Run the Soldier as the consumer of the Scout's wake channel (replaces the Phase-1 stub).
pub fn run(wake_rx: Receiver<WakeSignal>, spore_path: std::path::PathBuf) -> JoinHandle<()> {
    thread::spawn(move || {
        for signal in wake_rx {
            if let Err(e) = handle_wake(&signal, &spore_path) {
                eprintln!("[soldier] apoptosis failed: {e}");
            }
        }
    })
}
```

### Integration edit (D4) → `agents/scout.rs` `spawn_demo`
Replace the stub `wake_rx` logging thread with the real Soldier consumer:

```rust
// was: thread::spawn(move || for sig in wake_rx { println!("[soldier] wake …") });
let spore_path = std::env::temp_dir().join("bio-digital-defense.spore");
super::soldier::run(wake_rx, spore_path);
```

---

## Explicitly out of scope (Rule 3)
- **Sandbox setup + evolutionary fuzz** — Phase 4 (owner B, `evolution/`). `neutralize` is the
  seam, mocked.
- **Gene fetch from IPFS + Merkle verification + execution** — Phase 6.
- **Epigenetic_Status kill-switch check** — Phase 9. Phase 3 executes **no gene**, so there is
  nothing to suppress yet; the check belongs *before* the Phase-6 fetch/exec. Noted in
  `neutralize`'s doc as the seam. (⇒ `suppression-path-test` is **not** triggered by Phase 3.)
- **Threat Registry / Confidence_Score** — Phase 2 (owner B). We only *consume* a `WakeSignal`.
- **Real memory dump, real kill** — mocked per D1/D2.

## Source-of-truth conformance (pre-check)
- **Dormant un-executed spore on disk:** `Spore` serialized to a file; not running, resource-
  free. ✅
- **Wakes on a `Threat_ID` notification:** driven by `WakeSignal` (carries `threat_id`). ✅
- **Clones the frozen process's memory space:** `MemoryClone` snapshot of the suspended PID
  (mock, flagged). ✅
- **Apoptosis = re-serialize to a passive spore:** `apoptosis()` writes the spore, drops the
  active Soldier. ✅
- **Caste boundary:** the Soldier is the *executioner*, but its real kill is a sandboxed gene
  (Phase 4/6); Phase 3 mocks that step rather than killing on the host — no sandbox-boundary
  or Epigenetic violation. ✅
- Run **`source-of-truth-check`** before done.

## Definition of done for this batch
- `soldier.rs` exposes the `Spore`/`Soldier` lifecycle + `handle_wake` + `run`.
- A unit/integration test feeds a **mock `WakeSignal`** to `handle_wake` against a temp path
  and asserts: spore file exists afterward, `generation` incremented (dormancy → 1), a
  `MemoryClone` was produced. (Decoupled — no Phase 1 needed.)
- (If D4 accepted) `spawn_demo` rewired so the live Scout→Soldier chain runs; optionally an
  `#[ignore]`d end-to-end test asserting a real wake produces a spore file.
- `cargo check` + `cargo fmt` clean; `cargo test` green.
- Honest report on what was actually run vs. only compiled.

## Open questions for the user (answer at "implement")
- **Q1 — demo cleanup (D2):** have apoptosis `SIGCONT`+terminate the suspended helper as
  labeled demo cleanup (recommended), or leave `neutralize` purely a log?
- **Q2 — integration (D4):** rewire `spawn_demo` to the real Soldier now (recommended), or
  keep Phase 3 standalone/mock-tested only?
- **Q3 — spore path (D3):** OS temp dir (recommended) or Tauri app-data dir?
- **Q4 — apoptosis mode (D5):** re-serialize with a generation counter (recommended), or model
  programmed deletion?
