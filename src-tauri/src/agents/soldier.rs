//! Soldier: dormant on-disk remediation payload ("spore"). Wakes on a high-confidence
//! signal, isolates and neutralizes the threat inside the sandbox, then undergoes apoptosis
//! (re-serializes to a spore). Checks Epigenetic_Status before running any gene — a
//! suppressed gene (status 1) is never fetched or executed
//! (see ../../.claude/skills/suppression-path-test). Stage 2.

use std::path::Path;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use serde::{Deserialize, Serialize};

use super::WakeSignal;
use crate::core::Pid;
use crate::dashboard::Dashboard;

/// Lowest PID the Soldier will release/terminate — a coarse guard against touching
/// low-numbered system processes, paired with the self-PID check in [`release_target`].
#[cfg(unix)]
const PID_FLOOR: Pid = 1000;

/// A Soldier at rest: dormant, serialized, un-executed state on disk.
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
    fn dormant() -> Self {
        Self { generation: 0 }
    }

    /// Load the spore from disk, or start dormant if none exists / is unreadable.
    fn load_or_dormant(path: &Path) -> Self {
        std::fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_else(Self::dormant)
    }

    /// Serialize the passive spore back to disk — the apoptosis end-state.
    fn save(&self, path: &Path) -> std::io::Result<()> {
        let bytes = serde_json::to_vec(self).map_err(std::io::Error::other)?;
        std::fs::write(path, bytes)
    }

    /// Times this spore has woken and re-serialized.
    pub fn generation(&self) -> u64 {
        self.generation
    }
}

/// A mock snapshot of the frozen target's memory space.
///
/// Real cross-process memory cloning needs `task_for_pid`/`ptrace` (privileged); the PoC
/// captures a lightweight snapshot (PID + resident size) as the artifact the Phase-4 sandbox
/// will later replicate and fuzz against.
#[derive(Debug, Clone)]
pub struct MemoryClone {
    pid: Pid,
    rss_kib: u64,
}

/// An awakened Soldier: transient, holding the threat it's acting on and its memory clone.
pub struct Soldier {
    pid: Pid,
    clone: MemoryClone,
    generation: u64,
}

impl Soldier {
    /// Wake from a dormant spore on a `Threat_ID` notification, cloning the frozen target.
    fn wake(spore: Spore, signal: &WakeSignal) -> Self {
        let clone = clone_memory(signal.pid);
        Soldier {
            pid: signal.pid,
            clone,
            generation: spore.generation,
        }
    }

    /// Apoptosis: re-serialize to a passive spore (generation + 1), then drop self.
    fn apoptosis(self, path: &Path) -> std::io::Result<Spore> {
        let next = Spore {
            generation: self.generation + 1,
        };
        next.save(path)?;
        Ok(next)
    }
}

/// Clone the frozen process's memory space (mock: PID + resident size via `ps`).
fn clone_memory(pid: Pid) -> MemoryClone {
    let rss_kib = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &pid.to_string()])
        .output()
        .ok()
        .and_then(|out| {
            String::from_utf8_lossy(&out.stdout)
                .trim()
                .parse::<u64>()
                .ok()
        })
        .unwrap_or(0);
    MemoryClone { pid, rss_kib }
}

/// Demo cleanup: release the suspended target and terminate it.
///
/// This stands in for the real remediation (a sandboxed Wasm gene, Phase 4/6) *only* so the
/// scripted demo process doesn't stay frozen — it executes no gene and makes no correctness
/// claim about the cure. Guards the Soldier's own process and low-numbered system PIDs.
#[cfg(unix)]
fn release_target(pid: Pid) {
    if pid == std::process::id() || pid < PID_FLOOR {
        return;
    }
    use nix::sys::signal::{kill, Signal};
    let target = nix::unistd::Pid::from_raw(pid as i32);
    let _ = kill(target, Signal::SIGCONT);
    let _ = kill(target, Signal::SIGKILL);
}

/// Non-Unix stub: the PoC targets macOS/Linux.
#[cfg(not(unix))]
fn release_target(_pid: Pid) {}

/// Full lifecycle for one wake: load spore → wake + clone → dispense the cure through the
/// pharmacy (epigenetic kill-switch checked first) → demo-release the frozen target →
/// apoptosis. Returns the re-serialized spore and the cure outcome.
pub fn handle_wake(
    signal: &WakeSignal,
    spore_path: &Path,
    pharmacy: &Pharmacy,
    dashboard: Option<&Dashboard>,
) -> std::io::Result<(Spore, PharmacyOutcome)> {
    let spore = Spore::load_or_dormant(spore_path);
    let soldier = Soldier::wake(spore, signal);
    let outcome = soldier.dispense(&signal.threat_id, pharmacy, dashboard);
    release_target(soldier.pid); // demo cleanup: unfreeze/terminate the scripted helper
    if let Some(dashboard) = dashboard {
        dashboard.clear_ecosystem_node(soldier.pid);
    }
    let spore = soldier.apoptosis(spore_path)?;
    Ok((spore, outcome))
}

