//! Composition of literal declarations and observed syntax only. Manual native pair.
use super::capability_router::shell_command_tool;
use super::final_result::FinalResult;
use super::manifest_documentation::derive_manifest_documentation;
use super::observed_documentation::{
    complete as discovery_source, owner_of as discovery_owner, plan_observed_documentation,
};
use super::observed_source_usage::observed_source_usage_operation;
use super::planner::{AgenticPlan, Capability};
use super::source_documentation::{documentation_authority, source_documentation_policy};
use super::source_lexical_policy::source_lexical_rule;
use super::source_manifest_facts::observed_cargo_source_directories;
use super::work_item_evidence::work_item_evidence_paths;
use super::work_item_file_io::{
    FileIoAttempt, FileIoPlan, WorkItemFileProgress, file_io_pending_step, file_io_read_step,
    file_io_search_step,
};
use crate::protocol::ChatMessage;
use crate::source_fetch::sha256_hex;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
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
    value.as_str().filter(|source| {
        source.len() <= 512 && source_lexical_rule(source, "source-fence-forbidden") == Some(false)
    })
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

// Externally specified bootstrap input, not autonomous Formal AI synthesis.
struct Discovery {
    contract: Value,
    information: Value,
    step: Option<AgenticPlan>,
    gap: Option<String>,
}
impl Discovery {
    fn new(contract: &Value) -> Self {
        Self {
            contract: contract.clone(),
            step: None,
            gap: None,
            information: json!({"schema":"observed-consumer-discovery/v1","state":"explicit-only","queries":[],"candidates":[],
                "confirmedOwners":[],"gaps":[],"scope":"bounded-observed-cargo-source-scopes; syntax witnesses only; no whole-repository absence or runtime semantics"}),
        }
    }
    fn note(&mut self, reason: String) {
        let gaps = self.information["gaps"]
            .as_array_mut()
            .expect("DiscoveryGaps");
        if !gaps.contains(&json!(reason)) {
            gaps.push(json!(reason));
        }
    }
    fn fail(mut self, reason: &str) -> Self {
        self.gap = Some(reason.to_owned());
        self
    }
    fn planned(mut self, plan: FileIoPlan) -> Self {
        match plan {
            FileIoPlan::Pending(step) => self.step = Some(step),
            FileIoPlan::Gap(reason) => self.gap = Some(reason),
        }
        self
    }
}
fn discovery_reads<'a>(progress: &'a WorkItemFileProgress, owner: &str) -> Vec<&'a FileIoAttempt> {
    progress
        .research_attempts()
        .iter()
        .filter(|attempt| {
            attempt.capability == Capability::Read
                && discovery_owner(attempt).as_deref() == Some(owner)
        })
        .collect()
}
fn observation_position(attempt: &FileIoAttempt) -> Option<usize> {
    attempt.transport["completedAt"]
        .as_u64()
        .and_then(|value| usize::try_from(value).ok())
}
struct Consumer {
    path: String,
    sha256: String,
    after: usize,
    aliases: BTreeSet<String>,
}
fn repository_discovery_preflight(
    task: &str,
    messages: &[ChatMessage],
    tools: &[&str],
    contract: &Value,
    policy: &Value,
) -> Discovery {
    let mut discovery = Discovery::new(contract);
    let owned = |path: &str| {
        path.as_bytes()
            .first()
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
            && path
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"_./-".contains(&byte))
            && path.split('/').all(|part| !matches!(part, "" | "." | ".."))
    };
    let sources = contract["sources"].as_array();
    let target = contract["target"].as_str();
    let valid = sources.is_some_and(|rows| {
        !rows.is_empty()
            && rows.len() <= 16
            && rows.iter().all(|row| row.as_str().is_some_and(owned))
            && rows
                .iter()
                .filter_map(Value::as_str)
                .chain(target)
                .collect::<BTreeSet<_>>()
                .len()
                == rows.len() + 1
    }) && target.is_some_and(owned);
    if contract["authority"] != "owned" || !valid || shell_command_tool(tools).is_none() {
        return discovery;
    }
    let progress = WorkItemFileProgress::scan(messages);
    if let Some(reason) = progress.gap.as_ref() {
        return discovery.fail(reason);
    }
    let effects = progress
        .research_attempts()
        .iter()
        .filter(|attempt| matches!(attempt.capability, Capability::Edit | Capability::Write))
        .collect::<Vec<_>>();
    if effects.len() > 1
        || effects.iter().any(|attempt| {
            attempt.capability != Capability::Edit
                || discovery_owner(attempt).as_deref() != contract["target"].as_str()
        })
        || progress.research_attempts().iter().any(|attempt| {
            attempt.capability == Capability::Read && discovery_owner(attempt).is_none()
        })
    {
        return discovery;
    }
    if let Some(pending) = file_io_pending_step(&progress, tools) {
        return discovery.planned(pending);
    }
    let effect = effects.first().copied();
    let mut cutoff = messages.len();
    if let Some(effect) = effect {
        let Some(position) =
            observation_position(effect).filter(|position| *position < messages.len())
        else {
            return discovery.fail("ConsumerEffectProvenanceUnavailable");
        };
        let Some(call_identifier) = effect.transport["callId"].as_str() else {
            return discovery.fail("ConsumerEffectProvenanceUnavailable");
        };
        if !messages[position].role.eq_ignore_ascii_case("tool")
            || messages[position].tool_call_id.as_deref() != Some(call_identifier)
        {
            return discovery.fail("ConsumerEffectProvenanceUnavailable");
        }
        let declarations = messages[..position]
            .iter()
            .enumerate()
            .filter(|(_, message)| {
                message.role.eq_ignore_ascii_case("assistant")
                    && message.tool_calls.iter().any(|call| {
                        call.id == call_identifier
                            && Some(call.function.name.as_str())
                                == effect.transport["actualTool"].as_str()
                    })
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        if declarations.len() != 1 {
            return discovery.fail("ConsumerEffectProvenanceUnavailable");
        }
        cutoff = declarations[0];
    }
    let frozen = WorkItemFileProgress::scan(&messages[..cutoff]);
    let read = |discovery: Discovery, owner: &str| {
        if effect.is_some() {
            discovery.fail("ConsumerPreEffectObservationMissing")
        } else {
            discovery.planned(file_io_read_step(messages, tools, owner, task))
        }
    };
    let mut queries: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let explicit = contract["sources"].as_array().cloned().unwrap_or_default();
    for owner in explicit
        .iter()
        .filter_map(Value::as_str)
        .filter(|owner| owner.ends_with(".toml"))
    {
        let observations = discovery_reads(&frozen, owner);
        if observations.is_empty() {
            return read(discovery, owner);
        }
        let first = observations
            .iter()
            .find_map(|attempt| discovery_source(attempt));
        let latest = observations
            .last()
            .and_then(|attempt| discovery_source(attempt));
        let (Some(source), Some(latest)) = (first, latest) else {
            return discovery.fail("ConsumerManifestObservationIncomplete");
        };
        if source != latest {
            return discovery.fail("ConsumerManifestSnapshotChanged");
        }
        let layout = observed_cargo_source_directories(owner, source);
        let selected = derive_manifest_documentation(task, owner, source, "", Some(policy));
        let directories = layout["directories"].as_array();
        if layout["schema"] != "observed-cargo-source-directories/v1"
            || layout["manifest"] != owner
            || layout["sourceIdentity"] != sha256_hex(source.as_bytes())
            || layout["complete"] != true
            || directories.is_none_or(|rows| rows.len() > 4)
            || selected.get("gap").is_some()
        {
            discovery.note(format!("cargo-source-layout-unverified:{owner}"));
            continue;
        }
        let selected_facts = selected["facts"]
            .as_array()
            .or_else(|| selected["value"].as_array());
        let aliases = selected_facts
            .into_iter()
            .flatten()
            .filter(|fact| fact["kind"] == "manifest-declaration")
            .filter_map(|fact| fact["name"].as_str())
            .map(|name| name.replace('-', "_"))
            .collect::<BTreeSet<_>>();
        let identifier = |name: &str| {
            let mut characters = name.chars();
            characters
                .next()
                .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
                && name.len() <= 128
                && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
        };
        if aliases.is_empty() || aliases.iter().any(|alias| !identifier(alias)) {
            discovery.note(format!("cargo-dependency-alias-unavailable:{owner}"));
            continue;
        }
        for directory in directories.into_iter().flatten().filter_map(Value::as_str) {
            queries
                .entry(directory.to_owned())
                .or_default()
                .extend(aliases.iter().cloned());
        }
    }
    let mut planned_queries = Vec::new();
    for (directory, aliases) in &queries {
        let identifiers = aliases.iter().cloned().collect::<Vec<_>>();
        for chunk in identifiers.chunks(8) {
            planned_queries.push((directory.clone(), chunk.to_vec()));
        }
    }
    if queries.len() > 4 || planned_queries.len() > 4 {
        discovery.note("consumer-query-budget-exceeded".to_owned());
        discovery.information["state"] = json!("incomplete");
        return discovery;
    }
    if planned_queries.is_empty() {
        return discovery;
    }
    let mut candidates: BTreeMap<String, Consumer> = BTreeMap::new();
    let mut incomplete_search = false;
    for (directory, identifiers) in &planned_queries {
        let Some(observed) = frozen.search_for(directory, identifiers) else {
            if effect.is_some() {
                return discovery.fail("ConsumerPreEffectSearchMissing");
            }
            return discovery.planned(file_io_search_step(
                messages,
                tools,
                directory,
                identifiers,
                task,
            ));
        };
        discovery.information["queries"].as_array_mut().expect("DiscoveryQueries").push(json!({"directory":directory,"identifiers":identifiers,
            "requestIdentity":observed.request_identity,"status":observed.status,"complete":observed.complete,"root":observed.root,
            "searchGaps":observed.receipt.as_ref().map(|receipt|receipt["gaps"].clone()),"error":observed.error,
            "transport":{"actualTool":observed.transport["actualTool"],"callId":observed.transport["callId"],
                "actualExitCode":observed.transport["actualExitCode"],"completedAt":observed.transport["completedAt"]}}));
        let after = observed.transport["completedAt"]
            .as_u64()
            .and_then(|value| usize::try_from(value).ok());
        if observed.status != "reported-success"
            || !observed.complete
            || frozen.root.as_deref() != Some(observed.root.as_str())
            || after.is_none()
        {
            incomplete_search = true;
            discovery.note(format!("consumer-search-incomplete:{directory}"));
            continue;
        }
        for file in &observed.files {
            let owner = file["path"].as_str().unwrap_or_default();
            let hash = file["sha256"].as_str().unwrap_or_default();
            if let Some(prior) = candidates.get_mut(owner) {
                if prior.sha256 != hash {
                    return discovery.fail("ConsumerSearchSnapshotContradiction");
                }
                prior.after = prior.after.max(after.unwrap_or_default());
                prior.aliases.extend(identifiers.iter().cloned());
            } else {
                candidates.insert(
                    owner.to_owned(),
                    Consumer {
                        path: owner.to_owned(),
                        sha256: hash.to_owned(),
                        after: after.unwrap_or_default(),
                        aliases: identifiers.iter().cloned().collect(),
                    },
                );
            }
        }
    }
    if incomplete_search {
        discovery.information["state"] = json!("incomplete");
        return discovery;
    }
    if candidates.len() > 8 {
        discovery.note("consumer-source-budget-exceeded".to_owned());
        discovery.information["state"] = json!("incomplete");
        return discovery;
    }
    let mut confirmed = Vec::new();
    for candidate in candidates.values() {
        let reads = discovery_reads(&frozen, &candidate.path)
            .into_iter()
            .filter(|attempt| {
                observation_position(attempt).is_some_and(|position| position > candidate.after)
            })
            .collect::<Vec<_>>();
        let Some(observed) = reads.last() else {
            return read(discovery, &candidate.path);
        };
        let Some(source) = discovery_source(observed)
            .filter(|source| sha256_hex(source.as_bytes()) == candidate.sha256)
        else {
            return discovery.fail("ConsumerSearchReadCustodyMismatch");
        };
        let usage = match observed_source_usage_operation(
            &json!({"owner":candidate.path,"source":source,"language":"Rust"}),
        ) {
            Ok(usage) => usage,
            Err(_) => {
                discovery.note(format!("consumer-syntax-unavailable:{}", candidate.path));
                continue;
            }
        };
        if usage["schema"] != "observed-source-usage/v1"
            || usage["owner"] != candidate.path
            || usage["sourceIdentity"] != candidate.sha256
            || usage["language"] != "Rust"
        {
            discovery.note(format!("consumer-syntax-unbound:{}", candidate.path));
            continue;
        }
        let witnessed = ["imports", "calls"].iter().any(|key| {
            usage[*key].as_array().into_iter().flatten().any(|row| {
                row["module"].as_str().is_some_and(|name| {
                    candidate
                        .aliases
                        .contains(name.split("::").next().unwrap_or(name))
                })
            })
        }) || usage["references"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|row| {
                row["root"].as_str().is_some_and(|name| {
                    candidate
                        .aliases
                        .contains(name.split("::").next().unwrap_or(name))
                })
            });
        discovery.information["candidates"].as_array_mut().expect("DiscoveryCandidates").push(json!({"path":candidate.path,"sourceIdentity":candidate.sha256,"syntaxConfirmed":witnessed,"gaps":usage["gaps"]}));
        if witnessed {
            confirmed.push(candidate.path.clone());
        }
        if usage["gaps"]
            .as_array()
            .is_some_and(|rows| !rows.is_empty())
        {
            discovery.note(format!(
                "consumer-syntax-resolution-unverified:{}",
                candidate.path
            ));
        }
    }
    let mut sources = explicit
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    for owner in &confirmed {
        if !sources.contains(owner) {
            sources.push(owner.clone());
        }
    }
    if sources.len() > 16 {
        discovery.note("documentation-source-budget-exceeded".to_owned());
        discovery.information["state"] = json!("incomplete");
        return discovery;
    }
    discovery.information["confirmedOwners"] = json!(confirmed);
    discovery.information["state"] = json!(if discovery.information["gaps"]
        .as_array()
        .is_some_and(|rows| !rows.is_empty())
    {
        "partial"
    } else {
        "observed-scopes"
    });
    discovery.contract["sources"] = json!(sources);
    discovery.contract["coverage"] = discovery.information["scope"].clone();
    discovery
}

/// Run the shared custody state machine using pre-effect discovery observations.
pub(super) fn plan_repository_documentation(
    task: &str,
    messages: &[ChatMessage],
    tools: &[&str],
    work_item: Option<&str>,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let policy = source_documentation_policy()?;
    let contract = repository_documentation_contract(task, &policy)?;
    if contract["authority"] != "owned" || shell_command_tool(tools).is_none() {
        let mut contract = contract;
        contract["workItem"] = json!(work_item);
        return Some(plan_observed_documentation(
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
        ));
    }
    let mut preflight = repository_discovery_preflight(task, messages, tools, &contract, &policy);
    if let Some(step) = preflight.step.take() {
        return Some(step);
    }
    let mut contract = preflight.contract;
    contract["workItem"] = json!(work_item);
    let information = preflight.information;
    let gap = preflight.gap;
    Some(plan_observed_documentation(
        task,
        messages,
        tools,
        &contract,
        |sources, guide| {
            if let Some(reason) = &gap {
                return json!({"gap":reason});
            }
            let mut derived = derive_repository_documentation(
                task,
                sources,
                guide,
                &policy,
                observed_source_usage_operation,
            );
            if derived.get("gap").is_some() {
                return derived;
            }
            let fact = json!({"kind":"observed-consumer-discovery","schema":information["schema"],"state":information["state"],
            "queries":information["queries"],"candidates":information["candidates"],"confirmedOwners":information["confirmedOwners"],"gaps":information["gaps"],"scope":information["scope"]});
            derived["facts"]
                .as_array_mut()
                .expect("RepositoryFacts")
                .push(fact);
            derived["binding"]["discovery"] = information.clone();
            let remaining = derived["remainingWork"]
                .as_array_mut()
                .expect("RepositoryRemainingWork");
            for reason in information["gaps"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                let reason = json!(format!("consumer-discovery-unverified:{reason}"));
                if !remaining.contains(&reason) {
                    remaining.push(reason);
                }
            }
            derived
        },
        result,
    ))
}

#[cfg(test)]
#[path = "../../tests/unit/agentic-coding/repository_documentation.rs"]
mod tests;
