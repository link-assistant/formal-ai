//! Request-local constraints and dialogue facts. JavaScript twin: request-context.mjs.
//! Grammar/prose come from shared data, never from a persistent standing rule.
use std::sync::LazyLock;

use regex::{Regex, RegexBuilder};
use serde_json::Value;

use super::{ChatMessage, chat_prompt_and_history};
use crate::engine::{SymbolicAnswer, ThinkingStep};
use crate::solver::{ConversationRole, ConversationTurn, UniversalSolver};

static RULES: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!(
        "../../embedded/data/meta/request-context.json"
    ))
    .unwrap()
});

fn pattern(text: &str) -> Regex {
    RegexBuilder::new(text)
        .case_insensitive(true)
        .build()
        .unwrap()
}

fn capture(name: &str, text: &str) -> Option<Vec<String>> {
    RULES[name].as_array()?.iter().find_map(|rule| {
        pattern(rule.as_str()?).captures(text).map(|found| {
            found
                .iter()
                .skip(1)
                .map(|value| value.unwrap().as_str().to_owned())
                .collect()
        })
    })
}

fn language(value: &str) -> Option<&'static str> {
    RULES["languages"]
        .as_object()?
        .iter()
        .find_map(|(code, aliases)| {
            aliases
                .as_array()?
                .iter()
                .any(|alias| alias.as_str() == Some(value))
                .then_some(code.as_str())
        })
}

#[derive(Clone)]
struct Constraint {
    role: String,
    index: usize,
    text: String,
    kind: &'static str,
    value: String,
    status: &'static str,
}

pub(super) struct RequestContext {
    prompt: String,
    history: Vec<ConversationTurn>,
    constraints: Vec<Constraint>,
}

impl RequestContext {
    pub(super) fn solver(&self, solver: &UniversalSolver) -> UniversalSolver {
        let mut scoped = *solver;
        if let Some(language) = self.language() {
            scoped.config.forced_response_language = Some(language);
        }
        scoped
    }
    pub(super) fn new(messages: &[ChatMessage]) -> Self {
        let (prompt, history) = chat_prompt_and_history(messages);
        let mut constraints = Vec::new();
        for (index, message) in messages.iter().enumerate() {
            let role = message.role.to_lowercase();
            if !["system", "developer"].contains(&role.as_str()) {
                continue;
            }
            let text = message.plain_text();
            for text in pattern(RULES["clauseSeparator"].as_str().unwrap()).split(&text) {
                let text = text.trim();
                if text.is_empty() {
                    continue;
                }
                if pattern(RULES["negated"].as_str().unwrap()).is_match(text)
                    && (pattern(RULES["outputCue"].as_str().unwrap()).is_match(text)
                        || ["language", "format", "labels", "length"]
                            .iter()
                            .any(|kind| capture(kind, text).is_some()))
                {
                    constraints.push(Constraint {
                        role: role.clone(),
                        index,
                        text: text.to_owned(),
                        kind: "output",
                        value: text.to_owned(),
                        status: "pending",
                    });
                    continue;
                }
                let mut recognized = false;
                let mut covered = Vec::new();
                for kind in ["language", "format", "labels", "length"] {
                    let Some(found) = capture(kind, text) else {
                        continue;
                    };
                    for rule in RULES[kind].as_array().unwrap() {
                        if let Some(found) = pattern(rule.as_str().unwrap()).find(text) {
                            covered.push(found.range());
                            break;
                        }
                    }
                    let mut value = found[0].to_lowercase();
                    if kind == "language"
                        && RULES["formats"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|format| format.as_str() == Some(&value))
                    {
                        continue;
                    }
                    if kind == "labels" {
                        value.push('|');
                        value.push_str(&found[1].to_lowercase());
                    }
                    if kind == "length" {
                        let count = RULES["numberWords"][found[1].to_lowercase()]
                            .as_u64()
                            .map_or_else(|| found[1].clone(), |count| count.to_string());
                        value = format!("{value}|{count}");
                    }
                    constraints.push(Constraint {
                        role: role.clone(),
                        index,
                        text: text.to_owned(),
                        kind,
                        value,
                        status: "pending",
                    });
                    recognized = true;
                }
                let residue: String = text
                    .char_indices()
                    .map(|(index, character)| {
                        if covered.iter().any(|range| range.contains(&index)) {
                            ' '
                        } else {
                            character
                        }
                    })
                    .collect();
                if (!recognized && pattern(RULES["outputCue"].as_str().unwrap()).is_match(text))
                    || (recognized
                        && !pattern(RULES["ignorableResidue"].as_str().unwrap()).is_match(&residue))
                {
                    constraints.push(Constraint {
                        role: role.clone(),
                        index,
                        text: text.to_owned(),
                        kind: "output",
                        value: text.to_owned(),
                        status: "pending",
                    });
                }
            }
        }
        for index in 0..constraints.len() {
            let winner = (0..constraints.len())
                .filter(|other| constraints[*other].kind == constraints[index].kind)
                .max_by_key(|other| {
                    (
                        constraints[*other].role == "system",
                        constraints[*other].index,
                        *other,
                    )
                })
                .unwrap();
            if winner != index {
                constraints[index].status = "superseded";
            }
        }
        Self {
            prompt,
            history,
            constraints,
        }
    }

