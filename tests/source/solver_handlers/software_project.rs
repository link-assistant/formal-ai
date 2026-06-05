//! Generic software-project request handler.
//!
//! The handler covers open-ended "build/write/create an extension/app/bot/tool"
//! prompts. It first projects the surface text into a small Links Notation
//! meaning record, then derives reasoning and plan steps from that meaning.
//! Code is returned only after the user approves the plan in a later turn.

use std::fmt::Write as _;

use crate::engine::{normalize_prompt, stable_id, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::seed;
use crate::solver_handlers::finalize_simple;
use crate::solver_helpers::{last_assistant_turn, last_user_turn};

use super::software_project_code::implementation_code;

/// A matched software-artifact phrase: the surface word it was recognised by
/// (used to locate the target text after it) and the canonical English label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ArtifactMatch {
    surface: &'static str,
    label: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SoftwareProjectMeaning {
    action: &'static str,
    artifact_surface: &'static str,
    pub(super) artifact: &'static str,
    target: String,
    requirements: Vec<String>,
    subtasks: Vec<SoftwareSubtask>,
    delivery_mode: DeliveryMode,
    pub(super) implementation_language: &'static str,
    approval_gates: Vec<&'static str>,
    game_tracker: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SoftwareSubtask {
    requirement_id: String,
    category: &'static str,
    title: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeliveryMode {
    CodeGeneration,
    ManualInstructions,
    ScriptGeneration,
    ImmediateExecution,
}

impl DeliveryMode {
    const fn label(self) -> &'static str {
        match self {
            Self::CodeGeneration => "code_generation",
            Self::ManualInstructions => "manual_instructions",
            Self::ScriptGeneration => "script_generation",
            Self::ImmediateExecution => "immediate_execution",
        }
    }

    /// Map a `software_delivery_mode` meaning slug to its delivery mode. The
    /// default (code generation) has no slug — it is the fallback when no mode
    /// meaning is evidenced — so this returns `None` for any other slug.
    fn from_slug(slug: &str) -> Option<Self> {
        match slug {
            "delivery_manual_instructions" => Some(Self::ManualInstructions),
            "delivery_immediate_execution" => Some(Self::ImmediateExecution),
            "delivery_script_generation" => Some(Self::ScriptGeneration),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ApprovalState {
    Proposed,
    Approved,
}

impl ApprovalState {
    const fn label(self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
        }
    }
}

impl SoftwareProjectMeaning {
    pub(super) fn from_prompt(prompt: &str, normalized: &str) -> Option<Self> {
        if normalized.contains("hello") && normalized.contains("world") {
            return None;
        }

        let actions = action_surface_table();
        let artifacts = artifact_surface_table();
        let action = scan_match(normalized, |input| match_action(input, &actions))?;
        let artifact = scan_match(normalized, |input| match_artifact(input, &artifacts))?;
        let target = extract_target(prompt, artifact);
        let requirements = extract_requirements(prompt);
        let game_tracker = is_game_unit_tracker(normalized);
        let subtasks = derive_subtasks(&requirements, game_tracker);
        let delivery_mode = detect_delivery_mode(normalized);
        let implementation_language = detect_implementation_language(normalized);
        let approval_gates = approval_gates(normalized, delivery_mode);

        Some(Self {
            action,
            artifact_surface: artifact.surface,
            artifact: artifact.label,
            target,
            requirements,
            subtasks,
            delivery_mode,
            implementation_language,
            approval_gates,
            game_tracker,
        })
    }

    fn canonical_key(&self) -> String {
        let mut key = format!(
            "action={};artifact={};target={};game_tracker={}",
            self.action, self.artifact, self.target, self.game_tracker
        );
        let _ = write!(
            key,
            ";delivery_mode={};implementation_language={}",
            self.delivery_mode.label(),
            self.implementation_language
        );
        for requirement in &self.requirements {
            let _ = write!(key, ";requirement={requirement}");
        }
        for subtask in &self.subtasks {
            let _ = write!(key, ";subtask={}:{}", subtask.category, subtask.title);
        }
        key
    }

    pub(super) fn meaning_id(&self) -> String {
        stable_id("software_project_request", &self.canonical_key())
    }

    /// Expose the delivery-mode label to the follow-up module without leaking
    /// the private [`DeliveryMode`] enum or its backing field.
    pub(super) const fn delivery_mode_label(&self) -> &'static str {
        self.delivery_mode.label()
    }

    const fn domain_label(&self) -> &'static str {
        if self.game_tracker {
            "tabletop_game_unit_tracker"
        } else {
            "software_project"
        }
    }

    fn meaning_lino(&self, approval_state: ApprovalState) -> String {
        let mut buffer = String::from("software_project_request");
        let _ = writeln!(buffer, "\n  action {}", lino_string(self.action));
        let _ = writeln!(buffer, "  artifact {}", lino_string(self.artifact));
        let _ = writeln!(
            buffer,
            "  artifact_surface {}",
            lino_string(self.artifact_surface)
        );
        let _ = writeln!(buffer, "  target {}", lino_string(&self.target));
        let _ = writeln!(buffer, "  domain {}", lino_string(self.domain_label()));
        let _ = writeln!(buffer, "  delivery_mode {}", self.delivery_mode.label());
        let _ = writeln!(
            buffer,
            "  implementation_language {}",
            lino_string(self.implementation_language)
        );
        let _ = writeln!(buffer, "  approval_state {}", approval_state.label());
        let _ = writeln!(buffer, "  approval_required true");
        for gate in &self.approval_gates {
            let _ = writeln!(buffer, "  approval_gate {}", lino_string(gate));
        }
        for requirement in &self.requirements {
            let _ = writeln!(buffer, "  requirement {}", lino_string(requirement));
            let _ = writeln!(
                buffer,
                "  requirement_category {}",
                lino_string(classify_requirement(requirement, self.game_tracker))
            );
        }
        for subtask in &self.subtasks {
            let _ = writeln!(
                buffer,
                "  subtask {}",
                lino_string(&format!(
                    "{} [{}] {}",
                    subtask.requirement_id, subtask.category, subtask.title
                ))
            );
        }
        if self.game_tracker {
            buffer.push_str("  state_model \"unit_state\"\n");
            buffer.push_str("  command \"apply_damage\"\n");
            buffer.push_str("  command \"set_stacks\"\n");
            buffer.push_str("  command \"tick_cooldowns\"\n");
            buffer.push_str("  validation \"damage_mitigation_floor_at_zero\"\n");
            buffer.push_str("  validation \"cooldowns_decrement_without_negative_rounds\"\n");
        } else {
            buffer.push_str("  state_model \"project_records\"\n");
            buffer.push_str("  command \"create_record\"\n");
            buffer.push_str("  command \"update_record\"\n");
            buffer.push_str("  command \"export_state\"\n");
            buffer.push_str("  validation \"pure_state_transitions_before_host_api\"\n");
        }
        buffer
    }
}

pub fn try_software_project_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let canonical = normalize_prompt(prompt);
    let normalized = if canonical.is_empty() {
        normalized
    } else {
        canonical.as_str()
    };

    if is_approval_prompt(normalized) {
        if let Some(meaning) = prior_software_project_meaning(log) {
            record_meaning(log, &meaning, ApprovalState::Approved);
            let body = render_implementation_response(&meaning);
            return Some(finalize_simple(
                prompt,
                log,
                "software_project_implementation",
                "response:software_project_implementation",
                &body,
                0.82,
            ));
        }
    }

    let meaning = SoftwareProjectMeaning::from_prompt(prompt, normalized)?;
    record_meaning(log, &meaning, ApprovalState::Proposed);
    let body = render_plan_response(&meaning);
    Some(finalize_simple(
        prompt,
        log,
        "software_project_plan",
        "response:software_project_plan",
        &body,
        0.78,
    ))
}

fn prior_software_project_meaning(log: &EventLog) -> Option<SoftwareProjectMeaning> {
    let assistant = last_assistant_turn(log)?;
    if !assistant.contains("software_project_request") || !assistant.contains("approve plan") {
        return None;
    }
    let prior_prompt = last_user_turn(log)?;
    let normalized = normalize_prompt(prior_prompt);
    SoftwareProjectMeaning::from_prompt(prior_prompt, &normalized)
}

fn record_meaning(
    log: &mut EventLog,
    meaning: &SoftwareProjectMeaning,
    approval_state: ApprovalState,
) {
    log.append("formalization", "text_to_links_notation".to_owned());
    log.append("meaning", meaning.meaning_id());
    log.append("software_project:action", meaning.action.to_owned());
    log.append("software_project:artifact", meaning.artifact.to_owned());
    log.append("software_project:target", meaning.target.clone());
    log.append("software_project:domain", meaning.domain_label().to_owned());
    log.append(
        "software_project:delivery_mode",
        meaning.delivery_mode.label().to_owned(),
    );
    log.append(
        "software_project:implementation_language",
        meaning.implementation_language.to_owned(),
    );
    log.append("approval_state", approval_state.label().to_owned());
    for gate in &meaning.approval_gates {
        log.append("approval_gate", (*gate).to_owned());
    }
    log.append(
        "software_project:strategy",
        if meaning.game_tracker {
            "game_unit_tracker"
        } else {
            "bounded_project_plan"
        }
        .to_owned(),
    );
    for requirement in &meaning.requirements {
        log.append("requirement", requirement.clone());
        log.append(
            "requirement_category",
            classify_requirement(requirement, meaning.game_tracker).to_owned(),
        );
    }
    for subtask in &meaning.subtasks {
        log.append(
            "software_project:subtask",
            format!(
                "{}:{}:{}",
                subtask.requirement_id, subtask.category, subtask.title
            ),
        );
    }
    for step in reasoning_steps(meaning) {
        log.append("reasoning_step", step);
    }
    for step in plan_steps(meaning) {
        log.append("plan_step", step);
    }
}

/// Map a software-artifact-kind meaning slug to its canonical English label.
///
/// The lexicon owns the surface words a prompt is matched against (in every
/// supported language); this resolver owns only the stable slug→label mapping,
/// so adding a language never touches this code — exactly like
/// [`crate::solver_handlers::calendar`]'s `Weekday::from_slug`. Recognition
/// vocabulary lives in the data; the canonical output label lives in code.
/// Returns `None` for a slug this handler does not render, so a future artifact
/// kind in the lexicon is skipped here rather than silently mislabelled.
fn artifact_label(slug: &str) -> Option<&'static str> {
    let label = match slug {
        "artifact_browser_extension" => "browser extension",
        "artifact_command_line_tool" => "command-line tool",
        "artifact_github_action" => "action",
        "artifact_mobile_app" => "mobile app",
        "artifact_web_app" => "web app",
        "artifact_application" => "application",
        "artifact_extension" => "extension",
        "artifact_dashboard" => "dashboard",
        "artifact_scraper" => "scraper",
        "artifact_library" => "library",
        "artifact_website" => "website",
        "artifact_plugin" => "plugin",
        "artifact_service" => "service",
        "artifact_bot" => "bot",
        "artifact_app" => "app",
        "artifact_api" => "API",
        "artifact_sdk" => "SDK",
        "artifact_tool" => "tool",
        "artifact_mod" => "mod",
        _ => return None,
    };
    Some(label)
}

