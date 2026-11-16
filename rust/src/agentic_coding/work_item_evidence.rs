//! Bounded source observations before an unsupported repository task stops.
//! Native twin of `js/agentic/work_item_evidence.mjs`.
//!
//! Each file name the work item mentions is resolved through
//! `work_item_references` (issue #1558): the exact path first, then a search
//! by file name, then ranked candidates. A failed read is recorded and the
//! next action differs from it; a reference nothing resolves becomes an
//! explicit unresolved item while the remaining evidence is still read.
//!
//! A work item that names no file is still read for evidence (issue #1483):
//! attachments stay unresolved, linked documents are fetched as evidence and
//! code identifiers in backticks are searched and their best file read.
use super::final_result::{FinalDisposition, FinalResult};
use super::planner::{AgenticPlan, Capability, plan_one, tool_for};
use super::progress::{Progress, ToolAttempt};
use super::source_manifest_facts::observed_manifest_facts;
use super::work_item_leads::{
    MAXIMUM_DOCUMENTS, WorkItemLeads, document_action, document_attempt_for, symbol_actions,
    symbol_attempt_for, symbol_matches, work_item_leads, work_item_symbols,
};
use super::work_item_references::{
    MAXIMUM_CANDIDATE_READS, ReadOutcome, WorkItemReference, attempt_for, product_name,
    ranked_candidates, read_attempt_for, read_outcome, same_path, search_actions, search_matches,
    work_item_references, workspace_roots,
};
use super::work_item_report::{refusal, work_item_record_final};
use crate::protocol::{ChatMessage, client_working_directory};
use regex::Regex;
use serde_json::json;
use std::sync::OnceLock;

/// Parse bounded explicit source references without granting effect authority.
pub(super) fn work_item_evidence_paths(task: &str) -> Vec<&str> {
    super::work_item_references::work_item_evidence_paths(task, false)
}

/// The host root a relative read falls back to when the client names no workspace.
fn host_root() -> Option<String> {
    std::env::current_dir()
        .ok()
        .and_then(|root| root.into_os_string().into_string().ok())
}

/// What the collector does next: plan one action, finish with a reason, or
/// refuse the read of a path the user's turn forbids.
enum Step {
    Plan(AgenticPlan),
    Finish(&'static str),
    Refuse(String),
}

/// The evidence gathered so far, reported in the final payload.
#[derive(Default)]
struct Evidence {
    observations: Vec<serde_json::Value>,
    unresolved: Vec<serde_json::Value>,
    terms: Vec<String>,
    /// Complete sources read, kept to name an absent requested interface.
    sources: Vec<String>,
}

impl Evidence {
    fn leave_unresolved(
        &mut self,
        reference: &WorkItemReference,
        reason: &str,
        extra: Option<(&str, serde_json::Value)>,
    ) {
        let mut row = json!({"reference":reference.name,"target":reference.target,"reason":reason});
        if let Some((key, value)) = extra {
            row[key] = value;
        }
        self.unresolved.push(row);
    }

    /// Record the read of `path` for the reference called `name` and return its outcome.
    fn observe(
        &mut self,
        name: &str,
        path: &str,
        attempt: &ToolAttempt,
        resolved_by: &str,
    ) -> ReadOutcome {
        let outcome = read_outcome(attempt);
        let failed = outcome == ReadOutcome::Failed;
        let source = attempt
            .source_observation()
            .filter(|observed| !failed && observed.complete)
            .and_then(|observed| observed.source.as_deref())
            .filter(|source| source.len() <= 262_144);
        let complete = source.is_some();
        let mut observation = json!({"path":path,"complete":complete,
            "sourceIdentity":source.map(|source|crate::source_fetch::sha256_hex(source.as_bytes())),
            "excerpt":source.map(|source|source.chars().take(1024).collect::<String>())});
        if name != path {
            observation["reference"] = json!(name);
        }
        if resolved_by != "exact" {
            observation["resolvedBy"] = json!(resolved_by);
        }
        if failed {
            observation["readFailed"] = json!(true);
        }
        if let Some(source) = source
            && (path.ends_with(".toml") || path.ends_with(".json"))
        {
            match observed_manifest_facts(path, source) {
                Ok(facts) => {
                    observation["manifestFacts"] = json!(facts);
                    observation["factScope"] =
                        json!("literal-declarations-only; executable use remains unverified");
                }
                Err(_) => {
                    observation["manifestFacts"] = json!([]);
                    observation["factGap"] = json!("UnsupportedOrInvalidManifest");
                }
            }
        }
        self.observations.push(observation);
        if let Some(source) = source {
            self.sources.push(source.to_owned());
        }
        if failed {
            ReadOutcome::Failed
        } else if complete {
            ReadOutcome::Complete
        } else {
            ReadOutcome::Incomplete
        }
    }
}

fn read_step(read: &str, path: &str) -> AgenticPlan {
    plan_one(
        read,
        json!({"path":path,"filePath":path,"file_path":path}).to_string(),
    )
}

fn plan_read(messages: &[ChatMessage], task: &str, read: &str, path: &str) -> Step {
    let step = read_step(read, path);
    if super::file_read::user_read_policy_blocks_plan(messages, task, &step) {
        Step::Refuse(path.to_owned())
    } else {
        Step::Plan(step)
    }
}

/// What the work item points at: named files, or else its leads and identifiers.
struct Subject<'a> {
    references: &'a [WorkItemReference],
    leads: &'a WorkItemLeads,
    symbols: &'a [String],
}

