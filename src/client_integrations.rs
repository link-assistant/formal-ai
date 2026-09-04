use std::error::Error;
use std::fs;
use std::io::{IsTerminal as _, Read as _};
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use clap::{Args as ClapArgs, ValueEnum};

use crate::DEFAULT_MODEL;
use crate::context_capacity::ContextCapacity;
use crate::seed::{
    ClientIntegration, ModeArgPosition, ModelArgPosition,
    client_integrations as seed_client_integrations,
};

mod caller_args;
mod command;
mod completion;
mod completion_learning;
mod global_config;
mod global_verify;
mod server;
mod session_files;
mod tool_args;
mod url;
use caller_args::CallerArgs;
use command::{contains_model_arg, resolve_integration_command};
use completion::{AuthoringRun, CompletionInvocation, require_completed, run_to_completion};
use global_config::{
    render_json_settings, render_toml_settings, undo_global_config, write_global_config,
};
use server::maybe_start_server;
use session_files::{
    TempConfigDir, newest_changed_session_file, print_session_files, session_file_snapshot,
    user_home_dir,
};
pub use tool_args::delimit_tool_args;
use url::{base_url_with_port, join_url_path};

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:8080";
const EMPTY_BACKUP_SENTINEL: &str = "# formal-ai-empty-config-backup-v1\n";
const RENDERED_PLACEHOLDER: &str = concat!("{", "rendered", "}");
const ERROR_PLACEHOLDER: &str = concat!("{", "error", "}");

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ClientProtocol {
    Openai,
    Gemini,
    Vertex,
    Anthropic,
}

impl ClientProtocol {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Openai => "openai",
            Self::Gemini => "gemini",
            Self::Vertex => "vertex",
            Self::Anthropic => "anthropic",
        }
    }
}

#[derive(Debug, Clone, ClapArgs)]
#[command(trailing_var_arg = true)]
#[allow(clippy::struct_excessive_bools)]
pub struct WithFormalAiArgs {
    /// Permanently configure the selected tool instead of running it once.
    #[arg(
        short = 'g',
        long = "global",
        alias = "globally",
        default_value_t = false
    )]
    pub global: bool,

    /// Restore the backup created by a previous global configuration.
    #[arg(long, default_value_t = false)]
    pub undo: bool,

    /// Configure or undo every supported tool from the seed registry.
    #[arg(long, default_value_t = false)]
    pub all: bool,

    /// After `--global`, start the tool once non-interactively and fail on an
    /// auth refusal instead of reporting success.
    #[arg(long, default_value_t = false)]
    pub verify: bool,

    /// Formal AI server root URL. Protocol-specific paths are added from seed data.
    #[arg(long, default_value = DEFAULT_BASE_URL)]
    pub base_url: String,

    /// Override the port in --base-url.
    #[arg(long)]
    pub port: Option<u16>,

    /// Explicitly start `formal-ai serve` when the target loopback port is not listening.
    #[arg(long, default_value_t = false)]
    pub start_server: bool,

    /// Do not auto-start a temporary server when the target is not listening.
    #[arg(long, default_value_t = false, conflicts_with = "start_server")]
    pub no_start_server: bool,

    /// Keep the wrapped tool's normal summarization/compaction behavior.
    #[arg(long, alias = "keep-summarization", default_value_t = false)]
    pub summarize: bool,

    /// Force the wrapped CLI to stay in its interactive mode.
    #[arg(long, default_value_t = false, conflicts_with = "non_interactive")]
    pub interactive: bool,

    /// Force one-shot/headless output (aliases: --print and --one-shot).
    #[arg(long, alias = "print", alias = "one-shot", default_value_t = false)]
    pub non_interactive: bool,

    /// Apply the seed-defined permission-gated orchestration invocation overlay.
    #[arg(long, default_value_t = false, hide = true)]
    pub orchestration: bool,

    /// Stable client home used only by the permission-gated orchestrator.
    #[arg(long, hide = true, requires = "orchestration")]
    pub orchestration_home: Option<PathBuf>,

    /// Native client session id resumed only by the orchestrator.
    #[arg(long, hide = true, requires = "orchestration")]
    pub orchestration_resume: Option<String>,

    /// Protocol namespace to use for tools that support more than one protocol.
    #[arg(long, value_enum)]
    pub protocol: Option<ClientProtocol>,

    /// Model alias to configure for the target tool.
    #[arg(long, default_value = DEFAULT_MODEL)]
    pub model: String,

    /// External client target from the seed registry (for example codex or opencode-vscode).
    #[arg(value_name = "TOOL")]
    pub tool: Option<String>,

    /// Arguments passed through to the external CLI.
    #[arg(
        value_name = "ARGS",
        allow_hyphen_values = true,
        trailing_var_arg = true
    )]
    pub tool_args: Vec<String>,
}