/// The (surface, label) recognition table for software artifacts, sourced from
/// the lexicon: every `software_artifact_kind` meaning, in declaration order,
/// flat-mapped over its surface words in every supported language. Declaration
/// order preserves the specific-before-generic constraint [`scan_match`] relies
/// on (e.g. `application` precedes `app`, so the longer phrase is taken first).
fn artifact_surface_table() -> Vec<(&'static str, &'static str)> {
    let mut table = Vec::new();
    for meaning in seed::lexicon().meanings_with_role(seed::ROLE_SOFTWARE_ARTIFACT_KIND) {
        let Some(label) = artifact_label(&meaning.slug) else {
            continue;
        };
        for surface in meaning.words() {
            table.push((surface, label));
        }
    }
    table
}

/// The (surface, action-slug) recognition table for software-authoring verbs,
/// sourced from the lexicon: every `software_authoring_action` meaning, flat-
/// mapped over its surface words. The matched slug is stored verbatim as the
/// request's `action`, so the verb is recognised in every language it is
/// lexicalised in without this code naming a single word.
fn action_surface_table() -> Vec<(&'static str, &'static str)> {
    let mut table = Vec::new();
    for meaning in seed::lexicon().meanings_with_role(seed::ROLE_SOFTWARE_AUTHORING_ACTION) {
        let slug = meaning.slug.as_str();
        for surface in meaning.words() {
            table.push((surface, slug));
        }
    }
    table
}