/// Run the Soldier as the consumer of the Scout's wake channel (replaces the Phase-1 stub).
///
/// Loops for the channel's lifetime; each `WakeSignal` runs one full spore lifecycle against
/// the shared `pharmacy` — resolving and dispensing the cure, with the epigenetic kill-switch
/// checked before any fetch/exec. `on_outcome`, if given, gets each dispense's outcome — the
/// demo's sequencing signal for "wait for wave 1 to finish before suppressing and firing
/// wave 2" (`agents::scout::spawn_demo`), not something the Soldier itself needs.
pub fn run(
    wake_rx: Receiver<WakeSignal>,
    spore_path: std::path::PathBuf,
    pharmacy: Pharmacy,
    dashboard: Option<Arc<Dashboard>>,
    on_outcome: Option<Sender<PharmacyOutcome>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        for signal in wake_rx {
            match handle_wake(&signal, &spore_path, &pharmacy, dashboard.as_deref()) {
                Ok((spore, outcome)) => {
                    println!(
                        "[soldier] cure {outcome:?}; apoptosis → spore generation {}",
                        spore.generation()
                    );
                    if let Some(tx) = &on_outcome {
                        let _ = tx.send(outcome);
                    }
                }
                Err(e) => eprintln!("[soldier] apoptosis failed: {e}"),
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ThreatId;

    #[test]
    fn wake_acts_then_reserializes_to_spore() {
        let path = std::env::temp_dir().join(format!("bdd-test-spore-{}.json", std::process::id()));
        let _ = std::fs::remove_file(&path);

        // Target our own PID so the demo-release path is guarded off (never kill the test).
        let signal = WakeSignal {
            threat_id: ThreatId([7u8; 32]),
            pid: std::process::id(),
        };

        let pharmacy = Pharmacy::empty();
        let (spore, _) = handle_wake(&signal, &path, &pharmacy, None).expect("handle_wake");
        assert_eq!(
            spore.generation(),
            1,
            "dormant(0) → woke once → generation 1"
        );
        assert!(path.exists(), "spore re-serialized to disk");

        // A second wake loads generation 1 and re-serializes as 2.
        let (spore2, _) =
            handle_wake(&signal, &path, &pharmacy, None).expect("handle_wake again");
        assert_eq!(spore2.generation(), 2);

        let _ = std::fs::remove_file(&path);
    }

    /// End-to-end: a mock wake for a real suspended process spins up a Soldier that releases
    /// and terminates it, then re-serializes to a spore. Kills a real process, so `#[ignore]`d
    /// out of CI — run locally with `cargo test -- --ignored`.
    #[test]
    #[ignore = "spawns and kills a real process; run locally with --ignored"]
    #[cfg(unix)]
    fn wake_releases_real_target() {
        use nix::sys::signal::{kill, Signal};

        let mut child = std::process::Command::new("sleep")
            .arg("600")
            .spawn()
            .expect("spawn helper");
        let pid = child.id();
        // Mirror the Scout's hard interrupt: freeze the target first.
        let _ = kill(nix::unistd::Pid::from_raw(pid as i32), Signal::SIGSTOP);

        let path = std::env::temp_dir().join(format!("bdd-e2e-spore-{pid}.json"));
        let _ = std::fs::remove_file(&path);
        let signal = WakeSignal {
            threat_id: ThreatId([1u8; 32]),
            pid,
        };

        let pharmacy = Pharmacy::empty();
        let (spore, _) = handle_wake(&signal, &path, &pharmacy, None).expect("handle_wake");
        assert_eq!(spore.generation(), 1);
        assert!(path.exists());

        let status = child.wait().expect("reap target");
        assert!(
            !status.success(),
            "Soldier released and terminated the frozen target"
        );
        let _ = std::fs::remove_file(&path);
    }
}

use crate::core::ThreatId;
use crate::evolution::alleles::{fuzz, CompiledGene};
use crate::evolution::lymph_node::{LymphNode, RegressionOutcome};
use crate::evolution::sandbox::{FrozenProcess, Sandbox, TrialOutcome};
use crate::ledger::registry::{GeneCommitter, GenomeSource, SharedFakeLedger};

/// Outcome of a Soldier's resolve → verify → execute → apoptosis run (or, on
/// `NoCureAvailable`, the evolve-fuzz-commit-and-re-dispense loop that produces one live).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PharmacyOutcome {
    /// No Genome Registry row for this `Threat_ID`, and evolving one failed (fuzz found
    /// nothing safe, or the Lymph Node flagged the candidate as allergic).
    NoCureAvailable,
    /// `Epigenetic_Status = 1` — halted before any execution.
    Suppressed,
    /// The account's `gene_seq` doesn't hash to its own recorded `Wasm_Gene_Hash`.
    GeneHashUnverified,
    /// The Genome Registry read or the `commit_gene` write itself failed (RPC/network
    /// error) — distinct from a legitimate "no cure published" result.
    LedgerUnavailable,
    /// Gene ran in-sandbox but didn't neutralize the target; apoptosis still follows.
    Ineffective,
    /// Gene neutralized the target in-sandbox; apoptosis follows.
    Neutralized,
}

/// Resolves `threat_id` against the Genome Registry **on demand** (never a passive scan),
/// checks `Epigenetic_Status` *before* running anything, decodes the account's own
/// `gene_seq` bytes and checks their hash against `gene_hash` (reading a `confirmed`-
/// commitment Solana account already carries the integrity guarantee a Merkle proof used to
/// provide — see `../../ledger/state.rs` — so this is a corruption check, not a
/// fetch-verification step), runs it in-sandbox, and undergoes apoptosis. If no row exists
/// yet, hands off to [`evolve_and_commit`] instead of giving up.
pub fn resolve_and_run(
    threat_id: &ThreatId,
    pharmacy: &Pharmacy,
    mut sandbox: Sandbox,
    dashboard: Option<&Dashboard>,
) -> PharmacyOutcome {
    let entry = match pharmacy.genome.get(threat_id) {
        Ok(entry) => entry,
        Err(_) => return log_outcome(dashboard, threat_id, "ledger.unavailable", PharmacyOutcome::LedgerUnavailable),
    };

    let Some(entry) = entry else {
        return evolve_and_commit(threat_id, pharmacy, sandbox, dashboard);
    };

    if entry.epigenetic_status == 1 {
        return log_outcome(dashboard, threat_id, "gene.suppressed", PharmacyOutcome::Suppressed);
    }

    let compiled = match CompiledGene::from_bytes(&entry.gene_seq) {
        Some(compiled) if compiled.gene_hash().0 == entry.gene_hash => compiled,
        _ => {
            return log_outcome(
                dashboard,
                threat_id,
                "gene.hash_unverified",
                PharmacyOutcome::GeneHashUnverified,
            )
        }
    };

    let trial = sandbox.run_gene(&compiled.wasm_bytes);
    sandbox.teardown(); // apoptosis

    match trial {
        TrialOutcome::TargetCrashed => {
            log_outcome(dashboard, threat_id, "threat.neutralized", PharmacyOutcome::Neutralized)
        }
        TrialOutcome::TargetSurvived | TrialOutcome::HostDestabilized => log_outcome(
            dashboard,
            threat_id,
            "threat.ineffective",
            PharmacyOutcome::Ineffective,
        ),
    }
}

/// Stage 2 (fuzz) → Stage 3 (Lymph Node allergy check) → Stage 4 (`commit_gene` under
/// multisig) → re-dispense: the live path for a `Threat_ID` with no published cure yet.
/// Reuses the sandbox already cloned around the frozen target for fuzzing, then spins up a
/// fresh one (same PID) to run the freshly committed gene — closing the loop through the
/// ledger rather than trusting the local fuzz result directly.
fn evolve_and_commit(
    threat_id: &ThreatId,
    pharmacy: &Pharmacy,
    mut sandbox: Sandbox,
    dashboard: Option<&Dashboard>,
) -> PharmacyOutcome {
    let pid = sandbox.target_pid();
    let id = threat_id.to_hex();
    let strain = |suffix: &str, parent: &str, stage: &'static str| {
        if let Some(dashboard) = dashboard {
            dashboard.record_strain(
                format!("{id}-{suffix}"),
                Some(parent.to_string()),
                format!("strain-{}-{suffix}", &id[..8]),
                stage,
            );
        }
    };

    let Some(gene) = fuzz(&mut sandbox) else {
        sandbox.teardown();
        return log_outcome(dashboard, threat_id, "cure.unavailable", PharmacyOutcome::NoCureAvailable);
    };
    sandbox.teardown();
    log_event(dashboard, threat_id, "gene.fuzzed");
    strain("fuzzed", "genesis", "fuzzed");

    if let RegressionOutcome::AllergyFlagged(app) =
        LymphNode::standard_environment().run_regression(&gene)
    {
        println!("[soldier] gene allergy-flagged by {app}; dropped");
        log_event(dashboard, threat_id, "gene.allergy_flagged");
        return PharmacyOutcome::NoCureAvailable;
    }
    log_event(dashboard, threat_id, "allergy.checked");
    strain("regression", &format!("{id}-fuzzed"), "regression");

    let compiled = gene.compile();
    if pharmacy.committer.commit_gene(threat_id, &compiled).is_err() {
        return log_outcome(
            dashboard,
            threat_id,
            "ledger.unavailable",
            PharmacyOutcome::LedgerUnavailable,
        );
    }
    log_event(dashboard, threat_id, "gene.committed");
    strain("committed", &format!("{id}-regression"), "committed");

    let fresh_sandbox = Sandbox::spawn(FrozenProcess { pid, memory: Vec::new() });
    resolve_and_run(threat_id, pharmacy, fresh_sandbox, dashboard)
}

