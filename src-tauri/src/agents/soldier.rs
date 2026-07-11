//! Soldier: dormant on-disk remediation payload ("spore"). Wakes on a high-confidence
//! signal, isolates and neutralizes the threat inside the sandbox, then undergoes apoptosis
//! (re-serializes to a spore). Checks Epigenetic_Status before running any gene — a
//! suppressed gene (status 1) is never fetched or executed
//! (see ../../.claude/skills/suppression-path-test). Stage 2.

use std::path::Path;
use std::sync::mpsc::Receiver;
use std::thread::{self, JoinHandle};

use serde::{Deserialize, Serialize};

use super::WakeSignal;
use crate::core::Pid;

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

    /// Remediation seam. Phase 4 (sandbox + fuzz) / Phase 6 (verified gene) plug in here, and
    /// the Epigenetic_Status check (Phase 9) goes *before* any gene fetch/exec at that point.
    /// Mocked now: logs only, kills nothing.
    fn neutralize(&self) {
        println!(
            "[soldier] neutralize (mock) pid {} rss {} KiB",
            self.clone.pid, self.clone.rss_kib
        );
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

/// Full lifecycle for one wake: load spore → wake + clone → act → demo-release → apoptosis.
pub fn handle_wake(signal: &WakeSignal, spore_path: &Path) -> std::io::Result<Spore> {
    let spore = Spore::load_or_dormant(spore_path);
    let soldier = Soldier::wake(spore, signal);
    soldier.neutralize();
    release_target(soldier.pid); // demo cleanup (guarded); Phase 4/6 replace with the real gene
    soldier.apoptosis(spore_path)
}

/// Run the Soldier as the consumer of the Scout's wake channel (replaces the Phase-1 stub).
///
/// Loops for the channel's lifetime; each `WakeSignal` runs one full spore lifecycle.
pub fn run(wake_rx: Receiver<WakeSignal>, spore_path: std::path::PathBuf) -> JoinHandle<()> {
    thread::spawn(move || {
        for signal in wake_rx {
            match handle_wake(&signal, &spore_path) {
                Ok(spore) => {
                    println!(
                        "[soldier] apoptosis → spore generation {}",
                        spore.generation()
                    )
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

        let spore = handle_wake(&signal, &path).expect("handle_wake");
        assert_eq!(
            spore.generation(),
            1,
            "dormant(0) → woke once → generation 1"
        );
        assert!(path.exists(), "spore re-serialized to disk");

        // A second wake loads generation 1 and re-serializes as 2.
        let spore2 = handle_wake(&signal, &path).expect("handle_wake again");
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

        let spore = handle_wake(&signal, &path).expect("handle_wake");
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
//!
//! The full dormant-spore lifecycle (wake signal, apoptosis re-serialization) is Phase 3
//! and not yet built. [`resolve_and_run`] is the Pharmacy Flow slice of a Soldier that
//! Phase 6 needs: given a `Threat_ID` it already has (from a wake signal), resolve the cure
//! from the Genome Registry, verify it, run it in-sandbox, and undergo apoptosis.

use crate::core::ThreatId;
use crate::evolution::sandbox::{Sandbox, TrialOutcome};
use crate::ledger::registry::{EpigeneticStatus, GenomeRegistry, MockIpfsStore};
use crate::ledger::state::{MerkleProof, StateLedger};

/// Outcome of a Soldier's resolve → verify → fetch → execute → apoptosis run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PharmacyOutcome {
    /// No Genome Registry row for this `Threat_ID` yet.
    NoCureAvailable,
    /// `Epigenetic_Status = 1` — halted before any fetch or execution.
    Suppressed,
    /// The claimed `Wasm_Gene_Hash` didn't verify against the State Ledger Merkle root.
    GeneHashUnverified,
    /// Gene ran in-sandbox but didn't neutralize the target; apoptosis still follows.
    Ineffective,
    /// Gene neutralized the target in-sandbox; apoptosis follows.
    Neutralized,
}

/// Resolves `threat_id` against the Genome Registry **on demand** (never a passive scan),
/// checks `Epigenetic_Status` *before* touching IPFS or the sandbox, verifies the claimed
/// `Wasm_Gene_Hash` against the State Ledger's Merkle root, fetches the bytecode, runs it
/// in-sandbox, and undergoes apoptosis — the sandbox is torn down before returning either way.
pub fn resolve_and_run(
    threat_id: &ThreatId,
    genome_registry: &GenomeRegistry,
    ipfs: &MockIpfsStore,
    state_ledger: &StateLedger,
    gene_proof: &MerkleProof,
    mut sandbox: Sandbox,
) -> PharmacyOutcome {
    let Some(entry) = genome_registry.get(threat_id) else {
        return PharmacyOutcome::NoCureAvailable;
    };

    if entry.epigenetic_status == EpigeneticStatus::Suppressed {
        return PharmacyOutcome::Suppressed;
    }

    if !state_ledger.verify_gene(entry.gene_hash.0, gene_proof) {
        return PharmacyOutcome::GeneHashUnverified;
    }

    let Some(gene) = ipfs.fetch(&entry.gene_hash) else {
        return PharmacyOutcome::NoCureAvailable;
    };

    let outcome = sandbox.run(&gene.sequence);
    sandbox.teardown(); // apoptosis

    match outcome {
        TrialOutcome::TargetCrashed => PharmacyOutcome::Neutralized,
        TrialOutcome::TargetSurvived | TrialOutcome::HostDestabilized => {
            PharmacyOutcome::Ineffective
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evolution::alleles::{Allele, GenePayload};
    use crate::evolution::sandbox::FrozenProcess;
    use crate::ledger::client::MockConjugationLink;
    use crate::ledger::registry::BehavioralSchema;

    fn winning_gene() -> GenePayload {
        GenePayload { sequence: vec![Allele::Allele04, Allele::Allele12] }
    }

    fn mock_sandbox() -> Sandbox {
        Sandbox::spawn(FrozenProcess { pid: 4242, memory: vec![] })
    }

    fn sample_threat_id() -> ThreatId {
        BehavioralSchema(vec!["vssadmin".into()]).threat_id()
    }

    #[test]
    fn resolves_verifies_and_neutralizes_then_apoptoses() {
        let threat_id = sample_threat_id();
        let gene = winning_gene();

        let mut ipfs = MockIpfsStore::new();
        let uri = ipfs.store(gene.clone());

        let mut genome = GenomeRegistry::new();
        genome.publish(threat_id, gene.gene_hash(), uri);

        let mut link = MockConjugationLink::new();
        let header = link.commit_block(vec![], vec![gene.gene_hash().0], vec!["v1".into()], 0);
        let mut state_ledger = StateLedger::new();
        state_ledger.adopt(header);
        let gene_proof = link.genome_proof(0).unwrap();

        let outcome = resolve_and_run(
            &threat_id,
            &genome,
            &ipfs,
            &state_ledger,
            &gene_proof,
            mock_sandbox(),
        );
        assert_eq!(outcome, PharmacyOutcome::Neutralized);
    }

    #[test]
    fn suppressed_gene_is_halted_before_any_fetch() {
        let threat_id = sample_threat_id();
        let gene = winning_gene();

        let mut ipfs = MockIpfsStore::new();
        let uri = ipfs.store(gene.clone());

        let mut genome = GenomeRegistry::new();
        genome.publish(threat_id, gene.gene_hash(), uri);
        genome.suppress(&threat_id);

        let mut link = MockConjugationLink::new();
        let header = link.commit_block(vec![], vec![gene.gene_hash().0], vec!["v1".into()], 0);
        let mut state_ledger = StateLedger::new();
        state_ledger.adopt(header);
        let gene_proof = link.genome_proof(0).unwrap();

        let outcome = resolve_and_run(
            &threat_id,
            &genome,
            &ipfs,
            &state_ledger,
            &gene_proof,
            mock_sandbox(),
        );

        assert_eq!(outcome, PharmacyOutcome::Suppressed);
        assert_eq!(ipfs.fetch_calls(), 0, "must not fetch a suppressed gene");
    }

    #[test]
    fn unverified_gene_hash_is_rejected() {
        let threat_id = sample_threat_id();
        let gene = winning_gene();

        let mut ipfs = MockIpfsStore::new();
        let uri = ipfs.store(gene.clone());

        let mut genome = GenomeRegistry::new();
        genome.publish(threat_id, gene.gene_hash(), uri);

        // Commit a block whose genome root does NOT include this gene's hash.
        let mut link = MockConjugationLink::new();
        let header = link.commit_block(vec![], vec![[0xEE; 32]], vec!["v1".into()], 0);
        let mut state_ledger = StateLedger::new();
        state_ledger.adopt(header);
        let bogus_proof = link.genome_proof(0).unwrap();

        let outcome = resolve_and_run(
            &threat_id,
            &genome,
            &ipfs,
            &state_ledger,
            &bogus_proof,
            mock_sandbox(),
        );

        assert_eq!(outcome, PharmacyOutcome::GeneHashUnverified);
        assert_eq!(ipfs.fetch_calls(), 0, "must verify before fetching");
    }

    #[test]
    fn unknown_threat_id_has_no_cure() {
        let threat_id = sample_threat_id();
        let genome = GenomeRegistry::new();
        let ipfs = MockIpfsStore::new();

        let mut link = MockConjugationLink::new();
        let header = link.commit_block(vec![], vec![[0x11; 32]], vec!["v1".into()], 0);
        let mut state_ledger = StateLedger::new();
        state_ledger.adopt(header);
        let gene_proof = link.genome_proof(0).unwrap();

        let outcome = resolve_and_run(
            &threat_id,
            &genome,
            &ipfs,
            &state_ledger,
            &gene_proof,
            mock_sandbox(),
        );

        assert_eq!(outcome, PharmacyOutcome::NoCureAvailable);
        assert_eq!(ipfs.fetch_calls(), 0);
    }
}
