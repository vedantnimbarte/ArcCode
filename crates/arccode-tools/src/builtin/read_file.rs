use crate::{Tool, ToolCtx};
use arccode_core::{ToolOutcome, ToolSpec};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

pub struct ReadFile;

#[derive(Debug, Deserialize)]
struct Args {
    path: String,
    #[serde(default)]
    offset: Option<u32>,
    #[serde(default)]
    limit: Option<u32>,
}

#[async_trait]
impl Tool for ReadFile {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "read_file".into(),
            description: "Read a UTF-8 text file from disk. Optional 1-based `offset` and `limit` \
                          restrict the returned line range. Refuses files that look binary."
                .into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute or cwd-relative path." },
                    "offset": { "type": "integer", "minimum": 1, "description": "1-based starting line." },
                    "limit": { "type": "integer", "minimum": 1, "description": "Max lines to return." }
                },
                "required": ["path"],
                "additionalProperties": false
            }),
        }
    }

    async fn run(&self, args: Value, ctx: &ToolCtx) -> ToolOutcome {
        let args: Args = match serde_json::from_value(args) {
            Ok(a) => a,
            Err(e) => return ToolOutcome::err(format!("invalid args: {e}")),
        };
        let path = ctx.resolve(&args.path);
        let bytes = match tokio::fs::read(&path).await {
            Ok(b) => b,
            Err(e) => return ToolOutcome::err(format!("read {}: {e}", path.display())),
        };
        if looks_binary(&bytes) {
            return ToolOutcome::err(format!("refusing to read binary file {}", path.display()));
        }
        let text = String::from_utf8_lossy(&bytes).into_owned();
        let lines: Vec<&str> = text.lines().collect();
        let start = args
            .offset
            .map(|n| n.saturating_sub(1) as usize)
            .unwrap_or(0);
        let end = args
            .limit
            .map(|n| (start + n as usize).min(lines.len()))
            .unwrap_or(lines.len());
        if start >= lines.len() {
            return ToolOutcome::ok(String::new());
        }
        let slice = &lines[start..end];
        ToolOutcome::ok(slice.join("\n"))
    }
}

fn looks_binary(bytes: &[u8]) -> bool {
    let head = &bytes[..bytes.len().min(8192)];
    head.contains(&0)
}