/// Match a software-authoring verb at the start of `input`, returning the
/// consumed byte length and the matched meaning's slug.
fn match_action(
    input: &str,
    table: &[(&'static str, &'static str)],
) -> Option<(usize, &'static str)> {
    for &(surface, slug) in table {
        if input.starts_with(surface) {
            return Some((surface.len(), slug));
        }
    }
    None
}

/// Match a software-artifact phrase at the start of `input`, returning the
/// consumed byte length and the resolved [`ArtifactMatch`].
fn match_artifact(
    input: &str,
    table: &[(&'static str, &'static str)],
) -> Option<(usize, ArtifactMatch)> {
    for &(surface, label) in table {
        if input.starts_with(surface) {
            return Some((surface.len(), ArtifactMatch { surface, label }));
        }
    }
    None
}

/// Walk every word-boundary-aligned position in `normalized` left to right and
/// return the first match the `matcher` accepts that also ends on a word
/// boundary. Position-major: the surface appearing earliest in the prompt wins,
/// independent of table order (ties at one position fall to the matcher's own
/// first-match rule).
fn scan_match<T>(normalized: &str, matcher: impl Fn(&str) -> Option<(usize, T)>) -> Option<T> {
    for (index, _) in normalized.char_indices() {
        if !is_start_boundary(normalized, index) {
            continue;
        }
        if let Some((consumed, value)) = matcher(&normalized[index..]) {
            if consumed > 0 && is_end_boundary(normalized, index + consumed) {
                return Some(value);
            }
        }
    }
    None
}

