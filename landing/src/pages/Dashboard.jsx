// Dashboard — signed-in home. System stat cards, agent/network status, recent activity
// and the immune ledger (both live from the Rust `ledger` stream via useTauriEvents),
// machine stats, and a running-programs table live from the `ecosystem` stream.
// From Dashboard.dc.html. RAM/CPU/GPU/scouts/block/network stats have no backing Tauri
// stream yet, so they stay as clamped random-walk mocks like the design prototype.
import { useEffect, useState } from "react";
import SignedInNav from "../components/SignedInNav.jsx";
import { useTauriEvents } from "../hooks/useTauriEvents.js";
import { colors, fontDisplay } from "../tokens.js";

const card = { background: colors.panel, border: `1px solid ${colors.borderPanel}`, borderRadius: 14 };
const cardTitle = { fontFamily: fontDisplay, fontSize: 16, fontWeight: 600, letterSpacing: 0.5 };

const STATUS_STYLE = {
  normal: { label: "Nominal", color: colors.green },
  watching: { label: "Elevated — watching", color: colors.yellow },
  suspended: { label: "Suspended", color: colors.red },
};

const LEDGER_EVENT_COPY = {
  "threat.detected": { color: colors.green, text: (id) => `Scout reported a new signature — ${id} flagged, confidence low. Monitoring.` },
  "gene.proposed": { color: colors.lavender, text: (id) => `Soldier proposed a kill allele for ${id}. Awaiting validation.` },
  "allergy.checked": { color: colors.yellow, text: (id) => `Allergy check ran for ${id} — suppressed per host policy.` },
  "poi.verified": { color: colors.textFaint, text: (id) => `Proof of Immunity validated for ${id} — anchored to State Ledger.` },
  "gene.committed": { color: colors.green, text: (id) => `Kill module for ${id} committed to Genome Registry — immunity published network-wide.` },
};

const LEDGER_BLOCK_STATUS = {
  "gene.committed": { label: "Active & verified", color: colors.greenLight },
  "poi.verified": { label: "Active & verified", color: colors.greenLight },
  "allergy.checked": { label: "Suppressed (allergy)", color: colors.yellow },
};

