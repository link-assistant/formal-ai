//! Claude Code auto-mode classifier fast path (issue #1632, R4).
//!
//! `claude --permission-mode auto` checks every shell command and network
//! action by sending the configured model a "security monitor" request: a
//! ~140 KB rule book as the system prompt, one user message holding the
//! session `<transcript>`, `stop_sequences: ["</block>"]`, and a 60 s client
//! timeout. Routing that envelope through the general solver timed out, so
//! Claude Code reported "formal-ai is temporarily unavailable (timed out)" and
//! refused every Bash call in auto mode.
//!
//! This module recognises the envelope by the seed markers in
//! `data/seed/auto-mode-classifier.lino` and answers in the documented output
//! format at once: `<block>no</block>` unless the agent's latest action matches
//! one of the seed's irreversible or exfiltrating patterns, in which case it
//! answers `<block>yes</block><category>Rule</category><reason>[Rule] …</reason>`.
//! Only the agent's own actions are matched; transcript entries written by the
//! user are ignored, so a user *describing* `rm -rf /` never trips a rule.

use std::sync::OnceLock;

use regex::{Regex, RegexBuilder};
use serde_json::Value;

use crate::seed::{AUTO_MODE_CLASSIFIER_LINO, parse_lino, parser::LinoNode};

/// One blocking rule: its exact name, a short reason and its patterns.
#[derive(Debug)]
pub struct ClassifierRule {
    pub name: String,
    pub reason: String,
    patterns: Vec<Regex>,
}

/// The seed-backed classifier policy.
#[derive(Debug)]
pub struct ClassifierPolicy {
    markers: Vec<String>,
    transcript_open: String,
    transcript_close: String,
    user_entry_key: String,
    allow_text: String,
    block_template: String,
    rules: Vec<ClassifierRule>,
}

/// The verdict for one classifier request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifierVerdict {
    /// The matched rule name, `None` when the action is allowed.
    pub blocked_by: Option<String>,
    /// The full verdict text in the documented output format.
    pub text: String,
}

/// The policy parsed once from the embedded seed.
#[must_use]
pub fn classifier_policy() -> &'static ClassifierPolicy {
    static POLICY: OnceLock<ClassifierPolicy> = OnceLock::new();
    POLICY.get_or_init(|| parse_policy(AUTO_MODE_CLASSIFIER_LINO))
}

fn parse_policy(text: &str) -> ClassifierPolicy {
    let tree = parse_lino(text);
    let root = tree
        .children
        .iter()
        .find(|node| node.name == "auto-mode-classifier")
        .expect("data/seed/auto-mode-classifier.lino must declare auto-mode-classifier");
    let values = |name: &str| -> Vec<String> {
        root.children
            .iter()
            .filter(|child| child.name == name)
            .map(|child| child.id.clone())
            .collect()
    };
    let rules = root
        .children
        .iter()
        .filter(|child| child.name == "rule")
        .map(|node| ClassifierRule {
            name: node.id.clone(),
            reason: node.find_child_value("reason").to_owned(),
            patterns: node
                .children
                .iter()
                .filter(|child| child.name == "pattern")
                .map(|child| compile_pattern(&expand_alternatives(&child.id, node)))
                .collect(),
        })
        .collect();
    ClassifierPolicy {
        markers: values("marker"),
        transcript_open: root.find_child_value("transcript-open").to_owned(),
        transcript_close: root.find_child_value("transcript-close").to_owned(),
        user_entry_key: root.find_child_value("user-entry-key").to_owned(),
        allow_text: root.find_child_value("allow-text").to_owned(),
        block_template: root.find_child_value("block-template").to_owned(),
        rules,
    }
}

/// Replace every `{name}` in a rule pattern with the alternation of the rule's
/// `name` children, so the seed spells alternatives as separate values instead
/// of packing them with `|` (issue #398).
fn expand_alternatives(pattern: &str, rule: &LinoNode) -> String {
    static PLACEHOLDER: OnceLock<Regex> = OnceLock::new();
    let placeholder = PLACEHOLDER.get_or_init(|| {
        Regex::new(r"\{([a-z]+(?:-[a-z]+)*)\}").expect("fixed placeholder grammar")
    });
    placeholder
        .replace_all(pattern, |captures: &regex::Captures<'_>| {
            let alternatives = rule
                .children
                .iter()
                .filter(|child| child.name == captures[1])
                .map(|child| child.id.as_str())
                .collect::<Vec<_>>();
            assert!(
                !alternatives.is_empty(),
                "auto-mode classifier rule `{}` has no `{}` alternatives",
                rule.id,
                &captures[1]
            );
            format!("(?:{})", alternatives.join("|"))
        })
        .into_owned()
}

