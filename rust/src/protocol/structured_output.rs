//! Shared response-format obligation and projection for Chat and Responses.
mod arrays;

use super::{
    ChatCompletion, ChatCompletionRequest, MessageContent, ResponseObject, ResponseOutputContent,
    ResponseOutputItem, ResponsesRequest, chat_prompt_and_history,
    client_functions::sentiment_label, structured_schema as schema,
};
use crate::derivation::{Derivation, RECOMPOSE_BIND_KIND, RENDER_EMIT_KIND};
use crate::engine::{SymbolicAnswer, ThinkingStep, estimate_tokens, stable_id};
use crate::event_log::EventLog;
use regex::Regex;
use serde_json::{Map, Value, json};
use std::sync::LazyLock;

const UNKNOWN: &str = "unknown";
const NUMBER: &str = r"-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?";
static SUM: LazyLock<Regex> = LazyLock::new(|| arithmetic_pattern("structured-output-sum-pattern"));
static KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)\b(?:key|property|field)\s+["'`]([A-Za-z_][A-Za-z_0-9-]*)["'`]"#)
        .expect("output key")
});
static CITY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:in|based in|located in)\s+(\p{Lu}[\p{L}'’-]*(?:[ \t]+\p{Lu}[\p{L}'’-]*)*)(?:$|[,.!?;]|[ \t]+(?:with|is|at|and|where)\b)").expect("location statement")
});
static LOCATION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(?:wanted|hiring|jobs?|openings?|positions?|based|located)\b")
        .expect("location cue")
});
static ARITHMETIC: LazyLock<Regex> =
    LazyLock::new(|| arithmetic_pattern("structured-output-arithmetic-pattern"));
static YEARS: LazyLock<Regex> = LazyLock::new(|| {
    let qualifier = crate::seed::agentic_message("structured-output-experience-qualifier", &[]);
    Regex::new(&format!(
        r"\b{qualifier}\s+({NUMBER})\s+years?\s+(?:of\s+)?experience\b"
    ))
    .expect("experience statement")
});
static LABEL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^[ \t]*([\p{L}\p{N}_-]+(?:[ \t]+[\p{L}\p{N}_-]+)*?)[ \t]*(?::|=|[ \t]+is[ \t]+)[ \t]*(.+)").expect("labeled value")
});

fn arithmetic_pattern(key: &str) -> Regex {
    let pattern = crate::seed::agentic_message(key, &[("number", NUMBER)]);
    Regex::new(&format!("(?i){pattern}")).expect("structured arithmetic pattern")
}

struct Obligation<'a> {
    pub format: &'a Value,
    schema: Option<&'a Value>,
    schema_issue: Option<schema::SchemaIssue<'a>>,
}

pub(super) fn requested(format: Option<&Value>) -> bool {
    format.is_some_and(|format| {
        !format.is_null() && format.get("type").and_then(Value::as_str) != Some("text")
    })
}

impl<'a> Obligation<'a> {
    pub fn new(format: Option<&'a Value>, chat: bool) -> Option<Self> {
        static JSON_OBJECT: LazyLock<Value> = LazyLock::new(|| json!({"type":"object"}));
        let format = format.filter(|format| requested(Some(format)))?;
        let schema = match format.get("type").and_then(Value::as_str) {
            Some("json_object") => Some(&*JSON_OBJECT),
            Some("json_schema") => {
                let specification = if chat {
                    format.get("json_schema")
                } else {
                    Some(format)
                };
                specification
                    .filter(|specification| {
                        specification.get("strict").is_none_or(Value::is_boolean)
                    })
                    .and_then(|specification| specification.get("schema"))
            }
            _ => None,
        };
        let schema_issue = schema.and_then(|candidate| schema::check(candidate).err());
        Some(Self {
            format,
            schema: schema.filter(|_| schema_issue.is_none()),
            schema_issue,
        })
    }