#[derive(Debug, Clone)]
struct RenderContext {
    protocol: String,
    base_url: String,
    endpoint_base_url: String,
    openai_endpoint_base_url: String,
    anthropic_endpoint_base_url: String,
    provider_id: String,
    model: String,
    model_selector: String,
    api_key_env: String,
    api_key: String,
    protocol_base_env: String,
    google_auth_type: String,
    model_catalog_path: String,
    working_directory: String,
}

#[derive(Debug, Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
struct InvocationOptions {
    keep_summarization: bool,
    force_interactive: bool,
    force_non_interactive: bool,
    orchestration: bool,
    /// Whether standard input is a terminal. A piped invocation is headless
    /// even when the caller passed no prompt text.
    stdin_is_terminal: bool,
}

pub fn run_with_formal_ai(args: &WithFormalAiArgs) -> Result<(), Box<dyn Error>> {
    let integrations = seed_client_integrations();
    if args.global || args.undo {
        let selected = select_integrations(args, &integrations)?;
        for integration in selected {
            if args.undo {
                undo_global_config(integration, args)?;
            } else {
                write_global_config(integration, args)?;
            }
        }
        return Ok(());
    }

    if args.all {
        return Err("--all is only valid with --global or --undo".into());
    }
    if args.verify {
        return Err("--verify is only valid with --global".into());
    }
    let tool = args
        .tool
        .as_deref()
        .ok_or("missing tool; pass one of the supported tool names")?;
    let integration = find_integration(tool, &integrations)?;
    let context = render_context(integration, args)?;
    let server = if args.start_server || !args.no_start_server {
        let server = maybe_start_server(&context.base_url, args.port)?;
        if server.is_some() {
            eprintln!(
                "formal-ai: started a temporary server in agent mode (tool and shell execution enabled)"
            );
        }
        server
    } else {
        None
    };
    let stdin_is_terminal = std::io::stdin().is_terminal();
    let mut caller = CallerArgs::parse(&args.tool_args, integration, &integrations);
    if !caller.has_text() && !stdin_is_terminal && !args.interactive {
        // A piped prompt becomes the same structured prompt an argument would
        // have produced, so every client renders it in its own vocabulary
        // (`codex exec <prompt>`, `claude --print <prompt>`) instead of being
        // handed a mode flag with nothing to say.
        if let Some(piped) = read_piped_prompt()? {
            caller = caller.with_prompt(&piped);
        }
    }
    run_ephemeral(
        integration,
        &caller,
        &context,
        InvocationOptions {
            keep_summarization: args.summarize,
            force_interactive: args.interactive,
            force_non_interactive: args.non_interactive,
            orchestration: args.orchestration,
            stdin_is_terminal,
        },
        server.as_ref().map(|server| server.output_log.as_path()),
        args.orchestration_home.as_deref(),
        args.orchestration_resume.as_deref(),
    )
}

/// Read a prompt piped into the wrapper, or `None` when stdin carries nothing.
fn read_piped_prompt() -> Result<Option<String>, Box<dyn Error>> {
    let mut piped = String::new();
    std::io::stdin().read_to_string(&mut piped)?;
    let piped = piped.trim();
    if piped.is_empty() {
        return Ok(None);
    }
    Ok(Some(piped.to_owned()))
}

