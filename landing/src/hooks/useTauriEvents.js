// useTauriEvents — subscribes to a Rust→UI event stream by name and returns
// { data, dropped }: the latest payload (null until the first event) and whether the
// stream has gone quiet. The single data-access point for the live views; components
// stay presentational.
//
// Real wiring: inside a Tauri webview, listens via @tauri-apps/api/event. Detected via
// "__TAURI_INTERNALS__" in window — always injected by Tauri 2, unlike "__TAURI__" which
// needs the (unset) withGlobalTauri config flag.
// Outside Tauri (plain `vite dev` in a browser) or before a phase's Rust side emits real
// events, falls back to a local mock generator per stream so the dashboard is demoable.
// ponytail: swap/remove MOCK_GENERATORS entries as each phase wires its real `emit`.
import { useEffect, useRef, useState } from "react";

const DROP_TIMEOUT_MS = 5000;
const MOCK_INTERVAL_MS = 1500;

const PROCESS_NAMES = ["chrome.exe", "svchost.exe", "vssadmin.exe", "explorer.exe", "node.exe", "sshd"];

function mockEcosystem(prev) {
  const nodes = prev ? prev.map((n) => ({ ...n })) : PROCESS_NAMES.map((name, i) => ({
    pid: 1000 + i,
    name,
    score: 0,
    status: "normal",
  }));
  const target = nodes[Math.floor(Math.random() * nodes.length)];
  target.score = Math.min(100, target.score + [20, 40, 50][Math.floor(Math.random() * 3)]);
  target.status = target.score >= 100 ? "suspended" : target.score >= 60 ? "watching" : "normal";
  return nodes;
}

function mockLedger(prev) {
  const events = prev ? [...prev] : [];
  const kinds = ["threat.detected", "gene.proposed", "allergy.checked", "poi.verified", "gene.committed"];
  events.unshift({
    id: `${Date.now()}`,
    kind: kinds[Math.floor(Math.random() * kinds.length)],
    threatId: `T-${Math.floor(Math.random() * 9000 + 1000)}`,
    at: new Date().toLocaleTimeString(),
  });
  return events.slice(0, 25);
}

function mockStrains(prev) {
  const strains = prev ? [...prev] : [{ id: "genesis", parentId: null, label: "genesis", stage: "root" }];
  const parent = strains[Math.floor(Math.random() * strains.length)];
  strains.push({
    id: `S-${strains.length}`,
    parentId: parent.id,
    label: `strain-${strains.length}`,
    stage: ["fuzzed", "regression", "committed"][Math.floor(Math.random() * 3)],
  });
  return strains;
}

const MOCK_GENERATORS = {
  ecosystem: mockEcosystem,
  ledger: mockLedger,
  strains: mockStrains,
};

export function useTauriEvents(stream) {
  const [data, setData] = useState(null);
  const [dropped, setDropped] = useState(false);
  const dropTimer = useRef(null);

  useEffect(() => {
    let cancelled = false;
    let unlisten = () => {};

    const armDropTimer = () => {
      clearTimeout(dropTimer.current);
      setDropped(false);
      dropTimer.current = setTimeout(() => setDropped(true), DROP_TIMEOUT_MS);
    };

    const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

    if (isTauri) {
      import("@tauri-apps/api/event")
        .then(({ listen }) =>
          listen(stream, (event) => {
            setData(event.payload);
            armDropTimer();
          })
        )
        .then((fn) => {
          if (cancelled) fn();
          else unlisten = fn;
        })
        .catch((err) => {
          // Fails loud instead of leaving the view stuck on "no data" forever with no
          // clue why — e.g. a missing capability grant rejects listen() by ACL.
          console.error(`[useTauriEvents] failed to subscribe to "${stream}":`, err);
        });
    } else {
      const generate = MOCK_GENERATORS[stream];
      if (generate) {
        const id = setInterval(() => {
          setData((prev) => generate(prev));
          armDropTimer();
        }, MOCK_INTERVAL_MS);
        unlisten = () => clearInterval(id);
      }
    }

    return () => {
      cancelled = true;
      clearTimeout(dropTimer.current);
      unlisten();
    };
  }, [stream]);

  return { data, dropped };
}
