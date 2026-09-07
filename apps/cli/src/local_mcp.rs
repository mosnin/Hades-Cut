use std::path::PathBuf;

use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Implementation, ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct ProjectInput {
    project_path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct PatchInput {
    project_path: String,
    patch: Value,
    #[serde(default)]
    expected_revision: Option<String>,
    #[serde(default)]
    dry_run: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct TargetsInput {
    #[serde(default)]
    kind: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct RecordStartInput {
    #[serde(default)]
    screen: Option<String>,
    #[serde(default)]
    window: Option<String>,
    #[serde(default)]
    camera: Option<String>,
    #[serde(default)]
    mic: Option<String>,
    #[serde(default)]
    system_audio: bool,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    fps: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct RecordStopInput {
    recording_id: String,
    #[serde(default)]
    timeout_seconds: Option<f64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
struct ExportInput {
    project_path: String,
    output_path: String,
    #[serde(default)]
    format: Option<String>,
    #[serde(default)]
    fps: Option<u32>,
    #[serde(default)]
    resolution: Option<String>,
    #[serde(default)]
    quality: Option<String>,
}

#[derive(Clone)]
struct HadesLocalMcpServer {
    tool_router: ToolRouter<Self>,
}

impl HadesLocalMcpServer {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    fn result<T: serde::Serialize>(result: Result<T, String>) -> CallToolResult {
        match result {
            Ok(value) => match serde_json::to_value(value) {
                Ok(value) => CallToolResult::structured(value),
                Err(error) => CallToolResult::structured_error(json!({
                    "code": "SERIALIZATION_ERROR",
                    "message": error.to_string(),
                })),
            },
            Err(message) => CallToolResult::structured_error(json!({
                "code": "LOCAL_OPERATION_ERROR",
                "message": message,
            })),
        }
    }

    async fn run_cli(args: Vec<String>) -> Result<Value, String> {
        let executable = std::env::current_exe()
            .map_err(|error| format!("Failed to locate Hades Cut CLI: {error}"))?;
        let output = tokio::process::Command::new(executable)
            .arg("--json")
            .args(args)
            .output()
            .await
            .map_err(|error| format!("Failed to run Hades Cut CLI: {error}"))?;
        let stdout = String::from_utf8(output.stdout)
            .map_err(|error| format!("CLI returned invalid UTF-8: {error}"))?;
        let events = match serde_json::from_str::<Value>(&stdout) {
            Ok(value) => vec![value],
            Err(_) => stdout
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| {
                    serde_json::from_str::<Value>(line)
                        .map_err(|error| format!("CLI returned invalid JSON: {error}"))
                })
                .collect::<Result<Vec<_>, _>>()?,
        };
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(events
                .last()
                .and_then(|event| event.get("error"))
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| stderr.trim().to_string()));
        }
        match events.as_slice() {
            [event] => Ok(event.clone()),
            _ => Ok(json!({ "events": events })),
        }
    }
}

#[tool_router(router = tool_router)]
impl HadesLocalMcpServer {
    #[tool(
        name = "hades_targets",
        description = "List local screens, windows, cameras, or microphones available to Hades Cut",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn targets(&self, Parameters(input): Parameters<TargetsInput>) -> CallToolResult {
        let mut args = vec!["targets".to_string()];
        if let Some(kind) = input.kind {
            args.push(kind);
        }
        Self::result(Self::run_cli(args).await)
    }

    #[tool(
        name = "hades_record_start",
        description = "Start a detached local Studio recording and return its recording ID and .cap project path",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn record_start(
        &self,
        Parameters(input): Parameters<RecordStartInput>,
    ) -> CallToolResult {
        let mut args = vec![
            "record".to_string(),
            "start".to_string(),
            "--detach".to_string(),
        ];
        for (flag, value) in [
            ("--screen", input.screen),
            ("--window", input.window),
            ("--camera", input.camera),
            ("--mic", input.mic),
            ("--path", input.path),
        ] {
            if let Some(value) = value {
                args.extend([flag.to_string(), value]);
            }
        }
        if input.system_audio {
            args.push("--system-audio".to_string());
        }
        if let Some(fps) = input.fps {
            args.extend(["--fps".to_string(), fps.to_string()]);
        }
        Self::result(Self::run_cli(args).await)
    }

    #[tool(
        name = "hades_record_stop",
        description = "Stop and finalize a detached local recording by recording ID",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn record_stop(&self, Parameters(input): Parameters<RecordStopInput>) -> CallToolResult {
        let mut args = vec![
            "record".to_string(),
            "stop".to_string(),
            "--id".to_string(),
            input.recording_id,
        ];
        if let Some(timeout) = input.timeout_seconds {
            args.extend(["--timeout".to_string(), timeout.to_string()]);
        }
        Self::result(Self::run_cli(args).await)
    }

    #[tool(
        name = "hades_export",
        description = "Render a local .cap project to MP4, MOV, or GIF and return progress plus the output path",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn export(&self, Parameters(input): Parameters<ExportInput>) -> CallToolResult {
        let mut args = vec![
            "export".to_string(),
            input.project_path,
            "--output".to_string(),
            input.output_path,
        ];
        for (flag, value) in [
            ("--format", input.format),
            ("--resolution", input.resolution),
            ("--quality", input.quality),
        ] {
            if let Some(value) = value {
                args.extend([flag.to_string(), value]);
            }
        }
        if let Some(fps) = input.fps {
            args.extend(["--fps".to_string(), fps.to_string()]);
        }
        Self::result(Self::run_cli(args).await)
    }

    #[tool(
        name = "hades_project_get",
        description = "Read a local editable .cap project configuration and its optimistic-concurrency revision",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn project_get(&self, Parameters(input): Parameters<ProjectInput>) -> CallToolResult {
        Self::result(crate::project::config_snapshot(PathBuf::from(
            input.project_path,
        )))
    }

    #[tool(
        name = "hades_project_patch",
        description = "Apply an RFC 7396 merge patch to a local .cap edit, with dry-run validation, revision checking, and automatic reversible history",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn project_patch(&self, Parameters(input): Parameters<PatchInput>) -> CallToolResult {
        Self::result(crate::project::apply_config_patch(
            PathBuf::from(input.project_path),
            input.patch,
            input.expected_revision.as_deref(),
            input.dry_run,
        ))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for HadesLocalMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder().enable_tools().build(),
        )
        .with_server_info(Implementation::new(
            "hades-cut-local",
            env!("CARGO_PKG_VERSION"),
        ))
        .with_instructions(
            "Read the project first, pass its revision as expectedRevision, dry-run nontrivial edits, then apply. Every applied edit stores the previous configuration under .hades/history.",
        )
    }
}

pub async fn serve() -> Result<(), String> {
    let service = HadesLocalMcpServer::new()
        .serve(rmcp::transport::stdio())
        .await
        .map_err(|error| error.to_string())?;
    service.waiting().await.map_err(|error| error.to_string())?;
    Ok(())
}
