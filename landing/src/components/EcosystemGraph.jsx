// EcosystemGraph — live process/node map. Feeds from the Rust `ecosystem` event stream via
// useTauriEvents. Renders an empty state until the first event arrives.
import { useTauriEvents } from "../hooks/useTauriEvents.js";

export default function EcosystemGraph() {
  const nodes = useTauriEvents("ecosystem");
  if (!nodes) return <p>Waiting for ecosystem data…</p>;
  return <div>{/* TODO: canvas map of nodes */}</div>;
}
