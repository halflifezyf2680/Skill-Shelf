import React from 'react';
import {ArrowRight, Cpu, Database, FileCode2, ServerCog} from 'lucide-react';
import {AbsoluteFill, spring, useCurrentFrame, useVideoConfig} from 'remotion';

const steps = [
  ['Scan', 'Parse source and extract symbols', Cpu],
  ['Query', 'Ask structured questions with SQL', Database],
  ['Render', 'Shape output for the model', ServerCog],
  ['Act', 'Choose the next checks', FileCode2],
] as const;

export const FlowScene: React.FC = () => {
  const frame = useCurrentFrame();
  const {fps} = useVideoConfig();
  return (
    <AbsoluteFill className="scene">
      <div className="scene-heading">
        <span>Architecture Flow</span>
        <h2>Make the pipeline visible</h2>
      </div>
      <div className="flow-grid">
        {steps.map(([title, text, Icon], index) => {
          const itemIn = spring({frame: frame - index * 14, fps, config: {damping: 18, stiffness: 90}});
          return (
            <React.Fragment key={title}>
              <div className="flow-card" style={{opacity: itemIn}}>
                <Icon size={42} />
                <strong>{title}</strong>
                <span>{text}</span>
              </div>
              {index < steps.length - 1 ? <ArrowRight className="arrow" size={36} /> : null}
            </React.Fragment>
          );
        })}
      </div>
      <div className="result-strip">result: entry / owner / callers / next checks</div>
    </AbsoluteFill>
  );
};
