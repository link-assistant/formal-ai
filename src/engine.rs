use lino_objects_codec::{encode, LinoValue};
use serde::{Deserialize, Serialize};

pub const DEFAULT_MODEL: &str = "formal-symbolic-poc";

const GREETING_ANSWER: &str = "Hi, how may I help you?";
const RUST_HELLO_WORLD_ANSWER: &str = r#"Here is a minimal Rust hello world program:

```rust
fn main() {
    println!("Hello, world!");
}
```"#;
const UNKNOWN_ANSWER: &str = "I do not have a learned symbolic rule for that prompt yet. Add a Links Notation fact or rule, then run the request again.";

const GREETING_EXAMPLES: &[&str] = &["Hi", "Hello", "Hey"];
const RUST_HELLO_WORLD_EXAMPLES: &[&str] = &[
    "Write me hello world program in Rust",
    "hello world in rust",
    "Create a Rust hello world example",
];
const UNKNOWN_EXAMPLES: &[&str] = &["Any prompt without a matching symbolic rule"];

struct Rule {
    intent: &'static str,
    response_link: &'static str,
    answer: &'static str,
    examples: &'static [&'static str],
    source: &'static str,
}

const RULES: &[Rule] = &[
    Rule {
        intent: "greeting",
        response_link: "response:greeting",
        answer: GREETING_ANSWER,
        examples: GREETING_EXAMPLES,
        source: "local symbolic seed set",
    },
    Rule {
        intent: "hello_world_rust",
        response_link: "response:hello_world:rust",
        answer: RUST_HELLO_WORLD_ANSWER,
        examples: RUST_HELLO_WORLD_EXAMPLES,
        source: "Hello World Collection inspired seed",
    },
    Rule {
        intent: "unknown",
        response_link: "response:unknown",
        answer: UNKNOWN_ANSWER,
        examples: UNKNOWN_EXAMPLES,
        source: "fallback symbolic rule",
    },
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymbolicAnswer {
    pub intent: String,
    pub answer: String,
    pub confidence: f32,
    pub evidence_links: Vec<String>,
    pub links_notation: String,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FormalAiEngine;

impl FormalAiEngine {
    #[must_use]
    pub fn answer(&self, prompt: &str) -> SymbolicAnswer {
        let normalized = normalize_prompt(prompt);
        let rule = select_rule(&normalized);
        let evidence_links = vec![
            format!("prompt:{}", stable_id("prompt", prompt)),
            String::from(rule.response_link),
            format!("intent:{}", rule.intent),
        ];
        let confidence = if rule.intent == "unknown" { 0.0 } else { 1.0 };
        let answer = String::from(rule.answer);
        let links_notation = answer_links_notation(prompt, rule.intent, &answer, &evidence_links);

        SymbolicAnswer {
            intent: String::from(rule.intent),
            answer,
            confidence,
            evidence_links,
            links_notation,
        }
    }
}

#[must_use]
pub fn knowledge_links_notation() -> String {
    encode(&LinoValue::object([
        ("model", LinoValue::String(String::from(DEFAULT_MODEL))),
        (
            "policy",
            LinoValue::String(String::from(
                "deterministic symbolic rules; no neural network inference",
            )),
        ),
        (
            "rules",
            LinoValue::array(RULES.iter().map(rule_to_lino_value)),
        ),
    ]))
}

#[must_use]
pub fn estimate_tokens(text: &str) -> u32 {
    u32::try_from(text.split_whitespace().count()).unwrap_or(u32::MAX)
}

#[must_use]
pub fn stable_id(prefix: &str, text: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    format!("{prefix}_{hash:016x}")
}

fn select_rule(normalized_prompt: &str) -> &'static Rule {
    if is_greeting(normalized_prompt) {
        &RULES[0]
    } else if is_rust_hello_world_request(normalized_prompt) {
        &RULES[1]
    } else {
        &RULES[2]
    }
}

fn is_greeting(normalized_prompt: &str) -> bool {
    matches!(normalized_prompt, "hi" | "hello" | "hey")
}

fn is_rust_hello_world_request(normalized_prompt: &str) -> bool {
    let tokens: Vec<&str> = normalized_prompt.split_whitespace().collect();
    tokens.contains(&"rust") && tokens.contains(&"hello") && tokens.contains(&"world")
}

fn normalize_prompt(prompt: &str) -> String {
    let mut normalized = String::with_capacity(prompt.len());

    for character in prompt.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            normalized.push(character);
        } else {
            normalized.push(' ');
        }
    }

    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn answer_links_notation(
    prompt: &str,
    intent: &str,
    answer: &str,
    evidence_links: &[String],
) -> String {
    encode(&LinoValue::object([
        ("prompt", LinoValue::String(String::from(prompt))),
        ("intent", LinoValue::String(String::from(intent))),
        ("answer", LinoValue::String(String::from(answer))),
        (
            "evidence_links",
            LinoValue::array(
                evidence_links
                    .iter()
                    .map(|link| LinoValue::String(link.clone())),
            ),
        ),
    ]))
}

fn rule_to_lino_value(rule: &Rule) -> LinoValue {
    LinoValue::object([
        ("intent", LinoValue::String(String::from(rule.intent))),
        (
            "response_link",
            LinoValue::String(String::from(rule.response_link)),
        ),
        ("answer", LinoValue::String(String::from(rule.answer))),
        (
            "examples",
            LinoValue::array(
                rule.examples
                    .iter()
                    .map(|example| LinoValue::String(String::from(*example))),
            ),
        ),
        ("source", LinoValue::String(String::from(rule.source))),
    ])
}