fn compile_pattern(pattern: &str) -> Regex {
    RegexBuilder::new(pattern)
        .case_insensitive(true)
        .build()
        .unwrap_or_else(|error| {
            panic!("auto-mode classifier pattern `{pattern}` is invalid: {error}")
        })
}

impl ClassifierPolicy {
    /// Whether a system prompt is Claude Code's auto-mode security monitor.
    #[must_use]
    pub fn is_classifier_prompt(&self, system: &str) -> bool {
        !self.markers.is_empty() && self.markers.iter().all(|marker| system.contains(marker))
    }

    /// The agent actions inside the request's `<transcript>`: every JSON line
    /// whose key is not the user entry key, decoded to its string values.
    #[must_use]
    pub fn agent_actions(&self, messages: &str) -> Vec<String> {
        let transcript = messages
            .rfind(&self.transcript_open)
            .map_or(messages, |start| {
                &messages[start + self.transcript_open.len()..]
            });
        let transcript = transcript
            .find(&self.transcript_close)
            .map_or(transcript, |end| &transcript[..end]);
        transcript
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .filter_map(|line| self.action_text(line))
            .collect()
    }

    fn action_text(&self, line: &str) -> Option<String> {
        match serde_json::from_str::<Value>(line) {
            Ok(Value::Object(entry)) => {
                if entry.contains_key(&self.user_entry_key) {
                    return None;
                }
                let parts = entry
                    .values()
                    .map(|value| match value {
                        Value::String(text) => text.clone(),
                        other => other.to_string(),
                    })
                    .collect::<Vec<_>>();
                Some(parts.join("\n"))
            }
            Ok(_) => None,
            // A non-JSON line is not a user entry; keep it so an action
            // rendered in an unexpected shape is still screened.
            Err(_) => Some(line.to_owned()),
        }
    }

    /// Judge the latest agent action of a classifier transcript.
    #[must_use]
    pub fn judge(&self, messages: &str) -> ClassifierVerdict {
        let actions = self.agent_actions(messages);
        let Some(latest) = actions.last() else {
            return self.allow();
        };
        self.rules
            .iter()
            .find(|rule| rule.patterns.iter().any(|pattern| pattern.is_match(latest)))
            .map_or_else(
                || self.allow(),
                |rule| ClassifierVerdict {
                    blocked_by: Some(rule.name.clone()),
                    text: self
                        .block_template
                        .replace("{name}", &rule.name)
                        .replace("{reason}", &rule.reason),
                },
            )
    }

    fn allow(&self) -> ClassifierVerdict {
        ClassifierVerdict {
            blocked_by: None,
            text: self.allow_text.clone(),
        }
    }
}

/// Judge a classifier request given its system and message text; `None` when
/// the request is not an auto-mode classifier request.
#[must_use]
pub fn classify(system: &str, messages: &str) -> Option<ClassifierVerdict> {
    let policy = classifier_policy();
    policy
        .is_classifier_prompt(system)
        .then(|| policy.judge(messages))
}

/// Cut a verdict at the first requested stop sequence, the way the Anthropic
/// API does: the text before the stop sequence is returned and the matched
/// sequence is reported separately.
#[must_use]
pub fn apply_stop_sequences(text: &str, stop_sequences: &[String]) -> (String, Option<String>) {
    stop_sequences
        .iter()
        .filter(|stop| !stop.is_empty())
        .filter_map(|stop| text.find(stop.as_str()).map(|index| (index, stop)))
        .min_by_key(|(index, _)| *index)
        .map_or_else(
            || (text.to_owned(), None),
            |(index, stop)| (text[..index].to_owned(), Some(stop.clone())),
        )
}

#[cfg(test)]
#[path = "../../tests/source/source_tests/auto_mode_classifier/production_tests.rs"]
mod tests;