fn select_integrations<'a>(
    args: &WithFormalAiArgs,
    integrations: &'a [ClientIntegration],
) -> Result<Vec<&'a ClientIntegration>, Box<dyn Error>> {
    if args.all {
        return Ok(integrations.iter().collect());
    }
    let tool = args
        .tool
        .as_deref()
        .ok_or("missing tool; pass a tool name or --all")?;
    Ok(vec![find_integration(tool, integrations)?])
}

fn find_integration<'a>(
    tool: &str,
    integrations: &'a [ClientIntegration],
) -> Result<&'a ClientIntegration, Box<dyn Error>> {
    integrations
        .iter()
        .find(|integration| {
            integration.id == tool || integration.aliases.iter().any(|alias| alias == tool)
        })
        .ok_or_else(|| {
            let supported = integrations
                .iter()
                .flat_map(|integration| {
                    std::iter::once(integration.id.as_str())
                        .chain(integration.aliases.iter().map(String::as_str))
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("unsupported tool `{tool}`; supported tools: {supported}").into()
        })
}

fn render_context(
    integration: &ClientIntegration,
    args: &WithFormalAiArgs,
) -> Result<RenderContext, Box<dyn Error>> {
    let protocol = args
        .protocol
        .map_or(integration.default_protocol.as_str(), |protocol| {
            protocol.as_str()
        });
    if !integration
        .supported_protocols
        .iter()
        .any(|supported| supported == protocol)
    {
        return Err(format!("{} does not support protocol `{protocol}`", integration.id).into());
    }
    let endpoint_path = integration
        .endpoint_path_for(protocol)
        .ok_or_else(|| format!("{} has no endpoint for {protocol}", integration.id))?;
    let base_url = base_url_with_port(&args.base_url, args.port);
    let endpoint_base_url = join_url_path(&base_url, endpoint_path);
    let openai_endpoint_base_url = integration
        .endpoint_path_for("openai")
        .map_or_else(String::new, |path| join_url_path(&base_url, path));
    let anthropic_endpoint_base_url = integration
        .endpoint_path_for("anthropic")
        .map_or_else(String::new, |path| join_url_path(&base_url, path));
    let api_key = std::env::var(&integration.api_key_env)
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| std::env::var("FORMAL_AI_API_KEY").ok())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| integration.api_key_default.clone());
    let protocol_base_env = match protocol {
        "vertex" => "GOOGLE_VERTEX_BASE_URL",
        "gemini" => "GOOGLE_GEMINI_BASE_URL",
        "openai" => "OPENAI_BASE_URL",
        "anthropic" => "ANTHROPIC_BASE_URL",
        _ => "FORMAL_AI_BASE_URL",
    }
    .to_string();
    let google_auth_type = match protocol {
        "vertex" => "vertex-ai",
        "gemini" => "gemini-api-key",
        _ => "",
    }
    .to_string();

    let mut context = RenderContext {
        protocol: protocol.to_string(),
        base_url,
        endpoint_base_url,
        openai_endpoint_base_url,
        anthropic_endpoint_base_url,
        provider_id: integration.provider_id.clone(),
        model: args.model.clone(),
        model_selector: String::new(),
        api_key_env: integration.api_key_env.clone(),
        api_key,
        protocol_base_env,
        google_auth_type,
        model_catalog_path: String::new(),
        working_directory: std::env::current_dir()?.to_string_lossy().into_owned(),
    };
    // An already-qualified selector (`provider/model`) is passed through: the
    // seed template only supplies the provider a bare alias is missing.
    context.model_selector = if integration.model_selector.is_empty() || context.model.contains('/')
    {
        context.model.clone()
    } else {
        render_template(&integration.model_selector, &context)
    };
    Ok(context)
}

