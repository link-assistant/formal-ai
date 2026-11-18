use super::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::cnl_output::configuration::deserialize_standard"
    )]
    pub output_standard: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::cnl_output::configuration::deserialize_density"
    )]
    pub lexical_density: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub stream: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub functions: Vec<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_call: Option<Value>,
    /// OpenAI streaming knobs — most relevantly `include_usage`, which asks the
    /// server to emit a final chunk carrying the `usage` block. The AI SDK's
    /// openai-compatible provider ships this and drops token counts if it's
    /// missing (see issue link-assistant/agent#249).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<StreamOptions>,
}
