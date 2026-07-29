use super::embedded::CLIENT_INTEGRATIONS_LINO;
use super::parser::{parse_lino, split_pipe_list};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigFormat {
    Toml,
    Json,
    ShellEnv,
}

impl ConfigFormat {
    fn from_seed(value: &str) -> Option<Self> {
        match value {
            "toml" => Some(Self::Toml),
            "json" => Some(Self::Json),
            "shell_env" => Some(Self::ShellEnv),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelArgPosition {
    BeforeArgs,
    AfterFirstArg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeArgPosition {
    BeforeInvocation,
    BeforeUserArgs,
}

impl ModeArgPosition {
    fn from_seed(value: &str) -> Option<Self> {
        match value {
            "before_invocation" => Some(Self::BeforeInvocation),
            "before_user_args" => Some(Self::BeforeUserArgs),
            _ => None,
        }
    }
}

impl ModelArgPosition {
    fn from_seed(value: &str) -> Option<Self> {
        match value {
            "before_args" => Some(Self::BeforeArgs),
            "after_first_arg" => Some(Self::AfterFirstArg),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TemplateEnv {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Default)]
pub struct ClientIntegrationInvocation {
    pub prepend_args: Vec<String>,
    pub args: Vec<String>,
    pub no_summarize_args: Vec<String>,
    pub interactive_args: Vec<String>,
    pub interactive_args_require_prompt: bool,
    pub non_interactive_args: Vec<String>,
    pub mode_arg_position: Option<ModeArgPosition>,
    pub env: Vec<TemplateEnv>,
    pub config_content_env: String,
    pub config_env: String,
    pub config_dir_env: String,
    pub config_json_settings: Vec<(String, String)>,
    pub temp_home_env: String,
    pub temp_home_config_path: String,
    pub temp_home_json_settings: Vec<(String, String)>,
    pub temp_home_toml_settings: Vec<(String, String)>,
    pub model_catalog_path: String,
    pub model_arg: String,
    pub model_arg_position: Option<ModelArgPosition>,
    pub session_root: String,
    pub session_file_suffix: String,
    pub resume_command: String,
    pub session_id_query_args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ClientIntegrationGlobalConfig {
    pub protocol: String,
    pub format: ConfigFormat,
    pub path: String,
    pub backup_suffix: String,
    pub model_catalog_path: String,
    pub toml_settings: Vec<(String, String)>,
    pub json_settings: Vec<(String, String)>,
    pub shell_env: Vec<TemplateEnv>,
}

/// Seed-defined facts the real-client matrix needs to exercise an integration.
///
/// These facts live beside the invocation contract because they describe the
/// same upstream client. The matrix consumes them as data; adding a client or
/// changing an upstream limitation must not require another client-id branch in
/// shell or workflow code.
#[derive(Debug, Clone, Default)]
pub struct ClientVerification {
    /// Interaction surface: `cli`, `server`, `gui`, or `mcp`.
    pub surface: String,
    /// How a CLI makes fixture bytes available: `tool_call` or `in_band`.
    pub file_delivery: String,
    /// Arguments appended after the headless prompt for file-delivery clients.
    pub headless_args: Vec<String>,
    /// Environment required only while launching an interactive client.
    pub interactive_env: Vec<TemplateEnv>,
    /// Client-specific keystrokes performed before the common TUI task.
    pub interactive_preamble: Vec<String>,
    /// Tools that must be advertised by a constraint probe.
    pub required_request_tools: Vec<String>,
    /// Stable response tools already accepted as contract requirements.
    pub required_response_tools: Vec<String>,
    /// Text that must remain absent from headless client output.
    pub forbidden_output: Vec<String>,
    /// Arguments used to launch a server or GUI surface.
    pub launch_args: Vec<String>,
    /// Output proving that a launched server is ready.
    pub launch_ready: String,
    /// Additional rendered output required from a launched application.
    pub launch_required_output: Vec<String>,
    /// HTTP path that must answer after a server launch.
    pub launch_http_path: String,
    /// Exact upstream subcommand set for a server with no prompt path.
    pub launch_subcommands: Vec<String>,
    /// Required installed extension, expressed as a template/glob.
    pub extension_glob: String,
    /// Whether a GUI may need Chromium's no-sandbox fallback on restricted hosts.
    pub chromium_sandbox_fallback: bool,
    /// Whether CI must enable unprivileged namespaces for the client's sandbox.
    pub sandbox_user_namespaces: bool,
    /// Expected error when an MCP-shaped client has no vendor credentials.
    pub vendor_auth_error: String,
}

#[derive(Debug, Clone)]
pub struct ClientIntegration {
    pub id: String,
    pub aliases: Vec<String>,
    pub label: String,
    pub command: String,
    pub command_env: String,
    pub platform_commands: Vec<TemplateEnv>,
    pub provider_id: String,
    pub default_protocol: String,
    pub supported_protocols: Vec<String>,
    pub endpoints: Vec<(String, String)>,
    pub api_key_env: String,
    pub api_key_default: String,
    pub model_selector: String,
    pub invocation: ClientIntegrationInvocation,
    pub verification: ClientVerification,
    /// The default-protocol global configuration, retained for callers that
    /// inspect the registry directly.
    pub global_config: ClientIntegrationGlobalConfig,
    pub global_configs: Vec<ClientIntegrationGlobalConfig>,
}

impl ClientIntegration {
    #[must_use]
    pub fn endpoint_path_for(&self, protocol: &str) -> Option<&str> {
        self.endpoints
            .iter()
            .find_map(|(candidate, path)| (candidate == protocol).then_some(path.as_str()))
    }

    #[must_use]
    pub fn global_config_for(&self, protocol: &str) -> &ClientIntegrationGlobalConfig {
        self.global_configs
            .iter()
            .find(|config| config.protocol == protocol)
            .or_else(|| {
                self.global_configs
                    .iter()
                    .find(|config| config.protocol.is_empty())
            })
            .unwrap_or(&self.global_config)
    }
}

#[must_use]
pub fn client_integrations() -> Vec<ClientIntegration> {
    let tree = parse_lino(CLIENT_INTEGRATIONS_LINO);
    let mut out = Vec::new();
    for root in &tree.children {
        if root.name != "client_integrations" {
            continue;
        }
        for tool in root.children.iter().filter(|node| node.name == "tool") {
            let Some(integration) = parse_tool(tool) else {
                continue;
            };
            out.push(integration);
        }
    }
    out
}

fn parse_tool(tool: &super::parser::LinoNode) -> Option<ClientIntegration> {
    let id = tool.id.clone();
    if id.is_empty() {
        return None;
    }
    let aliases = split_pipe_list(tool.find_child_value("aliases"));
    let label = tool.find_child_value("label").to_string();
    let command = tool.find_child_value("command").to_string();
    let command_env = tool.find_child_value("command_env").to_string();
    let platform_commands = tool
        .children
        .iter()
        .filter_map(|child| {
            child
                .name
                .strip_prefix("command_")
                .filter(|platform| *platform != "env")
                .map(|platform| TemplateEnv {
                    key: platform.to_string(),
                    value: child.id.clone(),
                })
        })
        .collect();
    let provider_id = tool.find_child_value("provider_id").to_string();
    let default_protocol = tool.find_child_value("default_protocol").to_string();
    let supported_protocols = split_pipe_list(tool.find_child_value("supported_protocols"));
    let api_key_env = tool.find_child_value("api_key_env").to_string();
    let api_key_default = tool.find_child_value("api_key_default").to_string();
    let mut endpoints = Vec::new();
    for child in &tool.children {
        if let Some(protocol) = child.name.strip_prefix("endpoint_") {
            endpoints.push((protocol.to_string(), child.id.clone()));
        }
    }
    let model_selector = tool.find_child_value("model_selector").to_string();
    let invocation = tool
        .children
        .iter()
        .find(|node| node.name == "ephemeral")
        .map(parse_invocation)
        .unwrap_or_default();
    let verification = tool
        .children
        .iter()
        .find(|node| node.name == "verification")
        .map(parse_verification)
        .unwrap_or_default();
    let global_configs = tool
        .children
        .iter()
        .filter(|node| node.name == "global")
        .filter_map(parse_global_config)
        .collect::<Vec<_>>();
    if global_configs.is_empty() {
        return None;
    }
    let global_config = global_configs
        .iter()
        .find(|config| config.protocol == default_protocol)
        .unwrap_or(&global_configs[0])
        .clone();
    Some(ClientIntegration {
        id,
        aliases,
        label,
        command,
        command_env,
        platform_commands,
        provider_id,
        default_protocol,
        supported_protocols,
        endpoints,
        api_key_env,
        api_key_default,
        model_selector,
        invocation,
        verification,
        global_config,
        global_configs,
    })
}

fn parse_verification(node: &super::parser::LinoNode) -> ClientVerification {
    let mut verification = ClientVerification {
        surface: node.find_child_value("surface").to_string(),
        file_delivery: node.find_child_value("file_delivery").to_string(),
        launch_ready: node.find_child_value("launch_ready").to_string(),
        launch_http_path: node.find_child_value("launch_http_path").to_string(),
        extension_glob: node.find_child_value("extension_glob").to_string(),
        chromium_sandbox_fallback: node.find_child_value("chromium_sandbox_fallback") == "true",
        sandbox_user_namespaces: node.find_child_value("sandbox_user_namespaces") == "true",
        vendor_auth_error: node.find_child_value("vendor_auth_error").to_string(),
        ..ClientVerification::default()
    };
    for child in &node.children {
        match child.name.as_str() {
            "headless_arg" => verification.headless_args.push(child.id.clone()),
            "interactive_env" => {
                if let Some((key, value)) = split_once_equals(&child.id) {
                    verification
                        .interactive_env
                        .push(TemplateEnv { key, value });
                }
            }
            "interactive_preamble" => {
                verification.interactive_preamble.push(child.id.clone());
            }
            "required_request_tool" => {
                verification.required_request_tools.push(child.id.clone());
            }
            "required_response_tool" => {
                verification.required_response_tools.push(child.id.clone());
            }
            "forbidden_output" => verification.forbidden_output.push(child.id.clone()),
            "launch_arg" => verification.launch_args.push(child.id.clone()),
            "launch_required_output" => {
                verification.launch_required_output.push(child.id.clone());
            }
            "launch_subcommand" => verification.launch_subcommands.push(child.id.clone()),
            _ => {}
        }
    }
    verification
}

fn parse_invocation(node: &super::parser::LinoNode) -> ClientIntegrationInvocation {
    let mut invocation = ClientIntegrationInvocation::default();
    for child in &node.children {
        match child.name.as_str() {
            "prepend_arg" => invocation.prepend_args.push(child.id.clone()),
            "arg" => invocation.args.push(child.id.clone()),
            "no_summarize_arg" => invocation.no_summarize_args.push(child.id.clone()),
            "interactive_arg" => invocation.interactive_args.push(child.id.clone()),
            "interactive_args_require_prompt" => {
                invocation.interactive_args_require_prompt = child.id == "true";
            }
            "non_interactive_arg" => invocation.non_interactive_args.push(child.id.clone()),
            "mode_arg_position" => {
                invocation.mode_arg_position = ModeArgPosition::from_seed(&child.id);
            }
            "env" => {
                if let Some((key, value)) = split_once_equals(&child.id) {
                    invocation.env.push(TemplateEnv { key, value });
                }
            }
            "config_content_env" => invocation.config_content_env.clone_from(&child.id),
            "config_env" => invocation.config_env.clone_from(&child.id),
            "config_dir_env" => invocation.config_dir_env.clone_from(&child.id),
            "config_json_set" => {
                if let Some((key, value)) = split_once_equals(&child.id) {
                    invocation.config_json_settings.push((key, value));
                }
            }
            "temp_home_env" => invocation.temp_home_env.clone_from(&child.id),
            "temp_home_config_path" => invocation.temp_home_config_path.clone_from(&child.id),
            "temp_home_json_set" => {
                if let Some((key, value)) = split_once_equals(&child.id) {
                    invocation.temp_home_json_settings.push((key, value));
                }
            }
            "temp_home_toml_set" => {
                if let Some((key, value)) = split_once_equals(&child.id) {
                    invocation.temp_home_toml_settings.push((key, value));
                }
            }
            "model_catalog_path" => invocation.model_catalog_path.clone_from(&child.id),
            "model_arg" => invocation.model_arg.clone_from(&child.id),
            "model_arg_position" => {
                invocation.model_arg_position = ModelArgPosition::from_seed(&child.id);
            }
            "session_root" => invocation.session_root.clone_from(&child.id),
            "session_file_suffix" => invocation.session_file_suffix.clone_from(&child.id),
            "resume_command" => invocation.resume_command.clone_from(&child.id),
            "session_id_query_arg" => invocation.session_id_query_args.push(child.id.clone()),
            _ => {}
        }
    }
    invocation
}

fn parse_global_config(node: &super::parser::LinoNode) -> Option<ClientIntegrationGlobalConfig> {
    let format = ConfigFormat::from_seed(node.find_child_value("kind"))?;
    let mut config = ClientIntegrationGlobalConfig {
        protocol: node.id.clone(),
        format,
        path: node.find_child_value("path").to_string(),
        backup_suffix: node.find_child_value("backup_suffix").to_string(),
        model_catalog_path: node.find_child_value("model_catalog_path").to_string(),
        toml_settings: Vec::new(),
        json_settings: Vec::new(),
        shell_env: Vec::new(),
    };
    for child in &node.children {
        match child.name.as_str() {
            "toml_set" => {
                if let Some((key, value)) = split_once_equals(&child.id) {
                    config.toml_settings.push((key, value));
                }
            }
            "json_set" => {
                if let Some((key, value)) = split_once_equals(&child.id) {
                    config.json_settings.push((key, value));
                }
            }
            "shell_env" => {
                if let Some((key, value)) = split_once_equals(&child.id) {
                    config.shell_env.push(TemplateEnv { key, value });
                }
            }
            _ => {}
        }
    }
    Some(config)
}

fn split_once_equals(value: &str) -> Option<(String, String)> {
    let (key, value) = value.split_once('=')?;
    let key = key.trim();
    if key.is_empty() {
        return None;
    }
    Some((key.to_string(), value.trim().to_string()))
}
