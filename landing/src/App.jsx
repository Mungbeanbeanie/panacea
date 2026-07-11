// Application shell + layout. Hosts the three live observability views; holds no data of
// its own (see CLAUDE.md — the UI mirrors Rust state).
import EcosystemGraph from "./components/EcosystemGraph.jsx";
import LedgerTerminal from "./components/LedgerTerminal.jsx";
import StrainTree from "./components/StrainTree.jsx";

export default function App() {
  return (
    <main>
      <h1>Bio-Digital Defense</h1>
      <EcosystemGraph />
      <LedgerTerminal />
      <StrainTree />
    </main>
  );
}
