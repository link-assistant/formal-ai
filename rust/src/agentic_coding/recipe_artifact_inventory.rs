//! Read-only source-owned tracked artifacts; callers bind the original issue URL.

use crate::seed::parser::LinoNode;
use serde_json::{Value, json};
use std::sync::OnceLock;

fn policy() -> &'static LinoNode {
    static POLICY: OnceLock<LinoNode> = OnceLock::new();
    POLICY.get_or_init(|| {
        crate::seed::parser::parse_lino(include_str!(
            "../../embedded/data/meta/prepared-artifact-inventory.lino"
        ))
    })
}
fn value(name: &str) -> &'static str {
    policy()
        .children
        .first()
        .map_or("", |root| root.find_child_value(name))
}
fn lines(root: &LinoNode, name: &str) -> Vec<String> {
    root.children
        .iter()
        .flat_map(|root| &root.children)
        .filter(|node| node.name == name)
        .map(|node| node.id.clone())
        .collect()
}
/// Bind only the validated original issue, never an arbitrary task directory.
#[must_use]
pub(super) fn recipe_artifact_evidence_root(issue_reference: &str) -> Option<String> {
    let reference = super::general_planner::repository_work_reference(issue_reference)?;
    let segments = reference.split('/').collect::<Vec<_>>();
    let number = *segments.get(6)?;
    if reference != issue_reference
        || segments.get(5) != Some(&"issues")
        || number.is_empty()
        || number.starts_with('0')
        || !number.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    Some(crate::seed::fill_template_once(
        value("evidence-template"),
        &[("number", number)],
    ))
}
fn descriptor(source_paths: &[String], evidence_root: &str) -> Option<Value> {
    let portable = |path: &str| {
        !path.is_empty()
            && !path.starts_with('/')
            && !path.split('/').any(|part| part == "..")
            && path
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"_./-".contains(&byte))
    };
    if source_paths.is_empty()
        || source_paths.len() > 256
        || !source_paths.iter().all(|path| portable(path))
        || source_paths
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != source_paths.len()
        || regex::Regex::new(value("evidence-pattern"))
            .ok()?
            .find(evidence_root)?
            .as_str()
            != evidence_root
    {
        return None;
    }
    let hygiene = crate::seed::parser::parse_lino(include_str!(
        "../../embedded/data/meta/restart-hygiene.lino"
    ));
    let flags = lines(&hygiene, "output-flag");
    let mut outputs = Vec::new();
    for language in crate::coding::program_languages() {
        let source = language.save_as.as_ref();
        if !source_paths.iter().any(|path| path == source) {
            continue;
        }
        let words = language
            .execution
            .check_command
            .as_deref()
            .unwrap_or_default()
            .split_whitespace()
            .collect::<Vec<_>>();
        if !words.contains(&source) {
            continue;
        }
        for pair in words.windows(2) {
            if flags.iter().any(|flag| flag == pair[0]) && portable(pair[1]) && pair[1] != source {
                let entry = json!({"source":source,"output":pair[1]});
                if !outputs.contains(&entry) {
                    outputs.push(entry);
                }
            }
        }
    }
    let streams = lines(policy(), "stream");
    let mut candidates = Vec::<String>::new();
    for output in &outputs {
        let path = output.get("output")?.as_str()?.to_owned();
        if !candidates.contains(&path) {
            candidates.push(path);
        }
    }
    candidates.extend(
        streams
            .iter()
            .map(|member| format!("{evidence_root}{member}")),
    );
    candidates.push(format!("{evidence_root}session-id.txt"));
    let placeholders = lines(policy(), "empty-placeholder");
    candidates.extend(placeholders.iter().cloned());
    let library = crate::seed::parser::parse_lino(include_str!(
        "../../embedded/data/meta/workspace-source-io.lino"
    ));
    let mut python = lines(&library, "source-line");
    python.extend(lines(policy(), "python-line"));
    Some(
        json!({"sourcePaths":source_paths,"evidenceRoot":evidence_root,"outputs":outputs,
        "candidates":candidates,"streams":streams,"emptyPlaceholders":placeholders,"sessionPattern":value("session-pattern"),
        "sessionLimit":value("session-limit").parse::<usize>().ok()?,
        "binaryPrefixes":lines(&hygiene,"binary-prefix"),"python":python.join("\n")+"\n"}),
    )
}
/// Emit the canonical bounded supervisor plus the shared regular-file reader.
#[must_use]
pub(super) fn recipe_artifact_inventory_command(
    source_paths: &[String],
    evidence_root: &str,
) -> Option<String> {
    let plan = descriptor(source_paths, evidence_root)?;
    let kernel = crate::seed::parser::parse_lino(include_str!(
        "../../embedded/data/meta/bounded-command-capture-source.lino"
    ));
    let mut source = lines(&kernel, "line");
    source.extend(lines(policy(), "source-line"));
    let source = super::git_commit::shell_quote(&(source.join("\n") + "\n"));
    let plan = super::git_commit::shell_quote(&plan.to_string());
    Some(crate::seed::fill_template_once(
        value("command"),
        &[("source", &source), ("plan", &plan)],
    ))
}
/// Typed evidence; its caller still requires the exact successful command receipt.
#[derive(Debug, Clone)]
pub(super) struct RecipeArtifactInventory {
    pub owned_paths: Vec<String>,
    pub source_paths: Vec<String>,
    pub evidence_root: String,
    pub session_policy: Value,
}
/// Reject foreign paths, mismatched roots and partial source projections.
#[must_use]
pub(super) fn recipe_artifact_inventory_result(
    output: &str,
    source_paths: &[String],
    evidence_root: &str,
) -> Option<RecipeArtifactInventory> {
    let plan = descriptor(source_paths, evidence_root)?;
    let payload = super::tool_result::observed_payload(output)?;
    let result: Value = serde_json::from_str(&payload).ok()?;
    if result.get("schema")?.as_str()? != "prepared-artifact-inventory/v1"
        || !result.get("complete")?.as_bool()?
        || result.get("evidenceRoot")?.as_str()? != evidence_root
        || result.get("sourcePaths")? != &json!(source_paths)
    {
        return None;
    }
    let paths = result.get("ownedPaths")?.as_array()?;
    if paths.len() > 256 {
        return None;
    }
    let candidates = plan.get("candidates")?.as_array()?;
    let mut owned_paths = Vec::new();
    for path in paths {
        let text = path.as_str()?.to_owned();
        if !candidates.contains(path) || owned_paths.contains(&text) {
            return None;
        }
        owned_paths.push(text);
    }
    Some(RecipeArtifactInventory {
        owned_paths,
        source_paths: source_paths.to_vec(),
        evidence_root: evidence_root.to_owned(),
        session_policy: json!({"evidenceRoot":evidence_root,"filename":"session-id.txt","pattern":value("session-pattern"),"maximumBytes":plan.get("sessionLimit")?,
            "path":format!("{evidence_root}session-id.txt")}),
    })
}
