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
        if let Some(dashboard) = &self.dashboard {
            dashboard.note_scout_spawned();
        }
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
                let _ = self.wake_tx.send(WakeSignal {
                    threat_id: id,
                    pid,
                    schema: schema.clone(),
                });
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

/// Which real `fake_viruses/` specimen a watched PID is, and therefore which detector
/// `poll()` runs against it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecimenKind {
    /// `fake_viruses/virus` — bulk concurrent file reader.
    Virus,
    /// `fake_viruses/disease` — periodic TCP beacon.
    Disease,
    /// `fake_viruses/bacteria` — process replicator (bounded fork bursts).
    Bacteria,
}

/// Minimum open file descriptors (via `lsof -p`, header line excluded) to count `virus`'s
/// read burst as `Action::HighEntropyFileLoop`. Calibrated against a real run, not guessed:
/// `virus`'s idle baseline (its own threads/stdio/loaded libraries) measured ~6; a real
/// concurrent-8-file burst (after the specimens were widened to hold fds open long enough
/// to observe — see `kHoldOpenDuration` in `fake_viruses/virus.cpp`) measured ~13. This sits
/// between the two.
const VIRUS_FD_THRESHOLD: usize = 9;

/// Real behavior source: observes actual OS state of real target processes via `lsof`
/// shell-outs (no kernel-level tracer — eBPF/ETW/EndpointSecurity are each blocked for this
/// project, not just costly: eBPF is Linux-only and needs root; ETW is Windows-only,
/// already deprioritized; macOS's EndpointSecurity needs an Apple entitlement granted only
/// to vetted security vendors). Covers `Action::HighEntropyFileLoop` (`virus`),
/// `Action::NetEnumWithVssTamper` (`disease`), and `Action::HiddenChildFromTemp`
/// (`bacteria` — its fork-churn is really observed via `pgrep -P`, but the children are
/// plain forks, not exec'd from a Temp directory; the closest Stage-1 action stands in
/// until a real temp-dir-dropper specimen exists).
/// Each watched PID is independent: `Scout::tick()` sums `weight(action)` for every
/// observation with no dedup by type, so a specimen doing its one suspicious thing
/// continuously climbs toward the threshold on its own, no combining needed.
pub struct RealBehaviorSource {
    targets: Vec<(Pid, SpecimenKind)>,
}

impl RealBehaviorSource {
    pub fn new(targets: Vec<(Pid, SpecimenKind)>) -> Self {
        Self { targets }
    }
}

impl BehaviorSource for RealBehaviorSource {
    fn poll(&mut self) -> Vec<Observation> {
        self.targets
            .iter()
            .filter_map(|&(pid, kind)| {
                let detected = match kind {
                    SpecimenKind::Virus => open_fd_count(pid) > VIRUS_FD_THRESHOLD,
                    SpecimenKind::Disease => has_outbound_connection(pid),
                    SpecimenKind::Bacteria => has_live_child(pid),
                };
                detected.then_some(Observation {
                    pid,
                    action: match kind {
                        SpecimenKind::Virus => Action::HighEntropyFileLoop,
                        SpecimenKind::Disease => Action::NetEnumWithVssTamper,
                        SpecimenKind::Bacteria => Action::HiddenChildFromTemp,
                    },
                })
            })
            .collect()
    }
}

/// Real open-file-descriptor count for `pid`, via `lsof -p` (subtracting `lsof`'s own
/// header line). `0` if `lsof` fails or the process has already exited — never an error, a
/// dead PID just isn't suspicious.
#[cfg(unix)]
fn open_fd_count(pid: Pid) -> usize {
    std::process::Command::new("lsof")
        .args(["-p", &pid.to_string()])
        .output()
        .ok()
        .map(|out| {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .count()
                .saturating_sub(1) // header line
        })
        .unwrap_or(0)
}