fn log_event(dashboard: Option<&Dashboard>, threat_id: &ThreatId, kind: &'static str) {
    if let Some(dashboard) = dashboard {
        dashboard.log_event(kind, threat_id.to_hex());
    }
}

fn log_outcome(
    dashboard: Option<&Dashboard>,
    threat_id: &ThreatId,
    kind: &'static str,
    outcome: PharmacyOutcome,
) -> PharmacyOutcome {
    log_event(dashboard, threat_id, kind);
    outcome
}

/// The local pharmacy a running Soldier consults: the Genome Registry (Ledger 3) read side
/// and the `commit_gene` write side, behind trait objects so the offline test suite can swap
/// in fakes instead of hitting live devnet (Q8). Reading the Genome Registry fresh on every
/// dispense means there's no separate "absorb suppressor tokens" step anymore — the
/// on-chain account is always the live truth.
pub struct Pharmacy {
    pub genome: Box<dyn GenomeSource>,
    pub committer: Box<dyn GeneCommitter>,
}

impl Pharmacy {
    /// An empty, offline pharmacy backed by one `SharedFakeLedger` — no cures published yet,
    /// but a commit through `committer` is immediately visible through `genome`, the way a
    /// real `commit_gene` transaction becomes visible to the next RPC read.
    pub fn empty() -> Self {
        let ledger = SharedFakeLedger::new();
        Self {
            genome: Box::new(ledger.clone()),
            committer: Box::new(ledger),
        }
    }
}