    pub fn project(&self, source: &str, answer: &str, refusal: Option<String>) -> Projection {
        let value = refusal
            .is_none()
            .then(|| {
                self.schema
                    .and_then(|schema| derive(schema, source, answer))
            })
            .flatten();
        let refusal = refusal.or_else(|| {
            value.is_none().then(|| {
                let key = if self.schema.is_some() {
                    "structured-output-underivable"
                } else {
                    "structured-output-schema-unsupported"
                };
                let message = crate::seed::agentic_message(key, &[]);
                if let Some(issue) = &self.schema_issue {
                    format!("{message} {}", issue.message())
                } else {
                    message
                }
            })
        });
        let content = value.as_ref().map_or_else(String::new, Value::to_string);
        let no_evidence = self
            .schema
            .zip(value.as_ref())
            .map_or_else(Vec::new, |(schema, value)| {
                missing_array_evidence(schema, source, value)
            });
        let detail = if let Some(reason) = &refusal {
            json!({"format": self.format, "refusal": reason})
        } else if !no_evidence.is_empty() {
            json!({"format": self.format, "value": value, "no_evidence": no_evidence})
        } else {
            json!({"format": self.format, "value": value})
        }
        .to_string();
        let identifier = stable_id("answer", if refusal.is_some() { &detail } else { &content });
        let mut evidence = vec![format!("derivation:{identifier}")];
        if !crate::solver_config::ephemeral_enabled() {
            let mut log = EventLog::new();
            log.append(RECOMPOSE_BIND_KIND, &detail);
            log.append(
                RENDER_EMIT_KIND,
                format!("answer_id={identifier};format=json"),
            );
            match crate::runtime_cache::persist_derivation(&Derivation::record_for(
                &log,
                &identifier,
            )) {
                Ok(location) => {
                    evidence.push(location.path.display().to_string());
                    evidence.push(format!("derivation_record:{}", location.locator));
                }
                Err(error) => evidence.push(format!("derivation:persistence_failed:{error}")),
            }
        }
        Projection {
            content,
            refusal,
            detail,
            identifier,
            evidence,
        }
    }
}

struct Projection {
    pub content: String,
    pub refusal: Option<String>,
    detail: String,
    pub identifier: String,
    pub evidence: Vec<String>,
}

impl Projection {
    pub fn step(&self, steps: &[ThinkingStep]) -> ThinkingStep {
        ThinkingStep::new(
            u32::try_from(steps.len()).unwrap_or(u32::MAX),
            "structured_output",
            &self.detail,
            "high",
            RECOMPOSE_BIND_KIND,
        )
    }
}

fn key_name(name: &str) -> String {
    static CAMEL: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"([\p{Ll}\p{N}])([\p{Lu}\p{Lt}])").expect("camel case boundary")
    });
    static ACRONYM: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"([\p{Lu}\p{Lt}])([\p{Lu}\p{Lt}]\p{Ll})").expect("acronym boundary")
    });
    let words = CAMEL.replace_all(name, "$1 $2");
    ACRONYM
        .replace_all(&words, "$1 $2")
        .split(|ch: char| !ch.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

// Keep a matched but ambiguous label distinct from an absent one.
fn unique<T: PartialEq>(mut values: impl Iterator<Item = T>) -> Option<Option<T>> {
    let first = values.next()?;
    Some(values.all(|value| value == first).then_some(first))
}

fn labeled_evidence(source: &str, label: &str) -> Option<Option<Value>> {
    unique(source.split(['\n', ';', ',']).filter_map(|clause| {
        let captures = LABEL.captures(clause.trim_start())?;
        if key_name(&captures[1]) != label {
            return None;
        }
        let text = captures[2].trim().trim_end_matches(['.', '!', '?']);
        Some(serde_json::from_str(text).unwrap_or_else(|_| Value::String(text.to_owned())))
    }))
}

fn evidence(
    name: &str,
    field: &Value,
    description: Option<&str>,
    source: &str,
    answer: &str,
    number: Option<&Value>,
    key: Option<&str>,
) -> Option<Value> {
    let normalized = key_name(name);
    if let Some(value) = labeled_evidence(source, &normalized) {
        return value;
    }
    let described = || {
        let label = key_name(description?);
        if label.is_empty() {
            return None;
        }
        labeled_evidence(source, &label)
    };
    if let Some(number) = number
        && (key == Some(name)
            || (key.is_none()
                && matches!(normalized.as_str(), "answer" | "result" | "sum" | "total")))
    {
        return Some(number.clone());
    }
    if normalized == "city" {
        return unique(
            source
                .split(['.', '!', '?', ';', '\n'])
                .filter(|clause| LOCATION.is_match(clause))
                .flat_map(|clause| {
                    CITY.captures_iter(clause)
                        .map(|capture| Value::String(capture[1].to_owned()))
                }),
        )
        .or_else(described)
        .flatten();
    }
    if let Some(kind) = normalized
        .strip_prefix("min ")
        .or_else(|| normalized.strip_prefix("minimum "))
        && matches!(kind, "years" | "years of experience" | "experience years")
    {
        return unique(
            YEARS
                .captures_iter(source)
                .filter_map(|capture| serde_json::from_str(&capture[1]).ok()),
        )
        .or_else(described)
        .flatten();
    }
    if let Some(value) = described() {
        return value;
    }
    let labels = field.get("enum")?.as_array()?;
    for text in [answer.trim(), source.trim()] {
        if !text.is_empty() && labels.iter().any(|label| label == text) {
            return Some(Value::String(text.to_owned()));
        }
    }
    sentiment_label(&Value::Null, name, field, source).map(|label| Value::String(label.to_owned()))
}

fn resolved_field<'a>(root: &'a Value, field: &'a Value) -> &'a Value {
    field
        .get("$ref")
        .and_then(Value::as_str)
        .and_then(|pointer| schema::reference(root, pointer))
        .unwrap_or(field)
}

fn object_fields(schema: &Value) -> &Value {
    if schema.get("properties").is_some() {
        schema
    } else {
        resolved_field(schema, schema)
    }
}

fn field_description<'a>(field: &'a Value, resolved: &'a Value) -> Option<&'a str> {
    field
        .get("description")
        .and_then(Value::as_str)
        .or_else(|| resolved.get("description").and_then(Value::as_str))
}

fn missing_array_evidence<'a>(schema: &'a Value, source: &str, value: &Value) -> Vec<&'a str> {
    if source.len() > 8192
        || serde_json::from_str::<Value>(source).is_ok_and(|value| value.is_object())
    {
        return Vec::new();
    }
    let mut missing: Vec<_> = object_fields(schema)
        .get("properties")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .filter_map(|(name, field)| {
            if !value
                .get(name)
                .and_then(Value::as_array)
                .is_some_and(Vec::is_empty)
            {
                return None;
            }
            let resolved = resolved_field(schema, field);
            let items = arrays::item_rule(resolved, schema)?;
            arrays::evidence(source, name, field_description(field, resolved), items)
                .is_none()
                .then_some(name.as_str())
        })
        .collect();
    missing.sort_unstable();
    missing
}

