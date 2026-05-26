//! Wires config into a concrete `Provider` + `ToolRegistry` + `AgentLoop`.
//!
//! Keeps the per-provider plumbing in one place so command handlers
//! (headless --print, --json, future TUI) can just ask `Runtime::build(...)`.

use anyhow::{anyhow, Context, Result};
use arccode_config::{Config, PermissionMode, ProjectPaths};
use arccode_core::{AgentConfig, AgentLoop, Provider};
use arccode_providers::AnthropicProvider;
use arccode_tools::{ToolCtx, ToolRegistry};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Selection {
    pub provider_id: String,
    pub model: String,
}

/// Parse a model string. Either `provider/model` (preferred) or bare
/// `model` (uses `default_provider` from config).
pub fn resolve_selection(cfg: &Config, model_flag: Option<&str>) -> Result<Selection> {
    let raw = model_flag
        .map(str::to_string)
        .or_else(|| cfg.default_model.clone())
        .or_else(|| {
            cfg.default_provider
                .as_ref()
                .and_then(|p| cfg.providers.get(p).and_then(|pc| pc.model.clone()))
        });

    let (provider_id, model) = match raw {
        Some(s) if s.contains('/') => {
            let (p, m) = s.split_once('/').unwrap();
            (p.to_string(), m.to_string())
        }
        Some(s) => {
            let provider = cfg.default_provider.clone().ok_or_else(|| {
                anyhow!("no default_provider configured; pass --model provider/model")
            })?;
            (provider, s)
        }
        None => {
            let provider = cfg.default_provider.clone().ok_or_else(|| {
                anyhow!("no default_provider configured; run `arccode config init`")
            })?;
            let model = cfg
                .providers
                .get(&provider)
                .and_then(|pc| pc.model.clone())
                .ok_or_else(|| anyhow!("no model configured for provider {provider}"))?;
            (provider, model)
        }
    };

    Ok(Selection { provider_id, model })
}

pub fn build_provider(cfg: &Config, provider_id: &str) -> Result<Arc<dyn Provider>> {
    let pc = cfg
        .providers
        .get(provider_id)
        .with_context(|| format!("no [providers.{provider_id}] section in config"))?;

    match provider_id {
        "anthropic" => {
            let key = resolve_api_key(pc.api_key.as_deref(), "ANTHROPIC_API_KEY")?;
            let mut p = AnthropicProvider::new(key)?;
            if let Some(url) = &pc.base_url {
                p = p.with_base_url(url);
            }
            Ok(Arc::new(p))
        }
        other => Err(anyhow!(
            "provider '{other}' is not yet implemented (M1 ships Anthropic; OpenAI/Gemini/Ollama land in M2)"
        )),
    }
}

fn resolve_api_key(from_config: Option<&str>, env_name: &str) -> Result<String> {
    if let Some(s) = from_config {
        if !s.is_empty() && !looks_like_placeholder(s) {
            return Ok(s.to_string());
        }
    }
    std::env::var(env_name).map_err(|_| {
        anyhow!("missing API key: set [providers.*].api_key in config or {env_name} in env")
    })
}

fn looks_like_placeholder(s: &str) -> bool {
    s.trim().starts_with("${") && s.trim().ends_with('}')
}

pub fn build_registry(cfg: &Config, mode: PermissionMode) -> Result<ToolRegistry> {
    let cwd = std::env::current_dir()?;
    let paths = ProjectPaths::discover(&cwd);
    let ctx = ToolCtx::new(mode, cwd, paths.root.clone());
    let mut reg = ToolRegistry::new(ctx);
    let _ = cfg; // reserved for future per-tool config knobs
    reg = reg.with_builtins();
    Ok(reg)
}

pub fn build_agent(cfg: &Config, selection: &Selection, mode: PermissionMode) -> Result<AgentLoop> {
    let provider = build_provider(cfg, &selection.provider_id)?;
    let registry = build_registry(cfg, mode)?;
    let system = build_system_prompt(mode);
    let agent_cfg = AgentConfig {
        model: selection.model.clone(),
        system: Some(system),
        ..Default::default()
    };
    Ok(AgentLoop::new(provider, Arc::new(registry), agent_cfg))
}

pub fn build_system_prompt(mode: PermissionMode) -> String {
    let cwd = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "<unknown>".to_string());
    format!(
        "You are arccode, a terminal coding agent. You help the user inspect, \
         edit, and run code from the command line.\n\
         \n\
         Available tools: read_file, write_file, edit_file, run_shell, list_dir, glob, grep.\n\
         \n\
         Style rules:\n\
         - Prefer narrow, targeted reads (`read_file` with `offset`/`limit`) over reading whole files.\n\
         - Edit with `edit_file` and include enough surrounding context that `old_string` is unique.\n\
         - Verify your edits when reasonable (compile, run a test, re-read the diff).\n\
         - Be concise. Don't restate what the diff already shows.\n\
         \n\
         Environment:\n\
         - Working directory: {cwd}\n\
         - Permission mode: {mode}\n"
    )
}
