export const WIDTH = 1920;
export const HEIGHT = 1080;
export const FPS = 30;

export const scenes = [
  {key: 'cover', duration: 5},
  {key: 'flow', duration: 8},
  {key: 'compare', duration: 8},
  {key: 'evidence', duration: 9},
] as const;

let cursor = 0;
export const sceneTimings = Object.fromEntries(
  scenes.map((scene) => {
    const timing = [scene.key, {start: cursor, duration: scene.duration}];
    cursor += scene.duration;
    return timing;
  }),
) as Record<(typeof scenes)[number]['key'], {start: number; duration: number}>;

export const secondsToFrames = (seconds: number) => Math.round(seconds * FPS);
export const DURATION_IN_FRAMES = secondsToFrames(cursor);
