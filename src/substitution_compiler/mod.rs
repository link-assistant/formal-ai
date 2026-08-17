//! Rust-owned compiler for executable substitution-rule programs.
//!
//! Rules are lowered once into a target-neutral IR. Rust is the canonical
//! emitter; JavaScript is a generated embedding surface and WebAssembly is
//! compiled from generated Rust, so browser support does not fork semantics.

mod javascript;
mod rust;
mod rust_runtime;
mod webassembly;

use std::fmt;

use serde::Serialize;

use crate::engine::stable_id;
use crate::substitution::{PatternNode, SubstitutionRuleSet};

/// An output language supported by the substitution compiler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubstitutionCompilationTarget {
    Rust,
    JavaScript,
    WebAssembly,
}

impl SubstitutionCompilationTarget {
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::JavaScript => "javascript",
            Self::WebAssembly => "webassembly",
        }
    }
}

impl fmt::Display for SubstitutionCompilationTarget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.slug())
    }
}

/// One generated source or support file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledSubstitutionFile {
    pub name: String,
    pub contents: String,
}

/// A complete, inspectable compilation result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledSubstitutionProgram {
    pub target: SubstitutionCompilationTarget,
    pub ir: SubstitutionProgramIr,
    pub primary_file: CompiledSubstitutionFile,
    pub supporting_files: Vec<CompiledSubstitutionFile>,
    pub trace: String,
}

/// Target-neutral executable representation of a rule set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SubstitutionProgramIr {
    pub id: String,
    pub max_applications: usize,
    pub rules: Vec<SubstitutionRuleIr>,
}

/// One ordered rule in the compiler IR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SubstitutionRuleIr {
    pub id: String,
    pub order: i64,
    pub manual: bool,
    pub conditions: Vec<SubstitutionPatternIr>,
    pub actions: Vec<SubstitutionActionIr>,
}

/// One replace operation in the compiler IR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SubstitutionActionIr {
    pub remove: SubstitutionPatternIr,
    pub add: Vec<SubstitutionPatternIr>,
}

/// A directed link pattern in the compiler IR.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SubstitutionPatternIr {
    pub from: SubstitutionPatternNodeIr,
    pub to: SubstitutionPatternNodeIr,
}

/// A literal, whole-node variable, or prefix variable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SubstitutionPatternNodeIr {
    Literal { value: String },
    Variable { name: String },
    PrefixVariable { prefix: String, name: String },
}

impl SubstitutionProgramIr {
    #[must_use]
    pub fn lower(rules: &SubstitutionRuleSet) -> Self {
        Self {
            id: rules.id.clone(),
            max_applications: crate::substitution::DEFAULT_MAX_APPLICATIONS,
            rules: rules
                .rules
                .iter()
                .map(|rule| SubstitutionRuleIr {
                    id: rule.id.clone(),
                    order: rule.order,
                    manual: rule.events.iter().any(|event| event.as_str() == "manual"),
                    conditions: rule.conditions.iter().map(lower_pattern).collect(),
                    actions: rule
                        .actions
                        .iter()
                        .map(|action| SubstitutionActionIr {
                            remove: lower_pattern(&action.remove),
                            add: action.add.iter().map(lower_pattern).collect(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

/// Lower and emit an executable program for `target`.
#[must_use]
pub fn compile_substitution_rules(
    rules: &SubstitutionRuleSet,
    target: SubstitutionCompilationTarget,
) -> CompiledSubstitutionProgram {
    let ir = SubstitutionProgramIr::lower(rules);
    let stem = safe_file_stem(&ir.id);
    let (primary_file, mut supporting_files) = match target {
        SubstitutionCompilationTarget::Rust => rust::emit(&ir, &stem),
        SubstitutionCompilationTarget::JavaScript => javascript::emit(&ir, &stem),
        SubstitutionCompilationTarget::WebAssembly => webassembly::emit(&ir, &stem),
    };
    supporting_files.push(CompiledSubstitutionFile {
        name: format!("{stem}.substitution-ir.json"),
        contents: serde_json::to_string_pretty(&ir).expect("compiler IR is serializable"),
    });
    let trace = compilation_trace(&ir, target);
    CompiledSubstitutionProgram {
        target,
        ir,
        primary_file,
        supporting_files,
        trace,
    }
}

fn lower_pattern(pattern: &crate::substitution::LinkPattern) -> SubstitutionPatternIr {
    SubstitutionPatternIr {
        from: lower_node(&pattern.from),
        to: lower_node(&pattern.to),
    }
}

fn lower_node(node: &PatternNode) -> SubstitutionPatternNodeIr {
    match node {
        PatternNode::Literal(value) => SubstitutionPatternNodeIr::Literal {
            value: value.clone(),
        },
        PatternNode::Variable(name) => SubstitutionPatternNodeIr::Variable { name: name.clone() },
        PatternNode::PrefixVariable { prefix, variable } => {
            SubstitutionPatternNodeIr::PrefixVariable {
                prefix: prefix.clone(),
                name: variable.clone(),
            }
        }
    }
}

fn safe_file_stem(id: &str) -> String {
    let stem: String = id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' || character == '-' {
                character
            } else {
                '_'
            }
        })
        .collect();
    if stem.is_empty() {
        String::from("substitution_program")
    } else {
        stem
    }
}

fn compilation_trace(ir: &SubstitutionProgramIr, target: SubstitutionCompilationTarget) -> String {
    let serialized_ir = serde_json::to_string(ir).expect("compiler IR is serializable");
    let identity = stable_id(
        "substitution_compilation",
        &format!("{}:{}:{serialized_ir}", ir.id, target.slug()),
    );
    let execution = match target {
        SubstitutionCompilationTarget::Rust => "native_rust",
        SubstitutionCompilationTarget::JavaScript => "javascript_interop_to_rust_wasm",
        SubstitutionCompilationTarget::WebAssembly => "rust_to_wasm",
    };
    format!(
        "substitution_compilation {identity}\n  target {}\n  source_rule_set {}\n  ir_rule_count {}\n  stage lower_to_target_neutral_ir\n  stage emit_from_rust_owned_compiler\n  execution {execution}\n  verification executable_parity_required\n  doctrine javascript_interface_only",
        target.slug(), ir.id, ir.rules.len()
    )
}
