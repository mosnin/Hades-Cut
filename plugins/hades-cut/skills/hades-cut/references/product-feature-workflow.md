# Product-feature workflow

## Capture packet

Resolve these fields before starting the recorder:

- `feature`: the one product capability the video must prove
- `startState`: the visible state before the demonstration
- `actions`: the minimum observable interaction sequence
- `successState`: the visible proof that the feature worked
- `target`: a window ID when possible, otherwise a screen ID
- `durationBudget`: target duration plus a small recovery margin
- `privacyCheck`: notifications hidden and no secret-entry step in the take

If the feature requires authentication, prepare the authenticated state before recording. Never record a password, one-time code, recovery code, API key, or payment detail.

## Execution state machine

```text
planned -> recording -> stopped -> validated -> edited -> exported -> verified
                |           |           |          |
                +--------> stopped ----> needs-review
```

Only `hades_record_start` enters `recording`. Every exit from that state calls `hades_record_stop`, including failed product interaction. Retakes are new recordings; never silently overwrite a previous project.

## Editing pass

1. Read the project and revision.
2. Identify dead time before the first meaningful action and after the success state.
3. Preserve all timeline fields not intentionally changed.
4. Add zooms only when they clarify a control, result, or spatial transition.
5. Dry-run and inspect changed paths.
6. Apply against the inspected revision.
7. Re-read and verify the new revision.

## Acceptance gates

A finished result requires all of the following:

- the recording stopped cleanly
- project validation reports valid media
- the edit applied against the expected revision
- a recovery snapshot exists for a changed project
- export emitted a terminal completion event
- the output file exists
- the feature's success state is visible in the final video
- no secret or unrelated private content is visible

If visual inspection is unavailable, the last two gates remain explicitly unverified.
