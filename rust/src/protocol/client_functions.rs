//! Forced OpenAI functions deliver schema-validated data to the caller.
use super::{
    ChatChoice, ChatCompletion, ChatCompletionRequest, ChatMessage, ResponseFunctionToolCall,
    ResponseObject, ResponseOutputContent, ResponseOutputItem, ResponseOutputMessage,
    ResponseUsage, ResponsesRequest, TokenUsage, ToolCall, function_call_kind,
    message_input_tokens, resolved_request_model, response_timestamp, responses_input_tokens,
};
use crate::computer_use::ComputerUsePrimitive;
use crate::engine::{estimate_tokens, normalize_prompt, stable_id};
use crate::protocol_call_validation::invalid_arguments;
use crate::protocol_policy::{tool_choice_function_name, tool_definition_names};
use serde_json::{Map, Value, json};
use std::collections::HashMap;

enum ForcedFunction {
    Call(ToolCall),
    Refusal(String),
}

impl ForcedFunction {
    fn output_tokens(&self) -> u32 {
        match self {
            Self::Call(call) => estimate_tokens(&call.function.name)
                .saturating_add(estimate_tokens(&call.function.arguments)),
            Self::Refusal(reason) => estimate_tokens(reason),
        }
    }
}

fn selected_functions(request: &ChatCompletionRequest) -> Vec<(&str, &Value)> {
    if !request.functions.is_empty()
        || request
            .function_call
            .as_ref()
            .is_some_and(|choice| choice != "none")
    {
        return vec![];
    }
    let Some(choice) = &request.tool_choice else {
        return vec![];
    };
    let required = choice == "required";
    let name = (choice.get("type").and_then(Value::as_str) == Some("function"))
        .then(|| tool_choice_function_name(choice))
        .flatten();
    if !required && name.is_none() {
        return vec![];
    }

    let mut names = HashMap::new();
    for tool_name in request.tools.iter().flat_map(tool_definition_names) {
        *names.entry(tool_name).or_insert(0usize) += 1;
    }
    let registry = crate::seed::agentic_tool_capabilities();
    request
        .tools
        .iter()
        .filter_map(|tool| {
            if tool.get("type")?.as_str()? != "function" {
                return None;
            }
            let definition = tool.get("function").unwrap_or(tool);
            let candidate = definition.get("name")?.as_str()?;
            if (!required && name.as_deref() != Some(candidate))
                || candidate.is_empty()
                || candidate.len() > 64
                || !candidate
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
                || names.get(candidate) != Some(&1)
                || ComputerUsePrimitive::from_tool_name(candidate).is_some()
                || registry
                    .iter()
                    .flat_map(|entry| &entry.aliases)
                    .any(|alias| alias.eq_ignore_ascii_case(candidate))
            {
                return None;
            }
            Some((candidate, definition))
        })
        .collect()
}

pub(super) fn sentiment_label(
    definition: &Value,
    name: &str,
    field: &Value,
    prompt: &str,
) -> Option<&'static str> {
    let labels = ["negative", "neutral", "positive"];
    let choices = field.get("enum")?.as_array()?;
    if choices.len() < 2
        || !choices
            .iter()
            .all(|choice| choice.as_str().is_some_and(|label| labels.contains(&label)))
    {
        return None;
    }
    let description = format!(
        "{} {name} {}",
        definition
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or_default(),
        field
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or_default(),
    );
    let lexicon = crate::seed::lexicon();
    if !lexicon.mentions_role("client-function-sentiment", &normalize_prompt(&description)) {
        return None;
    }
    let text = normalize_prompt(prompt);
    if lexicon.mentions_role("client-function-sentiment-negation", &text) {
        return None;
    }
    let mut matched = labels.into_iter().filter(|label| {
        lexicon.mentions_role(&format!("client-function-sentiment-{label}"), &text)
    });
    let label = matched.next()?;
    matched.next().is_none().then_some(label)
}

fn derive_arguments(definition: &Value, schema: &Value, prompt: &str) -> Option<Value> {
    let supplied = serde_json::from_str::<Value>(prompt).ok();
    if supplied.as_ref().is_some_and(Value::is_object) {
        return supplied;
    }
    let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
        return Some(json!({}));
    };
    if properties.is_empty() {
        return Some(json!({}));
    }
    if properties.len() != 1 {
        return None;
    }
    let (name, field) = properties.iter().next()?;
    if !field.is_object() {
        return None;
    }
    let value = if let Some(supplied) = supplied {
        supplied
    } else {
        let text = Value::String(prompt.trim().to_owned());
        if field
            .get("enum")
            .and_then(Value::as_array)
            .is_some_and(|items| items.contains(&text))
        {
            text
        } else {
            Value::String(sentiment_label(definition, name, field, prompt)?.to_owned())
        }
    };
    Some(Value::Object(Map::from_iter([(name.clone(), value)])))
}

