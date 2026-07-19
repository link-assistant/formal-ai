use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::net::TcpStream;
use std::path::{Component, Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use clap::{Args as ClapArgs, ValueEnum};
use serde_json::Value;
use toml_edit::{value as toml_value, DocumentMut, Item, Table};

use crate::context_capacity::ContextCapacity;
use crate::seed::{
    client_integrations as seed_client_integrations, ClientIntegration, ConfigFormat,
    ModeArgPosition, ModelArgPosition,
};
use crate::DEFAULT_MODEL;

mod command;
mod session_files;
mod url;
use command::resolve_integration_command;
use session_files::{
    newest_changed_session_file, print_session_files, session_file_snapshot, user_home_dir,
    TempConfigDir,
};
use url::{base_url_with_port, join_url_path};

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:8080";
const EMPTY_BACKUP_SENTINEL: &str = "# formal-ai-empty-config-backup-v1\n";

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

    /// Protocol namespace to use for tools that support more than one protocol.
    #[arg(long, value_enum)]
    pub protocol: Option<ClientProtocol>,

    /// Model alias to configure for the target tool.
    #[arg(long, default_value = DEFAULT_MODEL)]
    pub model: String,

    /// External CLI: codex, opencode, agent, cursor, gemini, claude, qwen, grok, or aider.
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
}

struct ServerGuard {
    child: Child,
}

impl Drop for ServerGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
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
    let tool = args
        .tool
        .as_deref()
        .ok_or("missing tool; pass one of the supported tool names")?;
    let integration = find_integration(tool, &integrations)?;
    let context = render_context(integration, args)?;
    let _server = if args.start_server || !args.no_start_server {
        let server = maybe_start_server(&context.base_url, args.port)?;
        if server.is_some() {
            eprintln!("formal-ai: started a temporary server in agent mode (tool and shell execution enabled)");
        }
        server
    } else {
        None
    };
    run_ephemeral(
        integration,
        &args.tool_args,
        &context,
        args.summarize,
        args.interactive,
        args.non_interactive,
    )
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
    };
    context.model_selector = if integration.model_selector.is_empty() {
        context.model.clone()
    } else {
        render_template(&integration.model_selector, &context)
    };
    Ok(context)
}

