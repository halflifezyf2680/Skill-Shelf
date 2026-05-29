import React from 'react';
import {Database, GitBranch, History, Workflow} from 'lucide-react';
import {AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig} from 'remotion';

const nodes = [
  {label: 'index', icon: Database},
  {label: 'impact', icon: GitBranch},
  {label: 'workflow', icon: Workflow},
  {label: 'memory', icon: History},
] as const;

export const CoverScene: React.FC = () => {
  const frame = useCurrentFrame();
  const {fps} = useVideoConfig();
  const titleIn = spring({frame, fps, config: {damping: 18, stiffness: 90}});
  const sweep = interpolate(frame, [0, 180], [0, 360], {extrapolateRight: 'clamp'});
  return (
    <AbsoluteFill className="scene cover-scene">
      <div className="title-stack" style={{opacity: titleIn}}>
        <span className="eyebrow">Technical Infographic</span>
        <h1>Explain the system, not just the feature</h1>
        <p>Turn architecture, flow, evidence, and memory into a motion story.</p>
      </div>
      <div className="ring" style={{transform: `rotate(${sweep * 0.05}deg)`}}>
        {nodes.map((node, index) => {
          const Icon = node.icon;
          return (
            <div className="ring-node" key={node.label} style={{transform: `rotate(${index * 90}deg) translateX(260px) rotate(${-index * 90}deg)`}}>
              <Icon size={30} />
              <strong>{node.label}</strong>
            </div>
          );
        })}
      </div>
    </AbsoluteFill>
  );
};
