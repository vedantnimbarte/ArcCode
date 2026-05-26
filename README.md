# Arc-Code

`arccode` is a multi-provider, terminal-first coding agent written in Rust. It
runs as a TUI for interactive sessions and as a headless one-shot
(`--print "prompt"`) for scripting, talks to nine LLM providers behind a single
streaming interface, and ships a built-in tool layer for reading, searching,
and editing the project tree.

It is positioned as an open, provider-agnostic alternative to Claude Code,
Cursor, and Aider — with native support for Anthropic, OpenAI, Google Gemini,
OpenRouter, LiteLLM, LM Studio, vLLM, and Ollama, plus a planned MCP host.

---

## Highlights

- **Nine providers, one shape.** Anthropic is the reference implementation
  (streaming, tool use, explicit prompt caching). A single OpenAI-compatible
  adapter covers OpenAI, OpenRouter, LM Studio, vLLM, LiteLLM, and Ollama.
  Gemini has its own adapter. All speak the same `arccode_core::Message`
  contract.
- **Two surfaces.** A `ratatui`-based TUI for interactive coding and a
  headless `--print` mode that emits either text or newline-delimited JSON
  events — ready to pipe into other tools or CI.
- **Built-in tool layer.** File read/write/edit, glob, grep, directory
  listing, and shell execution, each gated by the active permission mode.
- **Live model swap.** Change provider/model mid-session with `/model
  <provider>/<id>` from inside the TUI — no restart, history preserved.
- **Token-aware pipeline.** Per-tool output budgets with head/tail
  truncation, history token estimation, and a compaction trigger
  (`compact_at_tokens`) so long sessions stay inside the active model's
  context window.
- **Layered configuration.** Defaults → global `~/.arccode/config.toml` →
  project `.arccode/config.toml` → `ARCCODE_*` env vars → CLI flags. TOML
  sub-tables merge instead of clobbering.
- **Permission modes.** `read-only` (default), `auto-edit` (writes/shell
  inside the project tree auto-allowed, denylist still prompts), and `yolo`
  (no prompts; per-session only, never persisted).

---

## Workspace layout

This is a Cargo workspace. Each crate has a narrow, well-defined responsibility.

| Crate                | Role                                                                                                  |
| -------------------- | ----------------------------------------------------------------------------------------------------- |
| `arccode-cli`        | Binary entry point. Argument parsing, logging, runtime wiring, headless mode.                          |
| `arccode-core`       | Provider-agnostic types: `Message`, `ContentBlock`, `CompletionRequest`, `Provider`, agent loop, streaming events, tool dispatch, token estimation. |
| `arccode-config`     | TOML config loading, layered merge, env-var resolution, permission model.                              |
| `arccode-providers`  | Concrete `Provider` implementations: Anthropic, Gemini, OpenAI-compatible (six variants).              |
| `arccode-tools`      | Built-in tool implementations (`read_file`, `write_file`, `edit_file`, `glob`, `grep`, `list_dir`, `run_shell`) and the `ToolRegistry`. |
| `arccode-tui`        | `ratatui` interactive surface: composer, transcript, status bar, slash commands.                       |
| `arccode-session`    | Session persistence scaffolding (M3).                                                                  |
| `arccode-rag`        | Repo index / embeddings scaffolding (M4).                                                              |
| `arccode-mcp`        | MCP host scaffolding (M3).                                                                             |

---

## Supported providers

| Provider           | Adapter                  | Notes                                                                    |
| ------------------ | ------------------------ | ------------------------------------------------------------------------ |
| Anthropic          | `AnthropicProvider`      | Reference impl. Streaming, tool use, explicit `cache_control` breakpoints. |
| OpenAI             | `OpenAiCompatProvider`   | Variant: `OpenAi`.                                                        |
| OpenRouter         | `OpenAiCompatProvider`   | Variant: `OpenRouter`. Aggregator — pass `provider/model` as model id.    |
| LiteLLM            | `OpenAiCompatProvider`   | Variant: `LiteLLM`. Self-hosted gateway.                                  |
| LM Studio          | `OpenAiCompatProvider`   | Variant: `LmStudio`. Local OpenAI-compatible shim.                        |
| vLLM               | `OpenAiCompatProvider`   | Variant: `Vllm`. Self-hosted inference server.                            |
| Ollama             | `OpenAiCompatProvider`   | Variant: `Ollama`. Hits `/v1` shim on localhost:11434.                    |
| Google Gemini      | `GeminiProvider`         | Native adapter.                                                          |

---

## Installation

### Prerequisites

- Rust 1.80 or later (uses 2021 edition; pinned in `Cargo.toml`).
- A working C toolchain for some transitive crates.
- (Optional) An API key for the provider(s) you intend to use.

### Build from source

