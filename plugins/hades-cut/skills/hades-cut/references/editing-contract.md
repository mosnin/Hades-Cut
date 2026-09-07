# Editing contract

Hades Cut edits Cap's `project-config.json` through RFC 7396 JSON Merge Patch. Objects merge recursively, scalar values replace the prior value, `null` removes a field, and arrays replace the complete prior array.

## Safe write protocol

1. Call `hades_project_get`.
2. Build a patch from that exact configuration.
3. Dry-run with `expectedRevision`.
4. Check `changedPaths` for unintended fields.
5. Apply the same patch with the same `expectedRevision`.
6. Keep the returned `historyPath` until the export is accepted.

A revision conflict means another person or agent edited the project. Re-read and rebase the intended edit; never retry with the new revision without reviewing the new configuration.

## Framing example

```json
{
  "aspectRatio": "wide",
  "background": {
    "source": {
      "type": "gradient",
      "from": [18, 12, 38],
      "to": [76, 29, 149],
      "angle": 135
    },
    "padding": 8,
    "rounding": 12,
    "shadow": 70
  },
  "cursor": {
    "size": 120,
    "hideWhenIdle": false,
    "motionBlur": 0.8
  }
}
```

## Trim contract

`timeline.segments` uses source time in seconds:

```json
{
  "recordingSegment": 0,
  "timescale": 1.0,
  "start": 1.25,
  "end": 18.5
}
```

To trim, copy the full `timeline.segments` array, retain each segment's `recordingSegment`, `timescale`, and `name`, and change only intentional `start` or `end` boundaries. Require `start >= 0`, `end > start`, and a positive `timescale`.

## Zoom contract

`timeline.zoomSegments` is also replaced as a whole array. An automatic zoom has this shape:

```json
{
  "start": 4.2,
  "end": 6.8,
  "amount": 1.6,
  "mode": "auto",
  "glideDirection": "none",
  "glideSpeed": 0.5,
  "instantAnimation": false,
  "edgeSnapRatio": 0.25
}
```

Keep zooms non-overlapping unless the renderer's transition behavior is intentionally being used. Leave visible time before and after the focal action so the viewer can maintain spatial context.

## Recovery

Every changed project returns `historyPath`, which contains the complete preceding configuration. Recovery is a full-config operation: inspect the snapshot, verify it belongs to the same `.cap` project, then restore it through the CLI's full `project config set` command or construct a reviewed patch. Do not delete history during an active workflow.
