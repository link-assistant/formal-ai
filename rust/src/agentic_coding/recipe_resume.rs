//! Client-owned source epochs and bounded repository completion.
use super::planner::{AgenticPlan, Capability, plan_one, tool_for};
use super::progress::Progress;
use super::qualified_tool_observation::Binding;
use super::tool_result::{
    StepOutcome, command_argument, incomplete_receipt, observed_payload, reported_exit_code,
    step_outcome,
};
use crate::engine::ExecutionRecipe;
use crate::protocol::ChatMessage;
use serde_json::json;
use std::sync::OnceLock;

/// Complete source evidence; absence never certifies a recipe artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RecipeSourceResult {
    pub absent: bool,
    pub source: Option<String>,
    pub sha256: Option<String>,
    pub identity: Option<Vec<String>>,
}

fn policy() -> &'static crate::seed::parser::LinoNode {
    static POLICY: OnceLock<crate::seed::parser::LinoNode> = OnceLock::new();
    POLICY.get_or_init(|| {
        crate::seed::parser::parse_lino(include_str!("../../embedded/data/meta/recipe-resume.lino"))
    })
}
fn value(name: &str) -> &'static str {
    policy()
        .children
        .first()
        .map_or("", |root| root.find_child_value(name))
}
fn lines(root: &crate::seed::parser::LinoNode, name: &str) -> Vec<String> {
    root.children
        .iter()
        .flat_map(|root| &root.children)
        .filter(|node| node.name == name)
        .map(|node| node.id.clone())
        .collect()
}
fn source_program() -> &'static str {
    static SOURCE: OnceLock<String> = OnceLock::new();
    SOURCE.get_or_init(|| {
        let library = crate::seed::parser::parse_lino(include_str!(
            "../../embedded/data/meta/workspace-source-io.lino"
        ));
        let mut source = lines(&library, "source-line");
        source.extend(lines(policy(), "source-line"));
        source.join("\n") + "\n"
    })
}
fn ci_program() -> &'static str {
    static SOURCE: OnceLock<String> = OnceLock::new();
    SOURCE.get_or_init(|| {
        let kernel = crate::seed::parser::parse_lino(include_str!(
            "../../embedded/data/meta/bounded-command-capture-source.lino"
        ));
        let mut source = lines(&kernel, "line");
        source.extend(lines(policy(), "ci-source-line"));
        source.join("\n") + "\n"
    })
}
fn command(template: &str, source: &str, plan: &serde_json::Value) -> String {
    let source = super::git_commit::shell_quote(source);
    let plan = super::git_commit::shell_quote(&plan.to_string());
    crate::seed::fill_template_once(template, &[("source", &source), ("plan", &plan)])
}
/// Lower a source-owned probe without server filesystem authority.
#[must_use]
pub(super) fn recipe_source_command(path: &str) -> String {
    command(value("command"), source_program(), &json!({"path":path}))
}
/// Decode only after the caller proves the exact successful command receipt.
#[must_use]
pub(super) fn recipe_source_result(output: &str, path: &str) -> Option<RecipeSourceResult> {
    let payload = observed_payload(output)?;
    let value: serde_json::Value = serde_json::from_str(&payload).ok()?;
    if value.get("schema")?.as_str()? != "workspace-source-v1"
        || value.get("path")?.as_str()? != path
        || !value.get("success")?.as_bool()?
        || !value.get("complete")?.as_bool()?
    {
        return None;
    }
    let absent = value.get("absent")?.as_bool()?;
    if absent && value.get("error_code")?.as_str()? == "ENOENT" && value.get("source").is_none() {
        return Some(RecipeSourceResult {
            absent: true,
            source: None,
            sha256: None,
            identity: None,
        });
    }
    if !absent && value.get("error_code").is_none() {
        return Some(RecipeSourceResult {
            absent: false,
            source: Some(value.get("source")?.as_str()?.to_owned()),
            sha256: value
                .get("sha256")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned),
            identity: value
                .get("identity")
                .and_then(|value| serde_json::from_value(value.clone()).ok()),
        });
    }
    None
}
/// Source constructors decode semantic receipts without inventing a missing process status.
#[must_use]
pub(super) fn recipe_source_receipt<'a>(progress: &'a Progress, command: &str) -> Option<&'a str> {
    let frame = progress.qualified_command_attempt(command)?;
    if frame.binding != Binding::Exact
        || !frame.succeeded
        || incomplete_receipt(&frame.detail)
        || reported_exit_code(&frame.detail).is_some_and(|code| code != 0)
    {
        return None;
    }
    Some(&frame.detail)
}