fn derive(schema: &Value, source: &str, answer: &str) -> Option<Value> {
    let satisfies = |value: &Value| {
        value.is_object() && schema::portable(value, 0) && schema::valid(value, schema, schema)
    };
    let formal = serde_json::from_str::<Value>(answer).ok();
    let supplied = formal.filter(Value::is_object).or_else(|| {
        serde_json::from_str::<Value>(source)
            .ok()
            .filter(Value::is_object)
    });
    if supplied.as_ref().is_some_and(&satisfies) {
        return supplied;
    }
    if supplied.is_none() && source.len() > 8192 {
        return None;
    }
    let text = if supplied.is_some() { "" } else { source };
    let key = KEY
        .captures(text)
        .and_then(|captures| captures.get(1))
        .map(|key| key.as_str());
    let expression = SUM
        .captures(text.trim())
        .map(|terms| format!("{} + {}", &terms[1], &terms[2]))
        .or_else(|| {
            ARITHMETIC
                .captures(text.trim())
                .map(|expression| expression[1].to_owned())
        });
    let number = expression
        .filter(|expression| expression.len() <= 80)
        .and_then(|expression| crate::arithmetic::evaluate_fallback_formatted(&expression).ok())
        .and_then(|number| serde_json::from_str::<Value>(&number).ok());
    let fields = object_fields(schema);
    let mut output = Map::new();
    if let Some(properties) = fields.get("properties").and_then(Value::as_object) {
        for (name, field) in properties {
            let is_required = fields
                .get("required")
                .and_then(Value::as_array)
                .is_some_and(|keys| keys.iter().any(|key| key == name.as_str()));
            let resolved = resolved_field(schema, field);
            let mut value = if let Some(supplied) = &supplied {
                supplied.get(name).cloned()
            } else {
                let description = field_description(field, resolved);
                if let Some(items) = arrays::item_rule(resolved, schema) {
                    arrays::evidence(text, name, description, items)
                        .map(Value::Array)
                        .or_else(|| {
                            let empty = Value::Array(Vec::new());
                            schema::valid(&empty, field, schema).then_some(empty)
                        })
                } else {
                    evidence(
                        name,
                        resolved,
                        description,
                        text,
                        answer,
                        number.as_ref(),
                        key,
                    )
                }
            };
            if value.is_none() && is_required {
                let unknown = Value::String(UNKNOWN.to_owned());
                if schema::valid(&Value::Null, field, schema) {
                    value = Some(Value::Null);
                } else if resolved
                    .get("enum")
                    .and_then(Value::as_array)
                    .is_some_and(|labels| labels.contains(&unknown))
                    && schema::valid(&unknown, field, schema)
                {
                    value = Some(unknown);
                }
            }
            if let Some(value) = value {
                output.insert(name.clone(), value);
            }
        }
    } else if let Some(number) = number {
        output.insert(key.unwrap_or("answer").to_owned(), number);
    }
    (!output.is_empty())
        .then_some(Value::Object(output))
        .filter(satisfies)
}

