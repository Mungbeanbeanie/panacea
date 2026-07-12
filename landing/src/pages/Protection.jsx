// Protection — kill log, allergy list, and suppression list for this host. From
// Protection.dc.html, rewired off the real `ledger` stream (useTauriEvents) instead of the
// design's static placeholders: kills from `threat.neutralized`, allergies from
// `gene.allergy_flagged`, suppressions from `gene.suppressed`.
import SignedInNav from "../components/SignedInNav.jsx";
import { useTauriEvents } from "../hooks/useTauriEvents.js";
import { colors, fontDisplay } from "../tokens.js";

const card = { background: colors.panel, border: `1px solid ${colors.borderPanel}`, borderRadius: 14 };
const cardTitle = { fontFamily: fontDisplay, fontSize: 16, fontWeight: 600, letterSpacing: 0.5 };

export default function Protection({ auth, navigate }) {
  const { data: ledger } = useTauriEvents("ledger");
  const events = ledger ?? [];
  const kills = events.filter((ev) => ev.kind === "threat.neutralized");
  const allergies = events.filter((ev) => ev.kind === "gene.allergy_flagged");
  const suppressions = events.filter((ev) => ev.kind === "gene.suppressed");
  const threatsTracked = new Set(events.map((ev) => ev.threatId)).size;

  return (
    <div className="pan-page" style={{ minHeight: "100vh", background: colors.bg, color: colors.text, display: "flex", flexDirection: "column" }}>
      <SignedInNav active="protection" userName={auth.userName} onNavigate={navigate} />

      <main style={{ maxWidth: 1180, width: "100%", margin: "0 auto", padding: 48, boxSizing: "border-box", display: "flex", flexDirection: "column", gap: 40 }}>
        <div style={{ display: "flex", alignItems: "flex-end", justifyContent: "space-between", gap: 24 }}>
          <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
            <h1 style={{ fontFamily: fontDisplay, fontSize: 32, fontWeight: 700, margin: 0 }}>Protection</h1>
            <p style={{ fontSize: 15, color: colors.textFaint, margin: 0 }}>Every threat this host has faced — and how it was killed.</p>
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: 10, background: colors.greenTint, border: `1px solid ${colors.greenTintBorder}`, borderRadius: 10, padding: "12px 20px" }}>
            <span style={{ width: 8, height: 8, borderRadius: "50%", background: colors.green, display: "inline-block" }} />
            <span style={{ fontSize: 13, fontWeight: 600, color: colors.greenLight, letterSpacing: 1, textTransform: "uppercase" }}>All systems immune</span>
          </div>
        </div>

        {/* SUMMARY */}
        <section style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: 24 }}>
          {[
            ["Threats killed", String(kills.length), colors.greenLight, "Neutralized this host"],
            ["Threats tracked", String(threatsTracked), undefined, "Distinct Threat_IDs seen"],
            ["Suppressed", String(suppressions.length), colors.red, "Epigenetic kill-switch"],
            ["Allergies", String(allergies.length), colors.yellow, "Flagged by Lymph Node"],
          ].map(([label, value, color, sub]) => (
            <div key={label} style={{ ...card, padding: "22px 26px", display: "flex", flexDirection: "column", gap: 4 }}>
              <div style={{ fontSize: 11, color: colors.textFaint, letterSpacing: 1, textTransform: "uppercase" }}>{label}</div>
              <div style={{ fontFamily: fontDisplay, fontSize: 28, fontWeight: 700, fontVariantNumeric: "tabular-nums", color }}>{value}</div>
              <div style={{ fontSize: 12, color: colors.textFaint }}>{sub}</div>
            </div>
          ))}
        </section>

        {/* KILL LOG */}
        <section style={{ ...card, padding: 28, display: "flex", flexDirection: "column", gap: 16 }}>
          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
            <div style={cardTitle}>Soldier deployment log</div>
            <div style={{ fontSize: 12, color: colors.textFaint }}>Most recent first</div>
          </div>
          {kills.length === 0 ? (
            <p style={{ fontSize: 13, color: colors.textFaint, margin: 0 }}>No threats neutralized yet.</p>
          ) : (
            kills.map((k) => (
              <div key={k.id} style={{ display: "flex", alignItems: "center", gap: 14, padding: "16px 0", borderBottom: `1px solid ${colors.border}` }}>
                <span style={{ width: 8, height: 8, borderRadius: "50%", background: colors.green, display: "inline-block", flexShrink: 0 }} />
                <span style={{ fontFamily: fontDisplay, fontSize: 14, fontWeight: 600 }}>{k.threatId}</span>
                <span style={{ fontSize: 12, color: colors.textFaint, marginLeft: "auto", fontVariantNumeric: "tabular-nums" }}>{k.at}</span>
                <span style={{ fontSize: 13, fontWeight: 600, color: colors.greenLight, minWidth: 100, textAlign: "right" }}>Neutralized</span>
              </div>
            ))
          )}
        </section>

        {/* ALLERGY LIST */}
        <section style={{ ...card, padding: 28, display: "flex", flexDirection: "column", gap: 16 }}>
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            <div style={cardTitle}>Allergy list</div>
            <div style={{ fontSize: 13, color: colors.textFaint }}>Candidate genes the Lymph Node flagged for breaking a whitelisted app and dropped.</div>
          </div>
          {allergies.length === 0 ? (
            <p style={{ fontSize: 13, color: colors.textFaint, margin: 0 }}>No allergies flagged.</p>
          ) : (
            allergies.map((a) => (
              <div key={a.id} style={{ display: "flex", alignItems: "center", gap: 14, padding: "16px 0", borderTop: `1px solid ${colors.border}` }}>
                <span style={{ width: 8, height: 8, borderRadius: "50%", background: colors.yellow, display: "inline-block", flexShrink: 0 }} />
                <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
                  <span style={{ fontSize: 14, fontWeight: 500 }}>{a.threatId}</span>
                  <span style={{ fontSize: 12, color: colors.textFaint }}>Allergy-flagged in Lymph Node regression — {a.at}</span>
                </div>
              </div>
            ))
          )}
        </section>

        {/* SUPPRESSION LIST */}
        <section style={{ ...card, padding: 28, display: "flex", flexDirection: "column", gap: 16, marginBottom: 40 }}>
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            <div style={cardTitle}>Suppressions</div>
            <div style={{ fontSize: 13, color: colors.textFaint }}>Cures halted by the epigenetic kill-switch before dispense.</div>
          </div>
          {suppressions.length === 0 ? (
            <p style={{ fontSize: 13, color: colors.textFaint, margin: 0 }}>No cures suppressed.</p>
          ) : (
            suppressions.map((s) => (
              <div key={s.id} style={{ display: "flex", alignItems: "center", gap: 14, padding: "16px 0", borderTop: `1px solid ${colors.border}` }}>
                <span style={{ width: 8, height: 8, borderRadius: "50%", background: colors.red, display: "inline-block", flexShrink: 0 }} />
                <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
                  <span style={{ fontSize: 14, fontWeight: 500 }}>{s.threatId}</span>
                  <span style={{ fontSize: 12, color: colors.textFaint }}>Suppressed via Epigenetic Suppressor Token — {s.at}</span>
                </div>
              </div>
            ))
          )}
        </section>
      </main>
    </div>
  );
}