fn is_start_boundary(value: &str, index: usize) -> bool {
    if index == 0 {
        return true;
    }
    value[..index]
        .chars()
        .next_back()
        .is_some_and(|character| !is_word_character(character))
}

fn is_end_boundary(value: &str, index: usize) -> bool {
    if index >= value.len() {
        return true;
    }
    value[index..]
        .chars()
        .next()
        .is_some_and(|character| !is_word_character(character))
}

/// A "word character" for the recognition scan: an alphanumeric that is *not*
/// CJK.
///
/// CJK scripts write without inter-word spaces, so a CJK surface must match as a
/// substring (every CJK codepoint is its own boundary). Latin, Cyrillic, and
/// Devanagari keep strict whole-token boundaries so a short surface like `апи`
/// never matches inside `напиши`. This mirrors the substring-vs-token contract
/// in [`crate::coding::contains_cjk`].
fn is_word_character(character: char) -> bool {
    character.is_alphanumeric() && !is_cjk_character(character)
}

/// Whether `character` belongs to a CJK script (per the codepoint ranges in
/// [`crate::coding::contains_cjk`]), and so matches as a substring not a token.
fn is_cjk_character(character: char) -> bool {
    let codepoint = character as u32;
    (0x3400..=0x4DBF).contains(&codepoint)
        || (0x4E00..=0x9FFF).contains(&codepoint)
        || (0xF900..=0xFAFF).contains(&codepoint)
        || (0x3040..=0x30FF).contains(&codepoint)
        || (0x3100..=0x312F).contains(&codepoint)
}

