//! Thin receipt syntax adapter for canonical generated research decisions.
use super::final_result::{
    ResearchDiscoveryContext, ResearchDiscoveryEvidence, ResearchFinalEvidence,
    ResearchObservations, ResearchProvenance, ResearchReadObservation, ResearchReadStatus,
    ResearchVerificationObservation,
};
use super::wording::agentic_message as message;
#[path = "../coding_research_kernel.rs"]
pub(super) mod kernel;

use super::tool_result::{StepOutcome, incomplete_receipt, normalized_payload, step_outcome};

/// Terminal TAP and Node spec evidence share strict generated counter decisions.
#[must_use]
pub fn verification_receipt_gap(raw: &str) -> Option<String> {
    let complete = !incomplete_receipt(raw) && step_outcome(raw) == StepOutcome::Succeeded;
    let payload: Vec<String> = normalized_payload(raw)
        .unwrap_or_default()
        .split('\n')
        .map(|line| {
            line.strip_prefix("ℹ ")
                .map_or_else(|| line.to_owned(), |tail| format!("# {tail}"))
        })
        .collect();
    let mut counts = [-1.0, -1.0, -1.0, 0.0, 0.0];
    let mut unique = true;
    for (index, field) in ["tests", "pass", "fail", "skipped", "cancelled"]
        .iter()
        .enumerate()
    {
        let prefix = format!("# {field} ");
        let values: Vec<&str> = payload
            .iter()
            .filter_map(|line| {
                line.strip_suffix('\r')
                    .unwrap_or(line)
                    .strip_prefix(&prefix)
            })
            .filter(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
            .collect();
        let label = format!("# {field}");
        let recognized = payload
            .iter()
            .filter(|line| {
                line.strip_prefix(label.as_str()).is_some_and(|suffix| {
                    suffix.is_empty()
                        || suffix.starts_with(' ')
                        || suffix.starts_with('\t')
                        || suffix.starts_with('\r')
                })
            })
            .count();
        if !kernel::tap_counter_syntax_valid(
            research_length_number(recognized),
            research_length_number(values.len()),
        ) {
            unique = false;
        }
        if let Some(value) = values.first() {
            counts[index] = value.parse::<f64>().unwrap_or(f64::NAN);
        }
    }
    let reason = kernel::tap_verification_decision(
        complete && unique,
        counts[0],
        counts[1],
        counts[2],
        counts[3],
        counts[4],
    );
    if reason.is_empty() {
        None
    } else {
        Some(reason)
    }
}

use super::coding_research_observation::{
    observe_coding_research_acceptance_tokens, observe_coding_research_rebinding_candidate,
};
use super::final_result::{FinalDisposition, FinalResult, record};
use super::module_function::ModuleFunctionRequest;
use super::planner::{AgenticPlan, Capability, fetch_arguments, plan_one, tool_for};
use super::progress::{Progress, ToolAttempt};
use crate::coding_research_generated as generated;
use crate::protocol::ChatMessage;
use serde_json::{Value, json};

/// Decimal observations use Rust's correctly rounded parser, matching JavaScript Number.
fn research_length_number(value: usize) -> f64 {
    value
        .to_string()
        .parse()
        .expect("ResearchLengthNumberInvariant")
}
/// Generated indices must remain finite integral platform indices; invalid values refuse.
fn research_number_index(value: f64) -> Option<usize> {
    if !value.is_finite() {
        return None;
    }
    let text = value.to_string();
    let unsigned_zero = if text == "-0" { "0" } else { &text };
    unsigned_zero.parse().ok()
}

fn text(value: &Value) -> String {
    value.as_str().unwrap_or_default().to_owned()
}
fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map_or_else(Vec::new, |items| items.iter().map(text).collect())
}
fn number(value: &Value) -> f64 {
    value.as_f64().unwrap_or_default()
}
fn flag(value: &Value) -> bool {
    value.as_bool().unwrap_or_default()
}
fn candidate(value: &Value) -> generated::ResearchRebindingCandidate {
    generated::ResearchRebindingCandidate::Candidate(
        text(&value["language"]),
        flag(&value["syntaxValid"]),
        flag(&value["moduleEffectsNone"]),
        flag(&value["moduleSyntaxSupported"]),
        number(&value["importsCount"]),
        number(&value["declarationsCount"]),
        flag(&value["contractSupported"]),
        flag(&value["callEffectsNone"]),
        text(&value["name"]),
        strings(&value["parameters"]),
        flag(&value["exported"]),
        strings(&value["pieces"]),
        strings(&value["pieceBindings"]),
    )
}
/// The same typed declaration/clause query kernel as JavaScript; no answer cache.
#[must_use]
pub fn coding_research_query(request: &ModuleFunctionRequest) -> String {
    let clause = super::module_function::outside_quotes(&request.clause);
    generated::coding_research_query_kernel(
        request.language.clone(),
        clause.chars().map(|value| value.to_string()).collect(),
        request.name.clone(),
        request.parameters.clone(),
    )
}
/// The same independent literal assertion grammar over existing tokenizer observations.
#[must_use]
pub fn independent_acceptance(request: &ModuleFunctionRequest, source: &str) -> bool {
    let observed = observe_coding_research_acceptance_tokens(source);
    let directory = |path: &str| {
        path.rsplit_once('/')
            .map_or("", |(parent, _)| parent)
            .to_owned()
    };
    let test = request.test.clone().unwrap_or_default();
    let module_url = format!(
        "./{}",
        request.module.rsplit('/').next().unwrap_or_default()
    );
    generated::independent_acceptance_kernel(
        strings(&observed["texts"]),
        strings(&observed["kinds"]),
        request.name.clone(),
        module_url,
        directory(&request.module),
        directory(&test),
        test,
        request.command.clone().unwrap_or_default(),
        research_length_number(request.parameters.len()),
        flag(&observed["complete"])
            && crate::coding::cst::validated_program_cst("javascript", source).is_some(),
    )
}
/// Source is proposed only by the generated catalog/binding/ambiguity decisions.
#[must_use]
pub fn derive_coding_procedure(request: &ModuleFunctionRequest, page: &Value) -> Option<Value> {
    let mut proposals = Vec::new();
    for block in page["codeBlocks"].as_array()? {
        let language = text(&block["language"]);
        if !generated::coding_research_language_eligible(request.language.clone(), language.clone())
        {
            continue;
        }
        let source = text(&block["text"]);
        let observed =
            observe_coding_research_rebinding_candidate(&source, &language, &request.module);
        let result = generated::coding_research_rebinding_kernel(
            candidate(&observed),
            request.language.clone(),
            request.name.clone(),
            request.parameters.clone(),
        );
        let generated::ResearchRebindingResult::Rendered(source, from, to) = result else {
            continue;
        };
        if crate::coding::cst::validated_program_cst(&request.language, &source).is_none() {
            continue;
        }
        let catalog = super::module_function::callable_catalog::observe_source_callables(
            &text(&block["text"]),
            &request.module,
        );
        let declaration = &catalog["declarations"][0];
        let bindings: Vec<Value> = from
            .iter()
            .zip(&to)
            .map(|(from, to)| json!({"from":from,"to":to}))
            .collect();
        proposals.push(json!({"source":source,"provenance":{"url":page["url"],"sha256":page["sha256"],
            "query":page["query"],"declaration":declaration["identity"],"contract":declaration["contract"],
            "bindings":bindings,"status":"proposal","verified":false}}));
        if !page["sourceObservation"].is_null() {
            proposals.last_mut()?["provenance"]["sourceObservation"] =
                page["sourceObservation"].clone();
        }
    }
    let selected = generated::canonical_unique_source_index(vec![true; proposals.len()]);
    if selected >= 0.0 {
        research_number_index(selected)
            .and_then(|index| proposals.get(index))
            .cloned()
    } else {
        None
    }
}
fn pure_research_program(request: &ModuleFunctionRequest, source: &str) -> bool {
    let observed =
        observe_coding_research_rebinding_candidate(source, &request.language, &request.module);
    let catalog =
        super::module_function::callable_catalog::observe_source_callables(source, &request.module);
    let exports = catalog["exports"].as_array().cloned().unwrap_or_default();
    generated::coding_research_pure_repair_kernel(
        flag(&observed["syntaxValid"]),
        flag(&observed["moduleEffectsNone"]),
        flag(&observed["moduleSyntaxSupported"]),
        flag(&observed["contractSupported"]),
        flag(&observed["callEffectsNone"]),
        number(&observed["declarationsCount"]),
        number(&observed["importsCount"]),
        text(&observed["name"]),
        strings(&observed["parameters"]),
        exports.iter().map(|item| text(&item["local"])).collect(),
        exports.iter().map(|item| text(&item["exposed"])).collect(),
        request.name.clone(),
        request.parameters.clone(),
    )
}
fn gap(
    decision: &str,
    request: &ModuleFunctionRequest,
    observations: ResearchObservations,
    context: ResearchDiscoveryContext,
    result: &mut Option<FinalResult>,
) -> AgenticPlan {
    let mut parts = decision.strip_prefix("gap:").unwrap_or(decision).split(':');
    let reason = parts.next().unwrap_or_default();
    let path = parts.next().and_then(|target| match target {
        "module" => Some(request.module.clone()),
        "test" => request.test.clone(),
        _ => None,
    });
    let plan = record(
        AgenticPlan::Final(message("coding-research-gap", &[("reason", reason)])),
        FinalDisposition::Gap,
        "coding-research-gap",
        result,
    );
    if let Some(result) = result {
        result.discovery = Some(ResearchDiscoveryEvidence {
            reason: reason.to_owned(),
            authored: false,
            verified: false,
            path,
            context,
            observations,
        });
    }
    plan
}
fn read_evidence(
    path: Option<&str>,
    observed: Option<&super::tool_result::SourceReadObservation>,
) -> ResearchReadObservation {
    ResearchReadObservation {
        path: path.map(str::to_owned),
        observed: observed.is_some(),
        status: observed.map(|value| match value.status {
            super::tool_result::SourceReadStatus::ReportedSuccess => {
                ResearchReadStatus::ReportedSuccess
            }
            super::tool_result::SourceReadStatus::ReportedFailure => {
                ResearchReadStatus::ReportedFailure
            }
            super::tool_result::SourceReadStatus::Unknown => ResearchReadStatus::Unknown,
        }),
        complete: observed.map(|value| value.complete),
        absent: observed.map(|value| value.absent),
        source_sha256: observed
            .and_then(|value| value.source.as_deref())
            .map(|source| crate::source_fetch::sha256_hex(source.as_bytes())),
        error: observed.and_then(|value| value.error.clone()),
    }
}