export default function Dashboard({ auth, navigate }) {
  const [on, setOn] = useState(true);
  const [sys, setSys] = useState({
    ram: 8.2, cpu: 22, gpu: 10, cpuTemp: 105, gpuTemp: 98,
    scouts: 46, block: 8412907, activeNodes: 41902, curesPerMin: 1208,
  });

  useEffect(() => {
    const id = setInterval(() => {
      if (!on) return;
      setSys((s) => ({
        ram: Math.max(6, Math.min(11, s.ram + (Math.random() - 0.5) * 0.4)),
        cpu: Math.max(4, Math.min(38, s.cpu + Math.round((Math.random() - 0.5) * 5))),
        gpu: Math.max(2, Math.min(24, s.gpu + Math.round((Math.random() - 0.5) * 3))),
        cpuTemp: Math.max(95, Math.min(115, s.cpuTemp + Math.round((Math.random() - 0.5) * 2))),
        gpuTemp: Math.max(88, Math.min(108, s.gpuTemp + Math.round((Math.random() - 0.5) * 2))),
        scouts: s.scouts,
        block: s.block + (Math.random() < 0.4 ? 1 : 0),
        activeNodes: s.activeNodes + Math.floor((Math.random() - 0.45) * 8),
        curesPerMin: Math.max(900, Math.min(1500, s.curesPerMin + Math.floor((Math.random() - 0.5) * 30))),
      }));
    }, 1500);
    return () => clearInterval(id);
  }, [on]);

  const { data: ecosystem } = useTauriEvents("ecosystem");
  const { data: ledger } = useTauriEvents("ledger");

  const processes = (ecosystem ?? []).map((node) => {
    const status = STATUS_STYLE[node.status] ?? { label: node.status, color: colors.textFaint };
    return {
      key: node.pid,
      name: node.name,
      scout: `Scout_${String(node.pid).slice(-2).padStart(2, "0")}`,
      cpu: node.score,
      mem: "—",
      status: status.label,
      color: status.color,
    };
  });

  const events = (ledger ?? []).slice(0, 5).map((ev) => {
    const copy = LEDGER_EVENT_COPY[ev.kind] ?? { color: colors.textFaint, text: (id) => `${ev.kind} — ${id}` };
    return { key: ev.id, time: ev.at, color: copy.color, text: copy.text(ev.threatId) };
  });

  const blocks = (ledger ?? []).slice(0, 2).map((ev, i) => {
    const status = LEDGER_BLOCK_STATUS[ev.kind] ?? { label: "Pending verification", color: colors.textFaint };
    return {
      key: ev.id,
      num: (sys.block - i).toLocaleString(),
      ago: ev.at,
      trajectory: ev.kind,
      vector: `ipfs://${ev.threatId.toLowerCase()}`,
      status: status.label,
      statusColor: status.color,
    };
  });

  const scouts = on ? sys.scouts : 0;
  const netDown = (2.4 + (sys.cpu - 22) * 0.05).toFixed(1);
  const netUp = (0.6 + (sys.gpu - 10) * 0.02).toFixed(1);

  return (
    <div className="pan-page" style={{ minHeight: "100vh", background: colors.bg, color: colors.text, display: "flex", flexDirection: "column" }}>
      <SignedInNav active="home" userName={auth.userName} onNavigate={navigate} power={{ on, onToggle: () => setOn((v) => !v) }} />

      {/* STATUS BANNER */}
      <div
        style={{
          display: "flex", alignItems: "center", justifyContent: "center", gap: 10, padding: 14,
          background: on ? colors.greenTint : colors.redTint, borderBottom: `1px solid ${colors.border}`,
          fontSize: 13, letterSpacing: 1.5, textTransform: "uppercase", fontWeight: 600,
          color: on ? colors.greenLight : colors.red,
        }}
      >
        <span style={{ width: 8, height: 8, borderRadius: "50%", background: on ? colors.greenLight : colors.red, display: "inline-block" }} />
        {on ? "Protection active — immune system online" : "Protection paused — scouts standing down"}
      </div>

      {/* MAIN */}
      <main style={{ maxWidth: 1180, width: "100%", margin: "0 auto", padding: 48, boxSizing: "border-box", display: "flex", flexDirection: "column", gap: 40 }}>
        {/* SYSTEM STATS */}
        <section style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: 24 }}>
          {[
            [`${sys.ram.toFixed(1)} MB`, "RAM in use", "Scout daemon footprint"],
            [`${sys.cpu}%`, "CPU in use", `${sys.cpuTemp}°F`],
            [`${sys.gpu}%`, "GPU in use", `${sys.gpuTemp}°F`],
          ].map(([value, label, sub]) => (
            <div key={label} style={{ ...card, padding: 30, display: "flex", flexDirection: "column", alignItems: "center", gap: 8, textAlign: "center" }}>
              <div style={{ fontFamily: fontDisplay, fontSize: 44, fontWeight: 700, fontVariantNumeric: "tabular-nums" }}>{value}</div>
              <div style={{ fontSize: 14, color: colors.lavender, letterSpacing: 1, textTransform: "uppercase" }}>{label}</div>
              <div style={{ fontSize: 12, color: colors.textFaint }}>{sub}</div>
            </div>
          ))}
        </section>

        {/* AGENT STATUS + ACTIVITY */}
        <section style={{ display: "grid", gridTemplateColumns: "1fr 1.4fr", gap: 24, alignItems: "start" }}>
          <div style={{ display: "flex", flexDirection: "column", gap: 24 }}>
            <div style={{ ...card, padding: 28, display: "flex", flexDirection: "column", gap: 18 }}>
              <div style={cardTitle}>Agent status</div>
              {[
                [colors.green, "Scouts patrolling", scouts],
                [colors.lavender, "Soldier spores dormant", 142],
                [colors.textFaint, "Soldiers active", 0],
              ].map(([dot, label, value]) => (
                <div key={label} style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                  <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
                    <span style={{ width: 8, height: 8, borderRadius: "50%", background: dot, display: "inline-block" }} />
                    <span style={{ fontSize: 14, color: colors.textSecondary }}>{label}</span>
                  </div>
                  <span style={{ fontFamily: fontDisplay, fontSize: 15, fontWeight: 600, fontVariantNumeric: "tabular-nums" }}>{value}</span>
                </div>
              ))}
            </div>

            <div style={{ ...card, padding: 28, display: "flex", flexDirection: "column", gap: 18 }}>
              <div style={cardTitle}>Network</div>
              <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                <span style={{ fontSize: 14, color: colors.textSecondary }}>Ledger sync</span>
                <span style={{ fontSize: 13, color: colors.greenLight, fontWeight: 600 }}>Block {sys.block.toLocaleString()}</span>
              </div>
              <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                <span style={{ fontSize: 14, color: colors.textSecondary }}>Threat Registry</span>
                <span style={{ fontSize: 13, color: colors.greenLight, fontWeight: 600 }}>Live</span>
              </div>
              <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                <span style={{ fontSize: 14, color: colors.textSecondary }}>Genome Registry</span>
                <span style={{ fontSize: 13, color: colors.textFaint, fontWeight: 600 }}>Synced 2 min ago</span>
              </div>
            </div>
          </div>

          <div style={{ ...card, padding: 28, display: "flex", flexDirection: "column", gap: 16 }}>
            <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
              <div style={cardTitle}>Recent activity</div>
              <a href="#log" style={{ fontSize: 13 }}>Full log</a>
            </div>
            <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", background: colors.block, border: `1px solid ${colors.borderPanel}`, borderRadius: 10, padding: "14px 18px", gap: 16 }}>
              <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
                <span style={{ fontFamily: fontDisplay, fontSize: 18, fontWeight: 700, fontVariantNumeric: "tabular-nums" }}>{sys.activeNodes.toLocaleString()}</span>
                <span style={{ fontSize: 11, color: colors.textFaint, letterSpacing: 1, textTransform: "uppercase" }}>Active nodes</span>
              </div>
              <div style={{ display: "flex", flexDirection: "column", gap: 2, textAlign: "right" }}>
                <span style={{ fontFamily: fontDisplay, fontSize: 18, fontWeight: 700, fontVariantNumeric: "tabular-nums", color: colors.greenLight }}>{sys.curesPerMin.toLocaleString()}</span>
                <span style={{ fontSize: 11, color: colors.textFaint, letterSpacing: 1, textTransform: "uppercase" }}>Cures / min network-wide</span>
              </div>
            </div>
            {events.length === 0 ? (
              <p style={{ fontSize: 13, color: colors.textFaint, margin: 0 }}>Waiting for ledger events…</p>
            ) : (
              events.map((ev) => (
                <div key={ev.key} style={{ display: "flex", alignItems: "baseline", gap: 14, padding: "13px 0", borderBottom: `1px solid ${colors.border}` }}>
                  <span style={{ fontFamily: fontDisplay, fontSize: 12, color: colors.textFaint, minWidth: 58, fontVariantNumeric: "tabular-nums" }}>{ev.time}</span>
                  <span style={{ width: 7, height: 7, borderRadius: "50%", background: ev.color, display: "inline-block", flexShrink: 0, position: "relative", top: -1 }} />
                  <span style={{ fontSize: 14, color: colors.textSecondary, lineHeight: 1.5 }}>{ev.text}</span>
                </div>
              ))
            )}
          </div>
        </section>

        {/* MACHINE STATS */}
        <section style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: 24 }}>
          <div style={{ ...card, padding: "22px 26px", display: "flex", flexDirection: "column", gap: 4 }}>
            <div style={{ fontSize: 11, color: colors.textFaint, letterSpacing: 1, textTransform: "uppercase" }}>Disk</div>
            <div style={{ fontFamily: fontDisplay, fontSize: 24, fontWeight: 700, fontVariantNumeric: "tabular-nums" }}>412 / 1000 GB</div>
            <div style={{ height: 4, borderRadius: 2, background: colors.borderPanel, marginTop: 6, overflow: "hidden" }}>
              <div style={{ width: "41%", height: "100%", background: colors.lavender }} />
            </div>
          </div>
          <div style={{ ...card, padding: "22px 26px", display: "flex", flexDirection: "column", gap: 4 }}>
            <div style={{ fontSize: 11, color: colors.textFaint, letterSpacing: 1, textTransform: "uppercase" }}>Network</div>
            <div style={{ fontFamily: fontDisplay, fontSize: 24, fontWeight: 700, fontVariantNumeric: "tabular-nums" }}>↓ {netDown} MB/s</div>
            <div style={{ fontSize: 12, color: colors.textFaint, fontVariantNumeric: "tabular-nums" }}>↑ {netUp} MB/s</div>
          </div>
          <div style={{ ...card, padding: "22px 26px", display: "flex", flexDirection: "column", gap: 4 }}>
            <div style={{ fontSize: 11, color: colors.textFaint, letterSpacing: 1, textTransform: "uppercase" }}>Uptime</div>
            <div style={{ fontFamily: fontDisplay, fontSize: 24, fontWeight: 700, fontVariantNumeric: "tabular-nums" }}>3d 14h 02m</div>
            <div style={{ fontSize: 12, color: colors.textFaint }}>Since last restart</div>
          </div>
          <div style={{ ...card, padding: "22px 26px", display: "flex", flexDirection: "column", gap: 4 }}>
            <div style={{ fontSize: 11, color: colors.textFaint, letterSpacing: 1, textTransform: "uppercase" }}>Battery</div>
            <div style={{ fontFamily: fontDisplay, fontSize: 24, fontWeight: 700, fontVariantNumeric: "tabular-nums", color: colors.greenLight }}>100%</div>
            <div style={{ fontSize: 12, color: colors.textFaint }}>Plugged in</div>
          </div>
        </section>

        {/* PROCESS GRAPH + IMMUNE LEDGER */}
        <section style={{ display: "grid", gridTemplateColumns: "1.2fr 1fr", gap: 24, alignItems: "start" }}>
          <div style={{ ...card, padding: 28, display: "flex", flexDirection: "column", gap: 16 }}>
            <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
              <div style={cardTitle}>Running programs</div>
              <div style={{ fontSize: 12, color: colors.textFaint }}>{scouts} scouts watching · {processes.length} processes</div>
            </div>
            <div style={{ display: "flex", alignItems: "center", gap: 14, padding: "4px 0 8px", borderBottom: `1px solid ${colors.border}`, fontSize: 11, color: colors.textFaint, letterSpacing: 1, textTransform: "uppercase" }}>
              <span style={{ width: 7, flexShrink: 0 }} />
              <span style={{ minWidth: 90 }}>Program</span>
              <span style={{ minWidth: 64 }}>Scout</span>
              <span style={{ minWidth: 52, textAlign: "right" }}>CPU</span>
              <span style={{ minWidth: 72, textAlign: "right" }}>Memory</span>
              <span style={{ marginLeft: "auto" }}>Status</span>
            </div>
            {processes.length === 0 ? (
              <p style={{ fontSize: 13, color: colors.textFaint, margin: 0 }}>Waiting for ecosystem data…</p>
            ) : (
              processes.map((proc) => (
                <div key={proc.key} style={{ display: "flex", alignItems: "center", gap: 14, padding: "12px 0", borderBottom: `1px solid ${colors.border}` }}>
                  <span style={{ width: 7, height: 7, borderRadius: "50%", background: proc.color, display: "inline-block", flexShrink: 0 }} />
                  <span style={{ fontSize: 14, color: colors.text, fontWeight: 500, minWidth: 90 }}>{proc.name}</span>
                  <span style={{ fontFamily: fontDisplay, fontSize: 12, color: colors.lavender, minWidth: 64 }}>{proc.scout}</span>
                  <span style={{ fontFamily: fontDisplay, fontSize: 13, color: colors.textSecondary, minWidth: 52, textAlign: "right", fontVariantNumeric: "tabular-nums" }}>{proc.cpu}%</span>
                  <span style={{ fontFamily: fontDisplay, fontSize: 13, color: colors.textSecondary, minWidth: 72, textAlign: "right", fontVariantNumeric: "tabular-nums" }}>{proc.mem}</span>
                  <span style={{ fontSize: 13, color: colors.textFaint, marginLeft: "auto" }}>{proc.status}</span>
                </div>
              ))
            )}
          </div>

          <div style={{ ...card, padding: 28, display: "flex", flexDirection: "column", gap: 16 }}>
            <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
              <div style={cardTitle}>Immune ledger</div>
              <a href="#ledger" style={{ fontSize: 13 }}>All blocks</a>
            </div>
            {blocks.length === 0 ? (
              <p style={{ fontSize: 13, color: colors.textFaint, margin: 0 }}>Waiting for ledger events…</p>
            ) : (
              blocks.map((blk) => (
                <div key={blk.key} style={{ background: colors.block, border: `1px solid ${colors.borderPanel}`, borderRadius: 10, padding: "16px 18px", display: "flex", flexDirection: "column", gap: 8 }}>
                  <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                    <span style={{ fontFamily: fontDisplay, fontSize: 14, fontWeight: 700 }}>BLCK #{blk.num}</span>
                    <span style={{ fontSize: 12, color: colors.textFaint, fontVariantNumeric: "tabular-nums" }}>{blk.ago}</span>
                  </div>
                  <div style={{ display: "flex", gap: 10, fontSize: 13 }}>
                    <span style={{ color: colors.textFaint, minWidth: 92 }}>Trajectory</span>
                    <span style={{ fontFamily: fontDisplay, color: colors.textSecondary }}>{blk.trajectory}</span>
                  </div>
                  <div style={{ display: "flex", gap: 10, fontSize: 13 }}>
                    <span style={{ color: colors.textFaint, minWidth: 92 }}>Gene vector</span>
                    <span style={{ fontFamily: fontDisplay, color: colors.lavender }}>{blk.vector}</span>
                  </div>
                  <div style={{ display: "flex", gap: 10, fontSize: 13 }}>
                    <span style={{ color: colors.textFaint, minWidth: 92 }}>Status</span>
                    <span style={{ fontWeight: 600, color: blk.statusColor }}>{blk.status}</span>
                  </div>
                </div>
              ))
            )}
          </div>
        </section>
      </main>
    </div>
  );
}