/// Walk every reference in order and return the first action still owed, or the terminal reason.
fn next_step(
    task: &str,
    messages: &[ChatMessage],
    subject: &Subject<'_>,
    progress: &Progress,
    roots: &[String],
    tools: &[&str],
    evidence: &mut Evidence,
) -> Step {
    let Subject {
        references,
        leads,
        symbols,
    } = *subject;
    let read = tool_for(tools, Capability::Read);
    let mut incomplete = false;
    // An attachment is never fetched or described: its content stays unresolved evidence.
    for url in &leads.attachments {
        evidence.unresolved.push(json!({"reference":url,"target":url,"kind":"attachment","reason":"AttachmentContentUnavailable"}));
    }
    for reference in references {
        let Some(target) = reference.path.as_deref().filter(|_| !reference.external) else {
            evidence.leave_unresolved(
                reference,
                "OutsideRepository",
                Some(("repository", json!(reference.repository))),
            );
            continue;
        };
        let Some(read) = read else {
            return Step::Finish("MissingReadTool");
        };
        let searches = search_actions(reference, tools);
        let product = product_name(target);
        // The exact path is read first unless the name is bare and a name search can find it without a failed read.
        let exact_first = !product
            && (target.contains('/')
                || !searches
                    .iter()
                    .any(|action| action.capability == Capability::Glob));
        let mut failed: Vec<&str> = Vec::new();
        if exact_first {
            let Some(attempt) = read_attempt_for(progress, target, roots) else {
                return plan_read(messages, task, read, target);
            };
            match evidence.observe(&reference.name, target, attempt, "exact") {
                ReadOutcome::Failed => failed.push(target),
                outcome => {
                    incomplete |= outcome == ReadOutcome::Incomplete;
                    continue;
                }
            }
        } else if super::file_read::user_read_policy_blocks_plan(
            messages,
            task,
            &read_step(read, target),
        ) {
            return Step::Refuse(target.to_owned());
        }
        let mut matches = None;
        for action in &searches {
            let Some(attempt) = attempt_for(progress, action) else {
                return Step::Plan(plan_one(action.tool, action.arguments.clone()));
            };
            let found = search_matches(&attempt.detail, reference, roots);
            if !found.is_empty() || attempt.succeeded {
                matches = Some(found);
                break;
            }
        }
        let Some(matches) = matches else {
            if product {
                evidence.terms.push(reference.name.clone());
            } else {
                let reason = if searches.is_empty() {
                    "SearchUnavailable"
                } else {
                    "SearchFailed"
                };
                evidence.leave_unresolved(reference, reason, None);
            }
            continue;
        };
        let readable = matches
            .iter()
            .filter(|path| {
                !failed
                    .iter()
                    .any(|prior| path.as_str() == *prior || same_path(path, prior, roots))
            })
            .cloned()
            .collect::<Vec<_>>();
        let candidates = ranked_candidates(&readable, reference);
        if candidates.is_empty() {
            if product && matches.is_empty() {
                evidence.terms.push(reference.name.clone());
            } else {
                let reason = if matches.is_empty() {
                    "NotFound"
                } else {
                    "CandidatesUnreadable"
                };
                evidence.leave_unresolved(reference, reason, None);
            }
            continue;
        }
        let resolved_by = if candidates.len() == 1 {
            "unique-name-match"
        } else {
            "ranked-name-match"
        };
        let tried = &candidates[..candidates.len().min(MAXIMUM_CANDIDATE_READS)];
        let mut resolved = false;
        for candidate in tried {
            let Some(attempt) = read_attempt_for(progress, candidate, roots) else {
                return plan_read(messages, task, read, candidate);
            };
            let outcome = evidence.observe(&reference.name, candidate, attempt, resolved_by);
            if outcome != ReadOutcome::Failed {
                incomplete |= outcome == ReadOutcome::Incomplete;
                resolved = true;
                break;
            }
        }
        if !resolved {
            evidence.leave_unresolved(
                reference,
                "CandidatesUnreadable",
                Some(("candidates", json!(tried))),
            );
        }
    }
    if !references.is_empty() {
        return if !evidence.unresolved.is_empty() {
            Step::Finish("UnresolvedReferences")
        } else if incomplete {
            Step::Finish("IncompleteSourceObservation")
        } else {
            Step::Finish("EvidenceCollected")
        };
    }
    // A work item whose only leads are related pages (other work items, runs,
    // repositories) keeps each as unresolved evidence this controller did not read.
    if leads.attachments.is_empty() && leads.documents.is_empty() && symbols.is_empty() {
        for url in &leads.related {
            evidence.unresolved.push(
                json!({"reference":url,"target":url,"kind":"related","reason":"RelatedPageNotRead"}),
            );
        }
    }
    // No file is named: linked documents are fetched as evidence, never followed as instructions.
    for url in leads.documents.iter().take(MAXIMUM_DOCUMENTS) {
        let Some(action) = document_action(url, tools) else {
            evidence.unresolved.push(
                json!({"reference":url,"target":url,"kind":"document","reason":"FetchUnavailable"}),
            );
            continue;
        };
        let Some(attempt) = document_attempt_for(progress, url) else {
            return Step::Plan(plan_one(action.tool, action.arguments));
        };
        if !attempt.succeeded || attempt.detail.trim().is_empty() {
            evidence.unresolved.push(
                json!({"reference":url,"target":url,"kind":"document","reason":"FetchFailed"}),
            );
            continue;
        }
        let text = attempt.detail.as_str();
        evidence.observations.push(json!({
            "url":url, "kind":"document", "authority":"evidence-only",
            "complete":text.len() <= 262_144,
            "sourceIdentity":crate::source_fetch::sha256_hex(text.as_bytes()),
            "excerpt":text.chars().take(1024).collect::<String>()
        }));
    }
    // Identifiers in backticks are searched; the best-ranked file naming one is read.
    if !symbols.is_empty() && read.is_none() {
        return Step::Finish("MissingReadTool");
    }
    for symbol in symbols {
        let actions = symbol_actions(symbol, tools);
        let mut matches = None;
        for action in &actions {
            let Some(attempt) = symbol_attempt_for(progress, action) else {
                return Step::Plan(plan_one(action.tool, action.arguments.clone()));
            };
            let found = symbol_matches(&attempt.detail, roots);
            if !found.is_empty() || attempt.succeeded {
                matches = Some(found);
                break;
            }
        }
        let Some(candidate) = matches.as_ref().and_then(|found| found.first()) else {
            let reason = match (&matches, actions.is_empty()) {
                (Some(_), _) => "SymbolNotFound",
                (None, true) => "SearchUnavailable",
                (None, false) => "SearchFailed",
            };
            evidence
                .unresolved
                .push(json!({"reference":symbol,"target":symbol,"kind":"symbol","reason":reason}));
            continue;
        };
        let Some(read) = read else {
            return Step::Finish("MissingReadTool");
        };
        let Some(attempt) = read_attempt_for(progress, candidate, roots) else {
            return plan_read(messages, task, read, candidate);
        };
        let outcome = evidence.observe(symbol, candidate, attempt, "symbol-search");
        if let Some(observation) = evidence.observations.last_mut() {
            observation["symbol"] = json!(symbol);
        }
        if outcome == ReadOutcome::Failed {
            evidence.unresolved.push(json!({"reference":symbol,"target":candidate,"kind":"symbol","reason":"CandidatesUnreadable"}));
        } else {
            incomplete |= outcome == ReadOutcome::Incomplete;
        }
    }
    Step::Finish(lead_reason(
        leads,
        symbols,
        &evidence.unresolved,
        incomplete,
    ))
}

