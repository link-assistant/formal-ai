//! Deterministic fallback planner for repository change requests (issue #654).
//!
//! Unlike the stored recipe fixtures, this planner derives its target and payload
//! from the formalized request.  The resulting plan is data: it is serialized to
//! Links Notation and written before execution, so the tool transcript is an
//! append-only record of the decision that caused the change.

use std::fmt::Write as _;

use crate::engine::stable_id;
use crate::intent_formalization::formalize_intent;

use super::planner::Capability;

/// Workspace-relative event-log artifact written before a general plan executes.
pub const PLAN_PATH: &str = ".formal-ai/general-change-plan.lino";

/// One ordered, capability-tagged operation in a general change plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneralPlanStep {
    pub capability: Capability,
    pub action: String,
    pub expected_evidence: String,
    pub command: Option<String>,
}

/// A deterministic plan composed from a formalized, previously unrecognised request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneralChangePlan {
    pub id: String,
    pub goal: String,
    pub target: String,
    pub content: String,
    pub steps: Vec<GeneralPlanStep>,
    pub verification_command: String,
}

impl GeneralChangePlan {
    /// Render the plan shape consumed by the driver and documented by the meta fixture.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("general_change_plan\n");
        field(&mut out, "id", &self.id);
        field(&mut out, "goal", &self.goal);
        field(&mut out, "target", &self.target);
        for (index, step) in self.steps.iter().enumerate() {
            let _ = writeln!(out, "  step {}", index + 1);
            field_nested(&mut out, "capability", capability_slug(step.capability));
            field_nested(&mut out, "action", &step.action);
            field_nested(&mut out, "expected_evidence", &step.expected_evidence);
            if let Some(command) = &step.command {
                field_nested(&mut out, "command", command);
            }
        }
        field(&mut out, "verification_command", &self.verification_command);
        out
    }
}

/// Compose a safe file-creation plan from arbitrary English or Russian wording.
///
/// The universal intent formalizer supplies the stable impulse identity.  The
/// decomposition is deliberately bounded to requests that state both a relative
/// target file and literal content; ambiguous requests continue to the ordinary
/// solver instead of inventing a patch or shell command.
#[must_use]
pub fn compose_general_change_plan(request: &str) -> Option<GeneralChangePlan> {
    let target = extract_target(request)?;
    let content = extract_content(request)?;
    if !safe_relative_path(&target) {
        return None;
    }
    let intent = formalize_intent(request, language(request), None);
    let verification_command = format!("cat {target}");
    let steps = vec![
        GeneralPlanStep {
            capability: Capability::Write,
            action: format!("append the composed plan to {PLAN_PATH}"),
            expected_evidence: format!("written plan event {}", intent.impulse_id),
            command: None,
        },
        GeneralPlanStep {
            capability: Capability::Write,
            action: format!("write the requested content to {target}"),
            expected_evidence: format!("workspace file {target}"),
            command: None,
        },
        GeneralPlanStep {
            capability: Capability::Run,
            action: String::from("run the request-derived verification command"),
            expected_evidence: content.clone(),
            command: Some(verification_command.clone()),
        },
    ];
    Some(GeneralChangePlan {
        id: stable_id(
            "general_change_plan",
            &format!("{}:{target}:{content}", intent.impulse_id),
        ),
        goal: intent.source_text,
        target,
        content,
        steps,
        verification_command,
    })
}

/// Words that, when they immediately precede a file-looking token, mark that token
/// as the write target. Includes the container words ("file"/"in"/…), the naming
/// words ("named"/"called" — issue #681's phrasing), and the write verbs
/// themselves ("write hello.py …") across the supported languages. Kept as data so
/// a new phrasing is one entry, never a code branch.
const TARGET_PRECEDING_WORDS: [&str; 17] = [
    "file",
    "файл",
    "in",
    "в",
    "create",
    "создай",
    "создать",
    "named",
    "called",
    "write",
    "save",
    "generate",
    "make",
    "запиши",
    "сохрани",
    "названием",
    "именем",
];

/// Verbs (any supported language) that mark a request as *producing* a file rather
/// than reading one. Substrings so inflected forms match (`create`/`creating`,
/// `создай`/`создать`). Kept deliberately narrow to a creation/mutation sense so an
/// ordinary read prompt never trips them.
const WRITE_VERBS: [&str; 12] = [
    "create",
    "write",
    "save",
    "generate",
    "append to",
    "add to",
    "созда",
    "запиш",
    "сохран",
    "записать",
    "допиш",
    "добав",
];

