// StrainTree — evolutionary phylogeny of strains. Feeds from the Rust `strains` event stream
// via useTauriEvents. Renders the parent/child lineage as a nested list.
import { useTauriEvents } from "../hooks/useTauriEvents.js";

function TreeNode({ strain, byParent }) {
  const children = byParent.get(strain.id) ?? [];
  return (
    <li>
      {strain.label} <em>({strain.stage})</em>
      {children.length > 0 && (
        <ul>
          {children.map((child) => (
            <TreeNode key={child.id} strain={child} byParent={byParent} />
          ))}
        </ul>
      )}
    </li>
  );
}

export default function StrainTree() {
  const { data: strains, dropped } = useTauriEvents("strains");

  if (!strains) return <p>Waiting for strain data…</p>;

  const byParent = new Map();
  for (const strain of strains) {
    if (!strain.parentId) continue;
    if (!byParent.has(strain.parentId)) byParent.set(strain.parentId, []);
    byParent.get(strain.parentId).push(strain);
  }
  const root = strains.find((s) => s.parentId === null);

  return (
    <section>
      <h2>Strains {dropped && <span style={{ color: "#f44336" }}>(stream dropped — showing last known state)</span>}</h2>
      {root ? (
        <ul>
          <TreeNode strain={root} byParent={byParent} />
        </ul>
      ) : (
        <p>No root strain yet.</p>
      )}
    </section>
  );
}