/// The terminal reason of a work item that names no repository file, from what its leads left unresolved.
fn lead_reason(
    leads: &WorkItemLeads,
    symbols: &[String],
    unresolved: &[serde_json::Value],
    incomplete: bool,
) -> &'static str {
    let kind = |row: &serde_json::Value, kind: &str| row["kind"].as_str() == Some(kind);
    if leads.attachments.is_empty() && leads.documents.is_empty() && symbols.is_empty() {
        if leads.related.is_empty() {
            "NoRepositoryReference"
        } else {
            "RelatedEvidenceUnread"
        }
    } else if !unresolved.is_empty() && unresolved.iter().all(|row| kind(row, "attachment")) {
        "AttachmentEvidenceUnavailable"
    } else if unresolved.iter().any(|row| kind(row, "document")) {
        "ExternalEvidenceUnavailable"
    } else if !unresolved.is_empty() {
        "UnresolvedReferences"
    } else if incomplete {
        "IncompleteSourceObservation"
    } else {
        "EvidenceCollected"
    }
}

/// A call-shaped compound identifier: `sameMeaning(a, b)`, `same_meaning(`.
fn interface_call() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(
            r"(?m)(?:^|[^A-Za-z0-9_.:])([a-z][a-z0-9]*(?:[A-Z][A-Za-z0-9]*|_[a-z0-9]+)+)\s*\(",
        )
        .expect("interface call pattern")
    })
}
const MAXIMUM_INTERFACES: usize = 8;

