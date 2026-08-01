//! Seeded natural-language memory-program compiler (issue #708).
//!
//! This extends the issue-#674 procedure-compiler discipline to associative
//! memory queries: language surfaces and ordered steps are data, compilation is
//! all-or-nothing, and canonical identity excludes the source language. The
//! closed primitive set is intentionally small; iteration and matching are
//! always bounded by [`MemoryProgramLimits`].

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::fmt::Write as _;
use std::sync::OnceLock;

use lino_objects_codec::format::parse_indented;

use crate::engine::stable_id;
use crate::links_format::push_lino_node;

mod execution;
pub use execution::{
    execute_memory_program, MemoryProgramAuthorization, MemoryProgramHalt, MemoryProgramOutcome,
};

const MEMORY_PROGRAMS_LINO: &str = include_str!("../data/seed/memory-programs.lino");

/// Explicit resource bounds carried by every compiled program and trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryProgramLimits {
    pub max_matches: usize,
    pub max_iterations: usize,
}

impl MemoryProgramLimits {
    /// Derive query bounds from the solver's recursive decomposition budget.
    #[must_use]
    pub fn from_decomposition_depth(max_decomposition_depth: u8) -> Self {
        let depth = usize::from(max_decomposition_depth.max(1));
        Self {
            max_matches: depth.saturating_mul(32),
            max_iterations: depth,
        }
    }
}

impl Default for MemoryProgramLimits {
    fn default() -> Self {
        Self::from_decomposition_depth(4)
    }
}

/// Permission class attached to a primitive by the seed catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemoryProgramPermission {
    Read,
    Write,
    Destructive,
}

impl MemoryProgramPermission {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Destructive => "destructive",
        }
    }
}

/// One primitive invocation after template bindings have been substituted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryProgramStep {
    pub primitive: String,
    pub permission: MemoryProgramPermission,
    pub arguments: BTreeMap<String, String>,
}

/// Language-independent executable memory program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledMemoryProgram {
    pub id: String,
    pub family: String,
    pub limits: MemoryProgramLimits,
    pub steps: Vec<MemoryProgramStep>,
    pub bindings: BTreeMap<String, String>,
    canonical_program: String,
}

impl CompiledMemoryProgram {
    #[must_use]
    pub fn primitive_names(&self) -> Vec<&str> {
        self.steps
            .iter()
            .map(|step| step.primitive.as_str())
            .collect()
    }

    #[must_use]
    pub fn required_permissions(&self) -> BTreeSet<MemoryProgramPermission> {
        self.steps.iter().map(|step| step.permission).collect()
    }

    #[must_use]
    pub fn canonical_program(&self) -> &str {
        &self.canonical_program
    }

    /// Reviewable Links Notation used verbatim in the compilation trace.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::new();
        push_lino_node(&mut out, 0, "memory_program", None);
        push_lino_node(&mut out, 2, "id", Some(&self.id));
        push_lino_node(&mut out, 2, "family", Some(&self.family));
        let _ = writeln!(out, "  max_matches {}", self.limits.max_matches);
        let _ = writeln!(out, "  max_iterations {}", self.limits.max_iterations);
        for (name, value) in &self.bindings {
            push_lino_node(&mut out, 2, "binding", Some(name));
            push_lino_node(&mut out, 4, "value", Some(value));
        }
        for (index, step) in self.steps.iter().enumerate() {
            let _ = writeln!(out, "  step {}", index + 1);
            push_lino_node(&mut out, 4, "primitive", Some(&step.primitive));
            push_lino_node(&mut out, 4, "permission", Some(step.permission.as_str()));
            for (name, value) in &step.arguments {
                push_lino_node(&mut out, 4, name, Some(value));
            }
        }
        if let Some(step) = self.steps.iter().find(|step| step.primitive == "update") {
            if let (Some(old), Some(new)) = (step.arguments.get("old"), step.arguments.get("new")) {
                push_lino_node(&mut out, 2, "replace", None);
                push_lino_node(&mut out, 4, "old", Some(old));
                push_lino_node(&mut out, 4, "new", Some(new));
            }
        }
        let first_effect = self.steps.iter().find(|step| {
            matches!(
                step.primitive.as_str(),
                "create" | "update" | "delete_with_retraction"
            )
        });
        if let (Some(condition), Some(effect)) = (self.steps.first(), first_effect) {
            push_lino_node(&mut out, 2, "when", Some(&condition.primitive));
            push_lino_node(&mut out, 4, "do", Some(&effect.primitive));
        }
        out.trim_end().to_owned()
    }
}

