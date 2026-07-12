//! Bridges core agent/ledger events to the Tauri event bus the dashboard's `useTauriEvents`
//! hook listens on (`ecosystem`/`ledger`/`strains` — see
//! `../../landing/src/hooks/useTauriEvents.js`). The core stays the source of truth; this
//! module only mirrors state changes out to the UI, never the other way around.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use sysinfo::{Disks, System};
use tauri::{AppHandle, Emitter};

use crate::core::Pid;

/// Matches `LedgerTerminal.jsx`'s cap on the mock generator's event list.
const LEDGER_CAP: usize = 25;

/// How often [`Dashboard::spawn_reemit`] re-broadcasts state and stats.
const REEMIT_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Serialize, Clone)]
struct EcosystemNode {
    pid: Pid,
    name: String,
    score: u32,
    status: &'static str,
}

#[derive(Serialize, Clone)]
struct LedgerEvent {
    id: String,
    kind: &'static str,
    #[serde(rename = "threatId")]
    threat_id: String,
    at: String,
}

#[derive(Serialize, Clone)]
struct StrainNode {
    id: String,
    #[serde(rename = "parentId")]
    parent_id: Option<String>,
    label: String,
    stage: &'static str,
}

/// Real host/network metrics for the `stats` stream (`Dashboard.jsx`). Emitted alongside
/// each re-emit tick — see [`Dashboard::spawn_reemit`].
#[derive(Serialize, Clone)]
struct Stats {
    #[serde(rename = "ramMb")]
    ram_mb: u64,
    #[serde(rename = "cpuPct")]
    cpu_pct: f32,
    #[serde(rename = "diskUsedGb")]
    disk_used_gb: f64,
    #[serde(rename = "diskTotalGb")]
    disk_total_gb: f64,
    #[serde(rename = "uptimeSecs")]
    uptime_secs: u64,
    scouts: usize,
    monitored: usize,
    slot: Option<u64>,
}

#[derive(Default)]
struct State {
    ecosystem: HashMap<Pid, EcosystemNode>,
    ledger: VecDeque<LedgerEvent>,
    strains: Vec<StrainNode>,
}

/// One shared handle, threaded through the Scout/Soldier/ledger paths (`agents::scout`,
/// `agents::soldier`) as `Option<Arc<Dashboard>>` — `None` in tests, since they have no
/// running Tauri app to emit into.
pub struct Dashboard {
    app: AppHandle,
    state: Mutex<State>,
    /// Bumped once per [`crate::agents::scout::Scout::spawn`] call; mirrored in the `stats`
    /// stream's `scouts` field. A plain counter, not part of `State`, since it only ever
    /// grows and needs no snapshot consistency with the other fields.
    scouts: AtomicUsize,
}

impl Dashboard {
    /// Builds the dashboard bridge and emits the `genesis` root strain once, so
    /// `StrainTree.jsx`'s single-root assumption always holds even before any gene evolves.
    pub fn new(app: AppHandle) -> Self {
        let dashboard = Self {
            app,
            state: Mutex::new(State {
                strains: vec![StrainNode {
                    id: "genesis".into(),
                    parent_id: None,
                    label: "genesis".into(),
                    stage: "root",
                }],
                ..State::default()
            }),
            scouts: AtomicUsize::new(0),
        };
        dashboard.emit_strains();
        dashboard
    }

    /// Records that one more Scout daemon started watching processes.
    pub fn note_scout_spawned(&self) {
        self.scouts.fetch_add(1, Ordering::Relaxed);
    }

    /// Records/updates one PID's Stage-1 trajectory for `EcosystemGraph.jsx`.
    pub fn report_score(&self, pid: Pid, name: &str, score: u32) {
        let status = if score >= 100 {
            "suspended"
        } else if score >= 60 {
            "watching"
        } else {
            "normal"
        };
        let mut state = self.state.lock().unwrap();
        state.ecosystem.insert(
            pid,
            EcosystemNode {
                pid,
                name: name.to_string(),
                score,
                status,
            },
        );
        let nodes: Vec<_> = state.ecosystem.values().cloned().collect();
        let _ = self.app.emit("ecosystem", nodes);
    }