fn run_ephemeral(
    integration: &ClientIntegration,
    caller: &CallerArgs,
    context: &RenderContext,
    mut options: InvocationOptions,
    temporary_server_log: Option<&Path>,
    orchestration_home: Option<&Path>,
    orchestration_resume: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    let invocation = &integration.invocation;
    let mut context = context.clone();
    let mut authoring = AuthoringRun::for_invocation(caller, &mut options)?;
    let mut temp_dirs = Vec::new();
    let mut session_home = None;
    let resolved_command = resolve_integration_command(integration);
    let mut command = Command::new(&resolved_command);
    for env in &invocation.env {
        command.env(
            render_template(&env.key, &context),
            render_template(&env.value, &context),
        );
    }
    if !invocation.config_json_settings.is_empty() {
        let config_json = render_json_settings(&invocation.config_json_settings, &context)?;
        if !invocation.config_content_env.is_empty() {
            command.env(
                render_template(&invocation.config_content_env, &context),
                &config_json,
            );
        }
        if !invocation.config_env.is_empty() || !invocation.config_dir_env.is_empty() {
            let temp = TempConfigDir::new(&integration.id)?;
            let config_path = temp.path.join(format!("{}.json", integration.id));
            fs::write(&config_path, config_json)?;
            if !invocation.config_env.is_empty() {
                command.env(&invocation.config_env, &config_path);
            }
            if !invocation.config_dir_env.is_empty() {
                command.env(&invocation.config_dir_env, &temp.path);
            }
            temp_dirs.push(temp);
        }
    }
    let temporary_home = if orchestration_home.is_none() && !invocation.temp_home_env.is_empty() {
        Some(TempConfigDir::new_home(&integration.id)?)
    } else {
        None
    };
    let scoped_home = orchestration_home
        .map(Path::to_path_buf)
        .or_else(|| temporary_home.as_ref().map(|temp| temp.path.clone()));
    if let Some(home) = scoped_home {
        fs::create_dir_all(&home)?;
        session_home = Some(home.clone());
        if !invocation.model_catalog_path.is_empty() {
            let relative_catalog_path = render_template(&invocation.model_catalog_path, &context);
            let catalog_path = temp_scoped_path(&home, &relative_catalog_path)?;
            context.model_catalog_path = catalog_path.display().to_string();
            write_file(&catalog_path, &codex_model_catalog(&context.model)?)?;
        }
        if !invocation.temp_home_config_path.is_empty() {
            let relative_config_path = render_template(&invocation.temp_home_config_path, &context);
            let config_path = temp_scoped_path(&home, &relative_config_path)?;
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let contents = if invocation.temp_home_toml_settings.is_empty() {
                render_json_settings(&invocation.temp_home_json_settings, &context)?
            } else {
                render_toml_settings(&invocation.temp_home_toml_settings, "", &context)?
            };
            fs::write(&config_path, contents)?;
        }
        let home_env = if invocation.temp_home_env.is_empty() {
            "HOME".to_string()
        } else {
            render_template(&invocation.temp_home_env, &context)
        };
        command.env(home_env, &home);
    }
    if let Some(temp) = temporary_home {
        temp_dirs.push(temp);
    }

    let session_root = if invocation.session_root.is_empty() {
        None
    } else {
        let base = session_home.map_or_else(user_home_dir, Ok)?;
        Some(base.join(&invocation.session_root))
    };
    let session_before = session_root
        .as_deref()
        .map(|root| session_file_snapshot(root, &invocation.session_file_suffix))
        .unwrap_or_default();

    let (status_success, status_label, completion_passed) =
        if let Some(authoring) = authoring.as_mut() {
            let outcome = run_to_completion(
                authoring,
                CompletionInvocation {
                    command: &command,
                    integration,
                    caller,
                    context: &context,
                    options,
                    session_root: session_root.as_deref(),
                    session_before: &session_before,
                    initial_resume: orchestration_resume,
                },
            )?;
            (
                outcome.status_success,
                outcome.status_label,
                Some(outcome.completion_passed),
            )
        } else {
            let final_args =
                build_invocation_args(integration, caller, &context, options, orchestration_resume);
            command.args(final_args);
            let status = command.status()?;
            (
                status.success(),
                status
                    .code()
                    .map_or_else(|| String::from("signal"), |code| code.to_string()),
                None,
            )
        };
    let session_file = session_root.as_deref().and_then(|root| {
        newest_changed_session_file(root, &invocation.session_file_suffix, &session_before)
    });
    let server_log = std::env::var_os("FORMAL_AI_PROXY_LOG")
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .map(|path| fs::canonicalize(&path).unwrap_or(path))
        .or_else(|| temporary_server_log.map(Path::to_path_buf));
    print_session_files(
        integration,
        session_file.as_deref(),
        server_log.as_deref(),
        options.orchestration,
    );
    let preserve_temp = session_file
        .as_deref()
        .is_some_and(|path| temp_dirs.iter().any(|temp| path.starts_with(&temp.path)));
    if preserve_temp {
        for temp in temp_dirs {
            temp.preserve();
        }
    } else {
        drop(temp_dirs);
    }
    require_completed(
        completion_passed,
        status_success,
        &status_label,
        &resolved_command,
    )
}