/// Why a natural-language request did not become a complete program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryProgramCompileError {
    NotMemoryProgram,
    ProgramGap { request: String, gap: String },
}

impl fmt::Display for MemoryProgramCompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotMemoryProgram => formatter.write_str("prompt is not a memory program"),
            Self::ProgramGap { gap, .. } => formatter.write_str(gap),
        }
    }
}

impl Error for MemoryProgramCompileError {}

/// A malformed or capability-escalating serialized memory program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryProgramParseError {
    pub message: String,
}

impl fmt::Display for MemoryProgramParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for MemoryProgramParseError {}

/// Parse the reviewable [`CompiledMemoryProgram::links_notation`] shape.
///
/// The stable id is recomputed and primitive permissions are checked against
/// the seed catalog, so editing arguments or order is supported while editing
/// a permission label cannot escalate a program.
pub fn parse_memory_program_links_notation(
    text: &str,
) -> Result<CompiledMemoryProgram, MemoryProgramParseError> {
    parse_indented(text).map_err(|error| memory_program_parse_error(format!("{error:?}")))?;
    let mut family = String::new();
    let mut max_matches = None;
    let mut max_iterations = None;
    let mut bindings = BTreeMap::new();
    let mut steps = Vec::new();
    let mut replacement = BTreeMap::new();
    let mut when_primitive = None;
    let mut do_primitive = None;
    let mut section = ParseSection::Root;

    for line in text.lines().skip(1) {
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim();
        if indent == 2 {
            if let ParseSection::Step(step) = std::mem::replace(&mut section, ParseSection::Root) {
                steps.push(step);
            }
            if trimmed == "replace" {
                section = ParseSection::Replace;
                continue;
            }
            let Some((name, value)) = trimmed.split_once(' ') else {
                continue;
            };
            match name {
                "family" => family = parse_lino_scalar(value),
                "max_matches" => max_matches = value.parse().ok(),
                "max_iterations" => max_iterations = value.parse().ok(),
                "binding" => section = ParseSection::Binding(parse_lino_scalar(value)),
                "step" => {
                    section = ParseSection::Step(MemoryProgramStep {
                        primitive: String::new(),
                        permission: MemoryProgramPermission::Read,
                        arguments: BTreeMap::new(),
                    });
                }
                "when" => {
                    when_primitive = Some(parse_lino_scalar(value));
                    section = ParseSection::When;
                }
                _ => {}
            }
        } else if indent == 4 {
            let Some((name, value)) = trimmed.split_once(' ') else {
                continue;
            };
            let value = parse_lino_scalar(value);
            match &mut section {
                ParseSection::Binding(binding) if name == "value" => {
                    bindings.insert(binding.clone(), value);
                }
                ParseSection::Step(step) if name == "primitive" => step.primitive = value,
                ParseSection::Step(step) if name == "permission" => {
                    step.permission = parse_permission(&value)?;
                }
                ParseSection::Step(step) => {
                    step.arguments.insert(name.to_owned(), value);
                }
                ParseSection::Replace if matches!(name, "old" | "new") => {
                    replacement.insert(name.to_owned(), value);
                }
                ParseSection::When if name == "do" => do_primitive = Some(value),
                ParseSection::Root
                | ParseSection::Binding(_)
                | ParseSection::Replace
                | ParseSection::When => {}
            }
        }
    }
    if let ParseSection::Step(step) = section {
        steps.push(step);
    }

    let limits = MemoryProgramLimits {
        max_matches: max_matches
            .ok_or_else(|| memory_program_parse_error("missing max_matches"))?,
        max_iterations: max_iterations
            .ok_or_else(|| memory_program_parse_error("missing max_iterations"))?,
    };
    if family.is_empty()
        || steps.is_empty()
        || limits.max_matches == 0
        || limits.max_iterations == 0
    {
        return Err(memory_program_parse_error(
            "a memory program needs a family, steps, and non-zero bounds",
        ));
    }
    for step in &steps {
        let Some(seed_permission) = catalog().primitives.get(&step.primitive) else {
            return Err(memory_program_parse_error(format!(
                "program_gap:unseeded_memory_primitive:{}",
                step.primitive
            )));
        };
        if seed_permission != &step.permission {
            return Err(memory_program_parse_error(format!(
                "memory_program_permission_mismatch:{}",
                step.primitive
            )));
        }
    }
    apply_reviewable_shapes(
        &mut steps,
        &replacement,
        when_primitive.as_deref(),
        do_primitive.as_deref(),
    )?;
    let canonical_program = canonical_program(&family, limits, &bindings, &steps);
    Ok(CompiledMemoryProgram {
        id: stable_id("memory_program", &canonical_program),
        family,
        limits,
        steps,
        bindings,
        canonical_program,
    })
}