/// Exact source evidence is invalid after a later write to that owner.
#[must_use]
pub(super) fn recipe_source_matches(progress: &Progress, path: &str, source: &str) -> bool {
    let command = recipe_source_command(path);
    let Some(receipt) = recipe_source_receipt(progress, &command) else {
        return false;
    };
    let Some(result) = recipe_source_result(receipt, path) else {
        return false;
    };
    if result.absent || result.source.as_deref() != Some(source) {
        return false;
    }
    let probe = progress.research_attempts().iter().rposition(|attempt| {
        attempt.capability == Capability::Run
            && attempt
                .arguments
                .as_deref()
                .and_then(command_argument)
                .is_some_and(|value| value == command)
    });
    probe.is_some_and(|probe| {
        progress
            .latest_source_mutation_index(path)
            .is_none_or(|write| write < probe)
    })
}
/// Emit the actual shared bounded kernel with a source-owned CI observer.
#[must_use]
pub(super) fn recipe_ci_command(reference: &str, workflow: &str, ready: bool) -> String {
    command(
        value("ci-command"),
        ci_program(),
        &json!({"reference":reference,"workflow":workflow,"ready":ready}),
    )
}
#[derive(Debug)]
pub(super) struct RecipeCiResult {
    pub head: String,
    pub is_draft: bool,
}
/// Bind the typed producer output to the exact requested PR and workflow.
#[must_use]
pub(super) fn recipe_ci_result(
    output: &str,
    reference: &str,
    workflow: &str,
    ready: bool,
) -> Option<RecipeCiResult> {
    let value: serde_json::Value = serde_json::from_str(&observed_payload(output)?).ok()?;
    if value.get("schema")?.as_str()? != "recipe-ci-v1"
        || !(1..=9_007_199_254_740_991).contains(&value.get("checks")?.as_u64()?)
    {
        return None;
    }
    receipt_identity(&value, reference, workflow, ready)
}
fn receipt_identity(
    value: &serde_json::Value,
    reference: &str,
    workflow: &str,
    ready: bool,
) -> Option<RecipeCiResult> {
    let head = value.get("head")?.as_str()?;
    let branch = value.get("branch")?.as_str()?;
    if !value.get("success")?.as_bool()?
        || !value.get("complete")?.as_bool()?
        || value.get("reference")?.as_str()? != reference
        || value.get("workflow")?.as_str()? != workflow
        || value.get("ready")?.as_bool()? != ready
        || head.len() != 40
        || !head
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        || branch.is_empty()
    {
        return None;
    }
    Some(RecipeCiResult {
        head: head.to_owned(),
        is_draft: value.get("isDraft")?.as_bool()?,
    })
}
fn staged_ready_command(
    reference: &str,
    workflow: &str,
    path: &str,
    source: &str,
    ready: bool,
) -> String {
    command(
        value("ci-command"),
        ci_program(),
        &json!({"reference":reference,"workflow":workflow,"ready":ready,
            "operation":"staged-ready","path":path,"source":source}),
    )
}
fn staged_ready_result(
    output: &str,
    reference: &str,
    workflow: &str,
    path: &str,
    ready: bool,
) -> Option<RecipeCiResult> {
    let value: serde_json::Value = serde_json::from_str(&observed_payload(output)?).ok()?;
    if value.get("schema")?.as_str()? != "recipe-staged-ready-v1"
        || value.get("path")?.as_str()? != path
        || value.get("checks")?.as_u64()? != 0
        || value.get("ciVerified")?.as_bool()?
    {
        return None;
    }
    receipt_identity(&value, reference, workflow, ready)
}

