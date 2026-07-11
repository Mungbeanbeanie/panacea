//! Scout: ultra-light background daemon. Traces process behavior (syscall anomalies,
//! memory boundary violations, I/O bursts) and accumulates a per-process trajectory score.
//! Observes only — never terminates. Crossing 100 pts suspends the target and signals the
//! local Soldier spore. See ../../.claude/docs/architecture.md, Stage 1.

use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use sha2::{Digest, Sha256};

use super::{ThreatReport, WakeSignal};
use crate::core::{Action, AnomalyScore, BehavioralSchema, Pid, ThreatId, ANOMALY_THRESHOLD};

/// How long the Scout daemon sleeps between observation ticks.
///
/// Tuned for "ultra-light": long enough that the idle loop costs effectively nothing, short
/// enough to catch a fast-moving trajectory. A PoC tunable — not a Source-of-Truth value.
const TICK_INTERVAL: Duration = Duration::from_millis(500);

/// Lowest PID the Scout will suspend — a coarse guard against freezing low-numbered system
/// processes (kernel, init/launchd). Combined with the self-PID check in [`suspend`], it
/// keeps the hard interrupt pointed only at processes the Scout is actively scoring.
const PID_FLOOR: Pid = 1000;

/// Weight one action contributes to a process trajectory (Source of Truth, Stage 1).
fn weight(action: Action) -> AnomalyScore {
    match action {
        Action::HiddenChildFromTemp => AnomalyScore(20),
        Action::NetEnumWithVssTamper => AnomalyScore(50),
        Action::HighEntropyFileLoop => AnomalyScore(40),
    }
}

/// One process's accumulating Stage-1 trajectory.
struct Trajectory {
    /// Cumulative anomaly weight so far.
    score: AnomalyScore,
    /// Actions observed, in order — becomes the [`BehavioralSchema`] when the threshold fires.
    actions: Vec<Action>,
    /// Set once the threshold is crossed, so a PID is suspended and reported exactly once.
    fired: bool,
}

impl Trajectory {
    fn new() -> Self {
        Self {
            score: AnomalyScore(0),
            actions: Vec::new(),
            fired: false,
        }
    }
}

/// A single behavioral observation: process `pid` performed `action`.
pub struct Observation {
    pub pid: Pid,
    pub action: Action,
}

/// Source of behavioral observations for the Scout.
///
/// The PoC feeds a scripted sequence ([`ScriptedSource`]); a real syscall / Endpoint-Security
/// tracer implements this same trait later, and the scoring loop is unchanged.
pub trait BehaviorSource: Send {
    /// Return the observations seen since the previous poll (may be empty).
    fn poll(&mut self) -> Vec<Observation>;
}

/// The Stage-1 scoring daemon: owns per-PID trajectories and the outbound signal channels.
pub struct Scout<S: BehaviorSource> {
    source: S,
    scores: HashMap<Pid, Trajectory>,
    /// Step 6 — wake the local Soldier spore (Phase 3 consumes this).
    wake_tx: Sender<WakeSignal>,
    /// Step 7 — submit the flagged vector to the Threat Registry (Phase 2 consumes this).
    threat_tx: Sender<ThreatReport>,
}