    fn active(&self, kind: &str) -> Option<&Constraint> {
        self.constraints
            .iter()
            .find(|item| item.kind == kind && item.status != "superseded")
    }

    pub(super) fn language(&self) -> Option<&'static str> {
        self.active("language")
            .and_then(|item| language(&item.value))
    }

    pub(super) fn answer(&self) -> Option<SymbolicAnswer> {
        if let Some(recall) = capture("recall", &self.prompt) {
            for (index, turn) in self.history.iter().enumerate().rev() {
                if turn.role != ConversationRole::User {
                    continue;
                }
                let sentences: Vec<_> = pattern(RULES["factSeparator"].as_str().unwrap())
                    .split(turn.content.trim())
                    .map(str::to_owned)
                    .collect();
                for sentence in sentences.iter().rev() {
                    if let Some(fact) =
                        pattern(RULES["assertion"].as_str().unwrap()).captures(sentence.trim())
                    {
                        if subject(&fact[1]) == subject(&recall[0]) {
                            let mut answer = answer(&fact[2], "dialogue_fact");
                            let detail = format!("user[{index}]:{}={}", &fact[1], &fact[2]);
                            answer.evidence_links.push(format!("request_fact:{detail}"));
                            answer.thinking_steps.push(ThinkingStep::new(
                                0,
                                "request_fact",
                                detail,
                                "high",
                                "request_context",
                            ));
                            return Some(answer);
                        }
                    }
                }
            }
            return Some(answer(
                RULES["missingFact"].as_str().unwrap(),
                "dialogue_fact_missing",
            ));
        }
        let labels: Vec<_> = RULES["binaryLabels"]
            .as_array()
            .unwrap()
            .iter()
            .map(|label| label.as_str().unwrap())
            .collect();
        if self
            .active("labels")
            .is_some_and(|item| item.value == labels.join("|"))
        {
            if let Some(found) =
                pattern(RULES["comparison"].as_str().unwrap()).captures(&self.prompt)
            {
                let comparison = compare_decimals(&found[1], &found[3]);
                let result = match RULES["relations"][found[2].to_lowercase()].as_str()? {
                    "gt" => comparison.is_gt(),
                    "lt" => comparison.is_lt(),
                    _ => comparison.is_eq(),
                };
                let mut answer = answer(labels[usize::from(!result)], "request_comparison");
                let detail = format!(
                    "{}:{}:{}:{result}",
                    &found[1],
                    RULES["relations"][found[2].to_lowercase()].as_str()?,
                    &found[3]
                );
                answer.thinking_steps.push(ThinkingStep::new(
                    0,
                    "request_comparison",
                    detail,
                    "high",
                    "request_context",
                ));
                return Some(answer);
            }
        }
        None
    }

    pub(super) fn compose(mut self, mut answer: SymbolicAnswer) -> SymbolicAnswer {
        if self.constraints.is_empty() {
            return answer;
        }
        let mut unmet = Vec::new();
        let target = self.language();
        let language_text = answer.answer.clone();
        if self
            .active("format")
            .is_some_and(|item| Some(item.value.as_str()) == RULES["supportedFormat"].as_str())
        {
            let mut object = serde_json::Map::new();
            object.insert(
                RULES["jsonAnswerKey"].as_str().unwrap().to_owned(),
                Value::String(answer.answer),
            );
            answer.answer = Value::Object(object).to_string();
        }
        for item in &mut self.constraints {
            if item.status == "superseded" {
                continue;
            }
            let satisfied = match item.kind {
                "language" => {
                    language(&item.value).is_some()
                        && (target != Some("en")
                            || !pattern(r"[\p{Cyrillic}\p{Han}\p{Devanagari}]")
                                .is_match(&language_text))
                        && (target != Some("ru")
                            || pattern(r"\p{Cyrillic}").is_match(&language_text)
                            || !pattern(r"\p{L}").is_match(&language_text))
                }
                "labels" => item
                    .value
                    .split('|')
                    .any(|label| label == answer.answer.to_lowercase()),
                "length" => {
                    let (mode, count) = item.value.split_once('|').unwrap();
                    let length = pattern(RULES["word"].as_str().unwrap())
                        .find_iter(&answer.answer)
                        .count();
                    let comparison = compare_decimals(&length.to_string(), count);
                    if mode == "exactly" {
                        comparison.is_eq()
                    } else {
                        !comparison.is_gt()
                    }
                }
                "format" => Some(item.value.as_str()) == RULES["supportedFormat"].as_str(),
                _ => false,
            };
            item.status = if satisfied { "applied" } else { "unmet" };
            if !satisfied {
                unmet.push(
                    RULES["refusal"]
                        .as_str()
                        .unwrap()
                        .replace("{kind}", item.kind)
                        .replace("{text}", &item.text),
                );
            }
        }
        if !unmet.is_empty() {
            let json = self
                .active("format")
                .is_some_and(|item| Some(item.value.as_str()) == RULES["supportedFormat"].as_str());
            let refuse = |unmet: &[String]| {
                let text = unmet.join(" ");
                if json {
                    let mut object = serde_json::Map::new();
                    object.insert(
                        RULES["jsonAnswerKey"].as_str().unwrap().to_owned(),
                        Value::String(text),
                    );
                    Value::Object(object).to_string()
                } else {
                    text
                }
            };
            for item in self
                .constraints
                .iter_mut()
                .filter(|item| item.kind == "labels" && item.status == "applied")
            {
                item.status = "unmet";
                unmet.push(
                    RULES["refusal"]
                        .as_str()
                        .unwrap()
                        .replace("{kind}", item.kind)
                        .replace("{text}", &item.text),
                );
            }
            for item in self.constraints.iter_mut().filter(|item| {
                item.kind == "language"
                    && item.status == "applied"
                    && language(&item.value) != RULES["refusalLanguage"].as_str()
            }) {
                item.status = "unmet";
                unmet.push(
                    RULES["refusal"]
                        .as_str()
                        .unwrap()
                        .replace("{kind}", item.kind)
                        .replace("{text}", &item.text),
                );
            }
            answer.answer = refuse(&unmet);
            for item in self
                .constraints
                .iter_mut()
                .filter(|item| item.kind == "length" && item.status == "applied")
            {
                let (mode, count) = item.value.split_once('|').unwrap();
                let words = pattern(RULES["word"].as_str().unwrap())
                    .find_iter(&answer.answer)
                    .count();
                let comparison = compare_decimals(&words.to_string(), count);
                if if mode == "exactly" {
                    !comparison.is_eq()
                } else {
                    comparison.is_gt()
                } {
                    item.status = "unmet";
                    unmet.push(
                        RULES["refusal"]
                            .as_str()
                            .unwrap()
                            .replace("{kind}", item.kind)
                            .replace("{text}", &item.text),
                    );
                    answer.answer = refuse(&unmet);
                }
            }
            answer.intent = "request_constraint_refusal".to_owned();
        }
        for item in &self.constraints {
            let detail = format!(
                "{}[{}]:{}:{}:{}",
                item.role, item.index, item.kind, item.status, item.text
            );
            answer.thinking_steps.push(ThinkingStep::new(
                answer.thinking_steps.len() as u32,
                "request_constraint",
                detail,
                "high",
                "request_context",
            ));
        }
        answer.links_notation.push_str(&format!(
            "\n({} ({} {}))",
            "request_context",
            "answer",
            serde_json::to_string(&answer.answer).unwrap()
        ));
        answer
    }
}

