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
    pub env: Vec<TemplateEnv>,
    pub config_env: String,
    pub config_dir_env: String,
    pub config_json_settings: Vec<(String, String)>,
    pub model_arg: String,
    pub model_arg_position: Option<ModelArgPosition>,
}

#[derive(Debug, Clone)]
pub struct ClientIntegrationGlobalConfig {
    pub format: ConfigFormat,
    pub path: String,
    pub backup_suffix: String,
    pub toml_settings: Vec<(String, String)>,
    pub json_settings: Vec<(String, String)>,
    pub shell_env: Vec<TemplateEnv>,
}

#[derive(Debug, Clone)]
pub struct ClientIntegration {
    pub id: String,
    pub label: String,
    pub command: String,
    pub provider_id: String,
    pub default_protocol: String,
    pub supported_protocols: Vec<String>,
    pub endpoints: Vec<(String, String)>,
    pub api_key_env: String,
    pub api_key_default: String,
    pub model_selector: String,
    pub invocation: ClientIntegrationInvocation,
    pub global_config: ClientIntegrationGlobalConfig,
}

impl ClientIntegration {
    #[must_use]
    pub fn endpoint_path_for(&self, protocol: &str) -> Option<&str> {
        self.endpoints
            .iter()
            .find_map(|(candidate, path)| (candidate == protocol).then_some(path.as_str()))
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
    let label = tool.find_child_value("label").to_string();
    let command = tool.find_child_value("command").to_string();
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
    let global_config = tool
        .children
        .iter()
        .find(|node| node.name == "global")
        .and_then(parse_global_config)?;
    Some(ClientIntegration {
        id,
        label,
        command,
        provider_id,
        default_protocol,
        supported_protocols,
        endpoints,
        api_key_env,
        api_key_default,
        model_selector,
        invocation,
        global_config,
    })
}

fn parse_invocation(node: &super::parser::LinoNode) -> ClientIntegrationInvocation {
    let mut invocation = ClientIntegrationInvocation::default();
    for child in &node.children {
        match child.name.as_str() {
            "prepend_arg" => invocation.prepend_args.push(child.id.clone()),
            "arg" => invocation.args.push(child.id.clone()),
            "env" => {
                if let Some((key, value)) = split_once_equals(&child.id) {
                    invocation.env.push(TemplateEnv { key, value });
                }
            }
            "config_env" => invocation.config_env.clone_from(&child.id),
            "config_dir_env" => invocation.config_dir_env.clone_from(&child.id),
            "config_json_set" => {
                if let Some((key, value)) = split_once_equals(&child.id) {
                    invocation.config_json_settings.push((key, value));
                }
            }
            "model_arg" => invocation.model_arg.clone_from(&child.id),
            "model_arg_position" => {
                invocation.model_arg_position = ModelArgPosition::from_seed(&child.id);
            }
            _ => {}
        }
    }
    invocation
}

fn parse_global_config(node: &super::parser::LinoNode) -> Option<ClientIntegrationGlobalConfig> {
    let format = ConfigFormat::from_seed(node.find_child_value("kind"))?;
    let mut config = ClientIntegrationGlobalConfig {
        format,
        path: node.find_child_value("path").to_string(),
        backup_suffix: node.find_child_value("backup_suffix").to_string(),
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