pub(super) fn answer_from_source(
    format: Option<&Value>,
    chat: bool,
    source: &str,
) -> Option<SymbolicAnswer> {
    let obligation = Obligation::new(format, chat)?;
    let value = obligation
        .schema
        .and_then(|schema| derive(schema, source, ""));
    if value.is_none()
        && obligation.schema.is_some()
        && !serde_json::from_str::<Value>(source).is_ok_and(|value| value.is_object())
    {
        return None;
    }
    Some(SymbolicAnswer {
        intent: "structured_output".into(),
        answer: value.map_or_else(String::new, |value| value.to_string()),
        confidence: 1.0,
        execution_recipe: None,
        evidence_links: Vec::new(),
        thinking_steps: Vec::new(),
        links_notation: String::new(),
    })
}

pub(super) fn project_chat(
    request: &ChatCompletionRequest,
    prompt: &str,
    completion: &mut ChatCompletion,
) -> bool {
    if let Some(obligation) = Obligation::new(request.response_format.as_ref(), true) {
        completion.response_format = Some(obligation.format.clone());
        if let Some(choice) = completion
            .choices
            .first_mut()
            .filter(|choice| choice.finish_reason == "stop" && choice.message.tool_calls.is_empty())
        {
            let message = &mut choice.message;
            let result = obligation.project(
                prompt,
                &message.content.plain_text(),
                message.refusal.take(),
            );
            message
                .thinking_steps
                .push(result.step(&message.thinking_steps));
            completion.usage.completion_tokens =
                estimate_tokens(result.refusal.as_deref().unwrap_or(&result.content));
            completion.usage.total_tokens = completion
                .usage
                .prompt_tokens
                .saturating_add(completion.usage.completion_tokens);
            message.content = MessageContent::Text(result.content);
            message.refusal = result.refusal;
            completion.derivation_identifier = Some(result.identifier);
            completion.evidence_links = result.evidence;
            completion.output_projection = None;
        }
        return true;
    }
    false
}

pub(super) fn project_response(request: &ResponsesRequest, response: &mut ResponseObject) {
    let format = request.text.as_ref().and_then(|text| text.get("format"));
    if let Some(obligation) = Obligation::new(format, false) {
        response.text = Some(serde_json::json!({"format": obligation.format}));
        let source = chat_prompt_and_history(&request.to_chat_completion_request().messages).0;
        if let Some(ResponseOutputItem::Message(message)) = response
            .output
            .iter_mut()
            .find(|item| matches!(item, ResponseOutputItem::Message(_)))
        {
            let answer = message
                .content
                .iter()
                .map(|part| part.text.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            let refusal = message
                .content
                .iter_mut()
                .find_map(|part| part.refusal.take());
            let result = obligation.project(&source, &answer, refusal);
            let returned_text = result.refusal.as_deref().unwrap_or(&result.content);
            response.usage.output_tokens = estimate_tokens(returned_text);
            response.usage.total_tokens = response
                .usage
                .input_tokens
                .saturating_add(response.usage.output_tokens);
            message.id = stable_id("msg", returned_text);
            message
                .thinking_steps
                .push(result.step(&message.thinking_steps));
            message.content = vec![ResponseOutputContent {
                kind: if result.refusal.is_some() {
                    "refusal"
                } else {
                    "output_text"
                }
                .into(),
                text: result.content,
                refusal: result.refusal,
            }];
            response.thinking_steps.clone_from(&message.thinking_steps);
            response.derivation_id = Some(result.identifier);
            let mut evidence = result.evidence;
            evidence.append(&mut response.evidence_links);
            response.evidence_links = evidence;
        }
    }
}