fn run_ephemeral(
    integration: &ClientIntegration,
    user_args: &[String],
    context: &RenderContext,
    keep_summarization: bool,
    force_interactive: bool,
    force_non_interactive: bool,
) -> Result<(), Box<dyn Error>> {
    let invocation = &integration.invocation;
    let mut context = context.clone();
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
    if !invocation.temp_home_env.is_empty() {
        let temp = TempConfigDir::new(&format!("{}-home", integration.id))?;
        session_home = Some(temp.path.clone());
        if !invocation.model_catalog_path.is_empty() {
            let relative_catalog_path = render_template(&invocation.model_catalog_path, &context);
            let catalog_path = temp_scoped_path(&temp.path, &relative_catalog_path)?;
            context.model_catalog_path = catalog_path.display().to_string();
            write_file(&catalog_path, &codex_model_catalog(&context.model)?)?;
        }
        if !invocation.temp_home_config_path.is_empty() {
            let relative_config_path = render_template(&invocation.temp_home_config_path, &context);
            let config_path = temp_scoped_path(&temp.path, &relative_config_path)?;
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
        command.env(
            render_template(&invocation.temp_home_env, &context),
            &temp.path,
        );
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

    let final_args = build_invocation_args(
        integration,
        user_args,
        &context,
        keep_summarization,
        force_interactive,
        force_non_interactive,
    );
    command.args(final_args);
    let status = command.status()?;
    let session_file = session_root.as_deref().and_then(|root| {
        newest_changed_session_file(root, &invocation.session_file_suffix, &session_before)
    });
    let server_log = std::env::var_os("FORMAL_AI_PROXY_LOG")
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .map(|path| fs::canonicalize(&path).unwrap_or(path));
    print_session_files(integration, session_file.as_deref(), server_log.as_deref());
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
    if status.success() {
        return Ok(());
    }
    Err(format!(
        "{} exited with status {}",
        resolved_command.display(),
        status
            .code()
            .map_or_else(|| String::from("signal"), |code| code.to_string())
    )
    .into())
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

fn build_invocation_args(
    integration: &ClientIntegration,
    user_args: &[String],
    context: &RenderContext,
    keep_summarization: bool,
    force_interactive: bool,
    force_non_interactive: bool,
) -> Vec<String> {
    let invocation = &integration.invocation;
    let mut args = invocation
        .prepend_args
        .iter()
        .chain(invocation.args.iter())
        .map(|arg| render_template(arg, context))
        .collect::<Vec<_>>();
    let interactive = force_interactive || (!force_non_interactive && user_args.is_empty());
    let mode_args: &[String] =
        if interactive && invocation.interactive_args_require_prompt && user_args.is_empty() {
            &[]
        } else if interactive {
            &invocation.interactive_args
        } else {
            &invocation.non_interactive_args
        };
    let rendered_mode_args = if mode_args
        .iter()
        .any(|mode_arg| user_args.contains(mode_arg))
    {
        Vec::new()
    } else {
        mode_args
            .iter()
            .map(|arg| render_template(arg, context))
            .collect::<Vec<_>>()
    };
    if invocation.mode_arg_position == Some(ModeArgPosition::BeforeInvocation) {
        args.splice(0..0, rendered_mode_args.iter().cloned());
    }
    if !keep_summarization {
        args.extend(
            invocation
                .no_summarize_args
                .iter()
                .map(|arg| render_template(arg, context)),
        );
    }
    let mut effective_user_args = Vec::new();
    if invocation.mode_arg_position != Some(ModeArgPosition::BeforeInvocation) {
        effective_user_args.extend(rendered_mode_args);
    }
    effective_user_args.extend(user_args.iter().cloned());
    if invocation.model_arg.is_empty() || contains_model_arg(user_args) {
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

fn contains_model_arg(args: &[String]) -> bool {
    args.iter()
        .any(|arg| matches!(arg.as_str(), "-m" | "--model") || arg.starts_with("--model="))
}

fn write_global_config(
    integration: &ClientIntegration,
    args: &WithFormalAiArgs,
) -> Result<(), Box<dyn Error>> {
    let mut context = render_context(integration, args)?;
    let global_config = integration.global_config_for(&context.protocol);
    if !global_config.model_catalog_path.is_empty() {
        let catalog_path = global_config_path(&global_config.model_catalog_path)?;
        let catalog_backup = backup_path(&catalog_path, &global_config.backup_suffix);
        ensure_backup(&catalog_path, &catalog_backup)?;
        context.model_catalog_path = catalog_path.display().to_string();
        write_file(&catalog_path, &codex_model_catalog(&context.model)?)?;
    }
    let path = global_config_path(&global_config.path)?;
    let backup_path = backup_path(&path, &global_config.backup_suffix);
    ensure_backup(&path, &backup_path)?;
    let existing = fs::read_to_string(&path).unwrap_or_default();
    let next = match global_config.format {
        ConfigFormat::Toml => {
            render_toml_settings(&global_config.toml_settings, &existing, &context)?
        }
        ConfigFormat::Json => merge_json_config(global_config, &existing, &context)?,
        ConfigFormat::ShellEnv => {
            merge_shell_env_config(&integration.id, global_config, &existing, &context)
        }
    };
    if next == existing {
        println!(
            "{} already configured at {}",
            integration.id,
            path.display()
        );
    } else {
        write_file(&path, &next)?;
        println!("configured {} at {}", integration.id, path.display());
    }
    Ok(())
}

fn undo_global_config(
    integration: &ClientIntegration,
    args: &WithFormalAiArgs,
) -> Result<(), Box<dyn Error>> {
    let context = render_context(integration, args)?;
    let global_config = integration.global_config_for(&context.protocol);
    let path = global_config_path(&global_config.path)?;
    let config_backup_path = backup_path(&path, &global_config.backup_suffix);
    let mut restored = if config_backup_path.exists() {
        restore_backup(&path, &config_backup_path)?;
        true
    } else {
        false
    };
    if !global_config.model_catalog_path.is_empty() {
        let catalog_path = global_config_path(&global_config.model_catalog_path)?;
        let catalog_backup_path = backup_path(&catalog_path, &global_config.backup_suffix);
        if catalog_backup_path.exists() {
            restore_backup(&catalog_path, &catalog_backup_path)?;
            restored = true;
        }
    }
    if !restored {
        println!(
            "no formal-ai backup for {} at {}",
            integration.id,
            path.display()
        );
        return Ok(());
    }
    println!(
        "restored {} from {}",
        integration.id,
        config_backup_path.display()
    );
    Ok(())
}

fn restore_backup(path: &Path, backup_path: &Path) -> Result<(), Box<dyn Error>> {
    if !backup_path.exists() {
        return Ok(());
    }
    let backup = fs::read_to_string(backup_path)?;
    if backup == EMPTY_BACKUP_SENTINEL {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    } else {
        write_file(path, &backup)?;
    }
    fs::remove_file(backup_path)?;
    Ok(())
}

fn ensure_backup(path: &Path, backup_path: &Path) -> Result<(), Box<dyn Error>> {
    if backup_path.exists() {
        return Ok(());
    }
    if let Some(parent) = backup_path.parent() {
        fs::create_dir_all(parent)?;
    }
    if path.exists() {
        fs::copy(path, backup_path)?;
    } else {
        fs::write(backup_path, EMPTY_BACKUP_SENTINEL)?;
    }
    Ok(())
}

fn render_toml_settings(
    settings: &[(String, String)],
    existing: &str,
    context: &RenderContext,
) -> Result<String, Box<dyn Error>> {
    let mut document = if existing.trim().is_empty() {
        DocumentMut::new()
    } else {
        existing.parse::<DocumentMut>()?
    };
    for (path, value) in settings {
        set_toml_string(
            document.as_table_mut(),
            &render_template(path, context),
            &render_template(value, context),
        )?;
    }
    Ok(ensure_trailing_newline(document.to_string()))
}

fn set_toml_string(
    table: &mut Table,
    dotted_path: &str,
    value: &str,
) -> Result<(), Box<dyn Error>> {
    let parts = dotted_path
        .split('.')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    let Some((last, parents)) = parts.split_last() else {
        return Err("empty TOML setting path".into());
    };
    let parent = table_at_path_mut(table, parents);
    parent[*last] = toml_value(value);
    Ok(())
}

fn table_at_path_mut<'a>(mut table: &'a mut Table, parts: &[&str]) -> &'a mut Table {
    for part in parts {
        let item = table
            .entry(part)
            .or_insert_with(|| Item::Table(Table::new()));
        if !item.is_table() {
            *item = Item::Table(Table::new());
        }
        table = item.as_table_mut().expect("table item");
    }
    table
}

fn merge_json_config(
    global_config: &crate::seed::ClientIntegrationGlobalConfig,
    existing: &str,
    context: &RenderContext,
) -> Result<String, Box<dyn Error>> {
    let mut base = if existing.trim().is_empty() {
        Value::Object(serde_json::Map::new())
    } else {
        serde_json::from_str(existing)?
    };
    let overlay = json_settings_value(&global_config.json_settings, context)?;
    merge_json_value(&mut base, overlay);
    Ok(format!("{}\n", serde_json::to_string_pretty(&base)?))
}

fn render_json_settings(
    settings: &[(String, String)],
    context: &RenderContext,
) -> Result<String, Box<dyn Error>> {
    Ok(format!(
        "{}\n",
        serde_json::to_string_pretty(&json_settings_value(settings, context)?)?
    ))
}

fn json_settings_value(
    settings: &[(String, String)],
    context: &RenderContext,
) -> Result<Value, Box<dyn Error>> {
    let mut value = Value::Object(serde_json::Map::new());
    for (path, setting_value) in settings {
        set_json_string(&mut value, path, setting_value, context)?;
    }
    Ok(value)
}

fn set_json_string(
    root: &mut Value,
    dotted_path: &str,
    value: &str,
    context: &RenderContext,
) -> Result<(), Box<dyn Error>> {
    let parts = dotted_path
        .split('.')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| render_template(part, context))
        .collect::<Vec<_>>();
    let Some((last, parents)) = parts.split_last() else {
        return Err("empty JSON setting path".into());
    };

    let mut current = root;
    for part in parents {
        let object = current
            .as_object_mut()
            .ok_or("JSON setting path conflicts with a scalar value")?;
        current = object
            .entry(part.clone())
            .or_insert_with(|| Value::Object(serde_json::Map::new()));
    }
    let object = current
        .as_object_mut()
        .ok_or("JSON setting path conflicts with a scalar value")?;
    object.insert(last.clone(), Value::String(render_template(value, context)));
    Ok(())
}

fn merge_json_value(base: &mut Value, overlay: Value) {
    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            for (key, overlay_value) in overlay_map {
                match base_map.get_mut(&key) {
                    Some(base_value) => merge_json_value(base_value, overlay_value),
                    None => {
                        base_map.insert(key, overlay_value);
                    }
                }
            }
        }
        (base_value, overlay_value) => *base_value = overlay_value,
    }
}