/// Repository obligations admitted by the source recipe route.
#[derive(Clone, Copy)]
pub(super) struct RecipeCompletion<'a> {
    pub recipe: &'a ExecutionRecipe,
    pub reference: &'a str,
    pub workflow: &'a str,
    pub followup: &'a [String],
    pub local_verified: bool,
    pub commit_reference: &'a str,
}
enum Observation<'a> {
    Missing,
    Failed,
    Succeeded(&'a str),
}
fn observation<'a>(progress: &'a Progress, command: &str) -> Observation<'a> {
    let Some(frame) = progress.qualified_command_attempt(command) else {
        return Observation::Missing;
    };
    if frame.binding == Binding::Exact
        && frame.succeeded
        && !incomplete_receipt(&frame.detail)
        && step_outcome(&frame.detail) != StepOutcome::Failed
    {
        Observation::Succeeded(&frame.detail)
    } else {
        Observation::Failed
    }
}
fn call(run: &str, command: &str) -> AgenticPlan {
    plan_one(run, json!({"command":command}).to_string())
}
fn gap(name: &str) -> AgenticPlan {
    AgenticPlan::Final(value(name).to_owned())
}
fn plan_staged_completion(
    run: &str,
    progress: &Progress,
    completion: RecipeCompletion<'_>,
    path: &str,
    source: &str,
) -> AgenticPlan {
    if let Some(comments) = completion.followup.first() {
        match observation(progress, comments) {
            Observation::Missing => return call(run, comments),
            Observation::Failed => return gap("scope-refusal"),
            Observation::Succeeded(_) => {}
        }
    }
    let view = staged_ready_command(
        completion.reference,
        completion.workflow,
        path,
        source,
        false,
    );
    let output = match observation(progress, &view) {
        Observation::Missing => return call(run, &view),
        Observation::Failed => return gap("scope-refusal"),
        Observation::Succeeded(output) => output,
    };
    let Some(before) = staged_ready_result(
        output,
        completion.reference,
        completion.workflow,
        path,
        false,
    ) else {
        return gap("scope-refusal");
    };
    if before.is_draft
        && let Some(edit) = completion.followup.get(1)
    {
        match observation(progress, edit) {
            Observation::Missing => return call(run, edit),
            Observation::Failed => return gap("scope-refusal"),
            Observation::Succeeded(_) => {}
        }
    }
    let ready = staged_ready_command(
        completion.reference,
        completion.workflow,
        path,
        source,
        true,
    );
    let output = match observation(progress, &ready) {
        Observation::Missing => return call(run, &ready),
        Observation::Failed => return gap("scope-refusal"),
        Observation::Succeeded(output) => output,
    };
    let Some(after) = staged_ready_result(
        output,
        completion.reference,
        completion.workflow,
        path,
        true,
    ) else {
        return gap("scope-refusal");
    };
    if after.is_draft || after.head != before.head {
        return gap("scope-refusal");
    }
    let report = if completion.local_verified {
        "staged-local-report"
    } else {
        "staged-unverified-report"
    };
    AgenticPlan::Final(crate::seed::fill_template_once(
        value(report),
        &[("head", &after.head)],
    ))
}
/// Preserve owned changes; staged readiness never certifies hosted CI completion.
#[must_use]
pub(super) fn plan_recipe_ci_fallback(
    messages: &[ChatMessage],
    tools: &[&str],
    completion: RecipeCompletion<'_>,
) -> AgenticPlan {
    let Some(run) = tool_for(tools, Capability::Run) else {
        return gap("scope-refusal");
    };
    let progress = Progress::scan(messages);
    let status = match observation(&progress, value("status-command")) {
        Observation::Missing => return call(run, value("status-command")),
        Observation::Failed => return gap("scope-refusal"),
        Observation::Succeeded(output) => output,
    };
    let Some(paths) = super::restart_feedback::prepared_work_paths(status) else {
        return gap("scope-refusal");
    };
    let source_paths = std::iter::once(completion.recipe.path.clone())
        .chain(
            completion
                .recipe
                .supporting_files
                .iter()
                .map(|file| file.path.clone()),
        )
        .collect::<Vec<_>>();
    let Some(evidence_root) = super::recipe_artifact_inventory::recipe_artifact_evidence_root(
        completion.commit_reference,
    ) else {
        return gap("scope-refusal");
    };
    let Some(inventory_command) =
        super::recipe_artifact_inventory::recipe_artifact_inventory_command(
            &source_paths,
            &evidence_root,
            Some(completion.commit_reference),
        )
    else {
        return gap("scope-refusal");
    };
    let inventory_output = match observation(&progress, &inventory_command) {
        Observation::Missing => return call(run, &inventory_command),
        Observation::Failed => return gap("scope-refusal"),
        Observation::Succeeded(output) => output,
    };
    let Some(inventory) = super::recipe_artifact_inventory::recipe_artifact_inventory_result(
        inventory_output,
        &source_paths,
        &evidence_root,
        Some(completion.commit_reference),
    ) else {
        return gap("scope-refusal");
    };
    if !paths.is_empty() || !inventory.owned_paths.is_empty() {
        let Some(hygiene) = super::restart_feedback::prepared_recipe_hygiene_with_inventory(
            &source_paths,
            &paths,
            Some(&inventory),
            Some(completion.commit_reference),
        ) else {
            return gap("scope-refusal");
        };
        let Some(commit) = super::restart_feedback::prepared_work_commit_command(
            &hygiene,
            completion.commit_reference,
        ) else {
            return gap("scope-refusal");
        };
        for command in [&hygiene.command, &commit, &hygiene.clean_command] {
            match observation(&progress, command) {
                Observation::Missing => return call(run, command),
                Observation::Failed => return gap("scope-refusal"),
                Observation::Succeeded(_) => {}
            }
        }
    }
    if let Some(file) = completion.recipe.supporting_files.iter().find(|file| {
        !value("staged-workflow-prefix").is_empty()
            && file.path.starts_with(value("staged-workflow-prefix"))
            && file
                .path
                .rsplit_once('.')
                .is_some_and(|(_, extension)| matches!(extension, "yml" | "yaml"))
            && file
                .source
                .lines()
                .find_map(|line| line.strip_prefix("name: "))
                == Some(completion.workflow)
    }) {
        return plan_staged_completion(run, &progress, completion, &file.path, &file.source);
    }
    let ci = recipe_ci_command(completion.reference, completion.workflow, false);
    let output = match observation(&progress, &ci) {
        Observation::Missing => return call(run, &ci),
        Observation::Failed => return gap("ci-pending-report"),
        Observation::Succeeded(output) => output,
    };
    let Some(evidence) = recipe_ci_result(output, completion.reference, completion.workflow, false)
    else {
        return gap("scope-refusal");
    };
    if let Some(comments) = completion.followup.first() {
        match observation(&progress, comments) {
            Observation::Missing => return call(run, comments),
            Observation::Failed => return gap("scope-refusal"),
            Observation::Succeeded(_) => {}
        }
    }
    if evidence.is_draft
        && let Some(edit) = completion.followup.get(1)
    {
        match observation(&progress, edit) {
            Observation::Missing => return call(run, edit),
            Observation::Failed => return gap("scope-refusal"),
            Observation::Succeeded(_) => {}
        }
    }
    let ready = recipe_ci_command(completion.reference, completion.workflow, true);
    let output = match observation(&progress, &ready) {
        Observation::Missing => return call(run, &ready),
        Observation::Failed => return gap("ci-pending-report"),
        Observation::Succeeded(output) => output,
    };
    let confirmed = recipe_ci_result(output, completion.reference, completion.workflow, true);
    if confirmed.is_none_or(|confirmed| confirmed.is_draft || confirmed.head != evidence.head) {
        return gap("ci-pending-report");
    }
    let name = if completion.local_verified {
        "ci-verified-local-report"
    } else {
        "ci-verified-report"
    };
    AgenticPlan::Final(crate::seed::fill_template_once(
        value(name),
        &[("head", &evidence.head)],
    ))
}