/// Call-shaped identifiers the requirements name, each once.
pub(super) fn requested_interfaces(requirements: &[serde_json::Value]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for requirement in requirements {
        let text = requirement["text"].as_str().unwrap_or_default();
        for captures in interface_call().captures_iter(text) {
            let name = &captures[1];
            if names.len() < MAXIMUM_INTERFACES && !names.iter().any(|known| known == name) {
                names.push(name.to_owned());
            }
        }
    }
    names
}

/// The narrowest gap a completed evidence collection leaves, and its work.
/// A requested interface that no read source defines is named; otherwise the
/// read files the requirements name (or every read file) are the targets no
/// change recipe of the supported subset covers.
pub(super) fn evidence_gap(
    requirements: &[serde_json::Value],
    observations: &[serde_json::Value],
    sources: &[String],
) -> Option<(&'static str, Vec<String>)> {
    let read = observations
        .iter()
        .filter(|row| row["complete"] == json!(true))
        .filter_map(|row| row["path"].as_str())
        .collect::<Vec<_>>();
    if read.is_empty() {
        return None;
    }
    let absent = requested_interfaces(requirements)
        .into_iter()
        .filter(|name| !sources.iter().any(|source| source.contains(name.as_str())))
        .collect::<Vec<_>>();
    if !absent.is_empty() {
        let work = absent
            .iter()
            .map(|name| format!("create-interface:{name}"))
            .collect();
        return Some(("RequestedInterfaceAbsent", work));
    }
    let named = read
        .iter()
        .copied()
        .filter(|path| {
            let base = path.rsplit('/').next().unwrap_or(path);
            requirements.iter().any(|requirement| {
                requirement["text"]
                    .as_str()
                    .unwrap_or_default()
                    .contains(base)
            })
        })
        .collect::<Vec<_>>();
    let mut targets: Vec<&str> = Vec::new();
    for path in if named.is_empty() { read } else { named } {
        if !targets.contains(&path) {
            targets.push(path);
        }
    }
    let work = targets
        .iter()
        .map(|path| format!("change-recipe-unavailable:{path}"))
        .collect();
    Some(("NoChangeRecipe", work))
}

