//! Agent-CLI recipe for auditing and generalizing the issue-#540 dreaming loop.
//!
//! The audit is **derived at runtime** from the grounded dreaming recipe
//! (`data/meta/dreaming-recipe.lino`) instead of echoing a hardcoded prose
//! checklist: the analyzer parses the recipe's `meta_step`, `meta_function`,
//! and `meta_test` records, cross-references which functions ground each stage
//! and which test suites pin it, and reports any stage that is *not* grounded
//! or pinned as an open gap. Editing the recipe therefore changes the audit
//! output without touching this module.

use std::fmt::Write as _;

use crate::dreaming::lexicon::load_data_document;

pub const DREAMING_AUDIT_PATH: &str = "dreaming-gap-analysis.lino";

pub const DREAMING_AUDIT_TASK: &str =
    "Audit issue 540 dreaming through Formal AI: inspect the grounded dreaming meta-algorithm, \
     identify every implementation gap from amendment application through replay, pattern \
     learning, storage consent, and idle runtime, then record the generalization that resolves \
     each gap as Links Notation in dreaming-gap-analysis.lino.";

const EMBEDDED_RECIPE: &str = include_str!("../../data/meta/dreaming-recipe.lino");
const EMBEDDED_CUES: &str = include_str!("../../data/meta/dreaming-cues.lino");

/// One record parsed from the recipe document: the quoted fields under a
/// top-level Links-Notation link.
#[derive(Debug, Default, Clone)]
struct RecipeRecord {
    fields: Vec<(String, String)>,
}

impl RecipeRecord {
    fn field(&self, key: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.as_str())
    }

    fn record_type(&self) -> &str {
        self.field("record_type").unwrap_or_default()
    }
}

/// A stage of the recipe together with the analytical evidence found for it.
#[derive(Debug)]
struct StageAnalysis {
    id: String,
    title: String,
    /// Functions from the recipe's `meta_function` records whose names appear
    /// in this stage's detail — the code that grounds the stage.
    grounding_functions: Vec<String>,
    /// Test suites from `meta_test` records whose pinned scope mentions this
    /// stage's functions or id.
    pinning_suites: Vec<String>,
    source_file: String,
}

impl StageAnalysis {
    /// A stage is resolved when at least one named function grounds it. Test
    /// pinning is reported but a purely descriptive stage may share a suite.
    const fn is_grounded(&self) -> bool {
        !self.grounding_functions.is_empty()
    }
}

/// Parse the flat two-level Links-Notation recipe into records.
fn parse_recipe(text: &str) -> Vec<RecipeRecord> {
    let mut records: Vec<RecipeRecord> = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim();
        if indent == 0 {
            records.push(RecipeRecord::default());
            continue;
        }
        let Some(record) = records.last_mut() else {
            continue;
        };
        if let Some((key, rest)) = trimmed.split_once(' ') {
            let value = rest.trim().trim_matches('"');
            record.fields.push((key.to_owned(), value.to_owned()));
        }
    }
    records
}

/// Cross-reference every recipe stage against the recipe's own function and
/// test records. This is the analytical core: nothing here is a canned answer.
fn analyze_recipe(records: &[RecipeRecord]) -> Vec<StageAnalysis> {
    let functions: Vec<&RecipeRecord> = records
        .iter()
        .filter(|record| record.record_type() == "meta_function")
        .collect();
    let tests: Vec<&RecipeRecord> = records
        .iter()
        .filter(|record| record.record_type() == "meta_test")
        .collect();

    let mut stages: Vec<(u64, StageAnalysis)> = records
        .iter()
        .filter(|record| record.record_type() == "meta_step")
        .map(|step| {
            let id = step.field("id").unwrap_or_default().to_owned();
            let detail = step.field("detail").unwrap_or_default().to_lowercase();
            let grounding_functions: Vec<String> = functions
                .iter()
                .filter_map(|function| function.field("function"))
                .filter(|name| detail.contains(&name.to_lowercase()))
                .map(str::to_owned)
                .collect();
            let pinning_suites: Vec<String> = tests
                .iter()
                .filter(|test| {
                    let pins = test.field("pins").unwrap_or_default().to_lowercase();
                    pins.contains(&id.replace('_', " "))
                        || grounding_functions
                            .iter()
                            .any(|name| pins.contains(&name.to_lowercase()))
                        || test
                            .field("test_file")
                            .is_some_and(|file| detail.contains(&file.to_lowercase()))
                })
                .filter_map(|test| test.field("suite"))
                .map(str::to_owned)
                .collect();
            let order = step
                .field("order")
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(u64::MAX);
            (
                order,
                StageAnalysis {
                    id,
                    title: step.field("title").unwrap_or_default().to_owned(),
                    grounding_functions,
                    pinning_suites,
                    source_file: step.field("source_file").unwrap_or_default().to_owned(),
                },
            )
        })
        .collect();
    stages.sort_by_key(|(order, _)| *order);
    stages.into_iter().map(|(_, stage)| stage).collect()
}