fn extract_target(prompt: &str, artifact: ArtifactMatch) -> String {
    let markers = [
        format!("{} for ", artifact.surface),
        format!("{} to ", artifact.surface),
        format!("{} for ", artifact.label),
        format!("{} to ", artifact.label),
        String::from(" for "),
        String::from(" to "),
    ];
    for marker in markers {
        if let Some(target) = extract_after_marker(prompt, &marker) {
            return target;
        }
    }
    String::from("the requested environment")
}

fn extract_after_marker(prompt: &str, marker: &str) -> Option<String> {
    let lower_prompt = prompt.to_lowercase();
    let lower_marker = marker.to_lowercase();
    let start = lower_prompt.find(&lower_marker)? + lower_marker.len();
    let tail = &prompt[start..];
    let stop = tail.find(['?', '.', ',', ';', '\n']).unwrap_or(tail.len());
    let raw = tail[..stop]
        .split(" with ")
        .next()
        .unwrap_or("")
        .split(" that ")
        .next()
        .unwrap_or("")
        .split(" and ")
        .next()
        .unwrap_or("")
        .trim();
    if raw.is_empty() {
        return None;
    }
    Some(capitalize_short_target(raw))
}

fn capitalize_short_target(raw: &str) -> String {
    let compact = raw.split_whitespace().take(5).collect::<Vec<_>>().join(" ");
    if compact.chars().any(char::is_uppercase) {
        return compact;
    }
    let mut chars = compact.chars();
    let Some(first) = chars.next() else {
        return compact;
    };
    format!("{}{}", first.to_uppercase(), chars.collect::<String>())
}

/// Every requirement-marker word — the union of all `software_requirement_category`
/// meanings' surface words, in every supported language, lowercased. A clause
/// containing any of these states a feature requirement. The words live once in
/// the lexicon; this code knows only the concept "a feature requirement".
fn requirement_marker_words() -> Vec<String> {
    seed::lexicon()
        .words_for_role(seed::ROLE_SOFTWARE_REQUIREMENT_CATEGORY)
        .into_iter()
        .filter(|word| !word.is_empty())
        .map(|word| word.to_lowercase())
        .collect()
}

fn extract_requirements(prompt: &str) -> Vec<String> {
    let markers = requirement_marker_words();
    let mut requirements = Vec::new();
    for segment in prompt.split(['.', ';', '\n']) {
        for clause in segment.split(',') {
            let cleaned = clause.trim();
            if cleaned.is_empty() {
                continue;
            }
            let lower = cleaned.to_lowercase();
            if markers.iter().any(|marker| lower.contains(marker.as_str())) {
                push_unique(&mut requirements, sentence_case(cleaned));
            }
        }
    }
    if requirements.is_empty() {
        requirements.push(String::from(
            "Capture state, user commands, persistence, validation, and tests.",
        ));
    }
    requirements
}

fn derive_subtasks(requirements: &[String], game_tracker: bool) -> Vec<SoftwareSubtask> {
    requirements
        .iter()
        .enumerate()
        .map(|(index, requirement)| {
            let category = classify_requirement(requirement, game_tracker);
            SoftwareSubtask {
                requirement_id: format!("R{}", index + 1),
                category,
                title: subtask_title(category, requirement),
            }
        })
        .collect()
}

/// Map a `software_requirement_category` meaning slug to its canonical category
/// label — the slug `subtask_title` matches on and the meaning record emits.
/// Recognition words live in the lexicon; the canonical label lives in code (the
/// calendar `from_slug` precedent). Returns `None` for a slug this handler does
/// not classify, so a future category is skipped rather than mislabelled.
fn requirement_category_label(slug: &str) -> Option<&'static str> {
    let label = match slug {
        "requirement_state_tracking" => "state_tracking",
        "requirement_data_exchange" => "data_exchange",
        "requirement_automation" => "automation",
        "requirement_validation" => "validation",
        "requirement_integration" => "integration",
        "requirement_user_interface" => "user_interface",
        "requirement_project_behavior" => "project_behavior",
        _ => return None,
    };
    Some(label)
}