fn temp_scoped_path(root: &Path, relative: &str) -> Result<PathBuf, Box<dyn Error>> {
    let path = Path::new(relative);
    if path.as_os_str().is_empty() || path.is_absolute() {
        return Err(format!("temporary config path must be relative: {relative}").into());
    }
    for component in path.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(format!("temporary config path escapes its root: {relative}").into());
            }
        }
    }
    Ok(root.join(path))
}

/// Render one client invocation from the seed-declared contract plus the
/// caller's parsed request.
///
/// Every attempt goes through here, so a completion retry re-renders the same
/// option set with only the prompt substituted instead of rebuilding an argv
/// from the correction text alone.
fn build_invocation_args(
    integration: &ClientIntegration,
    caller: &CallerArgs,
    context: &RenderContext,
    options: InvocationOptions,
    orchestration_resume: Option<&str>,
) -> Vec<String> {
    let invocation = &integration.invocation;
    let mut args = invocation
        .prepend_args
        .iter()
        .chain(invocation.args.iter())
        .map(|arg| render_template(arg, context))
        .collect::<Vec<_>>();
    if options.orchestration {
        for (from, to) in &invocation.orchestration_arg_replacements {
            let rendered_from = render_template(from, context);
            if let Some(argument) = args.iter_mut().find(|argument| **argument == rendered_from) {
                *argument = render_template(to, context);
            }
        }
    }
    // A request with no text still runs headless when stdin is a pipe: the
    // prompt arrives on stdin, so injecting the client's interactive flags
    // would hang a scripted invocation.
    let interactive = options.force_interactive
        || (!options.force_non_interactive && !caller.has_text() && options.stdin_is_terminal);
    let mode_args: &[String] =
        if interactive && invocation.interactive_args_require_prompt && caller.prompt().is_none() {
            &[]
        } else if interactive {
            &invocation.interactive_args
        } else {
            &invocation.non_interactive_args
        };
    let rendered_mode_args = mode_args
        .iter()
        .map(|arg| render_template(arg, context))
        .collect::<Vec<_>>();
    if invocation.mode_arg_position == Some(ModeArgPosition::BeforeInvocation) {
        args.splice(0..0, rendered_mode_args.iter().cloned());
    }
    if !options.keep_summarization {
        args.extend(
            invocation
                .no_summarize_args
                .iter()
                .map(|arg| render_template(arg, context)),
        );
    }
    let caller_options = caller.options();
    let mut effective_user_args = Vec::new();
    if options.orchestration {
        effective_user_args.extend(overlay_args(
            &invocation.orchestration_args,
            caller,
            context,
        ));
    }
    if let Some(session_id) = orchestration_resume {
        effective_user_args.extend(
            invocation
                .resume_args
                .iter()
                .map(|argument| argument.replace(concat!("{", "session_id", "}"), session_id)),
        );
    }
    effective_user_args.extend(caller_options.iter().cloned());
    if invocation.mode_arg_position != Some(ModeArgPosition::BeforeInvocation) {
        effective_user_args.extend(rendered_mode_args);
    }
    if let Some(prompt) = caller.prompt() {
        effective_user_args.push(prompt.to_owned());
    }
    if invocation.model_arg.is_empty() || contains_model_arg(&caller_options) {
        args.extend(effective_user_args);
        return args;
    }

    let model_arg = render_template(&invocation.model_arg, context);
    let model_value = context.model_selector.clone();
    match invocation.model_arg_position {
        Some(ModelArgPosition::AfterFirstArg)
            if invocation.mode_arg_position == Some(ModeArgPosition::BeforeInvocation)
                && !args.is_empty() =>
        {
            args.insert(1, model_value);
            args.insert(1, model_arg);
            args.extend(effective_user_args);
        }
        Some(ModelArgPosition::AfterFirstArg) if !effective_user_args.is_empty() => {
            args.push(effective_user_args[0].clone());
            args.push(model_arg);
            args.push(model_value);
            args.extend(effective_user_args.iter().skip(1).cloned());
        }
        _ => {
            args.push(model_arg);
            args.push(model_value);
            args.extend(effective_user_args);
        }
    }
    args
}