    /// Appends one Proof-of-Immunity pipeline event for `LedgerTerminal.jsx`.
    pub fn log_event(&self, kind: &'static str, threat_id_hex: String) {
        let mut state = self.state.lock().unwrap();
        state.ledger.push_front(LedgerEvent {
            id: format!("{}-{kind}", now_millis()),
            kind,
            threat_id: threat_id_hex,
            at: now_hms(),
        });
        state.ledger.truncate(LEDGER_CAP);
        let events: Vec<_> = state.ledger.iter().cloned().collect();
        let _ = self.app.emit("ledger", events);
    }

    /// Appends one node to the evolutionary lineage for `StrainTree.jsx`.
    pub fn record_strain(
        &self,
        id: String,
        parent_id: Option<String>,
        label: String,
        stage: &'static str,
    ) {
        let mut state = self.state.lock().unwrap();
        state.strains.push(StrainNode {
            id,
            parent_id,
            label,
            stage,
        });
        self.emit_strains_locked(&state);
    }

    fn emit_strains(&self) {
        let state = self.state.lock().unwrap();
        self.emit_strains_locked(&state);
    }

    fn emit_strains_locked(&self, state: &State) {
        let _ = self.app.emit("strains", state.strains.clone());
    }

    /// Re-broadcasts the current `ecosystem`/`ledger`/`strains` snapshot plus a `stats`
    /// event every [`REEMIT_INTERVAL`]. Fixes two gaps edge-triggered `emit` alone leaves:
    /// a view that mounts after the last state change (late subscriber) and a demo that has
    /// finished emitting (dead air read as a dropped stream). `get_slot` is `spawn_demo`'s
    /// devnet RPC slot lookup, called fresh each tick; `None` on any RPC failure never blocks
    /// the loop.
    pub fn spawn_reemit(
        self: &Arc<Self>,
        get_slot: impl Fn() -> Option<u64> + Send + 'static,
    ) -> JoinHandle<()> {
        let dashboard = self.clone();
        thread::spawn(move || {
            let mut sys = System::new_all();
            loop {
                thread::sleep(REEMIT_INTERVAL);

                let (ecosystem, ledger, strains, monitored) = {
                    let state = dashboard.state.lock().unwrap();
                    (
                        state.ecosystem.values().cloned().collect::<Vec<_>>(),
                        state.ledger.iter().cloned().collect::<Vec<_>>(),
                        state.strains.clone(),
                        state.ecosystem.len(),
                    )
                };
                let _ = dashboard.app.emit("ecosystem", ecosystem);
                let _ = dashboard.app.emit("ledger", ledger);
                let _ = dashboard.app.emit("strains", strains);

                sys.refresh_cpu_usage();
                sys.refresh_memory();
                let disks = Disks::new_with_refreshed_list();
                let (disk_used, disk_total) = disks.list().iter().fold(
                    (0u64, 0u64),
                    |(used, total), disk| {
                        (
                            used + disk.total_space().saturating_sub(disk.available_space()),
                            total + disk.total_space(),
                        )
                    },
                );

                let _ = dashboard.app.emit(
                    "stats",
                    Stats {
                        ram_mb: sys.used_memory() / (1024 * 1024),
                        cpu_pct: sys.global_cpu_usage(),
                        disk_used_gb: disk_used as f64 / 1e9,
                        disk_total_gb: disk_total as f64 / 1e9,
                        uptime_secs: System::uptime(),
                        scouts: dashboard.scouts.load(Ordering::Relaxed),
                        monitored,
                        slot: get_slot(),
                    },
                );
            }
        })
    }
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

/// `HH:MM:SS` (UTC) — enough for a live demo log; avoids pulling in a datetime crate for
/// display formatting alone.
fn now_hms() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let (h, m, s) = ((secs / 3600) % 24, (secs / 60) % 60, secs % 60);
    format!("{h:02}:{m:02}:{s:02}")
}
