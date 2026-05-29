import React from 'react';
import {AbsoluteFill, Sequence} from 'remotion';
import {CompareScene} from './scenes/CompareScene';
import {CoverScene} from './scenes/CoverScene';
import {EvidenceScene} from './scenes/EvidenceScene';
import {FlowScene} from './scenes/FlowScene';
import {sceneTimings, secondsToFrames} from './videoConfig';
import './styles.css';

const sceneList = [
  ['cover', CoverScene],
  ['flow', FlowScene],
  ['compare', CompareScene],
  ['evidence', EvidenceScene],
] as const;

export const InfographicVideo: React.FC = () => (
  <AbsoluteFill className="composition">
    {sceneList.map(([key, Scene]) => (
      <Sequence key={key} from={secondsToFrames(sceneTimings[key].start)} durationInFrames={secondsToFrames(sceneTimings[key].duration)}>
        <Scene />
      </Sequence>
    ))}
  </AbsoluteFill>
);
