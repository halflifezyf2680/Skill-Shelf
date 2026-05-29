import React from 'react';
import {Monitor, SearchCheck} from 'lucide-react';
import {AbsoluteFill, spring, useCurrentFrame, useVideoConfig} from 'remotion';

export const CompareScene: React.FC = () => {
  const frame = useCurrentFrame();
  const {fps} = useVideoConfig();
  const rightIn = spring({frame: frame - 20, fps, config: {damping: 18, stiffness: 90}});
  return (
    <AbsoluteFill className="scene">
      <div className="scene-heading">
        <span>Comparison</span>
        <h2>Show what each side is for</h2>
      </div>
      <div className="compare-grid">
        <div className="compare-card">
          <Monitor size={48} />
          <h3>Before</h3>
          <p>Broad search, mixed results, unclear next step.</p>
        </div>
        <div className="compare-card primary" style={{opacity: rightIn}}>
          <SearchCheck size={48} />
          <h3>After</h3>
          <p>Anchored evidence, visible flow, concrete action.</p>
        </div>
      </div>
    </AbsoluteFill>
  );
};
