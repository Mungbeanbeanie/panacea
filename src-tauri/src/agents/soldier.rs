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
) -> std::io::Result<(Spore, PharmacyOutcome)> {
    let spore = Spore::load_or_dormant(spore_path);
    let soldier = Soldier::wake(spore, signal);
    let outcome = soldier.dispense(&signal.threat_id, pharmacy);
    release_target(soldier.pid); // demo cleanup: unfreeze/terminate the scripted helper
    let spore = soldier.apoptosis(spore_path)?;
    Ok((spore, outcome))
}

/// Run the Soldier as the consumer of the Scout's wake channel (replaces the Phase-1 stub).
///
/// Loops for the channel's lifetime; each `WakeSignal` runs one full spore lifecycle against
/// the shared `pharmacy` — resolving and dispensing the cure, with the epigenetic kill-switch
/// checked before any fetch/exec.
pub fn run(
    wake_rx: Receiver<WakeSignal>,
    spore_path: std::path::PathBuf,
    pharmacy: Pharmacy,
) -> JoinHandle<()> {
    thread::spawn(move || {
        for signal in wake_rx {
            match handle_wake(&signal, &spore_path, &pharmacy) {
                Ok((spore, outcome)) => println!(
                    "[soldier] cure {outcome:?}; apoptosis → spore generation {}",
                    spore.generation()
                ),
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
        let (spore, _) = handle_wake(&signal, &path, &pharmacy).expect("handle_wake");
        assert_eq!(
            spore.generation(),
            1,
            "dormant(0) → woke once → generation 1"
        );
        assert!(path.exists(), "spore re-serialized to disk");

        // A second wake loads generation 1 and re-serializes as 2.
        let (spore2, _) = handle_wake(&signal, &path, &pharmacy).expect("handle_wake again");
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
        let (spore, _) = handle_wake(&signal, &path, &pharmacy).expect("handle_wake");
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

use crate::core::{GeneHandle, ThreatId};
use crate::evolution::sandbox::{FrozenProcess, Sandbox, TrialOutcome};
use crate::ledger::ipfs::{Cid, FakeGeneStore, GeneStore};
use crate::ledger::registry::{FakeGenomeSource, GenomeSource};
use crate::ledger::LedgerError;

/// Outcome of a Soldier's resolve → verify → fetch → execute → apoptosis run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PharmacyOutcome {
    /// No Genome Registry row for this `Threat_ID` yet.
    NoCureAvailable,
    /// `Epigenetic_Status = 1` — halted before any fetch or execution.
    Suppressed,
    /// The fetched gene's hash didn't match the on-chain `Wasm_Gene_Hash`.
    GeneHashUnverified,
    /// The Genome Registry read or the IPFS fetch itself failed (RPC/network error) —
    /// distinct from a legitimate "no cure published" result.
    LedgerUnavailable,
    /// Gene ran in-sandbox but didn't neutralize the target; apoptosis still follows.
    Ineffective,
    /// Gene neutralized the target in-sandbox; apoptosis follows.
    Neutralized,
}

/// Resolves `threat_id` against the Genome Registry **on demand** (never a passive scan),
/// checks `Epigenetic_Status` *before* touching IPFS or the sandbox, fetches the gene
/// bytecode by CID and checks its hash against the on-chain `Wasm_Gene_Hash` (reading a
/// `confirmed`-commitment Solana account already carries the integrity guarantee a Merkle
/// proof used to provide — see `../../ledger/state.rs`), runs it in-sandbox, and undergoes
/// apoptosis — the sandbox is torn down before returning either way.
pub fn resolve_and_run(
    threat_id: &ThreatId,
    genome_source: &dyn GenomeSource,
    ipfs: &dyn GeneStore,
    mut sandbox: Sandbox,
) -> PharmacyOutcome {
    let entry = match genome_source.get(threat_id) {
        Ok(Some(entry)) => entry,
        Ok(None) => return PharmacyOutcome::NoCureAvailable,
        Err(_) => return PharmacyOutcome::LedgerUnavailable,
    };

    if entry.epigenetic_status == 1 {
        return PharmacyOutcome::Suppressed;
    }

    let cid = Cid(entry.ipfs_cid.clone());
    let gene = match ipfs.fetch(&cid, GeneHandle(entry.gene_hash)) {
        Ok(gene) => gene,
        Err(LedgerError::GeneHashMismatch) => return PharmacyOutcome::GeneHashUnverified,
        Err(_) => return PharmacyOutcome::LedgerUnavailable,
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

/// The local pharmacy a running Soldier consults: the Genome Registry (Ledger 3) and gene
/// storage, behind trait objects so the offline test suite can swap in fakes instead of
/// hitting live devnet/IPFS (Q8). Reading the Genome Registry fresh on every dispense means
/// there's no separate "absorb suppressor tokens" step anymore — the on-chain account is
/// always the live truth (the old mock's broadcast/drain pair collapsed into one always-
/// fresh read).
pub struct Pharmacy {
    pub genome: Box<dyn GenomeSource>,
    pub ipfs: Box<dyn GeneStore>,
}

impl Pharmacy {
    /// An empty, offline pharmacy — no cures published. For tests that only exercise the
    /// spore lifecycle, not the pharmacy flow itself.
    pub fn empty() -> Self {
        Self {
            genome: Box::new(FakeGenomeSource::new()),
            ipfs: Box::new(FakeGeneStore::new()),
        }
    }
}

impl Soldier {
    /// Dispense the cure for `threat_id` through the full pharmacy flow: resolve →
    /// **kill-switch check** → fetch-and-verify → run in-sandbox.
    fn dispense(&self, threat_id: &ThreatId, pharmacy: &Pharmacy) -> PharmacyOutcome {
        println!(
            "[soldier] dispensing for pid {} (cloned rss {} KiB)",
            self.clone.pid, self.clone.rss_kib
        );
        let sandbox = Sandbox::spawn(FrozenProcess {
            pid: self.clone.pid,
            memory: Vec::new(),
        });
        resolve_and_run(
            threat_id,
            pharmacy.genome.as_ref(),
            pharmacy.ipfs.as_ref(),
            sandbox,
        )
    }
}

#[cfg(test)]
mod pharmacy_flow_tests {
    use super::*;
    use crate::evolution::alleles::{Allele, GenePayload};
    use crate::evolution::sandbox::FrozenProcess;
    use crate::ledger::ipfs::FakeGeneStore;
    use crate::ledger::registry::FakeGenomeSource;

    const TEST_CID: &str = "bafy-test-gene";

    fn winning_gene() -> GenePayload {
        GenePayload {
            sequence: vec![Allele::Allele04, Allele::Allele12],
        }
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

    #[test]
    fn resolves_verifies_and_neutralizes_then_apoptoses() {
        let threat_id = sample_threat_id();
        let gene = winning_gene();

        let mut ipfs = FakeGeneStore::new();
        ipfs.store(TEST_CID, gene.clone());

        let mut genome = FakeGenomeSource::new();
        genome.publish(threat_id, gene.gene_hash().0, TEST_CID.to_string());

        let outcome = resolve_and_run(&threat_id, &genome, &ipfs, mock_sandbox());
        assert_eq!(outcome, PharmacyOutcome::Neutralized);
    }

    #[test]
    fn suppressed_gene_is_halted_before_any_fetch() {
        let threat_id = sample_threat_id();
        let gene = winning_gene();

        let mut ipfs = FakeGeneStore::new();
        ipfs.store(TEST_CID, gene.clone());

        let mut genome = FakeGenomeSource::new();
        genome.publish(threat_id, gene.gene_hash().0, TEST_CID.to_string());
        genome.suppress(&threat_id);

        let outcome = resolve_and_run(&threat_id, &genome, &ipfs, mock_sandbox());

        assert_eq!(outcome, PharmacyOutcome::Suppressed);
        assert_eq!(ipfs.fetch_calls(), 0, "must not fetch a suppressed gene");
    }

    #[test]
    fn unverified_gene_hash_is_rejected() {
        let threat_id = sample_threat_id();
        let gene = winning_gene();

        let mut ipfs = FakeGeneStore::new();
        ipfs.store(TEST_CID, gene.clone());

        // Genome Registry claims a hash that doesn't match what's actually stored at the CID.
        let mut genome = FakeGenomeSource::new();
        genome.publish(threat_id, [0xEE; 32], TEST_CID.to_string());

        let outcome = resolve_and_run(&threat_id, &genome, &ipfs, mock_sandbox());

        assert_eq!(outcome, PharmacyOutcome::GeneHashUnverified);
        // Verification happens by comparing the fetched bytes' hash (see ../../ledger/ipfs.rs)
        // rather than a pre-fetch Merkle check, so the fetch itself does happen here — a
        // deliberate ordering change from the old mock, matching source-of-truth.md's
        // updated Ledger 3 wording ("fetches... checks its hash against Wasm_Gene_Hash").
        assert_eq!(ipfs.fetch_calls(), 1);
    }

    #[test]
    fn unknown_threat_id_has_no_cure() {
        let threat_id = sample_threat_id();
        let genome = FakeGenomeSource::new();
        let ipfs = FakeGeneStore::new();

        let outcome = resolve_and_run(&threat_id, &genome, &ipfs, mock_sandbox());

        assert_eq!(outcome, PharmacyOutcome::NoCureAvailable);
        assert_eq!(ipfs.fetch_calls(), 0);
    }
}

#[cfg(test)]
mod live_dispense_tests {
    use super::*;
    use crate::evolution::alleles::{Allele, GenePayload};
    use crate::ledger::ipfs::FakeGeneStore;
    use crate::ledger::registry::FakeGenomeSource;

    const TEST_CID: &str = "bafy-test-gene";

    fn seeded_pharmacy(threat_id: ThreatId, suppressed: bool) -> Pharmacy {
        let gene = GenePayload {
            sequence: vec![Allele::Allele04, Allele::Allele12],
        };
        let mut ipfs = FakeGeneStore::new();
        ipfs.store(TEST_CID, gene.clone());
        let mut genome = FakeGenomeSource::new();
        genome.publish(threat_id, gene.gene_hash().0, TEST_CID.to_string());
        if suppressed {
            // Suppressed directly on the Genome Registry entry — the on-chain migration
            // collapsed the old mock's separate "broadcast then drain" token exchange into
            // a single always-fresh account read (see ../../ledger/registry.rs).
            genome.suppress(&threat_id);
        }
        Pharmacy {
            genome: Box::new(genome),
            ipfs: Box::new(ipfs),
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
        let outcome = woken_soldier(threat_id).dispense(&threat_id, &pharmacy);
        assert_eq!(outcome, PharmacyOutcome::Neutralized);
    }

    #[test]
    fn suppressed_gene_halts_the_live_cure_before_fetch() {
        let threat_id = ThreatId([3u8; 32]);
        let pharmacy = seeded_pharmacy(threat_id, true);

        let outcome = woken_soldier(threat_id).dispense(&threat_id, &pharmacy);
        assert_eq!(outcome, PharmacyOutcome::Suppressed);
    }
}
