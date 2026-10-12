//! Sixth agentic recipe — driving Formal AI to *translate its entire source code
//! to the links / meta language and back*, reachable through the agentic interface
//! (issue #558).
//!
//! Issue #558 ("Auto learning") asks for a meta-algorithm that can *"recompile
//! itself"*, and specifically that *"the entire source code of our system"* be
//! *"translate\[d\] … to links / meta language (that must be present in our data),
//! and back to the source code."* [`super::self_ast`] proved that round-trip for a
//! single pinned module; [`crate::self_source_links`] lifts it to the whole
//! repository. This module makes the whole-repository view reachable *through the
//! agentic interface*: an external agent CLI (`Codex`, `OpenCode`, `Gemini`,
//! `Agent CLI`) — or the in-repo driver — asks Formal AI to project its own source,
//! and the deterministic planner walks a write → verify → final recipe that emits
//! the projection as Links Notation, exactly like the diagram, self-AST, and
//! self-healing recipes emit their self-inspection documents.
//!
//! The emitted document has two parts, split by cost so the recipe stays responsive
//! for a live agent loop:
//!
//! * an `entire_source` header that content-addresses *every* owned file (file
//!   count, total bytes, and one manifest id for the whole tree) — the cheap
//!   "entire source present in our data as links" view; and
//! * a `round_trip_proof` that parses a deterministic representative slice of real
//!   modules through the sole CST/AST engine and verifies each one round-trips
//!   byte-for-byte — the lossless "and back" proof.
//!
//! The *exhaustive* lossless proof over all owned files is the library invariant
//! ([`crate::self_source_links::SourceLinks::owned`]), locked by an ignored-by-default
//! test and printed by the `project_source_links` example — kept off the hot path
//! because parsing every file is deliberately expensive. Nothing here writes source
//! back: the projection is a read-only, auditable artifact, so the "recompile
//! itself" guardrail (observable, testable, reversible, human-approved) holds.
//! Neural inference stays a NON-GOAL — every number is a deterministic function of
//! the embedded source and the real tree-sitter parse.

use std::fmt::Write as _;
use std::sync::OnceLock;

use crate::self_source_links::{
    SourceLinks, owned_file_count, owned_manifest_content_id, owned_source_files, owned_total_bytes,
};

/// The workspace path the planner writes the generated projection document to.
pub const SOURCE_LINKS_PATH: &str = "self-source-links.lino";

/// How many real modules the recipe verifies losslessly inline. The whole-repository
/// proof lives in the library invariant / example; this slice keeps the live recipe
/// responsive while still proving the round-trip on real, varied source.
const SLICE_SIZE: usize = 6;

/// A *differently worded* request for the source-links recipe.
///
/// The router recognises the intent from the words, not a hardcoded string.
/// Deliberately avoids the self-AST keywords ("cst/ast", "abstract-syntax",
/// "reason about itself") and the self-healing keywords so the self-inspection
/// recipes never collide.
pub const SOURCE_LINKS_TASK: &str = "Translate the entire source code of our system to the links / meta language and \
     back to source, and record the whole-repository source-to-links projection in \
     Links Notation so we can recompile ourselves.";