/// Classify a requirement clause into its canonical category by walking the
/// `software_requirement_category` meanings in declaration order and taking the
/// first whose surface word appears in the clause. A game-unit tracker forces
/// `state_tracking` (the first category, so the order is consistent). Falls back
/// to `project_behavior` when no category matches.
fn classify_requirement(requirement: &str, game_tracker: bool) -> &'static str {
    let lower = requirement.to_lowercase();
    if game_tracker {
        return "state_tracking";
    }
    for meaning in seed::lexicon().meanings_with_role(seed::ROLE_SOFTWARE_REQUIREMENT_CATEGORY) {
        let Some(label) = requirement_category_label(&meaning.slug) else {
            continue;
        };
        if meaning
            .words()
            .any(|word| lower.contains(word.to_lowercase().as_str()))
        {
            return label;
        }
    }
    "project_behavior"
}

fn subtask_title(category: &str, requirement: &str) -> String {
    match category {
        "state_tracking" => format!("Model state fields and pure transitions for {requirement}"),
        "data_exchange" => {
            format!("Define parsers, serializers, and backup flow for {requirement}")
        }
        "automation" => {
            format!("Schedule deterministic jobs and delivery checks for {requirement}")
        }
        "validation" => format!("Encode validation rules and failure messages for {requirement}"),
        "integration" => format!("Isolate host API boundaries and mocks for {requirement}"),
        "user_interface" => format!("Design focused views and state updates for {requirement}"),
        _ => format!("Implement and test the smallest behavior for {requirement}"),
    }
}

/// Pick the delivery mode by walking the `software_delivery_mode` meanings in
/// declaration order (manual instructions → immediate execution → script
/// generation — the order encodes priority) and taking the first one evidenced
/// in the request. When none is, the default is generated code. The surface
/// words live in the lexicon, so the code knows only the concept "a request can
/// ask for a particular delivery mode".
fn detect_delivery_mode(normalized: &str) -> DeliveryMode {
    seed::lexicon()
        .first_role_match(seed::ROLE_SOFTWARE_DELIVERY_MODE, normalized)
        .and_then(|meaning| DeliveryMode::from_slug(&meaning.slug))
        .unwrap_or(DeliveryMode::CodeGeneration)
}

/// Resolve the target language by walking the `software_implementation_language`
/// meanings in declaration order (python → rust → javascript) and taking the
/// first one named in the request. The default is TypeScript.
fn detect_implementation_language(normalized: &str) -> &'static str {
    seed::lexicon()
        .first_role_match(seed::ROLE_SOFTWARE_IMPLEMENTATION_LANGUAGE, normalized)
        .and_then(|meaning| implementation_language_from_slug(&meaning.slug))
        .unwrap_or("typescript")
}

/// Map a `software_implementation_language` meaning slug to its canonical target
/// label. The recognition vocabulary lives in data while the rendered output
/// label stays stable in code (the calendar `from_slug` precedent).
fn implementation_language_from_slug(slug: &str) -> Option<&'static str> {
    match slug {
        "language_python" => Some("python"),
        "language_rust" => Some("rust"),
        "language_javascript" => Some("javascript"),
        _ => None,
    }
}