/// Whether `lower` (an already-lowercased request) is a file **write / create**
/// intent — a write verb applied to something file-shaped. This is the single
/// signal the router uses to keep a file-creation request from ever being
/// misrouted to the file-read recipe (issue #681): a request to *produce* a file
/// is a write, never a read of the not-yet-existing target.
///
/// It fires only when a write verb co-occurs with a file signal — the literal
/// word "file"/"файл", a naming word ("named"/"called"/"именем"/"названием"), or a
/// filename-looking token — so that "write a poem" or "save the world" (no file in
/// sight) fall through to the ordinary solver, while "create a file named …",
/// "save report.md …", and "generate data.json …" are recognised as writes.
#[must_use]
pub(super) fn has_file_write_intent(lower: &str) -> bool {
    let has_write_verb = WRITE_VERBS.iter().any(|verb| lower.contains(verb));
    if !has_write_verb {
        return false;
    }
    let names_a_file = lower.contains("file")
        || lower.contains("файл")
        || lower.contains(" named ")
        || lower.contains(" called ")
        || lower.contains(" именем ")
        || lower.contains(" названием ")
        || mentions_filename_token(lower);
    has_write_verb && names_a_file
}

/// Whether any whitespace-delimited token in `text` looks like a local filename
/// (`name.ext`, not a URL). Mirrors the write planner's own target heuristic so the
/// intent gate and the extractor agree on what "a file" is.
fn mentions_filename_token(text: &str) -> bool {
    text.split_whitespace().any(|word| {
        let cleaned = word.trim_matches(|c: char| matches!(c, '`' | '"' | '\'' | ',' | ':' | ';'));
        cleaned.contains('.')
            && !cleaned.contains("://")
            && cleaned.rsplit_once('.').is_some_and(|(stem, extension)| {
                !stem.is_empty()
                    && (1..=12).contains(&extension.len())
                    && extension.chars().all(|c| c.is_ascii_alphanumeric())
            })
    })
}

fn extract_target(request: &str) -> Option<String> {
    let words: Vec<&str> = request.split_whitespace().collect();
    words.iter().enumerate().find_map(|(index, word)| {
        let cleaned = word.trim_matches(|c: char| matches!(c, '`' | '"' | '\'' | ',' | ':' | ';'));
        let previous = index.checked_sub(1).and_then(|i| words.get(i)).map(|w| {
            w.trim_matches(|c: char| matches!(c, '`' | '"' | '\'' | ',' | ':' | ';'))
                .to_lowercase()
        });
        let looks_like_file = cleaned.contains('.') && !cleaned.contains("://");
        (looks_like_file
            && previous
                .as_deref()
                .is_some_and(|p| TARGET_PRECEDING_WORDS.contains(&p)))
        .then(|| cleaned.to_owned())
    })
}

/// Natural-language markers that introduce the literal content of a write request.
/// The longest, most specific markers come first so *"with the content"* wins over
/// a bare *"content"* substring (issue #681). Kept as data, ordered by descending
/// length, so a new phrasing is one entry rather than a code branch.
const CONTENT_MARKERS: [&str; 12] = [
    " with the content ",
    " with the text ",
    " with content ",
    " with text ",
    " that says ",
    " that contains ",
    " containing ",
    " saying ",
    " с содержанием ",
    " содержанием ",
    " с текстом ",
    " текстом ",
];

fn extract_content(request: &str) -> Option<String> {
    let lower = request.to_lowercase();
    CONTENT_MARKERS
        .iter()
        .find_map(|marker| lower.find(marker).map(|at| (marker, at)))
        .map(|(marker, at)| {
            request[at + marker.len()..]
                .trim()
                .trim_matches(|c: char| matches!(c, '`' | '"' | '\'' | '.' | '。'))
                .to_owned()
        })
        .filter(|content| !content.is_empty())
}

fn safe_relative_path(path: &str) -> bool {
    !path.starts_with('/')
        && !path.starts_with('-')
        && !path.split('/').any(|part| part == ".." || part.is_empty())
        && path
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '/' | '.' | '_' | '-'))
}

const fn capability_slug(capability: Capability) -> &'static str {
    match capability {
        Capability::Search => "Search",
        Capability::Fetch => "Fetch",
        Capability::Read => "Read",
        Capability::Write => "Write",
        Capability::Run => "Run",
    }
}

fn language(request: &str) -> &'static str {
    if request
        .chars()
        .any(|c| ('\u{0400}'..='\u{04ff}').contains(&c))
    {
        "ru"
    } else {
        "en"
    }
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn field(out: &mut String, name: &str, value: &str) {
    let _ = writeln!(out, "  {name} \"{}\"", escape(value));
}

fn field_nested(out: &mut String, name: &str, value: &str) {
    let _ = writeln!(out, "    {name} \"{}\"", escape(value));
}
