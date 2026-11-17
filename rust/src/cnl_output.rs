//! Output-only constraints. Accepting text never depends on its grammar.
//! JavaScript root: `js/agentic/crate/cnl_output.mjs`.
pub mod configuration;
mod dialogue;
mod projection;

pub use configuration::{
    OutputConfig, OutputDensity, OutputStandard, frequency_tier, output_configuration,
};
pub use dialogue::{OutputDirective, OutputTurn, output_directive, output_source, output_turn};
pub use projection::{
    OutputCheckers, OutputDocument, OutputPart, OutputReport, RenderedOutput, formalize_output,
    output_links_notation, output_meaning, output_words, project_output, render_output,
    validate_output,
};

use crate::seed::parser::parse_lino;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct Profile {
    pub id: String,
    pub aliases: Vec<String>,
    pub max_words: Option<usize>,
    pub forbidden: Vec<String>,
    pub required: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct OutputLanguage {
    pub id: String,
    pub notice: String,
    pub frequency: String,
    pub primes: Vec<String>,
    pub grammar: Vec<String>,
    pub directives: Vec<String>,
    pub switches: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct OutputForm {
    pub text: String,
    pub mode: String,
}

#[derive(Debug, Clone)]
pub struct OutputLexeme {
    pub id: String,
    pub language: String,
    pub forms: Vec<OutputForm>,
}

#[derive(Debug, Clone, Default)]
pub struct OutputPolicy {
    pub profiles: Vec<Profile>,
    pub languages: Vec<OutputLanguage>,
    pub lexemes: Vec<OutputLexeme>,
}

fn list(value: &str) -> Vec<String> {
    value
        .split('|')
        .filter(|part| !part.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Parse the same policy the worker reads; no vocabulary is embedded in code.
pub fn parse_output_policy(text: &str) -> OutputPolicy {
    let root = parse_lino(text);
    let Some(root) = (if root.name == "controlled-output" {
        Some(&root)
    } else {
        root.children
            .iter()
            .find(|node| node.name == "controlled-output")
    }) else {
        return OutputPolicy::default();
    };
    let mut policy = OutputPolicy::default();
    for node in &root.children {
        match node.name.as_str() {
            "profile" => policy.profiles.push(Profile {
                id: node.id.clone(),
                aliases: list(node.find_child_value("aliases")),
                max_words: node.find_child_value("maximum-words").parse().ok(),
                forbidden: list(node.find_child_value("forbidden")),
                required: list(node.find_child_value("required")),
            }),
            "language" => policy.languages.push(OutputLanguage {
                id: node.id.clone(),
                notice: node.find_child_value("notice").to_owned(),
                frequency: node.find_child_value("frequency").to_owned(),
                primes: list(node.find_child_value("primes")),
                grammar: list(node.find_child_value("grammar")),
                directives: node
                    .children
                    .iter()
                    .filter(|item| item.name == "directive")
                    .map(|item| item.id.clone())
                    .collect(),
                switches: node
                    .children
                    .iter()
                    .filter(|item| item.name == "switch")
                    .map(|item| (item.id.clone(), item.find_child_value("mode").to_owned()))
                    .collect(),
            }),
            "lexeme" => policy.lexemes.push(OutputLexeme {
                id: node.id.clone(),
                language: node.find_child_value("language").to_owned(),
                forms: node
                    .children
                    .iter()
                    .filter(|item| item.name == "form")
                    .map(|item| OutputForm {
                        text: item.id.clone(),
                        mode: item.find_child_value("mode").to_owned(),
                    })
                    .collect(),
            }),
            _ => {}
        }
    }
    policy
}

pub fn cnl_policy() -> &'static OutputPolicy {
    static POLICY: OnceLock<OutputPolicy> = OnceLock::new();
    POLICY
        .get_or_init(|| parse_output_policy(include_str!("../embedded/data/seed/cnl-output.lino")))
}