fn approval_gates(normalized: &str, delivery_mode: DeliveryMode) -> Vec<&'static str> {
    let lexicon = seed::lexicon();
    let mut gates = vec!["task_formalization", "implementation_plan"];
    if lexicon.mentions_role(seed::ROLE_SOFTWARE_FEATURE, normalized) {
        gates.push("requirements");
    }
    if lexicon.mentions_role(seed::ROLE_SOFTWARE_STEP_GRANULARITY, normalized) {
        gates.push("each_step");
    }
    match delivery_mode {
        DeliveryMode::CodeGeneration => gates.push("generated_code"),
        DeliveryMode::ManualInstructions => gates.push("manual_instructions"),
        DeliveryMode::ScriptGeneration | DeliveryMode::ImmediateExecution => {
            gates.push("generated_script");
            gates.push("bash_command");
        }
    }
    if lexicon.mentions_role(seed::ROLE_SOFTWARE_BASH_COMMAND, normalized) {
        gates.push("bash_command");
    }
    gates.sort_unstable();
    gates.dedup();
    gates
}

fn push_unique(items: &mut Vec<String>, value: String) {
    if !items.iter().any(|item| item == &value) {
        items.push(value);
    }
}

fn sentence_case(raw: &str) -> String {
    let trimmed = raw.trim().trim_matches(['-', '*', ' ']);
    let mut chars = trimmed.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    format!("{}{}", first.to_uppercase(), chars.collect::<String>())
}

/// A request is a game-unit tracker only when it pairs a game domain with a
/// combat mechanic — both the `game_tracker_domain` and `game_tracker_mechanic`
/// roles must be evidenced. The decomposition lives in the lexicon, so the code
/// knows only the concept "a tracker needs both a domain and a mechanic".
fn is_game_unit_tracker(normalized: &str) -> bool {
    let lexicon = seed::lexicon();
    lexicon.mentions_role(seed::ROLE_GAME_TRACKER_DOMAIN, normalized)
        && lexicon.mentions_role(seed::ROLE_GAME_TRACKER_MECHANIC, normalized)
}

/// Is the whole prompt a go-ahead that moves the dialogue from plan to
/// implementation? Unlike the passing-mention roles, an approval trigger must
/// match the *entire* compacted prompt (so "approve the validation step" is not
/// an approval), so this compares the compacted prompt against each
/// `software_approval_trigger` surface word. Compaction keeps Unicode letters
/// and digits, so a non-Latin go-ahead works the same way as an English one.
pub(super) fn is_approval_prompt(normalized: &str) -> bool {
    let compact = normalized
        .trim()
        .trim_matches(|character: char| !character.is_alphanumeric())
        .replace(['.', '!', ','], "");
    seed::lexicon()
        .meanings_with_role(seed::ROLE_SOFTWARE_APPROVAL_TRIGGER)
        .flat_map(seed::Meaning::words)
        .any(|word| compact == word)
}

fn reasoning_steps(meaning: &SoftwareProjectMeaning) -> Vec<String> {
    let mut steps = vec![
        format!(
            "Classify the impulse as a request to {} a {} instead of a fact lookup.",
            meaning.action, meaning.artifact
        ),
        format!(
            "Bind the target environment to {} and keep the first response reviewable.",
            meaning.target
        ),
        format!(
            "Extract {} requirement(s) into the meaning record before planning.",
            meaning.requirements.len()
        ),
        format!(
            "Decompose the requirement graph into {} implementation subtask(s) with category labels.",
            meaning.subtasks.len()
        ),
        format!(
            "Select delivery mode {} and approval gates: {}.",
            meaning.delivery_mode.label(),
            meaning.approval_gates.join(", ")
        ),
    ];
    if meaning.game_tracker {
        steps.push(String::from(
            "Map HP, Protection, Resistance, damage, and cooldown phrases to a unit-state domain model.",
        ));
    }
    steps.push(String::from(
        "Ask for approval before producing code, scripts, manual instructions, or execution steps.",
    ));
    steps
}