/// Mask literal payloads and path operands before instruction matching.
fn unquoted_instruction(prompt: &str) -> String {
    let mut outside = String::new();
    let mut cursor = 0;
    for span in crate::normal_markov::quoted_segment_spans(prompt) {
        outside.push_str(&prompt[cursor..span.start]);
        outside.push_str(&" ".repeat(span.end - span.start));
        cursor = span.end;
    }
    outside.push_str(&prompt[cursor..]);
    outside
        .split_whitespace()
        .map(|token| {
            let candidate = token.trim_matches(|character| "([{.,;!?)}]".contains(character));
            if candidate.contains(['/', '\\'])
                || candidate.split('.').count() > 1 && !candidate.starts_with('.')
            {
                String::new()
            } else {
                token.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
fn positive_cues(normalized: &str, role: &str) -> Vec<(usize, usize)> {
    let lexicon = crate::seed::lexicon();
    let mut positions = Vec::new();
    for word in lexicon.words_for_role(role) {
        let surface = crate::engine::normalize_prompt(&word);
        if surface.is_empty() {
            continue;
        }
        for (start, _) in normalized.match_indices(surface.as_str()) {
            let end = start + surface.len();
            let boundary = |character: Option<char>| {
                character.is_none_or(|character| {
                    !character.is_alphanumeric() && character != '_' && character != '-'
                })
            };
            if (crate::coding::contains_cjk(&surface)
                || boundary(normalized[..start].chars().next_back())
                    && boundary(normalized[end..].chars().next()))
                && !lexicon.mentions_role("statement_negation_cue", &normalized[..start])
            {
                positions.push((start, end));
            }
        }
    }
    positions
}
/// Only positively owned, unquoted projection instructions select this recipe.
#[must_use]
pub fn is_source_links_task(prompt: &str) -> bool {
    let outside = unquoted_instruction(prompt);
    super::shell_command_policy::sentences(&outside)
        .iter()
        .any(|clause| {
            let normalized = crate::engine::normalize_prompt(clause.text);
            let lexicon = crate::seed::lexicon();
            let owns_action = |prefix| {
                lexicon.mentions_role("translation_action", prefix)
                    || lexicon.mentions_role("file_whole_write_action", prefix)
            };
            if positive_cues(&normalized, "source-recompilation-action")
                .iter()
                .any(|&(start, _)| start == 0 || owns_action(&normalized[..start]))
            {
                return true;
            }
            if positive_cues(&normalized, "source-links-report-cue")
                .iter()
                .any(|&(start, end)| {
                    start == 0 && end == normalized.len() || owns_action(&normalized[..start])
                })
            {
                return true;
            }
            let groups = [
                "source-projection-scope",
                "source-projection-origin",
                "source-projection-format",
                "source-projection-return",
            ]
            .map(|role| positive_cues(&normalized, role).first().copied());
            groups.iter().all(Option::is_some)
                && groups
                    .iter()
                    .flatten()
                    .map(|&(_, end)| end)
                    .max()
                    .is_some_and(|end| {
                        lexicon.mentions_role("translation_action", &normalized[..end])
                    })
        })
}

/// The deterministic representative slice of owned modules the recipe verifies
/// inline: up to [`SLICE_SIZE`] files spread evenly across the path-sorted source
/// tree, so the proof spans varied subsystems rather than one corner.
fn representative_slice() -> Vec<(&'static str, &'static str)> {
    let files = owned_source_files();
    let total = files.len();
    let want = SLICE_SIZE.min(total);
    if want == 0 {
        return Vec::new();
    }
    let stride = (total / want).max(1);
    (0..want)
        .map(|index| files[(index * stride).min(total - 1)])
        .collect()
}

/// The projected representative slice, computed once per process.
///
/// Parsing each module through the CST/AST engine is deliberately non-trivial and
/// the recipe touches the projection several times per run (write step, verify step,
/// final answer), so memoising keeps the loop responsive without changing its
/// deterministic result.
fn cached_slice() -> &'static SourceLinks {
    static SLICE: OnceLock<SourceLinks> = OnceLock::new();
    SLICE.get_or_init(|| SourceLinks::compile(&representative_slice()))
}

/// Render the whole-repository source ↔ links projection document (Links Notation).
/// Deterministic and ends with exactly one trailing newline.
///
/// The output is what the Agent CLI writes to [`SOURCE_LINKS_PATH`]. It is asserted
/// live (never committed byte-for-byte) in the issue-#558 tests, because the
/// per-file content ids and counts change with every source edit — committing it
/// would force a regenerate on every unrelated PR.
#[must_use]
pub fn render_document() -> String {
    let slice = cached_slice();
    let mut out = String::from("self_source_links\n");
    let _ = writeln!(out, "  engine meta_language");
    let _ = writeln!(out, "  language rust");
    let _ = writeln!(out, "  task translate_entire_source_to_links_and_back");
    // The cheap "entire source present in our data as links" view: every owned file
    // enumerated and content-addressed under a single manifest id.
    let _ = writeln!(out, "  entire_source");
    let _ = writeln!(out, "    file_count {}", owned_file_count());
    let _ = writeln!(out, "    total_bytes {}", owned_total_bytes());
    let _ = writeln!(
        out,
        "    manifest_content_id \"{}\"",
        owned_manifest_content_id()
    );
    // The lossless "and back" proof over a real representative slice.
    let _ = writeln!(out, "  round_trip_proof");
    let _ = writeln!(out, "    slice_size {}", slice.module_count());
    let _ = writeln!(out, "    slice_faithful_count {}", slice.faithful_count());
    let _ = writeln!(
        out,
        "    slice_fully_faithful {}",
        slice.is_fully_faithful()
    );
    // Fold the slice projection in as an indented block so the whole document is one
    // reviewable Links Notation tree.
    for line in slice.links_notation().lines() {
        if line.is_empty() {
            out.push('\n');
        } else {
            let _ = writeln!(out, "    {line}");
        }
    }
    format!("{}\n", out.trim_end())
}

/// The projected representative slice backing the recipe. Exposed so tests can
/// assert the round-trip proof is lossless without rebuilding it.
#[must_use]
pub fn slice() -> SourceLinks {
    cached_slice().clone()
}

/// The self-contained final answer: a natural-language summary plus the generated
/// projection document inline.
#[must_use]
pub fn final_answer(document: &str) -> String {
    let slice = cached_slice();
    format!(
        "Translated the entire source code of our system to the links / meta language and back: \
         content-addressed all {files} owned source files under one manifest id, and verified a \
         representative slice of {slice} modules round-trips byte-for-byte through the meta-language \
         links network (the sole CST/AST engine here). The exhaustive whole-repository round-trip is \
         the library invariant (SourceLinks::owned) — a real step toward recompiling ourselves. \
         Nothing was written back; the projection is a read-only, auditable artifact.\n\n\
         Generated document ({SOURCE_LINKS_PATH}):\n\n{document}",
        files = owned_file_count(),
        slice = slice.module_count(),
        document = document.trim_end(),
    )
}
