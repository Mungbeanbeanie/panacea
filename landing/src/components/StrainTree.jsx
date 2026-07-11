// StrainTree — phylogenetic tree of malware strains. Feeds from the Rust `strains` event
// stream via useTauriEvents. Shows an empty state until data arrives.
import { useTauriEvents } from "../hooks/useTauriEvents.js";

export default function StrainTree() {
  const strains = useTauriEvents("strains");
  if (!strains) return <p>Waiting for strain data…</p>;
  return <div>{/* TODO: render strain tree */}</div>;
}
