import React from 'react';
import {Composition} from 'remotion';
import {InfographicVideo} from './InfographicVideo';
import {DURATION_IN_FRAMES, FPS, HEIGHT, WIDTH} from './videoConfig';

export const RemotionRoot: React.FC = () => (
  <Composition
    id="InfographicVideo"
    component={InfographicVideo}
    durationInFrames={DURATION_IN_FRAMES}
    fps={FPS}
    width={WIDTH}
    height={HEIGHT}
  />
);
