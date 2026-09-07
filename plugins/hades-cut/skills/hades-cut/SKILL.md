---
name: hades-cut
description: Record product features on macOS, edit local Hades Cut projects, and export verified demo videos. Use when a user asks to capture a workflow, produce a feature walkthrough, polish a screen recording, or operate Hades Cut.
---

# Hades Cut

Use the `hades_*` MCP tools for media state and the current computer-use capability for interacting with the product being demonstrated. Never substitute coordinate guesses for an available semantic target.

## Product feature recording

Before recording, establish the single feature outcome, target app or browser tab, intended audience, and approximate duration from available context. Prefer a window target over an entire display. Do not record password entry, API keys, private notifications, unrelated tabs, or other people's data.

Run `hades_targets`, then start with `hades_record_start`. Treat its `recordingId` as an open resource that must be finalized. Use computer use to perform the shortest coherent happy-path demonstration. If interaction fails or the user interrupts the run, call `hades_record_stop` before doing anything else and report that the take needs review.

After stopping, require the returned project path and validate it with `hades_project_validate`. Do not upload or share a recording unless the user separately asks.

## Editing

Read the project with `hades_project_get`. Preserve its `revision` and use it as `expectedRevision` for every write. For nontrivial edits, call `hades_project_patch` with `dryRun: true`, review `changedPaths`, then apply the identical patch against the same revision.

Favor edits that improve comprehension: remove dead time, frame the relevant surface, add restrained zooms around feature actions, keep cursor motion legible, and preserve enough lead-in and recovery time for each action. Do not fabricate clicks, states, captions, or outcomes that were not captured.

Timeline arrays are replaced as whole values by JSON Merge Patch. Read and preserve every unaffected segment before changing one. Consult [the editing contract](references/editing-contract.md) when trimming, adding zooms, changing framing, or recovering an edit.

## Export and verification

Export with `hades_export` only after the edit validates. A successful tool invocation is not enough: require a terminal completion event and output path. Inspect the resulting video or representative frames when the current environment supports it. Otherwise report that visual QA remains outstanding.

For the detailed capture loop, failure recovery, and acceptance gates, read [the product-feature workflow](references/product-feature-workflow.md).