#[cfg(not(unix))]
fn open_fd_count(_pid: Pid) -> usize {
    0
}

/// Whether `pid` currently holds any network connection, via `lsof -i -a -p`.
#[cfg(unix)]
fn has_outbound_connection(pid: Pid) -> bool {
    std::process::Command::new("lsof")
        .args(["-i", "-a", "-p", &pid.to_string()])
        .output()
        .ok()
        .map(|out| !out.stdout.is_empty())
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn has_outbound_connection(_pid: Pid) -> bool {
    false
}

/// Whether `pid` currently has any live child process, via `pgrep -P`. `bacteria`'s
/// children linger ~700ms of each 1s tick (`kChildLinger` in `fake_viruses/bacteria.cpp`),
/// so a 500ms-cadence poll catches a burst most ticks. `false` if `pgrep` fails or the
/// process is gone — a dead PID just isn't suspicious.
#[cfg(unix)]
fn has_live_child(pid: Pid) -> bool {
    std::process::Command::new("pgrep")
        .args(["-P", &pid.to_string()])
        .output()
        .ok()
        .map(|out| !out.stdout.is_empty())
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn has_live_child(_pid: Pid) -> bool {
    false
}

/// Spawns a real `fake_viruses/` specimen binary, failing loudly (not silently) if it's
/// missing — run `make` in `fake_viruses/` first (see
/// ../../.claude/docs/build-run-test.md).
fn spawn_specimen(name: &str, args: &[&str]) -> std::process::Child {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = format!("{manifest_dir}/../fake_viruses/{name}");
    std::process::Command::new(&path)
        .args(args)
        .spawn()
        .unwrap_or_else(|e| {
            panic!(
                "failed to spawn {path}: {e} — run `make` in fake_viruses/ first \
                 (see .claude/docs/build-run-test.md)"
            )
        })
}

/// The `Threat_ID` a real `disease` process deterministically produces: its one behavior
/// (`Action::NetEnumWithVssTamper`, 50 pts) fires the moment the trajectory hits exactly 2
/// detections (50 + 50 = 100 = `ANOMALY_THRESHOLD`) — `Scout::tick()` checks the threshold
/// immediately after each observation, so this is deterministic regardless of exactly how
/// many polls it takes to catch those 2 real beacons.
fn disease_threat_id() -> ThreatId {
    threat_id(&BehavioralSchema {
        actions: vec![Action::NetEnumWithVssTamper, Action::NetEnumWithVssTamper],
    })
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

/// Live demo entry point: wires two real, independent `fake_viruses/` specimens against the
/// **real** Solana devnet program and a **real** behavioral detector (Phase 13, item 1 —
/// closes the last major gap Phase 12 left: detection itself was still a hardcoded script).
///
/// 1. Wave 1: `virus`, `disease`, and `bacteria` are spawned as real processes and watched
///    concurrently by one Scout. Each is detected independently (no cure published yet for
///    any), so the Soldier evolves one live (fuzz → allergy check → `commit_gene`) and
///    dispenses it — `Neutralized` outcomes for `virus`/`disease`, while `bacteria`'s
///    hardened clone yields an Allele09 candidate the Lymph Node allergy-flags and drops
///    (`NoCureAvailable`): the live allergy example.
/// 2. Once both resolve, `suppress_gene` flips the kill-switch on `disease`'s cure
///    specifically (its schema is deterministic — see [`disease_threat_id`]), and wave 2 (a
///    fresh `disease` process, same real behavior, same `Threat_ID`) shows the halt.
///
/// Both `disease` submissions hit the same `Threat_ID`, so `Confidence_Score` also crosses
/// `MOBILIZATION_THRESHOLD` on wave 2 — the network-wide mobilization event fires alongside
/// the kill-switch demonstration, exactly as it did before this real-detection pass.
pub fn spawn_demo(app: AppHandle) -> JoinHandle<()> {
    use anchor_client::Cluster;
    use solana_keypair::read_keypair_file;
    use solana_rpc_client::rpc_client::RpcClient;

    use super::soldier::{Pharmacy, PharmacyOutcome};
    use crate::ledger::client::{SolanaConjugationLink, SolanaGeneCommitter};
    use crate::ledger::registry::{GenomeRegistry, ThreatRegistry};
    use crate::ledger::state::SolanaLightClient;

    let dashboard = Arc::new(Dashboard::new(app));

    // Own light RPC handle just for the `stats` stream's `slot` field — separate from the
    // `SolanaLightClient`/`SolanaConjugationLink` handles below, which need a signing keypair
    // this plain slot lookup doesn't.
    let slot_rpc = RpcClient::new(Cluster::Devnet.url().to_string());
    dashboard.spawn_reemit(move || slot_rpc.get_slot().ok());

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

    // Wave 1: several real, independent specimens, watched concurrently by one Scout.
    // `virus` needs a directory with enough files for its read burst to actually cross the
    // fd threshold — fake_viruses/ itself qualifies (Makefile + 3 .cpp + 3 binaries = 7).
    // `WAVE1_COPIES` concurrent copies each of `virus`/`disease`: same two real detectors,
    // more simultaneous catches to watch land in the dashboard. Repeat copies share their
    // specimen type's deterministic Threat_ID (the schema hash doesn't depend on PID), so
    // copies past the first one demonstrate Ledger 2's confidence-matching/mobilization path
    // rather than minting new threats. Capped at 3 (6 targets): `RealBehaviorSource::poll()`
    // shells out to `lsof` once per target, sequentially, and `lsof` alone costs ~40-50ms —
    // pushing much past this risks a poll() taking long enough, relative to `TICK_INTERVAL`,
    // to matter (both specimens loop forever though, so an occasional missed tick isn't fatal,
    // just a slower catch).
    const WAVE1_COPIES: usize = 3;
    let fake_viruses_dir = format!("{manifest_dir}/../fake_viruses");
    let mut targets = Vec::with_capacity(WAVE1_COPIES * 2 + 1);
    for _ in 0..WAVE1_COPIES {
        let virus_child = spawn_specimen("virus", &[fake_viruses_dir.as_str()]);
        targets.push((virus_child.id(), SpecimenKind::Virus));
        drop(virus_child); // Scout suspends on threshold cross; Soldier reaps on apoptosis.
        let disease_child = spawn_specimen("disease", &[]);
        targets.push((disease_child.id(), SpecimenKind::Disease));
        drop(disease_child);
    }
    // One `bacteria` alongside the copies: the allergy example. Its hardened clone only
    // dies to Allele09, which the Lymph Node allergy-flags (LegacyBackupAgent) — so its
    // outcome is NoCureAvailable and a live `gene.allergy_flagged` event, not a cure.
    // One copy is enough: repeats share the same deterministic Threat_ID.
    let bacteria_child = spawn_specimen("bacteria", &[]);
    targets.push((bacteria_child.id(), SpecimenKind::Bacteria));
    drop(bacteria_child);
    let source = RealBehaviorSource::new(targets);
    Scout::new(source, wake_tx.clone(), threat_tx.clone(), Some(dashboard.clone())).spawn();

    // One outcome per specimen this wave.
    for _ in 0..(WAVE1_COPIES * 2 + 1) {
        match outcome_rx.recv() {
            Ok(outcome) => println!("[demo] wave 1 outcome: {outcome:?}"),
            Err(_) => {
                eprintln!("[demo] soldier channel closed before wave 1 completed");
                break;
            }
        }
    }

    // Kill-switch, then wave 2: suppress disease's cure and re-trigger the identical real
    // behavior (same deterministic schema, same Threat_ID) to show the halt.
    let threat_id = disease_threat_id();
    match committer.suppress(threat_id) {
        Ok(sig) => {
            println!("[ledger3] suppress_gene {sig}");
            dashboard.log_event("gene.suppressed", threat_id.to_hex());
        }
        Err(e) => eprintln!("[ledger3] suppress_gene failed: {e}"),
    }

    let disease_child2 = spawn_specimen("disease", &[]);
    let source2 = RealBehaviorSource::new(vec![(disease_child2.id(), SpecimenKind::Disease)]);
    drop(disease_child2);
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

    /// End-to-end proof that real detection works: a real `fake_viruses/disease` process
    /// really beacons, `RealBehaviorSource` really observes it via `lsof`, and the
    /// trajectory really crosses 100 pts and wakes. Needs `make` run in `fake_viruses/`
    /// first and real wall-clock time for the specimen to actually beacon twice, so it's
    /// `#[ignore]`d out of CI — run locally with `cargo test -- --ignored`.
    #[test]
    #[ignore = "spawns a real fake_viruses/disease process; run locally with --ignored, \
                after `make` in fake_viruses/"]
    #[cfg(unix)]
    fn disease_specimen_is_really_detected_and_woken() {
        let mut child = spawn_specimen("disease", &[]);
        let pid = child.id();
        let (wake_tx, wake_rx) = channel();
        let (threat_tx, _threat_rx) = channel();
        let mut scout = Scout::new(
            RealBehaviorSource::new(vec![(pid, SpecimenKind::Disease)]),
            wake_tx,
            threat_tx,
            None,
        );

        // disease beacons roughly once/sec; give it up to 10 real ticks (~5s) to be
        // observed twice (50 + 50 = 100, crossing ANOMALY_THRESHOLD).
        let mut wake_signal = None;
        for _ in 0..10 {
            scout.tick();
            if let Ok(wake) = wake_rx.try_recv() {
                wake_signal = Some(wake);
                break;
            }
            thread::sleep(TICK_INTERVAL);
        }

        // Clean up regardless of the assertion outcome.
        let _ = child.kill();
        let _ = child.wait();

        let wake = wake_signal
            .expect("expected the real disease specimen to be detected and woken within 10 ticks");
        assert_eq!(wake.pid, pid, "wake signal targets the real specimen PID");
    }

    /// Same shape as the `disease` test, for `virus` — verifies `VIRUS_FD_THRESHOLD`'s
    /// recalibration (baseline ~6, burst ~13) actually distinguishes idle from bursting on
    /// a real run, not just the ad-hoc `lsof` probe used to pick the number.
    #[test]
    #[ignore = "spawns a real fake_viruses/virus process; run locally with --ignored, \
                after `make` in fake_viruses/"]
    #[cfg(unix)]
    fn virus_specimen_is_really_detected_and_woken() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let fake_viruses_dir = format!("{manifest_dir}/../fake_viruses");
        let mut child = spawn_specimen("virus", &[fake_viruses_dir.as_str()]);
        let pid = child.id();
        let (wake_tx, wake_rx) = channel();
        let (threat_tx, _threat_rx) = channel();
        let mut scout = Scout::new(
            RealBehaviorSource::new(vec![(pid, SpecimenKind::Virus)]),
            wake_tx,
            threat_tx,
            None,
        );

        // virus sweeps roughly once/sec; give it up to 10 real ticks (~5s) to be observed
        // 3 times (40 + 40 + 40 = 120, crossing ANOMALY_THRESHOLD at the 3rd).
        let mut wake_signal = None;
        for _ in 0..10 {
            scout.tick();
            if let Ok(wake) = wake_rx.try_recv() {
                wake_signal = Some(wake);
                break;
            }
            thread::sleep(TICK_INTERVAL);
        }

        let _ = child.kill();
        let _ = child.wait();

        let wake = wake_signal
            .expect("expected the real virus specimen to be detected and woken within 10 ticks");
        assert_eq!(wake.pid, pid, "wake signal targets the real specimen PID");
    }
}
