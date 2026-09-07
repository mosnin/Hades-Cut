# Hades Cut

Hades Cut is an agent-operated macOS screen recorder and nonlinear editor built as a downstream distribution of Cap. It keeps Cap's native capture, editor, renderer, and `.cap` project format, then adds a deterministic control plane for AI agents.

## Product contract

An agent can complete the full loop without guessing UI coordinates:

1. Inspect macOS capture permissions and enumerate stable screen, window, camera, and microphone IDs.
2. Start a detached Studio recording and receive a recording ID plus project path.
3. Stop and validate the recording.
4. Read the current edit configuration and its content revision.
5. dry-run a partial edit, apply it only against the revision it inspected, and recover the prior configuration from automatic history.
6. Export locally and verify the terminal completion event.

The visible Mac app remains available for human review and repair. Computer-use access is a supported fallback target; release builds must pass the accessibility audit in the milestones below. CLI or MCP is the primary agent interface because it is deterministic and independently verifiable.

## CLI workflow

Build the CLI:

```sh
cargo build -p cap
```

Discover and record:

```sh
target/debug/cap doctor --json
target/debug/cap targets --json
target/debug/cap record start --screen <screen-id> --detach --json
target/debug/cap record stop --id <recording-id> --json
target/debug/cap project validate <project.cap> --json
```

Read the edit and capture its `revision`:

```sh
target/debug/cap project config patch <project.cap> --patch-json '{}' --dry-run --json
```

Apply a partial edit using [JSON Merge Patch (RFC 7396)](https://www.rfc-editor.org/rfc/rfc7396):

```sh
target/debug/cap project config patch <project.cap> \
  --expected-revision <revision> \
  --patch-file edit.json \
  --json
```

An applied edit returns the old and new revision, exact changed JSON Pointer paths, and the recovery file written under `<project.cap>/.hades/history/`. Arrays are replaced as complete values, so an agent must read the current configuration before editing timeline arrays.

Export:

```sh
target/debug/cap export <project.cap> --output finished.mp4 --quality maximum --json
```

## Local MCP

Run the credential-free local server over stdio:

```sh
target/debug/cap mcp local
```

Configure an MCP client with:

```json
{
  "mcpServers": {
    "hades-cut": {
      "command": "/absolute/path/to/target/debug/cap",
      "args": ["mcp", "local"]
    }
  }
}
```

The server exposes:

- `hades_targets`
- `hades_record_start`
- `hades_record_stop`
- `hades_record_status`
- `hades_project_get`
- `hades_project_validate`
- `hades_project_patch`
- `hades_editor_open`
- `hades_export`

Local MCP never needs a Cap account token. Network library, upload, comment, and organization operations stay on the separate authenticated `cap mcp serve` boundary.

## ChatGPT Work connection

The repository contains a distributable ChatGPT/Codex plugin at `plugins/hades-cut`. Import that directory from the GitHub repository into a workspace, or install it locally while developing. Its skill teaches ChatGPT Work the capture, privacy, recovery, edit, and verification protocol; its MCP declaration starts `cap mcp local` on the Mac where Hades Cut is installed.

The `cap` executable must be on `PATH`. A source build can be exposed temporarily with:

```sh
export PATH="$(pwd)/target/debug:$PATH"
```

Then ask ChatGPT Work to use the Hades Cut plugin to record a product feature. The agent uses its computer-use capability to operate the product and the Hades tools to control capture and editing. A workspace administrator can constrain or disable write actions through workspace plugin controls.

This local stdio connection is intentionally desktop-only. A cloud ChatGPT session cannot reach a private Mac process directly; supporting unattended cloud jobs requires a separately authenticated HTTPS MCP bridge, explicit machine enrollment, and per-job consent. That bridge is not included yet, so this version is designed for ChatGPT Work or Codex running with local desktop tool access.

## Agent editing protocol

For every edit, the agent follows `read → propose → dry-run → apply → export → inspect`.

- `hades_project_get` returns the whole typed editor configuration and revision.
- `hades_project_patch` accepts a partial object, `expectedRevision`, and `dryRun`.
- A stale revision fails rather than overwriting a person's intervening edit.
- The updated configuration is validated by Cap's project model before any write.
- A successful write stores the preceding full configuration in `.hades/history`.
- The desktop editor sees the same `project-config.json`, so agent and human edits converge on one source of truth.

## macOS application build

The Hades distribution overlay changes the bundle name and identifier and disables Cap's production updater, preventing a Cap update from replacing the fork:

```sh
pnpm install
pnpm env-setup
pnpm cap-setup
pnpm tauri:build:hades
```

The signed/notarized release pipeline still needs Hades-owned Apple Developer credentials, updater keys, icons, and release URLs. Those are deployment secrets and brand assets, not source-code blockers.

## Architecture boundary

```text
AI agent
  ├─ local MCP (typed tools, stdio)
  ├─ CLI (JSON/NDJSON, exit codes)
  └─ Computer Use (accessible human review/fallback)
           │
           ▼
Hades control plane
  ├─ capture lifecycle
  ├─ revision-safe merge patches
  ├─ automatic edit history
  └─ export completion evidence
           │
           ▼
Cap media kernel
  ├─ ScreenCaptureKit / audio / camera
  ├─ .cap project model
  ├─ GPU editor and renderer
  └─ MP4 / MOV / GIF export
```

## Next product milestones

1. Add semantic high-level edit operations (`trimSilence`, `focusWindow`, `addZoom`, `addCaption`, `redact`) that compile into the existing project configuration.
2. Add perception tools for preview frames, timeline thumbnails, transcript alignment, and post-export visual QA.
3. Add a job ledger with resumable multi-step plans and policy limits for output directory, duration, and disk usage.
4. Audit every interactive desktop control for an accessible name, role, state, keyboard action, and deterministic focus order.
5. Replace inherited Cap marks, URLs, signing identities, telemetry destinations, and updater infrastructure before public distribution.

## License

Cap's repository states that most of the project is AGPLv3, with specified crate families under MIT and third-party components under their original licenses. A distributed Hades Cut build must preserve the applicable notices and offer corresponding source as required by AGPLv3.
