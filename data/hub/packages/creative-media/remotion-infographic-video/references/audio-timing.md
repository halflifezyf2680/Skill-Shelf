# Audio Timing

Use a manifest for narration-driven timing.

## Manifest Shape

```json
{
  "segments": [
    {
      "key": "cover",
      "file": "narration/00-cover.wav",
      "targetDuration": 5,
      "actualDuration": 4.7,
      "text": "..."
    }
  ]
}
```

## Rules

- Use one audio file per scene.
- Set scene duration to `max(targetDuration, actualDuration + padding)`.
- Use `0.2s` to `0.4s` visual padding.
- Keep display text free to use tool names; rewrite TTS text for pronunciation.
- Never attach a concatenated preview file to the final composition.

## Remotion Pattern

```tsx
<Sequence from={secondsToFrames(timing.start)} durationInFrames={secondsToFrames(timing.duration)}>
  <Audio src={staticFile(timing.audioFile)} />
  <Scene />
</Sequence>
```
