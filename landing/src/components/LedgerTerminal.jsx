// LedgerTerminal — rolling Proof-of-Immunity event log. Feeds from the Rust `ledger` event
// stream via useTauriEvents. Shows an empty state until events arrive.
import { useTauriEvents } from "../hooks/useTauriEvents.js";

export default function LedgerTerminal() {
  const events = useTauriEvents("ledger");
  if (!events) return <p>Waiting for ledger events…</p>;
  return <ul>{/* TODO: render PoI events */}</ul>;
}