enum ParseSection {
    Root,
    Binding(String),
    Step(MemoryProgramStep),
    Replace,
    When,
}

fn apply_reviewable_shapes(
    steps: &mut [MemoryProgramStep],
    replacement: &BTreeMap<String, String>,
    when_primitive: Option<&str>,
    do_primitive: Option<&str>,
) -> Result<(), MemoryProgramParseError> {
    if !replacement.is_empty() {
        let old = replacement
            .get("old")
            .ok_or_else(|| memory_program_parse_error("memory_program_replace_missing_old"))?;
        let new = replacement
            .get("new")
            .ok_or_else(|| memory_program_parse_error("memory_program_replace_missing_new"))?;
        let update = steps
            .iter_mut()
            .find(|step| step.primitive == "update")
            .ok_or_else(|| memory_program_parse_error("memory_program_replace_missing_update"))?;
        update.arguments.insert(String::from("old"), old.clone());
        update.arguments.insert(String::from("new"), new.clone());
    }

    if let Some(primitive) = when_primitive {
        let first = steps
            .first_mut()
            .ok_or_else(|| memory_program_parse_error("memory_program_when_missing_condition"))?;
        primitive.clone_into(&mut first.primitive);
        first.permission = seeded_permission(primitive)?;
    }
    if let Some(primitive) = do_primitive {
        let effect = steps
            .iter_mut()
            .find(|step| is_effect_primitive(&step.primitive))
            .ok_or_else(|| memory_program_parse_error("memory_program_when_missing_effect"))?;
        primitive.clone_into(&mut effect.primitive);
        effect.permission = seeded_permission(primitive)?;
    }
    Ok(())
}

fn seeded_permission(primitive: &str) -> Result<MemoryProgramPermission, MemoryProgramParseError> {
    catalog().primitives.get(primitive).copied().ok_or_else(|| {
        memory_program_parse_error(format!("program_gap:unseeded_memory_primitive:{primitive}"))
    })
}

fn is_effect_primitive(primitive: &str) -> bool {
    matches!(primitive, "create" | "update" | "delete_with_retraction")
}

fn parse_lino_scalar(value: &str) -> String {
    crate::memory::parse_quoted(value).unwrap_or_else(|| value.trim().to_owned())
}

fn parse_permission(value: &str) -> Result<MemoryProgramPermission, MemoryProgramParseError> {
    match value {
        "read" => Ok(MemoryProgramPermission::Read),
        "write" => Ok(MemoryProgramPermission::Write),
        "destructive" => Ok(MemoryProgramPermission::Destructive),
        _ => Err(memory_program_parse_error(format!(
            "memory_program_unknown_permission:{value}"
        ))),
    }
}

fn memory_program_parse_error(message: impl Into<String>) -> MemoryProgramParseError {
    MemoryProgramParseError {
        message: message.into(),
    }
}

#[derive(Debug)]
struct Catalog {
    primitives: BTreeMap<String, MemoryProgramPermission>,
    cues: Vec<String>,
    scopes: Vec<String>,
    families: Vec<Family>,
}

#[derive(Debug, Default)]
struct Family {
    id: String,
    steps: Vec<String>,
    templates: Vec<String>,
}

/// Compile a request using only reviewed templates and steps in the seed.
pub fn compile_memory_program(
    request: &str,
    limits: MemoryProgramLimits,
) -> Result<CompiledMemoryProgram, MemoryProgramCompileError> {
    let catalog = catalog();
    let surface = normalize_surface_preserving_case(request);
    let normalized = surface.to_lowercase();
    for family in &catalog.families {
        for template in &family.templates {
            let Some(bindings) = match_template(template, &surface) else {
                continue;
            };
            return compile_family(catalog, family, bindings, limits);
        }
    }
    let names_memory_resource = catalog.cues.iter().any(|cue| normalized.contains(cue));
    let requests_set_operation = catalog
        .scopes
        .iter()
        .any(|scope| contains_scope_cue(&normalized, scope));
    if names_memory_resource && requests_set_operation {
        return Err(MemoryProgramCompileError::ProgramGap {
            request: request.trim().to_owned(),
            gap: String::from("program_gap:no_complete_seeded_family"),
        });
    }
    Err(MemoryProgramCompileError::NotMemoryProgram)
}