impl<S: BehaviorSource + 'static> Scout<S> {
    /// Build a Scout over `source`, emitting wake signals and threat reports on the channels.
    pub fn new(source: S, wake_tx: Sender<WakeSignal>, threat_tx: Sender<ThreatReport>) -> Self {
        Self {
            source,
            scores: HashMap::new(),
            wake_tx,
            threat_tx,
        }
    }

    /// Run the ultra-light loop on a background thread; returns its join handle.
    ///
    /// The daemon runs for the life of the process, sleeping [`TICK_INTERVAL`] between ticks.
    pub fn spawn(mut self) -> JoinHandle<()> {
        thread::spawn(move || loop {
            self.tick();
            thread::sleep(TICK_INTERVAL);
        })
    }

    /// One observation cycle: fold new observations into per-PID trajectories and, on the
    /// first threshold crossing for a PID, suspend it and signal downstream.
    fn tick(&mut self) {
        for Observation { pid, action } in self.source.poll() {
            let trajectory = self.scores.entry(pid).or_insert_with(Trajectory::new);
            if trajectory.fired {
                continue;
            }
            trajectory.score = AnomalyScore(trajectory.score.0 + weight(action).0);
            trajectory.actions.push(action);

            if trajectory.score >= ANOMALY_THRESHOLD {
                trajectory.fired = true;
                let schema = BehavioralSchema {
                    actions: trajectory.actions.clone(),
                };
                let id = threat_id(&schema);
                suspend(pid);
                let _ = self.wake_tx.send(WakeSignal { threat_id: id, pid });
                let _ = self.threat_tx.send(ThreatReport {
                    threat_id: id,
                    schema,
                });
            }
        }
    }
}

/// `Threat_ID` = SHA-256 over the ordered action bytes.
///
/// Deterministic and serialization-format-free, so any node computes the same id from the
/// same behavioral sequence.
fn threat_id(schema: &BehavioralSchema) -> ThreatId {
    let mut hasher = Sha256::new();
    for action in &schema.actions {
        hasher.update([*action as u8]);
    }
    let digest = hasher.finalize();
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&digest);
    ThreatId(bytes)
}

/// Suspend all threads of `pid` — the Stage-1 hard interrupt.
///
/// Guards the Scout's own process and low-numbered system PIDs so the freeze only ever lands
/// on a process the Scout is scoring. Unix uses `SIGSTOP`; other targets are out of PoC scope.
#[cfg(unix)]
fn suspend(pid: Pid) {
    if pid == std::process::id() || pid < PID_FLOOR {
        return;
    }
    use nix::sys::signal::{kill, Signal};
    let _ = kill(nix::unistd::Pid::from_raw(pid as i32), Signal::SIGSTOP);
}

/// Non-Unix stub: the PoC targets macOS/Linux; Windows suspension (`NtSuspendProcess`) is
/// out of scope.
#[cfg(not(unix))]
fn suspend(_pid: Pid) {}

/// The scripted trajectory the PoC target performs — A → B → C, summing 20 + 50 + 40 = 110,
/// crossing the 100-pt threshold on the third action. Shared so the demo pharmacy can publish
/// a cure under the same `Threat_ID` the Scout will emit for it.
pub const SCRIPTED_TRAJECTORY: [Action; 3] = [
    Action::HiddenChildFromTemp,
    Action::NetEnumWithVssTamper,
    Action::HighEntropyFileLoop,
];

/// The `Threat_ID` the Scout emits for [`SCRIPTED_TRAJECTORY`].
pub fn scripted_threat_id() -> ThreatId {
    threat_id(&BehavioralSchema {
        actions: SCRIPTED_TRAJECTORY.to_vec(),
    })
}

/// PoC behavior source: replays [`SCRIPTED_TRAJECTORY`] against one real target PID, then goes
/// quiet. One action per tick, so the trajectory climbs 20 → 70 → 110 and fires on the third.
pub struct ScriptedSource {
    target: Pid,
    script: std::vec::IntoIter<Action>,
}

impl ScriptedSource {
    /// Spawn a benign helper process to stand in for the "bad" process and script its
    /// trajectory across the 100-pt threshold. Returns the source and the helper handle;
    /// the caller keeps the handle alive for the demo's lifetime.
    pub fn spawn_target() -> std::io::Result<(Self, std::process::Child)> {
        let child = std::process::Command::new("sleep").arg("600").spawn()?;
        let target = child.id();
        let script = SCRIPTED_TRAJECTORY.to_vec().into_iter();
        Ok((Self { target, script }, child))
    }
}

impl BehaviorSource for ScriptedSource {
    fn poll(&mut self) -> Vec<Observation> {
        match self.script.next() {
            Some(action) => vec![Observation {
                pid: self.target,
                action,
            }],
            None => Vec::new(),
        }
    }
}

