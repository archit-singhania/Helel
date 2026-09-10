const foundations = ["Desktop shell", "Rust core", "Python ML workspace", "Shared contracts"];

export function App() {
  return (
    <main className="shell">
      <section className="hero">
        <p className="eyebrow">PHASE 0 · FOUNDATION</p>
        <h1>Helel</h1>
        <p className="tagline">Build. Reason. Execute.</p>
        <p className="status">Local-first foundations are ready. Product features begin in Phase 1.</p>
        <ul aria-label="Phase 0 foundations">
          {foundations.map((foundation) => <li key={foundation}>{foundation}<span>ready</span></li>)}
        </ul>
      </section>
    </main>
  );
}