fn compile_family(
    catalog: &Catalog,
    family: &Family,
    bindings: BTreeMap<String, String>,
    limits: MemoryProgramLimits,
) -> Result<CompiledMemoryProgram, MemoryProgramCompileError> {
    if limits.max_matches == 0 || limits.max_iterations == 0 {
        return Err(MemoryProgramCompileError::ProgramGap {
            request: family.id.clone(),
            gap: String::from("program_gap:bounds_must_be_nonzero"),
        });
    }
    let mut steps = Vec::with_capacity(family.steps.len());
    for specification in &family.steps {
        let mut fields = specification.split_whitespace();
        let primitive = fields.next().unwrap_or_default().to_owned();
        let Some(&permission) = catalog.primitives.get(&primitive) else {
            return Err(MemoryProgramCompileError::ProgramGap {
                request: family.id.clone(),
                gap: format!("program_gap:unseeded_memory_primitive:{primitive}"),
            });
        };
        let arguments = fields
            .filter_map(|field| field.split_once('='))
            .map(|(name, value)| {
                let resolved = value
                    .strip_prefix('$')
                    .and_then(|key| bindings.get(key))
                    .cloned()
                    .unwrap_or_else(|| value.to_owned());
                (name.to_owned(), resolved)
            })
            .collect();
        steps.push(MemoryProgramStep {
            primitive,
            permission,
            arguments,
        });
    }
    let canonical_program = canonical_program(&family.id, limits, &bindings, &steps);
    Ok(CompiledMemoryProgram {
        id: stable_id("memory_program", &canonical_program),
        family: family.id.clone(),
        limits,
        steps,
        bindings,
        canonical_program,
    })
}

fn canonical_program(
    family: &str,
    limits: MemoryProgramLimits,
    bindings: &BTreeMap<String, String>,
    steps: &[MemoryProgramStep],
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "family={family}");
    let _ = writeln!(out, "max_matches={}", limits.max_matches);
    let _ = writeln!(out, "max_iterations={}", limits.max_iterations);
    for (name, value) in bindings {
        let _ = writeln!(out, "binding:{name}={}", normalize_binding(value));
    }
    for step in steps {
        let _ = write!(out, "step:{}:{}", step.primitive, step.permission.as_str());
        for (name, value) in &step.arguments {
            let _ = write!(out, ":{name}={}", normalize_binding(value));
        }
        out.push('\n');
    }
    out
}

fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(parse_catalog)
}

fn parse_catalog() -> Catalog {
    let mut catalog = Catalog {
        primitives: BTreeMap::new(),
        cues: Vec::new(),
        scopes: Vec::new(),
        families: Vec::new(),
    };
    let mut current_family: Option<Family> = None;
    let mut current_primitive: Option<String> = None;
    for line in MEMORY_PROGRAMS_LINO.lines().skip(1) {
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim();
        if indent == 2 {
            if let Some(family) = current_family.take() {
                catalog.families.push(family);
            }
            current_primitive = None;
            if let Some(id) = trimmed.strip_prefix("primitive ") {
                current_primitive = Some(id.to_owned());
            } else if let Some(cue) = trimmed.strip_prefix("cue ") {
                catalog.cues.push(normalize_surface(cue));
            } else if let Some(scope) = trimmed.strip_prefix("scope ") {
                catalog.scopes.push(normalize_surface(&unquote(scope)));
            } else if let Some(id) = trimmed.strip_prefix("family ") {
                current_family = Some(Family {
                    id: id.to_owned(),
                    ..Family::default()
                });
            }
        } else if indent == 4 {
            if let Some(permission) = trimmed.strip_prefix("permission ") {
                let permission = match permission {
                    "read" => MemoryProgramPermission::Read,
                    "write" => MemoryProgramPermission::Write,
                    "destructive" => MemoryProgramPermission::Destructive,
                    _ => continue,
                };
                if let Some(primitive) = current_primitive.take() {
                    catalog.primitives.insert(primitive, permission);
                }
            } else if let Some(family) = current_family.as_mut() {
                let Some((field, value)) = trimmed.split_once(' ') else {
                    continue;
                };
                let value = unquote(value);
                if field.starts_with("step_") {
                    family.steps.push(value);
                } else if field.starts_with("template_") {
                    family.templates.push(normalize_surface(&value));
                }
            }
        }
    }
    if let Some(family) = current_family {
        catalog.families.push(family);
    }
    catalog
}