/// `Behavioral_Schema` hash submitted to the Threat Registry PDA — mirrors [`threat_id`]'s
/// own hashing so both are deterministic over the same action sequence.
fn behavioral_schema_hash(schema: &BehavioralSchema) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for action in &schema.actions {
        hasher.update([*action as u8]);
    }
    hasher.finalize().into()
}

/// Live demo entry point: wires a scripted target against the **real** Solana devnet
/// program and Pinata IPFS (Phase 11 — "no mocks left in the ledger path"). Seeds the
/// scripted trajectory's cure via a real `commit_gene` + `suppress_gene` pair before the
/// Scout starts, so the live wake demonstrates the kill-switch halting the cure before any
/// fetch/exec — now as genuine devnet transactions instead of local mocks. Requires
/// `PINATA_JWT` to be set; without it the IPFS upload fails loudly and the demo seed is
/// skipped (the Scout/Soldier lifecycle still runs, just with nothing to dispense).
pub fn spawn_demo() -> JoinHandle<()> {
    use std::sync::Arc;

    use anchor_client::Cluster;
    use solana_keypair::read_keypair_file;

    use super::soldier::Pharmacy;
    use crate::evolution::alleles::{Allele, GenePayload};
    use crate::ledger::client::SolanaConjugationLink;
    use crate::ledger::ipfs::IpfsClient;
    use crate::ledger::registry::GenomeRegistry;
    use crate::ledger::state::SolanaLightClient;

    let (wake_tx, wake_rx) = channel::<WakeSignal>();
    let (threat_tx, threat_rx) = channel::<ThreatReport>();

    let deployer = Arc::new(
        read_keypair_file("~/.config/solana/id.json").expect("read deployer keypair"),
    );
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let validators: Vec<_> = (1..=5)
        .map(|i| {
            let path = format!("{manifest_dir}/../keys/lymph-nodes/validator-{i}.json");
            read_keypair_file(&path).expect("read Lymph Node validator keypair")
        })
        .collect();

    let conjugation = SolanaConjugationLink::new(Cluster::Devnet, deployer.clone())
        .expect("connect SolanaConjugationLink to devnet");
    let light_client = SolanaLightClient::new(Cluster::Devnet, deployer.clone())
        .expect("connect SolanaLightClient to devnet");
    let ipfs = IpfsClient::new();

    // Demo seed: publish the cure for the scripted trajectory, then immediately suppress
    // it — so the live wake demonstrates the kill-switch (Phase 9) via real transactions.
    let threat_id = scripted_threat_id();
    let gene = GenePayload {
        sequence: vec![Allele::Allele04, Allele::Allele12],
    };
    match ipfs.upload(&gene) {
        Ok(cid) => {
            match conjugation.commit_gene(threat_id, gene.gene_hash(), cid, &validators) {
                Ok(sig) => {
                    println!("[ledger3] commit_gene {sig}");
                    match conjugation.suppress_gene(threat_id, &validators) {
                        Ok(sig) => println!("[ledger3] suppress_gene {sig}"),
                        Err(e) => eprintln!("[ledger3] suppress_gene failed: {e}"),
                    }
                }
                Err(e) => eprintln!("[ledger3] commit_gene failed: {e}"),
            }
        }
        Err(e) => eprintln!("[ledger3] IPFS upload failed (is PINATA_JWT set?): {e}"),
    }

    let pharmacy = Pharmacy {
        genome: Box::new(GenomeRegistry::new(light_client)),
        ipfs: Box::new(ipfs),
    };

    // Real Soldier consumes the wake channel against the pharmacy: Phase-3 spore lifecycle +
    // Phase-6/11 pharmacy resolution against live devnet + Phase-9 kill-switch.
    let spore_path = std::env::temp_dir().join("bio-digital-defense.spore");
    super::soldier::run(wake_rx, spore_path, pharmacy);

    // Threat Registry (Ledger 2): each report is now a real `submit_threat` transaction.
    thread::spawn(move || {
        for report in threat_rx {
            let schema_hash = behavioral_schema_hash(&report.schema);
            match conjugation.submit_threat(report.threat_id, schema_hash) {
                Ok(sig) => println!(
                    "[ledger2] submit_threat {sig} ({} actions)",
                    report.schema.actions.len()
                ),
                Err(e) => eprintln!("[ledger2] submit_threat failed: {e}"),
            }
        }
    });

    let (source, child) =
        ScriptedSource::spawn_target().expect("failed to spawn scripted target process");
    // Drop the helper handle: the Scout suspends it on threshold cross and the Soldier
    // releases and terminates it during apoptosis; any residue is reaped on app exit.
    drop(child);

    Scout::new(source, wake_tx, threat_tx).spawn()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A source that yields all its observations on the first poll, then nothing.
    struct VecSource(Vec<Observation>);
    impl BehaviorSource for VecSource {
        fn poll(&mut self) -> Vec<Observation> {
            std::mem::take(&mut self.0)
        }
    }

    #[test]
    fn crosses_threshold_fires_once_and_emits() {
        // Use our own PID so the real SIGSTOP path is guarded off (never freeze the test).
        let pid = std::process::id();
        let (wake_tx, wake_rx) = channel();
        let (threat_tx, threat_rx) = channel();
        let observations = vec![
            Observation {
                pid,
                action: Action::HiddenChildFromTemp,
            }, // 20
            Observation {
                pid,
                action: Action::NetEnumWithVssTamper,
            }, // 70
            Observation {
                pid,
                action: Action::HighEntropyFileLoop,
            }, // 110 -> fire
        ];
        let mut scout = Scout::new(VecSource(observations), wake_tx, threat_tx);
        scout.tick();

        let wake = wake_rx.try_recv().expect("expected a wake signal");
        assert_eq!(wake.pid, pid);
        let report = threat_rx.try_recv().expect("expected a threat report");
        assert_eq!(report.schema.actions.len(), 3);
        assert_eq!(wake.threat_id, report.threat_id);

        // Fires exactly once, even though the source is drained.
        scout.tick();
        assert!(wake_rx.try_recv().is_err());
    }

    /// End-to-end proof of the done-when: a scripted target really crosses 100 pts, is
    /// suspended by the OS, and the spore is signaled. Spawns a real process and `SIGSTOP`s
    /// it, so it's `#[ignore]`d out of CI — run locally with `cargo test -- --ignored`.
    #[test]
    #[ignore = "spawns and SIGSTOPs a real process; run locally with --ignored"]
    #[cfg(unix)]
    fn scripted_target_is_really_suspended() {
        use std::process::Command;

        let (source, child) = ScriptedSource::spawn_target().expect("spawn helper");
        let pid = child.id();
        let (wake_tx, wake_rx) = channel();
        let (threat_tx, _threat_rx) = channel();
        let mut scout = Scout::new(source, wake_tx, threat_tx);

        // One scripted action per tick: A (20) → B (70) → C (110, fires).
        scout.tick();
        scout.tick();
        scout.tick();

        let wake = wake_rx.try_recv().expect("expected a wake signal");
        assert_eq!(wake.pid, pid, "wake signal targets the real helper PID");

        // The OS should report the helper stopped ('T').
        let out = Command::new("ps")
            .args(["-o", "stat=", "-p", &pid.to_string()])
            .output()
            .expect("run ps");
        let stat = String::from_utf8_lossy(&out.stdout).trim().to_string();

        // Clean up no matter what the assertion finds: continue, then terminate.
        let _ = Command::new("kill")
            .args(["-CONT", &pid.to_string()])
            .status();
        let _ = Command::new("kill").arg("-9").arg(pid.to_string()).status();

        assert!(
            stat.starts_with('T'),
            "expected helper {pid} to be stopped (T); ps stat was {stat:?}"
        );
    }
}