/// Work still owed after evidence collection: the change itself (or, when a
/// gap is named, its narrower parts), and every lead left unresolved.
fn evidence_remaining_work(
    unresolved: &[serde_json::Value],
    gap: Option<&(&'static str, Vec<String>)>,
    unbound: Option<&str>,
) -> Vec<String> {
    let mut work = vec!["bind-requirements-to-source".to_owned()];
    // A file the solver selected from a citation alone stays unwritten until
    // an obligation names it (issue #1562).
    if let Some(path) = unbound {
        work.push(format!("unbound-write:{path}"));
    }
    match gap {
        Some((_, parts)) => work.extend(parts.iter().cloned()),
        None => work.push("implement-requested-change".to_owned()),
    }
    work.push("verify-artifact-and-requirements".to_owned());
    for row in unresolved {
        work.push(format!(
            "unresolved-evidence:{}:{}",
            row["reason"].as_str().unwrap_or_default(),
            row["reference"].as_str().unwrap_or_default()
        ));
    }
    work
}

/// Native twin of `js/agentic/work_item_evidence.mjs`. References grant read evidence only.
/// `unbound` is the file a recipe would have written although no obligation of
/// the work item names it (issue #1562); the record and its report name it.
pub(super) fn plan_work_item_evidence(
    task: &str,
    messages: &[ChatMessage],
    tools: &[&str],
    work_item: &str,
    unbound: Option<&str>,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let named = !work_item_evidence_paths(task).is_empty();
    let references = if named {
        work_item_references(task, work_item)
    } else {
        Vec::new()
    };
    let leads = work_item_leads(task, work_item);
    // Identifiers are searched only when the work item names no file: a named file is the narrower evidence.
    let symbols = if references.is_empty() {
        work_item_symbols(task)
    } else {
        Vec::new()
    };
    // A work item with neither a file nor a lead is left to the general planner.
    if !named
        && leads.attachments.is_empty()
        && leads.documents.is_empty()
        && leads.related.is_empty()
        && symbols.is_empty()
    {
        return None;
    }
    let progress = Progress::scan(messages);
    let roots = workspace_roots(client_working_directory(messages), host_root());
    let mut evidence = Evidence::default();
    // The work item is split into requirements before any evidence is read, so
    // a refusal names the requirement it concerns (issue #1560).
    // A clause that lies wholly inside an address, a code span, a table of
    // evidence or a quotation is a citation, not an obligation (issue #1562).
    let obligations = super::work_item_citations::obligation_text(task);
    let requirements = super::work_item_requirements::work_item_requirements(task)
        .into_iter()
        .map(|(text, (start, end))| {
            let status = if super::work_item_citations::is_citation(&obligations, (start, end)) {
                "citation"
            } else {
                "unbound"
            };
            json!({"text":text,"span":[start,end],"status":status})
        })
        .collect::<Vec<_>>();
    let (reason, refused) = match next_step(
        task,
        messages,
        &Subject {
            references: &references,
            leads: &leads,
            symbols: &symbols,
        },
        &progress,
        &roots,
        tools,
        &mut evidence,
    ) {
        Step::Plan(plan) => return Some(plan),
        Step::Finish(reason) => (reason, None),
        Step::Refuse(path) => ("ReadPolicyRefused", Some(refusal(&requirements, &path))),
    };
    let gap = (reason == "EvidenceCollected")
        .then(|| evidence_gap(&requirements, &evidence.observations, &evidence.sources))
        .flatten();
    let mut payload = json!({
        "schema":"repository-work-item-evidence/v1", "reason":reason,"workItem":work_item,
        "taskIdentity":crate::source_fetch::sha256_hex(task.as_bytes()),
        "requirements":requirements,
        "decomposition":[
            {"step":"collect_repository_evidence", "status":if reason == "EvidenceCollected" {"observed"} else {"incomplete"}},
            {"step":"bind_requirements_to_source", "status":"unverified"},
            {"step":"implement_requested_change", "status":"not_executed"},
            {"step":"verify_artifact_and_requirements", "status":"not_executed"}
        ],
        "remainingWork":evidence_remaining_work(&evidence.unresolved, gap.as_ref(), unbound),
        "observations":evidence.observations, "unresolved":evidence.unresolved, "terms":evidence.terms,
        "coverage":"explicit-references; exact path, then file-name search; at most eight files; attachments unresolved; at most two linked documents and three identifiers when no file is named",
        "candidateEdits":[], "checks":[], "effects":[],
        "authored":false, "verified":false
    });
    if let Some((kind, _)) = &gap {
        payload["gap"] = json!(kind);
    }
    if !leads.related.is_empty() {
        payload["related"] = json!(leads.related);
    }
    if let Some(refused) = refused {
        payload["refused"] = refused;
    }
    if let Some(path) = unbound {
        payload["unboundWrite"] = json!({"path":path,"reason":"NamedByNoObligation"});
    }
    Some(work_item_record_final(
        payload,
        task,
        FinalDisposition::Gap,
        "repository_work_item_evidence_gap",
        result,
    ))
}

#[cfg(test)]
#[path = "../../tests/unit/agentic-coding/work_item_evidence_gap.rs"]
mod tests;
