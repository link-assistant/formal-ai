//! Composition of literal declarations and observed syntax only. Manual native pair.
use super::final_result::FinalResult;
use super::manifest_documentation::derive_manifest_documentation;
use super::observed_documentation::plan_observed_documentation;
use super::observed_source_usage::observed_source_usage_operation;
use super::planner::AgenticPlan;
use super::source_documentation::{documentation_authority, source_documentation_policy};
use super::work_item_evidence::work_item_evidence_paths;
use crate::protocol::ChatMessage;
use serde_json::{Value, json};
use std::sync::OnceLock;

fn manifest_path(path: &str) -> bool {
    path.ends_with(".json") || path.ends_with(".toml")
}
fn source_path(path: &str) -> bool {
    [".mjs", ".js", ".ts", ".rs"]
        .iter()
        .any(|suffix| path.ends_with(suffix))
}
fn policy_words(policy: &Value, key: &str) -> Vec<String> {
    policy
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}
/// A unique explicit guide and sources identify custody; authority remains separate.
pub(super) fn repository_documentation_contract(task: &str, policy: &Value) -> Option<Value> {
    let paths = work_item_evidence_paths(task);
    let guides: Vec<_> = paths
        .iter()
        .copied()
        .filter(|path| path.ends_with(".md"))
        .collect();
    let manifests: Vec<_> = paths
        .iter()
        .copied()
        .filter(|path| manifest_path(path))
        .collect();
    let source_paths: Vec<_> = paths
        .iter()
        .copied()
        .filter(|path| source_path(path))
        .collect();
    if paths.len() < 3
        || guides.len() != 1
        || manifests.is_empty()
        || paths.len() != guides.len() + manifests.len() + source_paths.len()
    {
        return None;
    }
    let authority = documentation_authority(
        task,
        guides[0],
        &policy_words(policy, "actions"),
        &policy_words(policy, "preserve"),
        &policy_words(policy, "negations"),
        &policy_words(policy, "prefixes"),
        false,
        false,
    )?;
    let sources: Vec<_> = manifests.into_iter().chain(source_paths).collect();
    Some(
        json!({"sources":sources,"target":guides[0],"authority":authority,
        "schema":"repository-work-item-controller/v1",
        "coverage":"explicit-source-paths-only; consumer-discovery-pending"}),
    )
}
fn fill(template: &str, bindings: &[(&str, &str)]) -> String {
    bindings
        .iter()
        .fold(template.to_owned(), |text, (key, value)| {
            text.replace(&format!("{{{key}}}"), value)
        })
}
fn caption_safe(value: &str) -> bool {
    static UNSAFE_CAPTION: OnceLock<regex::Regex> = OnceLock::new();
    !UNSAFE_CAPTION
        .get_or_init(|| regex::Regex::new(r"[`\r\n\p{Cc}\p{Cf}]").expect("StaticSyntaxCaption"))
        .is_match(value)
}
fn snippet(value: &Value) -> Option<&str> {
    value
        .as_str()
        .filter(|source| source.len() <= 512 && !source.contains("```"))
}
fn syntax_paragraph(owner: &str, usage: &Value, policy: &Value) -> Option<String> {
    let forms = policy.get("sourceUsage")?;
    let heading = forms.get("heading")?.as_str()?;
    let call_form = forms.get("call")?.as_str()?;
    let reference_form = forms.get("reference")?.as_str()?;
    let code_open = forms.get("codeOpen")?.as_str()?;
    let code_close = forms.get("codeClose")?.as_str()?;
    let language = usage.get("language")?.as_str()?.to_lowercase();
    let fence = fill(code_open, &[("language", &language)]);
    let mut rows = Vec::new();
    if let Some(calls) = usage.get("calls").filter(|value| !value.is_null()) {
        for call in calls.as_array()? {
            let source = snippet(&call["source"])?;
            let imported = call["imported"]
                .as_str()
                .filter(|value| caption_safe(value))?;
            let module = call["module"]
                .as_str()
                .filter(|value| caption_safe(value))?;
            let caption = fill(
                call_form,
                &[("owner", owner), ("imported", imported), ("module", module)],
            );
            rows.push(format!("{caption}\n\n{fence}\n{source}\n{code_close}"));
        }
    }
    if let Some(references) = usage.get("references").filter(|value| !value.is_null()) {
        for reference in references.as_array()? {
            let source = snippet(&reference["source"])?;
            let caption = fill(reference_form, &[("owner", owner)]);
            rows.push(format!("{caption}\n\n{fence}\n{source}\n{code_close}"));
        }
    }
    if rows.is_empty() {
        return None;
    }
    Some(format!(
        "{}\n\n{}",
        fill(heading, &[("owner", owner)]),
        rows.join("\n\n")
    ))
}