impl Soldier {
    /// Dispense the cure for `threat_id` through the full pharmacy flow: resolve →
    /// **kill-switch check** → decode-and-verify → run in-sandbox (evolving one first if
    /// none is published yet).
    fn dispense(
        &self,
        threat_id: &ThreatId,
        pharmacy: &Pharmacy,
        dashboard: Option<&Dashboard>,
    ) -> PharmacyOutcome {
        println!(
            "[soldier] dispensing for pid {} (cloned rss {} KiB)",
            self.clone.pid, self.clone.rss_kib
        );
        let sandbox = Sandbox::spawn(FrozenProcess {
            pid: self.clone.pid,
            memory: Vec::new(),
        });
        resolve_and_run(threat_id, pharmacy, sandbox, dashboard)
    }
}

#[cfg(test)]
mod pharmacy_flow_tests {
    use super::*;
    use crate::evolution::alleles::{Allele, GenePayload};
    use crate::evolution::sandbox::FrozenProcess;
    use crate::ledger::registry::FakeGenomeSource;

    fn winning_gene() -> CompiledGene {
        GenePayload {
            sequence: vec![Allele::Allele04, Allele::Allele12],
        }
        .compile()
    }

    fn mock_sandbox() -> Sandbox {
        Sandbox::spawn(FrozenProcess {
            pid: 4242,
            memory: vec![],
        })
    }

    fn sample_threat_id() -> ThreatId {
        ThreatId([9u8; 32])
    }

    fn pharmacy_over(ledger: SharedFakeLedger) -> Pharmacy {
        Pharmacy {
            genome: Box::new(ledger.clone()),
            committer: Box::new(ledger),
        }
    }

    #[test]
    fn resolves_verifies_and_neutralizes_then_apoptoses() {
        let threat_id = sample_threat_id();
        let ledger = SharedFakeLedger::new();
        ledger.commit_gene(&threat_id, &winning_gene()).expect("seed cure");
        let pharmacy = pharmacy_over(ledger);

        let outcome = resolve_and_run(&threat_id, &pharmacy, mock_sandbox(), None);
        assert_eq!(outcome, PharmacyOutcome::Neutralized);
    }