/// Recognize the explicit dreaming-audit task.
///
/// Scoped narrowly (issue #540 review): the prompt must name the audit
/// artifact itself — merely mentioning "dreaming" and "audit" no longer
/// hijacks unrelated agentic tasks.
#[must_use]
pub fn is_dreaming_audit_task(prompt: &str) -> bool {
    prompt.to_lowercase().contains("dreaming-gap-analysis")
}

#[must_use]
pub fn render_document() -> String {
    render_document_from(
        &load_data_document("dreaming-recipe.lino", EMBEDDED_RECIPE),
        &load_data_document("dreaming-cues.lino", EMBEDDED_CUES),
    )
}

/// Render the audit for an explicit recipe/cues document pair. Exposed so
/// tests can prove the analysis is computed from the data: altering the
/// recipe alters the reported gaps.
#[must_use]
pub fn render_document_from(recipe: &str, cues: &str) -> String {
    let records = parse_recipe(recipe);
    let stages = analyze_recipe(&records);
    let cue_count = cues
        .lines()
        .filter(|line| line.trim_start().starts_with("cue \""))
        .count();
    let open_gaps = stages.iter().filter(|stage| !stage.is_grounded()).count();

    let mut out = String::from("dreaming_gap_analysis\n");
    out.push_str("  record_type \"agent_cli_gap_analysis\"\n");
    out.push_str("  issue \"540\"\n");
    out.push_str("  method \"derived at runtime by cross-referencing meta_step, meta_function, and meta_test records of the grounded recipe\"\n");
    let _ = writeln!(out, "  grounded_recipe_steps \"{}\"", stages.len());
    let _ = writeln!(out, "  multilingual_cues \"{cue_count}\"");
    let _ = writeln!(out, "  open_gaps \"{open_gaps}\"");
    let conclusion = if open_gaps == 0 {
        "every recipe stage is grounded in named functions; no open gaps"
    } else {
        "ungrounded stages remain; see resolution records with status open_gap"
    };
    field_at(&mut out, 2, "conclusion", conclusion);
    for (index, stage) in stages.iter().enumerate() {
        let _ = writeln!(out, "  resolution_{:02}", index + 1);
        field_at(&mut out, 4, "stage", &stage.id);
        field_at(&mut out, 4, "generalization", &stage.title);
        field_at(&mut out, 4, "source_file", &stage.source_file);
        field_at(
            &mut out,
            4,
            "grounding_functions",
            &join_or_none(&stage.grounding_functions),
        );
        field_at(
            &mut out,
            4,
            "pinned_by_suites",
            &join_or_none(&stage.pinning_suites),
        );
        let status = if stage.is_grounded() {
            "grounded"
        } else {
            "open_gap"
        };
        field_at(&mut out, 4, "status", status);
    }
    out
}

#[must_use]
pub fn final_answer(document: &str) -> String {
    let stage_count = document
        .lines()
        .filter(|line| line.trim().starts_with("resolution_"))
        .count();
    let open_gaps = document
        .lines()
        .filter(|line| line.trim() == "status \"open_gap\"")
        .count();
    format!(
        "Audited the grounded issue-#540 dreaming meta-algorithm through Formal AI by \
         cross-referencing its recipe stages against the functions and test suites the recipe \
         itself declares. {stage_count} stages analyzed, {open_gaps} open gap(s) \
         found.\n\nGenerated document ({DREAMING_AUDIT_PATH}):\n\n{}",
        document.trim_end(),
    )
}

fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        String::from("none")
    } else {
        values.join(", ")
    }
}

fn field_at(out: &mut String, indent: usize, name: &str, value: &str) {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    let _ = writeln!(out, "{:indent$}{name} \"{escaped}\"", "");
}