fn merge_shell_env_config(
    integration_id: &str,
    global_config: &crate::seed::ClientIntegrationGlobalConfig,
    existing: &str,
    context: &RenderContext,
) -> String {
    let mut next = remove_managed_block(existing, integration_id);
    if !next.is_empty() && !next.ends_with('\n') {
        next.push('\n');
    }
    let _ = writeln!(next, "# >>> formal-ai {integration_id}");
    for env in &global_config.shell_env {
        next.push_str("export ");
        next.push_str(&render_template(&env.key, context));
        next.push('=');
        next.push_str(&shell_double_quote(&render_template(&env.value, context)));
        next.push('\n');
    }
    let _ = writeln!(next, "# <<< formal-ai {integration_id}");
    next
}

fn remove_managed_block(existing: &str, tool: &str) -> String {
    let start = format!("# >>> formal-ai {tool}");
    let end = format!("# <<< formal-ai {tool}");
    let mut out = String::new();
    let mut skipping = false;
    for line in existing.lines() {
        if line == start {
            skipping = true;
            continue;
        }
        if skipping {
            if line == end {
                skipping = false;
            }
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn shell_double_quote(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
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

fn ensure_trailing_newline(mut value: String) -> String {
    if !value.ends_with('\n') {
        value.push('\n');
    }
    value
}

fn maybe_start_server(
    base_url: &str,
    port_override: Option<u16>,
) -> Result<Option<ServerGuard>, Box<dyn Error>> {
    let (host, port) = parse_host_port(base_url, port_override)?;
    let address = format!("{host}:{port}");
    if TcpStream::connect(&address).is_ok() {
        return Ok(None);
    }
    let binary = formal_ai_binary_path()?;
    let mut child = Command::new(binary)
        .args([
            "serve",
            "--agent-mode",
            "--host",
            &host,
            "--port",
            &port.to_string(),
        ])
        .spawn()?;
    wait_for_server(&address, &mut child)?;
    Ok(Some(ServerGuard { child }))
}

fn parse_host_port(
    base_url: &str,
    port_override: Option<u16>,
) -> Result<(String, u16), Box<dyn Error>> {
    let (_, rest) = base_url
        .split_once("://")
        .ok_or("base URL must include a scheme, for example http://127.0.0.1:8080")?;
    let authority = rest.split('/').next().unwrap_or(rest);
    let (host, parsed_port) = if let Some(stripped) = authority.strip_prefix('[') {
        let (inside, after) = stripped
            .split_once(']')
            .ok_or("invalid bracketed IPv6 host in base URL")?;
        let port = after.strip_prefix(':').and_then(|value| value.parse().ok());
        (inside.to_string(), port)
    } else if let Some((host, port)) = authority.split_once(':') {
        (host.to_string(), port.parse().ok())
    } else {
        (authority.to_string(), None)
    };
    let port = port_override.or(parsed_port).unwrap_or(8080);
    Ok((host, port))
}

fn formal_ai_binary_path() -> Result<PathBuf, Box<dyn Error>> {
    let current = std::env::current_exe()?;
    let stem = current.file_stem().and_then(|value| value.to_str());
    if stem == Some("formal-ai") {
        return Ok(current);
    }
    let sibling = current.with_file_name(format!("formal-ai{}", std::env::consts::EXE_SUFFIX));
    if sibling.exists() {
        return Ok(sibling);
    }
    Ok(PathBuf::from("formal-ai"))
}

fn wait_for_server(address: &str, child: &mut Child) -> Result<(), Box<dyn Error>> {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait()? {
            return Err(format!("formal-ai serve exited before listening: {status}").into());
        }
        if TcpStream::connect(address).is_ok() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Err(format!("formal-ai serve did not listen on {address}").into())
}