// Refuse constraints outside the protocol validator's supported vocabulary.
fn supported_schema(schema: &Value, depth: usize) -> bool {
    if depth > 32 {
        return false;
    }
    if schema.is_boolean() {
        return true;
    }
    let nested = |value| supported_schema(value, depth + 1);
    let supported_type = |value: &Value| {
        matches!(
            value.as_str(),
            Some("null" | "boolean" | "object" | "array" | "string" | "number" | "integer")
        )
    };
    schema.as_object().is_some_and(|fields| {
        fields.iter().all(|(key, value)| match key.as_str() {
            "properties" => value
                .as_object()
                .is_some_and(|fields| fields.values().all(nested)),
            "additionalProperties" | "items" => nested(value),
            "allOf" | "anyOf" => value
                .as_array()
                .is_some_and(|items| !items.is_empty() && items.iter().all(nested)),
            "required" => value
                .as_array()
                .is_some_and(|items| items.iter().all(Value::is_string)),
            "enum" => value.as_array().is_some_and(|items| !items.is_empty()),
            "minLength" | "maxLength" | "minItems" | "maxItems" => value
                .as_u64()
                .is_some_and(|bound| bound <= MAXIMUM_SAFE_INTEGER),
            "minimum" | "maximum" => value.is_number(),
            "type" => {
                supported_type(value)
                    || value
                        .as_array()
                        .is_some_and(|types| !types.is_empty() && types.iter().all(supported_type))
            }
            "title" | "description" | "default" | "examples" | "const" => true,
            _ => false,
        })
    })
}

const MAXIMUM_SAFE_INTEGER: u64 = (1 << 53) - 1;

fn serializable_arguments(value: &Value, depth: usize) -> bool {
    if depth > 32 {
        return false;
    }
    let nested = |value| serializable_arguments(value, depth + 1);
    match value {
        Value::Object(fields) => fields.values().all(nested),
        Value::Array(items) => items.iter().all(nested),
        Value::Number(number) => number.as_f64().is_some_and(|number| {
            number.fract() != 0.0 || number.abs() <= MAXIMUM_SAFE_INTEGER as f64
        }),
        _ => true,
    }
}

fn forced_function(request: &ChatCompletionRequest, prompt: &str) -> Option<ForcedFunction> {
    let definitions = selected_functions(request);
    let (first, _) = definitions.first().copied()?;
    let empty_schema = json!({"additionalProperties": false});
    for (name, definition) in definitions {
        let schema = definition.get("parameters").unwrap_or(&empty_schema);
        if schema.is_object()
            && supported_schema(schema, 0)
            && let Some(value) = derive_arguments(definition, schema, prompt)
            && serializable_arguments(&value, 0)
            && !invalid_arguments(&value, schema, 0)
        {
            let arguments = value.to_string();
            let seed = format!("{prompt}|{}|0|{name}|{arguments}", request.messages.len());
            return Some(ForcedFunction::Call(ToolCall::function(
                stable_id("call", &seed),
                name.to_owned(),
                arguments,
            )));
        }
    }
    Some(ForcedFunction::Refusal(crate::seed::agentic_message(
        "client-function-arguments-underivable",
        &[("tool", first)],
    )))
}

pub(super) fn client_function_completion(
    request: &ChatCompletionRequest,
    prompt: &str,
) -> Option<ChatCompletion> {
    let outcome = forced_function(request, prompt)?;
    let completion_tokens = outcome.output_tokens();
    let prompt_tokens = message_input_tokens(&request.messages);
    let (message, finish_reason) = match outcome {
        ForcedFunction::Call(call) => (ChatMessage::assistant_tool_calls(vec![call]), "tool_calls"),
        ForcedFunction::Refusal(reason) => (
            ChatMessage {
                refusal: Some(reason),
                ..ChatMessage::assistant("")
            },
            "stop",
        ),
    };
    Some(ChatCompletion {
        id: stable_id("chatcmpl", prompt),
        object: "chat.completion".into(),
        created: response_timestamp(),
        model: resolved_request_model(request.model.as_deref()),
        choices: vec![ChatChoice {
            index: 0,
            message,
            finish_reason: finish_reason.into(),
        }],
        usage: TokenUsage {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens.saturating_add(completion_tokens),
        },
        learning_trace: None,
        response_format: None,
        derivation_identifier: None,
        evidence_links: Vec::new(),
        output_projection: None,
    })
}

pub(super) fn client_function_response(
    request: &ResponsesRequest,
    chat: &ChatCompletionRequest,
    prompt: &str,
    user_text: &str,
) -> Option<ResponseObject> {
    let outcome = forced_function(chat, user_text)?;
    let output_tokens = outcome.output_tokens();
    let input_tokens = responses_input_tokens(request);
    let item = match outcome {
        ForcedFunction::Call(call) => ResponseOutputItem::FunctionCall(ResponseFunctionToolCall {
            id: stable_id("fc", &call.id),
            kind: function_call_kind(),
            call_id: call.id,
            name: call.function.name,
            namespace: None,
            arguments: call.function.arguments,
            status: "completed".into(),
        }),
        ForcedFunction::Refusal(reason) => ResponseOutputItem::Message(ResponseOutputMessage {
            id: stable_id("msg", &reason),
            kind: "message".into(),
            role: "assistant".into(),
            content: vec![ResponseOutputContent {
                kind: "refusal".into(),
                text: String::new(),
                refusal: Some(reason),
            }],
            thinking_steps: vec![],
        }),
    };
    Some(ResponseObject {
        id: stable_id("resp", prompt),
        object: "response".into(),
        created_at: response_timestamp(),
        status: "completed".into(),
        model: resolved_request_model(request.model.as_deref()),
        output: vec![item],
        usage: ResponseUsage {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens.saturating_add(output_tokens),
        },
        evidence_links: vec![],
        derivation_id: None,
        text: None,
        thinking_steps: vec![],
    })
}