/// Pure composition: preserve manifest and syntax evidence as separate fact kinds.
pub(super) fn derive_repository_documentation(
    task: &str,
    sources: &[Value],
    guide: &str,
    policy: &Value,
    usage: impl Fn(&Value) -> Result<Value, String>,
) -> Value {
    let mut content = guide.to_owned();
    let mut facts = Vec::new();
    let mut remaining_work = vec![
        "independent-integration-semantics".to_owned(),
        "consumer-discovery".to_owned(),
    ];
    for source in sources
        .iter()
        .filter(|row| row["path"].as_str().is_some_and(manifest_path))
    {
        let path = source["path"].as_str().unwrap_or_default();
        let source_text = source["source"].as_str().unwrap_or_default();
        let derived =
            derive_manifest_documentation(task, path, source_text, &content, Some(policy));
        if derived
            .get("gap")
            .is_some_and(|value| value.as_str().is_some_and(|value| !value.is_empty()))
        {
            return derived;
        }
        let Some(next) = derived.get("content").and_then(Value::as_str) else {
            return json!({"gap":"DocumentationDerivationInvalid"});
        };
        content = next.to_owned();
        if let Some(rows) = derived
            .get("facts")
            .filter(|value| !value.is_null())
            .or_else(|| derived.get("value"))
            .and_then(Value::as_array)
        {
            facts.extend(rows.iter().cloned());
        }
    }
    for source in sources
        .iter()
        .filter(|row| row["path"].as_str().is_some_and(source_path))
    {
        let owner = source["path"].as_str().unwrap_or_default();
        let language = if owner.ends_with(".rs") {
            "Rust"
        } else if owner.ends_with(".ts") {
            "TypeScript"
        } else {
            "JavaScript"
        };
        let request = json!({"owner":owner,"source":source["source"],"language":language});
        let observed = match usage(&request) {
            Ok(observed) => observed,
            Err(_) => {
                remaining_work.push(format!("source-syntax-unavailable:{owner}"));
                continue;
            }
        };
        if observed["schema"] != "observed-source-usage/v1"
            || observed["owner"] != owner
            || observed["sourceIdentity"] != source["sourceIdentity"]
            || observed["language"] != language
        {
            remaining_work.push(format!("source-syntax-unbound:{owner}"));
            continue;
        }
        let Some(paragraph) = syntax_paragraph(owner, &observed, policy) else {
            remaining_work.push(format!("source-syntax-rendering-unavailable:{owner}"));
            continue;
        };
        let heading = fill(
            policy["sourceUsage"]["heading"]
                .as_str()
                .unwrap_or_default(),
            &[("owner", owner)],
        );
        if !content.contains(&paragraph) && content.contains(&heading) {
            return json!({"gap":"StaleSourceUsageDocumentation"});
        }
        if !content.contains(&paragraph) {
            content = format!(
                "{}\n\n{paragraph}\n",
                content.strip_suffix('\n').unwrap_or(&content)
            );
        }
        facts.push(json!({"kind":"observed-source-usage","owner":observed["owner"],"sourceIdentity":observed["sourceIdentity"],
            "language":observed["language"],"imports":observed["imports"],"calls":observed["calls"],
            "references":observed["references"],"gaps":observed["gaps"]}));
        if observed
            .get("gaps")
            .and_then(Value::as_array)
            .is_some_and(|gaps| !gaps.is_empty())
        {
            remaining_work.push(format!("source-resolution-unverified:{owner}"));
        }
    }
    json!({"content":content,"facts":facts,"remainingWork":remaining_work,
        "binding":{"kind":"composed-source-observations",
            "sources":sources.iter().map(|row| json!({"path":row["path"],"sourceIdentity":row["sourceIdentity"]})).collect::<Vec<_>>(),
            "coverage":"explicit-source-paths-only; no executable-role or whole-task acceptance credit"}})
}

/// Run the shared custody state machine using the actual native syntax operation.
pub(super) fn plan_repository_documentation(
    task: &str,
    messages: &[ChatMessage],
    tools: &[&str],
    work_item: Option<&str>,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let policy = source_documentation_policy()?;
    let mut contract = repository_documentation_contract(task, &policy)?;
    contract["workItem"] = json!(work_item);
    Some(plan_observed_documentation(
        task,
        messages,
        tools,
        &contract,
        |sources, guide| {
            derive_repository_documentation(
                task,
                sources,
                guide,
                &policy,
                observed_source_usage_operation,
            )
        },
        result,
    ))
}

#[cfg(test)]
#[path = "../../tests/unit/agentic-coding/repository_documentation.rs"]
mod tests;