fn plan_steps(meaning: &SoftwareProjectMeaning) -> Vec<String> {
    let mut steps = Vec::new();
    steps.push(String::from(
        "Review the formalized task, requirement graph, approval gates, and delivery mode with the user.",
    ));
    if meaning.game_tracker {
        steps.extend([
            format!("Confirm the {} storage and selected-token API boundaries.", meaning.target),
            String::from(
                "Define `UnitState` with HP, max HP, Protection, Resistance, and cooldowns.",
            ),
            String::from(
                "Write pure transition functions for damage mitigation, stack edits, and round ticks.",
            ),
            String::from(
                "Add tests for zero damage, overkill damage, stack changes, and cooldown expiry.",
            ),
            String::from("Wire the tested core into the extension panel and host persistence."),
        ]);
        return steps;
    }

    steps.extend([
        format!(
            "Confirm the host API and data boundaries for {}.",
            meaning.target
        ),
        String::from("Define the smallest serializable state records for the requirements."),
    ]);
    for subtask in &meaning.subtasks {
        steps.push(format!(
            "Implement {}: {}.",
            subtask.category, subtask.title
        ));
    }
    steps.push(format!(
        "Generate a {} starter core plus language-appropriate repository initialization and checks.",
        meaning.implementation_language
    ));
    steps.push(String::from(
        "Keep shell, Docker, or WebVM commands behind the configured approval gates.",
    ));
    steps
}

fn render_plan_response(meaning: &SoftwareProjectMeaning) -> String {
    let mut body = String::new();
    let _ = writeln!(
        body,
        "Implementation plan pending approval for a {} targeting {}.",
        meaning.artifact, meaning.target
    );
    body.push('\n');
    body.push_str("Formalized meaning:\n```lino\n");
    body.push_str(&meaning.meaning_lino(ApprovalState::Proposed));
    body.push_str("```\n\nReasoning steps:\n");
    for (index, step) in reasoning_steps(meaning).iter().enumerate() {
        let _ = writeln!(body, "{}. {step}", index + 1);
    }
    body.push_str("\nRequirement model:\n");
    for (index, requirement) in meaning.requirements.iter().enumerate() {
        let category = classify_requirement(requirement, meaning.game_tracker);
        let _ = writeln!(body, "{}. [{category}] {requirement}", index + 1);
    }
    body.push_str("\nSubtasks:\n");
    for (index, subtask) in meaning.subtasks.iter().enumerate() {
        let _ = writeln!(
            body,
            "{}. {} -> {}",
            index + 1,
            subtask.requirement_id,
            subtask.title
        );
    }
    body.push_str("\nApproval gates:\n");
    for gate in &meaning.approval_gates {
        let _ = writeln!(body, "- {gate}");
    }
    body.push_str("\nProposed plan:\n");
    for (index, step) in plan_steps(meaning).iter().enumerate() {
        let _ = writeln!(body, "{}. {step}", index + 1);
    }
    body.push_str(
        "\nReply `approve plan` to generate the starter implementation, or describe what to change.",
    );
    body
}

fn render_implementation_response(meaning: &SoftwareProjectMeaning) -> String {
    let mut body = String::new();
    let _ = writeln!(
        body,
        "Approved implementation starter for a {} targeting {}.",
        meaning.artifact, meaning.target
    );
    body.push('\n');
    body.push_str("Formalized meaning:\n```lino\n");
    body.push_str(&meaning.meaning_lino(ApprovalState::Approved));
    body.push_str("```\n\nImplementation steps:\n");
    for (index, step) in plan_steps(meaning).iter().enumerate() {
        let _ = writeln!(body, "{}. {step}", index + 1);
    }
    let code = implementation_code(meaning.game_tracker, meaning.implementation_language);
    let _ = write!(
        body,
        "\nStarter {} core:\n\n```{}\n",
        code.label, code.fence
    );
    body.push_str(code.body);
    body.push_str("\n```\n");
    body.push_str("\nGenerated code checks:\n");
    let _ = writeln!(
        body,
        "1. Initialize a {} project in an isolated workspace.",
        code.label
    );
    let _ = writeln!(
        body,
        "2. Run the language-native syntax/type check before host integration."
    );
    body
}

pub(super) fn lino_string(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r");
    format!("\"{escaped}\"")
}