fn research_observations(
    request: &ModuleFunctionRequest,
    progress: &Progress,
    verification: Option<ResearchVerificationObservation>,
) -> ResearchObservations {
    ResearchObservations {
        module_read: read_evidence(
            Some(&request.module),
            progress.source_read_for(&request.module),
        ),
        test_read: read_evidence(
            request.test.as_deref(),
            request
                .test
                .as_deref()
                .and_then(|path| progress.source_read_for(path)),
        ),
        successful_write_sha256: progress
            .successful_write_content_for(&request.module)
            .map(|source| crate::source_fetch::sha256_hex(source.as_bytes())),
        verification,
    }
}

fn capability(attempt: &ToolAttempt) -> String {
    attempt
        .capability
        .permission_key()
        .strip_prefix("tool:capability:")
        .unwrap_or_default()
        .to_owned()
}
fn outcome(attempt: Option<&ToolAttempt>) -> String {
    let Some(attempt) = attempt else {
        return "missing".to_owned();
    };
    if !attempt.succeeded {
        return "failed".to_owned();
    }
    match step_outcome(&attempt.detail) {
        StepOutcome::Succeeded => "succeeded",
        StepOutcome::Failed => "failed",
        StepOutcome::Unreported => "unreported",
    }
    .to_owned()
}
/// Thin effect/observation adapter. Every research decision comes from generated canonical source.
pub(super) fn plan_coding_research_step(
    task: &str,
    messages: &[ChatMessage],
    tools: &[&str],
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let request = super::module_function::module_function_research_request(task)?;
    let progress = Progress::scan(messages);
    let read_tool = tool_for(tools, Capability::Read);
    let search_tool = tool_for(tools, Capability::Search);
    let fetch_tool = tool_for(tools, Capability::Fetch);
    let run_tool = tool_for(tools, Capability::Run);
    let module_read = progress.source_read_for(&request.module);
    let test_read = request
        .test
        .as_deref()
        .and_then(|path| progress.source_read_for(path));
    let names: Vec<String> = std::iter::once(&request.name)
        .chain(&request.parameters)
        .cloned()
        .collect();
    let characters = names
        .iter()
        .map(|name| name.chars().map(|value| value.to_string()).collect())
        .collect();
    let keywords = names
        .iter()
        .map(|name| crate::es_tokenizer::is_keyword(name))
        .collect();
    let preflight = generated::coding_research_preflight_kernel(
        request.language.clone(),
        crate::coding::cst::parse_program_cst(&request.language, "").is_some(),
        generated::coding_research_bindings_kernel(names, characters, keywords),
        request.name.clone(),
        request.parameters.clone(),
        request
            .command
            .as_ref()
            .is_some_and(|value| !value.is_empty()),
        request.test.as_ref().is_some_and(|value| !value.is_empty()),
        module_read.is_some(),
        module_read.is_some_and(|value| value.complete),
        module_read.is_some_and(|value| value.absent),
        test_read.is_some(),
        test_read.is_some_and(|value| value.complete),
        test_read.is_some_and(|value| value.error.is_some()),
        test_read
            .and_then(|value| value.source.as_deref())
            .is_some_and(|source| !crate::meta_reasoner::text::js_trim(source).is_empty()),
        test_read
            .and_then(|value| value.source.as_deref())
            .is_some_and(|source| independent_acceptance(&request, source)),
        read_tool.is_some(),
    );
    if preflight.starts_with("gap:") {
        return Some(gap(
            &preflight,
            &request,
            research_observations(&request, &progress, None),
            ResearchDiscoveryContext::None,
            result,
        ));
    }
    if let Some(target) = preflight.strip_prefix("read:") {
        let path = if target == "module" {
            &request.module
        } else {
            request.test.as_ref()?
        };
        return read_tool.map(|tool| plan_one(tool, super::workspace_change::read_arguments(path)));
    }
    let query = coding_research_query(&request);
    let attempts = progress.research_attempts();
    let capabilities: Vec<String> = attempts.iter().map(capability).collect();
    let succeeded: Vec<bool> = attempts.iter().map(|attempt| attempt.succeeded).collect();
    let arguments: Vec<Option<Value>> = attempts
        .iter()
        .map(|attempt| {
            attempt
                .arguments
                .as_ref()
                .and_then(|value| serde_json::from_str(value).ok())
        })
        .collect();
    let valid: Vec<bool> = arguments
        .iter()
        .map(|value| value.as_ref().is_some_and(Value::is_object))
        .collect();
    let keys: Vec<f64> = arguments
        .iter()
        .map(|value| {
            value
                .as_ref()
                .and_then(Value::as_object)
                .map_or(-1.0, |object| research_length_number(object.len()))
        })
        .collect();
    let queries: Vec<String> = arguments
        .iter()
        .map(|value| {
            value
                .as_ref()
                .map_or_else(String::new, |value| text(&value["query"]))
        })
        .collect();
    let commands: Vec<String> = attempts
        .iter()
        .map(|attempt| {
            attempt
                .arguments
                .as_deref()
                .and_then(super::tool_result::command_argument)
                .unwrap_or_default()
        })
        .collect();
    let search_index = generated::coding_research_search_index(
        capabilities.clone(),
        succeeded,
        valid.clone(),
        queries,
        keys,
        query.clone(),
    );
    let search_decision =
        generated::coding_research_search_decision(search_index >= 0.0, search_tool.is_some());
    if search_decision.starts_with("gap:") {
        return Some(gap(
            &search_decision,
            &request,
            research_observations(&request, &progress, None),
            ResearchDiscoveryContext::None,
            result,
        ));
    }
    if search_decision == "search" {
        return search_tool.map(|tool| plan_one(tool, json!({"query":query}).to_string()));
    }
    let search = attempts.get(research_number_index(search_index)?)?;
    let search_text = normalized_payload(&search.detail).unwrap_or_default();
    let capture_decision = generated::coding_research_capture_decision(
        research_length_number(search_text.len()),
        true,
    );
    if capture_decision.starts_with("gap:") {
        return Some(gap(
            &capture_decision,
            &request,
            research_observations(&request, &progress, None),
            ResearchDiscoveryContext::None,
            result,
        ));
    }
    let found = super::web_research::urls_in(&search_text);
    let secure = found
        .iter()
        .map(|url| url::Url::parse(url).is_ok_and(|parsed| parsed.scheme() == "https"))
        .collect();
    let urls = generated::coding_research_source_ur_ls(found, secure);
    let mut proposals = Vec::new();
    for (url, raw) in &progress.fetched_pages {
        let source_index = generated::coding_research_source_index(urls.clone(), url.clone());
        let body = raw;
        if !generated::coding_research_page_eligible(
            source_index,
            research_length_number(body.len()),
        ) {
            continue;
        }
        let (_, blocks) =
            crate::web_formalize::formalize_page_with_context(body.as_bytes(), None, Some(url));
        let code_blocks: Vec<Value> = blocks
            .into_iter()
            .filter_map(|(_, block)| {
                if let crate::web_formalize::PageBlock::CodeBlock { language, text } = block {
                    Some(json!({"language":language,"text":text}))
                } else {
                    None
                }
            })
            .collect();
        let page = json!({"url":url,"sha256":crate::source_fetch::sha256_hex(body.as_bytes()),"query":query,"codeBlocks":code_blocks,
            "sourceObservation":progress.source_observation_for(url, raw)});
        if let Some(proposal) = derive_coding_procedure(&request, &page) {
            proposals.push(proposal);
        }
    }
    let next_url = generated::coding_research_next_url(urls, progress.attempted_fetches.clone());
    let discovery = generated::coding_research_discovery_kernel(
        research_length_number(proposals.len()),
        fetch_tool.is_some(),
        next_url.clone(),
    );
    if discovery.starts_with("gap:") {
        return Some(gap(
            &discovery,
            &request,
            research_observations(&request, &progress, None),
            ResearchDiscoveryContext::Search {
                query,
                attempted: progress.attempted_fetches.len(),
            },
            result,
        ));
    }
    if discovery == "fetch" {
        return fetch_tool.map(|tool| plan_one(tool, fetch_arguments(&next_url)));
    }
    let proposal = proposals.first()?;
    let proposed_source = text(&proposal["source"]);
    let module_source = module_read
        .and_then(|value| value.source.as_deref())
        .unwrap_or_default();
    let latest_source = progress.successful_write_content_for(&request.module);
    let recipe = crate::engine::ExecutionRecipe {
        language: request.language.clone(),
        path: request.module.clone(),
        source: proposed_source.clone(),
        supporting_files: Vec::new(),
        commands: vec![request.command.clone()?],
    };
    let mut plan = if crate::meta_reasoner::text::js_trim(module_source).is_empty() {
        super::command_reroute::plan_symbolic_command_reroute(
            messages,
            tools,
            &crate::engine::SymbolicAnswer {
                intent: "coding_research".to_owned(),
                answer: String::new(),
                confidence: 1.0,
                evidence_links: Vec::new(),
                thinking_steps: Vec::new(),
                links_notation: String::new(),
                execution_recipe: Some(Box::new(recipe)),
            },
        )
    } else {
        None
    };
    let planned_write_pure = plan.as_ref().is_none_or(|plan| {
        let AgenticPlan::ToolCalls(calls) = plan else {
            return true;
        };
        calls
            .iter()
            .filter(|call| Some(call.tool.as_str()) == tool_for(tools, Capability::Write))
            .all(|call| {
                let Ok(value) = serde_json::from_str::<Value>(&call.arguments) else {
                    return false;
                };
                let path = ["path", "filePath", "file_path"]
                    .iter()
                    .find_map(|key| value[*key].as_str());
                path != Some(request.module.as_str())
                    || value["content"]
                        .as_str()
                        .is_some_and(|source| pure_research_program(&request, source))
            })
    });
    let boundary = if crate::meta_reasoner::text::js_trim(module_source).is_empty() {
        progress
            .latest_successful_write_index(&request.module)
            .map_or_else(
                || research_length_number(attempts.len()),
                research_length_number,
            )
    } else {
        generated::coding_research_latest_read_index(capabilities.clone())
    };
    let run_index = generated::coding_research_run_index(
        capabilities,
        valid,
        commands,
        boundary,
        request.command.clone()?,
    );
    let observation = if run_index >= 0.0 {
        research_number_index(run_index).and_then(|index| attempts.get(index))
    } else {
        None
    };
    let last_outcome = outcome(observation);
    let receipt_gap = observation
        .and_then(|attempt| verification_receipt_gap(&attempt.detail))
        .unwrap_or_default();
    let reroute_kind = match &plan {
        Some(AgenticPlan::Final(_)) => "final",
        Some(AgenticPlan::ToolCalls(_)) => "tool_calls",
        None => "",
    };
    let effect = generated::coding_research_effect_kernel(
        module_source.to_owned(),
        proposed_source,
        latest_source.is_some(),
        latest_source
            .as_deref()
            .is_none_or(|source| pure_research_program(&request, source)),
        planned_write_pure,
        plan.is_some(),
        reroute_kind.to_owned(),
        last_outcome.clone(),
        receipt_gap.clone(),
        run_tool.is_some(),
    );
    let observations = research_observations(
        &request,
        &progress,
        Some(ResearchVerificationObservation {
            command: request.command.clone(),
            receipt: observation.and_then(|attempt| attempt.raw_receipt.clone()),
            tool_succeeded: observation.map(|attempt| attempt.succeeded),
            outcome: last_outcome,
            receipt_gap: if receipt_gap.is_empty() {
                None
            } else {
                Some(receipt_gap)
            },
            decision: effect.clone(),
        }),
    );
    if effect.starts_with("gap:") {
        return Some(gap(
            &effect,
            &request,
            observations,
            ResearchProvenance::from_observed(&proposal["provenance"]).map_or(
                ResearchDiscoveryContext::None,
                ResearchDiscoveryContext::Source,
            ),
            result,
        ));
    }
    if effect == "run" {
        return run_tool.map(|tool| plan_one(tool, json!({"command":request.command}).to_string()));
    }
    if effect == "reroute" {
        return plan;
    }
    let verified = effect.ends_with(":verified");
    let existing = effect.starts_with("existing:");
    let answer = if existing {
        if verified {
            message("coding-research-verified-existing", &[])
        } else {
            message(
                "coding-research-existing-failed",
                &[(
                    "detail",
                    observation.map_or("", |attempt| attempt.detail.as_str()),
                )],
            )
        }
    } else {
        match plan.take()? {
            AgenticPlan::Final(answer) => answer,
            AgenticPlan::ToolCalls(_) => return None,
        }
    };
    let origin = match (existing, verified) {
        (true, true) => "coding-research-verified-existing",
        (true, false) => "coding-research-existing-verification-failed",
        (false, true) => "coding-research-verified-created",
        (false, false) => "coding-research-verification-failed",
    };
    let plan = record(
        AgenticPlan::Final(answer),
        if verified {
            FinalDisposition::Finding
        } else {
            FinalDisposition::Failure
        },
        origin,
        result,
    );
    if let Some(result) = result {
        result.research =
            ResearchProvenance::from_observed(&proposal["provenance"]).map(|provenance| {
                ResearchFinalEvidence {
                    provenance: provenance.terminal(
                        verified,
                        existing,
                        crate::source_fetch::sha256_hex(
                            test_read
                                .and_then(|read| read.source.as_deref())
                                .unwrap_or_default()
                                .as_bytes(),
                        ),
                    ),
                    observations,
                }
            });
    }
    Some(plan)
}
/// Successful ordinary plans remain first; only exact source-composer gaps fall through.
pub(super) fn plan_module_function_research_step(
    task: &str,
    messages: &[ChatMessage],
    tools: &[&str],
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let ordinary = super::module_function::plan_module_function_step(task, messages, tools, result);
    if !matches!(ordinary, Some(AgenticPlan::Final(_)))
        || !result.as_ref().is_some_and(|value| {
            matches!(
                value.origin.as_str(),
                "module-function-request-unconsumed" | "module-function-recipe-missing"
            )
        })
    {
        return ordinary;
    }
    let mut research_result = None;
    plan_coding_research_step(task, messages, tools, &mut research_result).map_or(
        ordinary,
        |plan| {
            *result = research_result;
            Some(plan)
        },
    )
}

#[cfg(test)]
#[path = "../../tests/fixtures/coding-research-numeric-boundaries.rs"]
mod numeric_boundary_tests;

#[cfg(test)]
#[path = "../../tests/fixtures/coding-research-terminal-evidence.rs"]
mod terminal_evidence_tests;

#[cfg(test)]
#[path = "../../tests/fixtures/coding-research-tap-counters.rs"]
mod tap_counter_tests;
