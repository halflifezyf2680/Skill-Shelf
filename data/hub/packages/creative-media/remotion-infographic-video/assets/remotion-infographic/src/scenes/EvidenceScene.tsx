import React from 'react';
import {CheckCircle2, Filter, SearchCheck} from 'lucide-react';
import {AbsoluteFill, spring, useCurrentFrame, useVideoConfig} from 'remotion';

const evidence = ['entry point', 'owner file', 'direct caller'];

export const EvidenceScene: React.FC = () => {
  const frame = useCurrentFrame();
  const {fps} = useVideoConfig();
  const filterIn = spring({frame: frame - 54, fps, config: {damping: 18, stiffness: 90}});
  return (
    <AbsoluteFill className="scene">
      <div className="scene-heading">
        <span>Evidence Filter</span>
        <h2>Recall broadly, return only proof</h2>
      </div>
      <div className="query-bar">
        <SearchCheck size={34} />
        <strong>Ambiguous production issue</strong>
      </div>
      <div className="evidence-panel" style={{opacity: filterIn}}>
        <div className="panel-title">
          <Filter size={32} />
          <strong>kept evidence</strong>
        </div>
        {evidence.map((item) => (
          <div className="evidence-row" key={item}>
            <CheckCircle2 size={24} />
            <code>{item}</code>
            <span>ready for context</span>
          </div>
        ))}
      </div>
    </AbsoluteFill>
  );
};