    #[test]
    fn suppressed_gene_is_halted_before_any_run() {
        let threat_id = sample_threat_id();
        let ledger = SharedFakeLedger::new();
        ledger.commit_gene(&threat_id, &winning_gene()).expect("seed cure");
        ledger.suppress(&threat_id);
        let pharmacy = pharmacy_over(ledger);

        let outcome = resolve_and_run(&threat_id, &pharmacy, mock_sandbox(), None);
        assert_eq!(outcome, PharmacyOutcome::Suppressed);
    }

    #[test]
    fn unverified_gene_hash_is_rejected() {
        let threat_id = sample_threat_id();
        // Genome Registry entry claims a hash that doesn't match its own gene_seq — models
        // corrupt/tampered account data, since a normal `commit_gene` can't produce this.
        let mut genome = FakeGenomeSource::new();
        genome.publish_raw(threat_id, [0xEE; 32], winning_gene().to_bytes());
        let pharmacy = Pharmacy {
            genome: Box::new(genome),
            committer: Box::new(SharedFakeLedger::new()),
        };

        let outcome = resolve_and_run(&threat_id, &pharmacy, mock_sandbox(), None);
        assert_eq!(outcome, PharmacyOutcome::GeneHashUnverified);
    }

    #[test]
    fn unknown_threat_id_evolves_fuzzes_and_commits_a_cure() {
        let threat_id = sample_threat_id();
        let ledger = SharedFakeLedger::new();
        let pharmacy = pharmacy_over(ledger.clone());

        let outcome = resolve_and_run(&threat_id, &pharmacy, mock_sandbox(), None);

        assert_eq!(outcome, PharmacyOutcome::Neutralized);
        assert!(
            ledger.get(&threat_id).unwrap().is_some(),
            "evolve_and_commit published the cure it found"
        );
    }

    struct FailingCommitter;
    impl GeneCommitter for FailingCommitter {
        fn commit_gene(&self, _threat_id: &ThreatId, _gene: &CompiledGene) -> Result<(), crate::ledger::LedgerError> {
            Err(crate::ledger::LedgerError::GeneHashMismatch)
        }
    }

    #[test]
    fn commit_failure_surfaces_as_ledger_unavailable() {
        let threat_id = sample_threat_id();
        let pharmacy = Pharmacy {
            genome: Box::new(FakeGenomeSource::new()),
            committer: Box::new(FailingCommitter),
        };

        let outcome = resolve_and_run(&threat_id, &pharmacy, mock_sandbox(), None);
        assert_eq!(outcome, PharmacyOutcome::LedgerUnavailable);
    }
}

#[cfg(test)]
mod live_dispense_tests {
    use super::*;
    use crate::evolution::alleles::{Allele, GenePayload};

    fn seeded_pharmacy(threat_id: ThreatId, suppressed: bool) -> Pharmacy {
        let ledger = SharedFakeLedger::new();
        let gene = GenePayload {
            sequence: vec![Allele::Allele04, Allele::Allele12],
        }
        .compile();
        ledger.commit_gene(&threat_id, &gene).expect("seed cure");
        if suppressed {
            // The on-chain migration collapsed the old mock's separate "broadcast then
            // drain" token exchange into a single always-fresh account read
            // (see ../../ledger/registry.rs).
            ledger.suppress(&threat_id);
        }
        Pharmacy {
            genome: Box::new(ledger.clone()),
            committer: Box::new(ledger),
        }
    }

    fn woken_soldier(threat_id: ThreatId) -> Soldier {
        // Own PID so the release path is guarded off; dispense doesn't release anyway.
        Soldier::wake(
            Spore::dormant(),
            &WakeSignal {
                threat_id,
                pid: std::process::id(),
            },
        )
    }

    #[test]
    fn active_cure_is_dispensed_in_sandbox() {
        let threat_id = ThreatId([3u8; 32]);
        let pharmacy = seeded_pharmacy(threat_id, false);
        let outcome = woken_soldier(threat_id).dispense(&threat_id, &pharmacy, None);
        assert_eq!(outcome, PharmacyOutcome::Neutralized);
    }

    #[test]
    fn suppressed_gene_halts_the_live_cure_before_run() {
        let threat_id = ThreatId([3u8; 32]);
        let pharmacy = seeded_pharmacy(threat_id, true);

        let outcome = woken_soldier(threat_id).dispense(&threat_id, &pharmacy, None);
        assert_eq!(outcome, PharmacyOutcome::Suppressed);
    }
}