```bash
git clone git@github.com:vedantnimbarte/Arc-Code.git
cd Arc-Code
cargo build --release
```

The resulting binary is at `target/release/arccode` (or `arccode.exe` on
Windows).

To install onto your `PATH`:

```bash
cargo install --path crates/arccode-cli
```

---

## Quick start

### 1. Scaffold a config

```bash
arccode config init
```

This writes a starter `~/.arccode/config.toml` populated with entries for every
supported provider, each pointing at a `${ENV_VAR}` placeholder for the API
key.

### 2. Set an API key

Pick one of the supported providers and export its key. Anthropic is the
default:

```bash
export ANTHROPIC_API_KEY=sk-ant-...
# or
export OPENAI_API_KEY=sk-...
export OPENROUTER_API_KEY=...
export GOOGLE_API_KEY=...
```

For local providers (Ollama, LM Studio, vLLM) no key is needed — just point
the `base_url` at the running instance.

### 3. Use it

```bash
# Interactive TUI in the current project
arccode

# Headless one-shot
arccode --print "explain the agent loop in crates/arccode-core"

# Headless, streaming JSON events (newline-delimited)
arccode --print "list the public types in arccode-core" --json

# Pick a model for this session only
arccode --model anthropic/claude-opus-4-7
arccode --model openai/gpt-4.1
arccode --model gemini/gemini-2.5-pro
arccode --model openrouter/anthropic/claude-opus-4-7

# Loosen the permission model for this session
arccode --mode auto-edit
arccode --mode yolo            # no prompts; per-session only
```

### Inside the TUI

- Type a prompt and hit Enter to send.
- `/model <provider>/<model-id>` — swap the active model live.
- Tool calls render inline with their output (head/tail truncated per the
  active budget) and the token-usage strip updates after each turn.

---

## Configuration

`arccode` resolves configuration in this order (lowest to highest precedence):

1. Built-in defaults.
2. `~/.arccode/config.toml` (global).
3. `<project>/.arccode/config.toml` (project-local).
4. `ARCCODE_*` environment variables.
5. CLI flags.

TOML sub-tables are merged at the raw-TOML level, so an absent section in the
project file does **not** wipe out the global values for that section.

### Example `~/.arccode/config.toml`

```toml
default_provider = "anthropic"
permission_mode = "read-only"

[tokens]
compact_at_tokens = 120000
tool_output_max_lines = 400
prompt_cache = true

[router]
fast_model = "anthropic/claude-haiku-4-5-20251001"

[tui]
theme = "default"
show_token_usage = true

[providers.anthropic]
api_key = "${ANTHROPIC_API_KEY}"
model = "claude-opus-4-7"

[providers.openai]
api_key = "${OPENAI_API_KEY}"
model = "gpt-4.1"

[providers.gemini]
api_key = "${GOOGLE_API_KEY}"
model = "gemini-2.5-pro"

[providers.openrouter]
api_key = "${OPENROUTER_API_KEY}"
model = "anthropic/claude-opus-4-7"

[providers.ollama]
base_url = "http://localhost:11434/v1"
model = "llama3.1:8b"

[providers.lmstudio]
base_url = "http://localhost:1234/v1"
model = "local-model"

[providers.vllm]
base_url = "http://localhost:8000/v1"
model = "local-model"

[providers.litellm]
api_key = "${LITELLM_API_KEY}"
base_url = "http://localhost:4000/v1"
model = "anthropic/claude-opus-4-7"

[logging]
filter = "info,arccode=info"
file = true
```

### Environment variables

| Variable                            | Effect                                                              |
| ----------------------------------- | ------------------------------------------------------------------- |
| `ARCCODE_MODEL`                     | Overrides `default_model`. Same syntax as `--model`.                |
| `ARCCODE_PROVIDER`                  | Overrides `default_provider`.                                       |
| `ARCCODE_PERMISSION_MODE`           | `read-only` \| `auto-edit` \| `yolo`.                               |
| `ARCCODE_LOG`                       | `tracing-subscriber` env-filter directive.                          |
| `ARCCODE_<PROVIDER>_API_KEY`        | Sets `providers.<provider>.api_key`.                                |
| `ARCCODE_<PROVIDER>_BASE_URL`       | Sets `providers.<provider>.base_url`.                               |
| `ARCCODE_<PROVIDER>_MODEL`          | Sets `providers.<provider>.model`.                                  |

Any string field of the form `${ENV_VAR}` (e.g. `api_key = "${ANTHROPIC_API_KEY}"`)
is resolved against the environment at load time.

### Permission modes

| Mode         | Reads / Search | Writes inside project | Shell                       | Out-of-tree paths |
| ------------ | -------------- | --------------------- | --------------------------- | ----------------- |
| `read-only`  | allowed        | prompts               | prompts                     | prompts           |
| `auto-edit`  | allowed        | auto-allowed          | auto-allowed except denylist | prompts           |
| `yolo`       | allowed        | auto-allowed          | auto-allowed                | auto-allowed      |

