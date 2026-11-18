//! Output-only constraints. Accepting text never depends on its grammar.
//! JavaScript root: `js/agentic/crate/cnl_output.mjs`.
pub mod configuration;
mod controlled;
mod dialogue;
pub mod grammar;
mod projection;
mod validation;

pub use configuration::{
    OutputConfig, OutputDensity, OutputStandard, frequency_tier, output_configuration,
};
pub use controlled::{ControlledDialogue, ControlledSolve, controlled_dialogue, controlled_solve};
pub use dialogue::{OutputDirective, OutputTurn, output_directive, output_source, output_turn};
pub use projection::{
    OutputDocument, OutputPart, RenderedOutput, formalize_output, output_candidates,
    output_links_notation, output_meaning, output_words, project_output, render_output,
};
pub use validation::{OutputCheckers, OutputReport, validate_output};

use crate::seed::parser::parse_lino;
use grammar::{CompiledGrammar, SharedPolicy, compile_grammar, parse_language_grammar};
use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};

pub const CNL_POLICY_PATH: &str = "data/seed/cnl-output.lino";

#[derive(Debug, Clone)]
pub struct Profile {
    pub id: String,
    pub aliases: Vec<String>,
    pub max_words: Option<usize>,
    pub density_max_words: Option<usize>,
    pub styles: Vec<String>,
    pub registers: Vec<String>,
    pub density_required: Vec<String>,
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
    /// Frames, facts and checks shared by every language.
    pub shared: Arc<SharedPolicy>,
    /// Compiled grammars of the declared languages whose grammar file exists.
    pub grammars: BTreeMap<String, Arc<CompiledGrammar>>,
}

impl OutputPolicy {
    #[must_use]
    pub fn grammar(&self, language: &str) -> Option<&Arc<CompiledGrammar>> {
        self.grammars.get(language)
    }

    /// Grammars in declared language order (the order prompts are analyzed in).
    #[must_use]
    pub fn ordered_grammars(&self) -> Vec<Arc<CompiledGrammar>> {
        self.languages
            .iter()
            .filter_map(|language| self.grammar(&language.id).cloned())
            .collect()
    }
}

fn list(value: &str) -> Vec<String> {
    value
        .split('|')
        .filter(|part| !part.is_empty())
        .map(str::to_owned)
        .collect()
}

/// JavaScript `Number(text) || null` for a positive word count.
fn positive_count(text: &str) -> Option<usize> {
    text.trim().parse().ok().filter(|count| *count > 0)
}

/// Parse the same policy the worker reads, without language grammars.
pub fn parse_output_policy(text: &str) -> OutputPolicy {
    parse_output_policy_with(text, |_| None)
}

/// Parse the policy; `grammar_text(language)` returns that language's grammar
/// file, or `None` when the language has no grammar yet.
pub fn parse_output_policy_with(
    text: &str,
    grammar_text: impl Fn(&str) -> Option<String>,
) -> OutputPolicy {
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
                max_words: positive_count(node.find_child_value("maximum-words")),
                density_max_words: positive_count(node.find_child_value("density-maximum-words")),
                styles: list(node.find_child_value("styles")),
                registers: list(node.find_child_value("registers")),
                density_required: list(node.find_child_value("density-required")),
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
    let shared = Arc::new(grammar::parse_grammar_policy(root));
    for language in &policy.languages {
        let Some(source) = grammar_text(&language.id).filter(|text| !text.is_empty()) else {
            continue;
        };
        if let Ok(data) = parse_language_grammar(&source) {
            let compiled = compile_grammar(data, Arc::clone(&shared));
            policy
                .grammars
                .insert(language.id.clone(), Arc::new(compiled));
        }
    }
    policy.shared = shared;
    policy
}

/// The embedded seed file at `path`, if the registry has one.
fn seed_text(path: &str) -> Option<String> {
    crate::seed::seed_files()
        .into_iter()
        .find(|(name, _)| *name == path)
        .map(|(_, text)| text.to_owned())
}

/// The shared policy plus every declared language whose grammar file is registered.
pub fn cnl_policy() -> &'static OutputPolicy {
    static POLICY: OnceLock<OutputPolicy> = OnceLock::new();
    POLICY.get_or_init(|| {
        parse_output_policy_with(
            &seed_text(CNL_POLICY_PATH).unwrap_or_default(),
            |language| seed_text(&grammar::CNL_LANGUAGE_PATH.replace("{language}", language)),
        )
    })
}
