//! Scout: ultra-light background daemon. Traces process behavior (syscall anomalies,
//! memory boundary violations, I/O bursts) and accumulates a per-process trajectory score.
//! Observes only — never terminates. Crossing 100 pts suspends the target and signals the
//! local Soldier spore. See ../../.claude/docs/architecture.md, Stage 1.

use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use sha2::{Digest, Sha256};

use tauri::AppHandle;

use super::{ThreatReport, WakeSignal};
use crate::core::{Action, AnomalyScore, BehavioralSchema, Pid, ThreatId, ANOMALY_THRESHOLD};
use crate::dashboard::Dashboard;

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
    /// Mirrors each observed PID's trajectory to `EcosystemGraph.jsx`. `None` in tests, which
    /// have no running Tauri app to emit into.
    dashboard: Option<Arc<Dashboard>>,
}

impl<S: BehaviorSource + 'static> Scout<S> {
    /// Build a Scout over `source`, emitting wake signals and threat reports on the channels.
    pub fn new(
        source: S,
        wake_tx: Sender<WakeSignal>,
        threat_tx: Sender<ThreatReport>,
        dashboard: Option<Arc<Dashboard>>,
    ) -> Self {
        Self {
            source,
            scores: HashMap::new(),
            wake_tx,
            threat_tx,
            dashboard,
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

            if let Some(dashboard) = &self.dashboard {
                dashboard.report_score(pid, &process_name(pid), trajectory.score.0);
            }

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

/// A display name for the dashboard's ecosystem view — purely cosmetic, never fed back into
/// scoring/correlation logic. Unix: the process's `comm` via `ps`; falls back to a synthetic
/// name if that fails or on non-Unix.
#[cfg(unix)]
fn process_name(pid: Pid) -> String {
    std::process::Command::new("ps")
        .args(["-o", "comm=", "-p", &pid.to_string()])
        .output()
        .ok()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| format!("pid-{pid}"))
}

#[cfg(not(unix))]
fn process_name(pid: Pid) -> String {
    format!("pid-{pid}")
}

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
/// program (Phase 12 — closes the live-path gaps Phase 11 left: real events, live
/// evolution, happy-path-before-kill-switch, and mobilization). Runs the scripted
/// trajectory twice against the same `Threat_ID`:
///
/// 1. Wave 1: no cure is published yet, so the Soldier evolves one live (fuzz → allergy
///    check → `commit_gene`) and dispenses it — `Neutralized`, shown before any suppression.
/// 2. Once wave 1 completes, `suppress_gene` flips the kill-switch, and wave 2 (a fresh
///    scripted target, same trajectory) shows the halt — `Suppressed`.
///
/// Both waves' `submit_threat` calls hit the same `Threat_ID`, so `Confidence_Score` also
/// crosses `MOBILIZATION_THRESHOLD` on wave 2 — the network-wide mobilization event fires
/// alongside the kill-switch demonstration.
pub fn spawn_demo(app: AppHandle) -> JoinHandle<()> {
    use anchor_client::Cluster;
    use solana_keypair::read_keypair_file;

    use super::soldier::{Pharmacy, PharmacyOutcome};
    use crate::ledger::client::{SolanaConjugationLink, SolanaGeneCommitter};
    use crate::ledger::registry::{GenomeRegistry, ThreatRegistry};
    use crate::ledger::state::SolanaLightClient;

    let dashboard = Arc::new(Dashboard::new(app));

    let (wake_tx, wake_rx) = channel::<WakeSignal>();
    let (threat_tx, threat_rx) = channel::<ThreatReport>();
    let (outcome_tx, outcome_rx) = channel::<PharmacyOutcome>();

    let home = std::env::var("HOME").expect("HOME not set");
    let deployer = Arc::new(
        read_keypair_file(format!("{home}/.config/solana/id.json"))
            .expect("read deployer keypair"),
    );
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let validators: Vec<_> = (1..=5)
        .map(|i| {
            let path = format!("{manifest_dir}/../keys/lymph-nodes/validator-{i}.json");
            read_keypair_file(&path).expect("read Lymph Node validator keypair")
        })
        .collect();

    let conjugation_for_threats = SolanaConjugationLink::new(Cluster::Devnet, deployer.clone())
        .expect("connect SolanaConjugationLink to devnet");
    let threat_registry = ThreatRegistry::new(
        SolanaLightClient::new(Cluster::Devnet, deployer.clone())
            .expect("connect SolanaLightClient to devnet"),
    );

    let genome_link = SolanaConjugationLink::new(Cluster::Devnet, deployer.clone())
        .expect("connect SolanaConjugationLink to devnet");
    let committer = Arc::new(SolanaGeneCommitter::new(genome_link, validators));
    let light_client = SolanaLightClient::new(Cluster::Devnet, deployer.clone())
        .expect("connect SolanaLightClient to devnet");
    let pharmacy = Pharmacy {
        genome: Box::new(GenomeRegistry::new(light_client)),
        committer: Box::new(committer.clone()),
    };

    // Real Soldier consumes the wake channel against the pharmacy: Phase-3 spore lifecycle +
    // Phase-6/11/12 pharmacy resolution (evolving a cure live when none exists yet) +
    // Phase-9 kill-switch. `outcome_tx` is this function's own sequencing signal — see (2).
    let spore_path = std::env::temp_dir().join("bio-digital-defense.spore");
    super::soldier::run(
        wake_rx,
        spore_path,
        pharmacy,
        Some(dashboard.clone()),
        Some(outcome_tx),
    );

    // Threat Registry (Ledger 2): each report is a real `submit_threat` transaction. Reads
    // Confidence_Score back and logs the network-wide mobilization event once it crosses
    // MOBILIZATION_THRESHOLD — both demo waves report the same Threat_ID, so wave 2's
    // submission is what actually crosses it. This doesn't gate the Soldier's wake, which
    // stays Stage 1's own immediate, local mechanism (source-of-truth.md keeps the two
    // separate).
    let threat_dashboard = dashboard.clone();
    thread::spawn(move || {
        for report in threat_rx {
            let schema_hash = behavioral_schema_hash(&report.schema);
            match conjugation_for_threats.submit_threat(report.threat_id, schema_hash) {
                Ok(sig) => {
                    println!(
                        "[ledger2] submit_threat {sig} ({} actions)",
                        report.schema.actions.len()
                    );
                    threat_dashboard.log_event("threat.detected", report.threat_id.to_hex());
                    if let Ok(Some(entry)) = threat_registry.get(&report.threat_id) {
                        if entry.confidence_score >= bio_digital_defense::MOBILIZATION_THRESHOLD {
                            println!(
                                "[ledger2] network-wide mobilization triggered for {:?} \
                                 (confidence {})",
                                report.threat_id, entry.confidence_score
                            );
                            threat_dashboard
                                .log_event("threat.mobilized", report.threat_id.to_hex());
                        }
                    }
                }
                Err(e) => eprintln!("[ledger2] submit_threat failed: {e}"),
            }
        }
    });

    // Wave 1.
    let (source, child) =
        ScriptedSource::spawn_target().expect("failed to spawn scripted target process");
    // Drop the helper handle: the Scout suspends it on threshold cross and the Soldier
    // releases and terminates it during apoptosis; any residue is reaped on app exit.
    drop(child);
    Scout::new(source, wake_tx.clone(), threat_tx.clone(), Some(dashboard.clone())).spawn();

    match outcome_rx.recv() {
        Ok(outcome) => println!("[demo] wave 1 outcome: {outcome:?}"),
        Err(_) => eprintln!("[demo] soldier channel closed before wave 1 completed"),
    }

    // Kill-switch, then wave 2 against a fresh target running the identical trajectory
    // (same Threat_ID) to show the halt.
    let threat_id = scripted_threat_id();
    match committer.suppress(threat_id) {
        Ok(sig) => {
            println!("[ledger3] suppress_gene {sig}");
            dashboard.log_event("gene.suppressed", threat_id.to_hex());
        }
        Err(e) => eprintln!("[ledger3] suppress_gene failed: {e}"),
    }

    let (source2, child2) = ScriptedSource::spawn_target()
        .expect("failed to spawn scripted target process (wave 2)");
    drop(child2);
    Scout::new(source2, wake_tx, threat_tx, Some(dashboard)).spawn()
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
        let mut scout = Scout::new(VecSource(observations), wake_tx, threat_tx, None);
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
        let mut scout = Scout::new(source, wake_tx, threat_tx, None);

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