`yolo` is per-session only — never persisted to config.

---

## CLI reference

```text
arccode [OPTIONS] [COMMAND]
```

**Top-level flags**

| Flag                     | Description                                                                 |
| ------------------------ | --------------------------------------------------------------------------- |
| `--mode <MODE>`          | `read-only` \| `auto-edit` \| `yolo`.                                       |
| `--model <MODEL>`        | Model id, optionally prefixed: `anthropic/claude-opus-4-7`. Env: `ARCCODE_MODEL`. |
| `--print <PROMPT>`       | Run a single prompt and exit (non-interactive).                              |
| `--json`                 | Emit newline-delimited JSON events instead of text. Use with `--print`.      |
| `-v`, `-vv`              | Increase log verbosity.                                                      |
| `--quiet`                | Suppress non-error stderr output.                                            |
| `--version`              | Print version and exit.                                                      |
| `--help`                 | Print help.                                                                  |

**Subcommands**

| Command              | Description                                            |
| -------------------- | ------------------------------------------------------ |
| `config init`        | Write a starter `~/.arccode/config.toml`. `--force` to overwrite. |
| `config show`        | Print the merged effective configuration. `--json` for JSON output. |
| `config paths`       | Print the resolved global and project config paths.    |

Running `arccode` with no subcommand launches the TUI against the resolved
provider and model.

---

## Built-in tools

Each tool runs through the registry, which receives a `ToolCtx` carrying the
active permission mode, current working directory, and project root. Tools
decide whether to act, prompt, or refuse based on that context.

| Tool          | Purpose                                                                 |
| ------------- | ----------------------------------------------------------------------- |
| `read_file`   | Read a file by absolute path. Returns content with line numbers.        |
| `write_file`  | Create or overwrite a file.                                             |
| `edit_file`   | Exact string replacement inside an existing file.                       |
| `glob_tool`   | Find files by glob pattern (e.g. `**/*.rs`).                            |
| `grep_tool`   | Content search via ripgrep semantics.                                   |
| `list_dir`    | List a directory.                                                       |
| `run_shell`   | Execute a shell command. Subject to the permission denylist.            |

Tool output is bounded by `tokens.tool_output_max_lines`; anything longer is
head/tail truncated before being fed back into the model.

---

## Roadmap

The project is being built milestone by milestone:

- **M0** — Workspace scaffold, CLI surface, layered config loader. *(shipped)*
- **M1** — Headless and TUI agent loop against Anthropic with built-in tools. *(shipped)*
- **M2** — Six more providers, token pipeline, live `/model` swap. *(shipped — current `main`)*
- **M3** — Session persistence (`arccode-session`) and MCP host (`arccode-mcp`).
- **M4** — Repo index / RAG (`arccode-rag`).

---

## Development

### Build & test

```bash
cargo build              # debug build
cargo build --release    # release build
cargo test               # full test suite
cargo fmt                # formatting (rustfmt.toml is project-pinned)
cargo clippy             # lints
```

### Run the TUI from source

```bash
cargo run -- --mode auto-edit
```

### Run a headless one-shot from source

```bash
cargo run -- --print "what does crates/arccode-core do?"
```

### Logs

By default, logs are written to `~/.arccode/logs/`. Override with
`ARCCODE_LOG=debug` or via the `[logging]` block in config.

---

## Project layout on disk

```
.
├── Cargo.toml              # workspace manifest
├── Cargo.lock
├── rustfmt.toml
├── crates/
│   ├── arccode-cli/        # binary entry point
│   ├── arccode-config/     # config loading + merge
│   ├── arccode-core/       # provider-agnostic types + agent loop
│   ├── arccode-mcp/        # MCP host (M3)
│   ├── arccode-providers/  # Anthropic, Gemini, OpenAI-compat
│   ├── arccode-rag/        # repo index (M4)
│   ├── arccode-session/    # session persistence (M3)
│   ├── arccode-tools/      # built-in tools + registry
│   └── arccode-tui/        # ratatui surface
└── target/                 # build output (gitignored)
```

On the user's machine:

```
~/.arccode/
├── config.toml             # global config
├── credentials.toml        # provider credentials (optional)
└── logs/                   # tracing output
```

```
<project-root>/.arccode/
├── config.toml             # project-local overrides
└── (future) sessions/, index/
```

---

## License

Dual-licensed under either:

- MIT License
- Apache License, Version 2.0

at your option.

---

## Contributing

Issues and pull requests are welcome. Before opening a PR:

1. `cargo fmt` and `cargo clippy` cleanly.
2. `cargo test` passes.
3. New behavior is covered by a test where reasonable.
