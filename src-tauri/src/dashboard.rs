//! Bridges core agent/ledger events to the Tauri event bus the dashboard's `useTauriEvents`
//! hook listens on (`ecosystem`/`ledger`/`strains` — see
//! `../../landing/src/hooks/useTauriEvents.js`). The core stays the source of truth; this
//! module only mirrors state changes out to the UI, never the other way around.

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::core::Pid;

/// Matches `LedgerTerminal.jsx`'s cap on the mock generator's event list.
const LEDGER_CAP: usize = 25;

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
        };
        dashboard.emit_strains();
        dashboard
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