/// Render a seed-declared argument overlay, dropping any flag the caller
/// already passed so the client never receives the same option twice.
fn overlay_args(overlay: &[String], caller: &CallerArgs, context: &RenderContext) -> Vec<String> {
    let mut rendered = Vec::new();
    let mut index = 0;
    while index < overlay.len() {
        let argument = &overlay[index];
        let is_flag = argument.len() > 1 && argument.starts_with('-');
        let value_count = usize::from(
            is_flag
                && overlay
                    .get(index + 1)
                    .is_some_and(|value| !value.starts_with('-')),
        );
        if is_flag && caller.contains_flag(argument) {
            index += 1 + value_count;
            continue;
        }
        for argument in &overlay[index..=index + value_count] {
            rendered.push(render_template(argument, context));
        }
        index += 1 + value_count;
    }
    rendered
}

fn render_template(template: &str, context: &RenderContext) -> String {
    template
        .replace("{provider_id}", &context.provider_id)
        .replace("{model}", &context.model)
        .replace("{model_selector}", &context.model_selector)
        .replace("{endpoint_base_url}", &context.endpoint_base_url)
        .replace(
            "{openai_endpoint_base_url}",
            &context.openai_endpoint_base_url,
        )
        .replace(
            "{anthropic_endpoint_base_url}",
            &context.anthropic_endpoint_base_url,
        )
        .replace("{base_url}", &context.base_url)
        .replace("{api_key_env}", &context.api_key_env)
        .replace("{api_key}", &context.api_key)
        .replace("{protocol_base_env}", &context.protocol_base_env)
        .replace("{google_auth_type}", &context.google_auth_type)
        .replace("{model_catalog_path}", &context.model_catalog_path)
        .replace("{working_directory}", &context.working_directory)
}

fn codex_model_catalog(model: &str) -> Result<String, Box<dyn Error>> {
    let context = ContextCapacity::current()?;
    let catalog = serde_json::json!({
        "models": [{
            "slug": model,
            "display_name": model,
            "description": "Formal AI symbolic model",
            "default_reasoning_level": "none",
            "supported_reasoning_levels": [],
            "shell_type": "shell_command",
            "visibility": "list",
            "supported_in_api": true,
            "priority": 0,
            "availability_nux": null,
            "upgrade": null,
            "base_instructions": "",
            "supports_reasoning_summaries": false,
            "supports_reasoning_summary_parameter": false,
            "default_reasoning_summary": "none",
            "support_verbosity": false,
            "default_verbosity": null,
            "apply_patch_tool_type": "freeform",
            "web_search_tool_type": "text",
            "truncation_policy": {"mode": "tokens", "limit": 8192},
            "supports_parallel_tool_calls": true,
            "context_window": context.context_window_tokens,
            "max_context_window": context.context_window_tokens,
            "context": context,
            "effective_context_window_percent": 100,
            "experimental_supported_tools": [],
            "input_modalities": ["text"]
        }]
    });
    Ok(format!("{}\n", serde_json::to_string_pretty(&catalog)?))
}

fn global_config_path(relative: &str) -> Result<PathBuf, Box<dyn Error>> {
    let path = Path::new(relative);
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .ok_or("HOME is not set; cannot resolve global config path")?;
    Ok(PathBuf::from(home).join(path))
}

fn backup_path(path: &Path, suffix: &str) -> PathBuf {
    let mut backup = path.as_os_str().to_os_string();
    backup.push(suffix);
    PathBuf::from(backup)
}

fn write_file(path: &Path, contents: &str) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)?;
    Ok(())
}
