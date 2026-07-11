// EcosystemGraph — live process/node map. Feeds from the Rust `ecosystem` event stream via
// useTauriEvents. Each node is a monitored PID with its Stage-1 trajectory score and status.
import { useTauriEvents } from "../hooks/useTauriEvents.js";

const STATUS_COLOR = { normal: "#4caf50", watching: "#ff9800", suspended: "#f44336" };

export default function EcosystemGraph() {
  const { data: nodes, dropped } = useTauriEvents("ecosystem");

  if (!nodes) return <p>Waiting for ecosystem data…</p>;

  return (
    <section>
      <h2>Ecosystem {dropped && <span style={{ color: "#f44336" }}>(stream dropped — showing last known state)</span>}</h2>
      <div style={{ display: "flex", flexWrap: "wrap", gap: "0.75rem" }}>
        {nodes.map((node) => (
          <div
            key={node.pid}
            title={`PID ${node.pid} · score ${node.score}`}
            style={{
              border: `2px solid ${STATUS_COLOR[node.status] ?? "#999"}`,
              borderRadius: "8px",
              padding: "0.5rem 0.75rem",
              minWidth: "8rem",
            }}
          >
            <div>{node.name}</div>
            <div>pid {node.pid}</div>
            <div>score {node.score}</div>
          </div>
        ))}
      </div>
    </section>
  );
}
