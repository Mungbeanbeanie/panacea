// LedgerTerminal — rolling Proof-of-Immunity event log. Feeds from the Rust `ledger` event
// stream via useTauriEvents. Newest event on top, capped list from the hook.
import { useTauriEvents } from "../hooks/useTauriEvents.js";

export default function LedgerTerminal() {
  const { data: events, dropped } = useTauriEvents("ledger");

  if (!events) return <p>Waiting for ledger events…</p>;

  return (
    <section>
      <h2>Ledger {dropped && <span style={{ color: "#f44336" }}>(stream dropped — showing last known state)</span>}</h2>
      <ul style={{ fontFamily: "monospace", listStyle: "none", padding: 0 }}>
        {events.map((event) => (
          <li key={event.id}>
            [{event.at}] {event.kind} — {event.threatId}
          </li>
        ))}
      </ul>
    </section>
  );
}
