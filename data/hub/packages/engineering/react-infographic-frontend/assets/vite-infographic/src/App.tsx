import React, {useState} from 'react';
import {Archive, CheckCircle2, Database, Filter, GitBranch, SearchCheck, Workflow} from 'lucide-react';

const flow = [
  ['Index', 'Structure source into a queryable project map', Database],
  ['Impact', 'Expand anchors into callers and risk', GitBranch],
  ['Execute', 'Run phases with gates and visible state', Workflow],
  ['Memory', 'Keep decisions recall-ready', Archive],
] as const;

const candidates = ['write state mismatch', 'gate rollback', 'memo missing timeline', 'transaction boundary', 'permission state', 'cache refresh'];

export const App: React.FC = () => {
  const [selected, setSelected] = useState(candidates.slice(0, 3));

  return (
    <main>
      <section className="hero">
        <div>
          <span className="eyebrow">Technical Infographic</span>
          <h1>Make the system legible</h1>
          <p>Use flows, evidence filters, and timelines to explain engineering work without burying the user in prose.</p>
        </div>
        <div className="system-ring">
          {flow.map(([label, , Icon]) => (
            <div className="ring-node" key={label}>
              <Icon size={28} />
              <strong>{label}</strong>
            </div>
          ))}
        </div>
      </section>

      <section className="flow-section">
        <div className="section-heading">
          <span>Architecture</span>
          <h2>Turn abstract process into visible structure</h2>
        </div>
        <div className="flow-grid">
          {flow.map(([label, detail, Icon]) => (
            <article className="flow-card" key={label}>
              <Icon size={34} />
              <strong>{label}</strong>
              <span>{detail}</span>
            </article>
          ))}
        </div>
      </section>

      <section className="evidence-section">
        <div className="query-bar">
          <SearchCheck size={30} />
          <strong>Ambiguous issue: write behavior drifted</strong>
        </div>
        <div className="evidence-grid">
          <article className="candidate-panel">
            <div className="panel-title">
              <Archive size={28} />
              <strong>Wide recall</strong>
            </div>
            <div className="chips">
              {candidates.map((candidate) => (
                <button
                  className={selected.includes(candidate) ? 'chip kept' : 'chip'}
                  key={candidate}
                  onClick={() =>
                    setSelected((current) =>
                      current.includes(candidate)
                        ? current.filter((item) => item !== candidate)
                        : current.length < 3
                          ? [...current, candidate]
                          : current,
                    )
                  }
                >
                  {candidate}
                </button>
              ))}
            </div>
          </article>
          <article className="strict-panel">
            <div className="panel-title">
              <Filter size={28} />
              <strong>Strict evidence</strong>
            </div>
            {selected.map((item) => (
              <div className="evidence-row" key={item}>
                <CheckCircle2 size={22} />
                <code>{item}</code>
                <span>ready for context</span>
              </div>
            ))}
          </article>
        </div>
      </section>
    </main>
  );
};