fn unquote(value: &str) -> String {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value)
        .to_owned()
}

fn match_template(template: &str, request: &str) -> Option<BTreeMap<String, String>> {
    let mut bindings: BTreeMap<String, String> = BTreeMap::new();
    let mut template_cursor = 0;
    let mut request_cursor = 0;
    while let Some(open_relative) = template[template_cursor..].find('{') {
        let open = template_cursor + open_relative;
        let close = open + template[open..].find('}')?;
        let literal = &template[template_cursor..open];
        request_cursor += case_insensitive_prefix_len(&request[request_cursor..], literal)?;
        let name = &template[open + 1..close];
        template_cursor = close + 1;
        let next_open = template[template_cursor..]
            .find('{')
            .map_or(template.len(), |position| template_cursor + position);
        let next_literal = &template[template_cursor..next_open];
        let capture_end = if next_literal.is_empty() {
            request.len()
        } else {
            find_case_insensitive(&request[request_cursor..], next_literal)? + request_cursor
        };
        let captured = request[request_cursor..capture_end].trim();
        if captured.is_empty() {
            return None;
        }
        if let Some(existing) = bindings.get(name) {
            if normalize_binding(existing) != normalize_binding(captured) {
                return None;
            }
        } else {
            bindings.insert(name.to_owned(), captured.to_owned());
        }
        request_cursor = capture_end;
    }
    let suffix = &template[template_cursor..];
    case_insensitive_prefix_len(&request[request_cursor..], suffix)
        .is_some_and(|length| request_cursor + length == request.len())
        .then_some(bindings)
}

fn normalize_surface(text: &str) -> String {
    normalize_surface_preserving_case(text).to_lowercase()
}

fn normalize_surface_preserving_case(text: &str) -> String {
    text.trim()
        .trim_end_matches(|character: char| {
            matches!(character, '.' | '!' | '?' | ';' | '।' | '。' | '！' | '？')
        })
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn case_insensitive_prefix_len(text: &str, normalized_prefix: &str) -> Option<usize> {
    if normalized_prefix.is_empty() {
        return Some(0);
    }
    text.char_indices()
        .map(|(index, character)| index + character.len_utf8())
        .find(|&end| text[..end].to_lowercase() == normalized_prefix)
}

fn find_case_insensitive(text: &str, normalized_needle: &str) -> Option<usize> {
    text.char_indices()
        .map(|(index, _)| index)
        .chain(std::iter::once(text.len()))
        .find(|&start| case_insensitive_prefix_len(&text[start..], normalized_needle).is_some())
}

fn contains_scope_cue(text: &str, cue: &str) -> bool {
    text.match_indices(cue).any(|(start, matched)| {
        let end = start + matched.len();
        let before_is_word = text[..start]
            .chars()
            .next_back()
            .is_some_and(char::is_alphanumeric);
        let after_is_word = text[end..]
            .chars()
            .next()
            .is_some_and(char::is_alphanumeric);
        let cue_uses_han = cue.chars().any(
            |character| matches!(character, '\u{3400}'..='\u{4dbf}' | '\u{4e00}'..='\u{9fff}'),
        );
        if cue_uses_han {
            true
        } else {
            !before_is_word && !after_is_word
        }
    })
}

fn normalize_binding(text: &str) -> String {
    normalize_surface(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_closed_primitives_and_fifteen_families() {
        assert_eq!(catalog().primitives.len(), 8);
        assert!(catalog().families.len() >= 15);
    }

    #[test]
    fn a_memory_shaped_unknown_request_is_an_explicit_gap() {
        let error = compile_memory_program(
            "transpose every fact matrix",
            MemoryProgramLimits::default(),
        )
        .expect_err("no seeded family provides matrix transposition");
        assert!(matches!(
            error,
            MemoryProgramCompileError::ProgramGap { .. }
        ));
    }

    #[test]
    fn generic_fact_checks_are_not_misclassified_as_memory_program_gaps() {
        for request in [
            "fact-check this dialogue",
            "проверь факты в диалоге",
            "इस संवाद के तथ्यों की जाँच करें",
            "核查此对话中的事实",
            "verifica los hechos de este diálogo",
        ] {
            assert_eq!(
                compile_memory_program(request, MemoryProgramLimits::default()),
                Err(MemoryProgramCompileError::NotMemoryProgram),
                "{request} must remain available to the fact-checking route",
            );
        }
    }
}