/// Delivery binding from an explicit, unquoted latest-user handoff.
pub(super) enum RecipeDeliveryReference {
    Absent,
    Bound(String),
    Refused(String),
}

/// Preserve the issue reference for requirements and admit only its repository's PR.
#[must_use]
pub(super) fn recipe_delivery_reference(
    request: &str,
    requirements_reference: Option<&str>,
) -> RecipeDeliveryReference {
    let patterns: Vec<String> = serde_json::from_str(value("prepared-pull-request-patterns"))
        .expect("valid prepared pull request policy");
    let fences: Vec<String> = serde_json::from_str(&super::work_item_steps::fill(
        "owned-clause-fence-prefixes",
        &[],
    ))
    .expect("valid owned clause fence policy");
    let quoted = crate::normal_markov::quoted_segment_spans(request);
    let mut references = std::collections::BTreeSet::new();
    let mut inside_code_fence = false;
    let mut offset = 0;
    for raw in request.split_inclusive('\n') {
        let start = offset;
        offset += raw.len();
        let line = raw.trim();
        if fences
            .iter()
            .any(|prefix| line.starts_with(prefix.as_str()))
        {
            inside_code_fence = !inside_code_fence;
            continue;
        }
        let owned_at = start + raw.find(line).unwrap_or(0);
        if inside_code_fence
            || quoted
                .iter()
                .any(|segment| segment.start <= owned_at && owned_at < segment.end)
        {
            continue;
        }
        for pattern in &patterns {
            let regex = regex::RegexBuilder::new(pattern)
                .case_insensitive(true)
                .build()
                .expect("valid prepared pull request pattern");
            let Some(matched) = regex.captures(line).and_then(|captures| captures.get(1)) else {
                continue;
            };
            let candidate = matched.as_str();
            let Some(reference) = super::general_planner::repository_work_reference(candidate)
            else {
                return RecipeDeliveryReference::Refused(value("scope-refusal").to_owned());
            };
            let segments: Vec<&str> = reference.split('/').collect();
            let requirements: Vec<&str> = requirements_reference
                .unwrap_or_default()
                .split('/')
                .collect();
            let number = segments.get(6).copied().unwrap_or_default();
            if reference != candidate
                || segments.get(5) != Some(&"pull")
                || number.is_empty()
                || !number.bytes().all(|byte| byte.is_ascii_digit())
                || !number.bytes().any(|byte| byte != b'0')
                || segments.get(3) != requirements.get(3)
                || segments.get(4) != requirements.get(4)
            {
                return RecipeDeliveryReference::Refused(value("scope-refusal").to_owned());
            }
            references.insert(reference);
        }
    }
    match references.len() {
        0 => RecipeDeliveryReference::Absent,
        1 => RecipeDeliveryReference::Bound(references.into_iter().next().expect("one reference")),
        _ => RecipeDeliveryReference::Refused(value("scope-refusal").to_owned()),
    }
}