pub(super) fn compose_final(
    messages: &[ChatMessage],
    text: &str,
    steps: Vec<ThinkingStep>,
) -> SymbolicAnswer {
    let mut answer = answer(text, "agentic_final");
    answer.thinking_steps = steps;
    RequestContext::new(messages).compose(answer)
}

pub(super) fn final_message(
    messages: &[ChatMessage],
    text: &str,
    steps: Vec<ThinkingStep>,
) -> ChatMessage {
    let composed = compose_final(messages, text, steps);
    let mut message = ChatMessage::assistant(composed.answer);
    if !composed.thinking_steps.is_empty() {
        message.reasoning = crate::engine::render_thinking_steps(&composed.thinking_steps);
        message.reasoning_content.clone_from(&message.reasoning);
        message.thinking_steps = composed.thinking_steps;
    }
    message
}

fn subject(text: &str) -> String {
    text.to_lowercase()
        .split_whitespace()
        .map(|word| RULES["subjectAliases"][word].as_str().unwrap_or(word))
        .filter(|word| {
            !RULES["subjectModifiers"]
                .as_array()
                .unwrap()
                .iter()
                .any(|modifier| modifier.as_str() == Some(word))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn compare_decimals(left: &str, right: &str) -> std::cmp::Ordering {
    fn parts(value: &str) -> (bool, &str, &str) {
        let clean = value.strip_prefix('-').unwrap_or(value);
        let (whole, fraction) = clean.split_once('.').unwrap_or((clean, ""));
        let whole = whole.trim_start_matches('0');
        let fraction = fraction.trim_end_matches('0');
        (
            value.starts_with('-') && (!whole.is_empty() || !fraction.is_empty()),
            whole,
            fraction,
        )
    }
    let (negative_a, whole_a, fraction_a) = parts(left);
    let (negative_b, whole_b, fraction_b) = parts(right);
    if negative_a != negative_b {
        return negative_b.cmp(&negative_a);
    }
    let width = fraction_a.len().max(fraction_b.len());
    let pad = |fraction: &str| format!("{fraction}{}", "0".repeat(width - fraction.len()));
    let result = whole_a
        .len()
        .cmp(&whole_b.len())
        .then_with(|| whole_a.cmp(whole_b))
        .then_with(|| pad(fraction_a).cmp(&pad(fraction_b)));
    if negative_a { result.reverse() } else { result }
}

fn answer(text: &str, intent: &str) -> SymbolicAnswer {
    SymbolicAnswer {
        answer: text.to_owned(),
        intent: intent.to_owned(),
        confidence: 1.0,
        thinking_steps: Vec::new(),
        evidence_links: Vec::new(),
        links_notation: format!(
            "({} ({} {intent}) ({} {}))",
            "request_context",
            "intent",
            "answer",
            serde_json::to_string(text).unwrap()
        ),
        execution_recipe: None,
    }
}