/// A later source mutation cannot be verified by an earlier artifact observation.
#[must_use]
pub(super) fn recipe_source_refusal() -> AgenticPlan {
    gap("scope-refusal")
}

/// Construct only an admitted full-source replacement tied to a complete physical preimage.
#[must_use]
pub(super) fn recipe_source_update_command(
    path: &str,
    source: &str,
    before: &RecipeSourceResult,
) -> Option<String> {
    static SOURCE: OnceLock<String> = OnceLock::new();
    let digest = before.sha256.as_deref()?;
    let identity = before.identity.as_ref()?;
    if before.absent
        || digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        || identity.len() != 5
        || !identity
            .iter()
            .all(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return None;
    }
    let program = SOURCE.get_or_init(|| {
        let library = crate::seed::parser::parse_lino(include_str!(
            "../../embedded/data/meta/workspace-source-io.lino"
        ));
        let mut source = lines(&library, "source-line");
        source.extend(lines(policy(), "update-source-line"));
        source.join("\n") + "\n"
    });
    Some(command(
        value("command"),
        program,
        &json!({"path":path,"source":source,"before":{"sha256":digest,"identity":identity}}),
    ))
}
/// The exact bound command must also have an authoritative successful exit status.
#[must_use]
pub(super) fn recipe_source_update_result(output: &str, path: &str) -> bool {
    observed_payload(output)
        .and_then(|payload| serde_json::from_str::<serde_json::Value>(&payload).ok())
        .is_some_and(|value| {
            value.get("schema").and_then(serde_json::Value::as_str)
                == Some("workspace-source-update-v1")
                && value.get("path").and_then(serde_json::Value::as_str) == Some(path)
                && value.get("success").and_then(serde_json::Value::as_bool) == Some(true)
                && value.get("complete").and_then(serde_json::Value::as_bool) == Some(true)
        })
}

#[cfg(test)]
#[path = "../../tests/fixtures/recipe-staged-readiness.rs"]
mod staged_receipt_tests;
