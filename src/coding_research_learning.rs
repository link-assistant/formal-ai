//! Research-backed learning for coding skill gaps (issue #919).
//!
//! A normal program-synthesis miss supplies the stable gap identity. This
//! module plans a query, retrieves exact bytes through the opt-in source cache,
//! compiles a narrowly typed coding procedure into Links Notation, and runs it
//! through the same bounded workspace-rewrite executor as hand-seeded
//! procedures. Search prose never becomes a capability: only an exact execution
//! match plus named review enters the content-addressed procedure ledger.

use std::collections::BTreeMap;
use std::fmt;

use crate::engine::stable_id;
use crate::links_format::push_lino_node;
use crate::program_skill_gap;
use crate::research_learning::{
    AutonomyMode, CycleConfig, KnowledgeKind, RecoveryOption, ResearchLearningCycle,
    VerificationGate,
};
use crate::seed::parser::{parse_lino, LinoNode};
use crate::source_fetch::{CachedSourceClient, SourceCapture, SourceTransport};
use crate::source_research::{execute_source_research, SourceResearchExecution};
use crate::workspace_change_learning::{
    execute_workspace_rewrite, WorkspaceRewriteExecution, WORKSPACE_CHANGE_TASK_FAMILY,
};

const SEARCH_PAGE_LIMIT: usize = 3;
const SOURCE_HEADER: &str = "Formal AI coding procedure";

/// Data-authored policy interpreted before every research round.
pub const CODING_RESEARCH_LEARNING_CONTRACT: &str =
    include_str!("../data/meta/coding-research-learning-contract.lino");

/// A failed research or verification stage. `cycle` retains the audit trail up
/// to the failure so callers can inspect it without accepting a procedure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodingResearchError {
    pub reason: String,
    pub cycle: String,
}

impl fmt::Display for CodingResearchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.reason)
    }
}

impl std::error::Error for CodingResearchError {}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResearchRound {
    query: String,
    status: &'static str,
    detail: String,
}

/// Durable scheduling state for one genuine `write_program` skill gap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodingResearchGap {
    task: String,
    language: String,
    name: String,
    next_query: String,
    rounds: Vec<ResearchRound>,
    resolved_procedure: Option<String>,
}

impl CodingResearchGap {
    /// Construct from the same identity emitted after every built-in synthesis
    /// route misses.
    #[must_use]
    pub fn for_program_task(task: impl Into<String>, language: impl Into<String>) -> Self {
        let task = task.into();
        let language = language.into();
        let name = program_skill_gap::gap_name(Some(&task), Some(&language));
        let next_query = base_query(&task, &language);
        Self {
            task,
            language,
            name,
            next_query,
            rounds: Vec::new(),
            resolved_procedure: None,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn next_query(&self) -> &str {
        &self.next_query
    }

    #[must_use]
    pub fn failed_rounds(&self) -> usize {
        self.rounds
            .iter()
            .filter(|round| round.status == "failed")
            .count()
    }

    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::new();
        push_lino_node(&mut out, 0, "coding_research_gap", Some(&self.name));
        push_lino_node(&mut out, 2, "task", Some(&self.task));
        push_lino_node(&mut out, 2, "language", Some(&self.language));
        push_lino_node(
            &mut out,
            2,
            "status",
            Some(if self.resolved_procedure.is_some() {
                "resolved"
            } else {
                "open"
            }),
        );
        if let Some(procedure) = &self.resolved_procedure {
            push_lino_node(&mut out, 2, "resolved_procedure", Some(procedure));
        } else {
            push_lino_node(&mut out, 2, "next_query", Some(&self.next_query));
        }
        for (index, round) in self.rounds.iter().enumerate() {
            push_lino_node(
                &mut out,
                2,
                "research_round",
                Some(&(index + 1).to_string()),
            );
            push_lino_node(&mut out, 4, "query", Some(&round.query));
            push_lino_node(&mut out, 4, "status", Some(round.status));
            push_lino_node(&mut out, 4, "detail", Some(&round.detail));
        }
        out
    }

    fn record_failure(&mut self, query: String, reason: &str) {
        self.rounds.push(ResearchRound {
            query,
            status: "failed",
            detail: reason.to_owned(),
        });
        let next_round = self.failed_rounds() + 1;
        self.next_query = format!(
            "{} alternative evidence round {next_round}",
            base_query(&self.task, &self.language)
        );
    }

    fn record_success(&mut self, query: String, procedure_id: &str) {
        self.rounds.push(ResearchRound {
            query,
            status: "verified",
            detail: procedure_id.to_owned(),
        });
        self.resolved_procedure = Some(procedure_id.to_owned());
    }
}

fn base_query(task: &str, language: &str) -> String {
    format!("{language} {task} verified coding procedure SPDX license")
}

/// Named review gate retained from the existing procedure-learning contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodingResearchApproval {
    reviewer: String,
    granted: bool,
}

impl CodingResearchApproval {
    #[must_use]
    pub fn granted(reviewer: impl Into<String>) -> Self {
        Self {
            reviewer: reviewer.into(),
            granted: true,
        }
    }

    #[must_use]
    pub fn declined(reviewer: impl Into<String>) -> Self {
        Self {
            reviewer: reviewer.into(),
            granted: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceProcedure {
    task: String,
    language: String,
    license: String,
    operation: String,
    pattern: String,
    replacement: String,
}

impl SourceProcedure {
    fn parse(bytes: &[u8]) -> Result<Self, &'static str> {
        let text = std::str::from_utf8(bytes).map_err(|_| "coding_research_source_not_utf8")?;
        let mut lines = text.lines();
        if lines.next() != Some(SOURCE_HEADER) {
            return Err("coding_research_source_format_unrecognized");
        }
        let mut fields = BTreeMap::new();
        for line in lines {
            let Some((name, value)) = line.split_once(": ") else {
                return Err("coding_research_source_field_invalid");
            };
            if value.trim().is_empty() || fields.insert(name, value).is_some() {
                return Err("coding_research_source_field_invalid");
            }
        }
        let value = |name| {
            fields
                .get(name)
                .copied()
                .ok_or("coding_research_source_field_missing")
        };
        let license = value("SPDX-License-Identifier")?;
        if !valid_spdx_expression(license) {
            return Err("coding_research_source_license_invalid");
        }
        Ok(Self {
            task: value("Task")?.to_owned(),
            language: value("Language")?.to_owned(),
            operation: value("Operation")?.to_owned(),
            pattern: value("Pattern")?.to_owned(),
            replacement: value("Replacement")?.to_owned(),
            license: license.to_owned(),
        })
    }

    fn formalize(&self, gap_name: &str) -> String {
        let body = self.formalization_body(gap_name);
        let id = stable_id("researched_coding_procedure_formalization", &body);
        let mut out = String::new();
        push_lino_node(&mut out, 0, "coding_procedure", Some(&id));
        out.push_str(&body);
        out
    }

    fn formalization_body(&self, gap_name: &str) -> String {
        let mut out = String::new();
        push_lino_node(&mut out, 2, "origin", Some("research"));
        push_lino_node(&mut out, 2, "gap_name", Some(gap_name));
        push_lino_node(&mut out, 2, "task", Some(&self.task));
        push_lino_node(&mut out, 2, "language", Some(&self.language));
        push_lino_node(&mut out, 2, "operation", Some(&self.operation));
        push_lino_node(&mut out, 2, "pattern", Some(&self.pattern));
        push_lino_node(&mut out, 2, "replacement", Some(&self.replacement));
        out
    }
}

fn valid_spdx_expression(expression: &str) -> bool {
    !matches!(expression, "NONE" | "NOASSERTION")
        && expression.bytes().any(|byte| byte.is_ascii_alphanumeric())
        && expression.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(byte, b'-' | b'.' | b'+' | b':' | b'(' | b')' | b' ')
        })
}

/// One execution-verified procedure plus immutable capture provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResearchedCodingProcedure {
    pub id: String,
    pub gap_name: String,
    pub task: String,
    pub language: String,
    pub query: String,
    pub source_url: String,
    pub source_license: String,
    pub fetched_at: String,
    pub source_sha256: String,
    pub formalization: String,
    pub executor: String,
    pub pattern: String,
    pub replacement: String,
    pub verified_output_sha256: String,
    pub verification_steps: usize,
    pub reviewer: String,
}

impl ResearchedCodingProcedure {
    fn identity(&self) -> String {
        let mut out = String::new();
        for value in [
            self.gap_name.as_str(),
            self.task.as_str(),
            self.language.as_str(),
            self.query.as_str(),
            self.source_url.as_str(),
            self.source_license.as_str(),
            self.fetched_at.as_str(),
            self.source_sha256.as_str(),
            self.formalization.as_str(),
            self.executor.as_str(),
            self.pattern.as_str(),
            self.replacement.as_str(),
            self.verified_output_sha256.as_str(),
            self.reviewer.as_str(),
        ] {
            out.push_str(value);
            out.push('\n');
        }
        out.push_str(&self.verification_steps.to_string());
        out
    }

    fn expected_id(&self) -> String {
        stable_id("researched_coding_procedure", &self.identity())
    }

    fn write_body(&self, out: &mut String) {
        push_lino_node(out, 2, "procedure", Some(&self.id));
        push_lino_node(out, 4, "origin", Some("research"));
        push_lino_node(out, 4, "status", Some("execution_verified"));
        push_lino_node(out, 4, "gap_name", Some(&self.gap_name));
        push_lino_node(out, 4, "task", Some(&self.task));
        push_lino_node(out, 4, "language", Some(&self.language));
        push_lino_node(out, 4, "query", Some(&self.query));
        push_lino_node(out, 4, "source_url", Some(&self.source_url));
        push_lino_node(out, 4, "source_license", Some(&self.source_license));
        push_lino_node(out, 4, "fetched_at", Some(&self.fetched_at));
        push_lino_node(out, 4, "source_sha256", Some(&self.source_sha256));
        push_lino_node(out, 4, "formalization", Some(&self.formalization));
        push_lino_node(out, 4, "executor", Some(&self.executor));
        push_lino_node(out, 4, "pattern", Some(&self.pattern));
        push_lino_node(out, 4, "replacement", Some(&self.replacement));
        push_lino_node(
            out,
            4,
            "verified_output_sha256",
            Some(&self.verified_output_sha256),
        );
        push_lino_node(
            out,
            4,
            "verification_steps",
            Some(&self.verification_steps.to_string()),
        );
        push_lino_node(out, 4, "reviewer", Some(&self.reviewer));
    }
}

/// Content-addressed durable memory for source-derived coding capabilities.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResearchedCodingProcedureLedger {
    procedures: BTreeMap<String, ResearchedCodingProcedure>,
}

impl ResearchedCodingProcedureLedger {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            procedures: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.procedures.is_empty()
    }

    #[must_use]
    pub fn procedure_for(&self, gap_name: &str) -> Option<&ResearchedCodingProcedure> {
        self.procedures.get(gap_name)
    }

    fn promote(&mut self, procedure: ResearchedCodingProcedure) -> Result<(), CodingResearchError> {
        validate_procedure(&procedure)?;
        self.procedures
            .insert(procedure.gap_name.clone(), procedure);
        Ok(())
    }

    #[must_use]
    pub fn links_notation(&self) -> String {
        let body = self.review_body();
        let id = stable_id("researched_coding_procedure_ledger", &body);
        let mut out = String::new();
        push_lino_node(&mut out, 0, "researched_coding_procedure_ledger", Some(&id));
        push_lino_node(&mut out, 2, "schema_version", Some("1"));
        push_lino_node(&mut out, 2, "human_gated", Some("true"));
        out.push_str(&body);
        out
    }

    fn review_body(&self) -> String {
        let mut out = String::new();
        for procedure in self.procedures.values() {
            procedure.write_body(&mut out);
        }
        out
    }

    pub fn from_links_notation(document: &str) -> Result<Self, CodingResearchError> {
        let tree = parse_lino(document);
        let root = root_named(&tree, "researched_coding_procedure_ledger")?;
        if required(root, "schema_version")? != "1" || required(root, "human_gated")? != "true" {
            return Err(error("coding_research_ledger_policy_invalid"));
        }
        let mut ledger = Self::new();
        for node in root.children.iter().filter(|node| node.name == "procedure") {
            if required(node, "origin")? != "research"
                || required(node, "status")? != "execution_verified"
            {
                return Err(error("coding_research_procedure_status_invalid"));
            }
            let procedure = ResearchedCodingProcedure {
                id: node.id.clone(),
                gap_name: required(node, "gap_name")?,
                task: required(node, "task")?,
                language: required(node, "language")?,
                query: required(node, "query")?,
                source_url: required(node, "source_url")?,
                source_license: required(node, "source_license")?,
                fetched_at: required(node, "fetched_at")?,
                source_sha256: required(node, "source_sha256")?,
                formalization: required(node, "formalization")?,
                executor: required(node, "executor")?,
                pattern: required(node, "pattern")?,
                replacement: required(node, "replacement")?,
                verified_output_sha256: required(node, "verified_output_sha256")?,
                verification_steps: parse_usize(node, "verification_steps")?,
                reviewer: required(node, "reviewer")?,
            };
            ledger.promote(procedure)?;
        }
        let expected = stable_id("researched_coding_procedure_ledger", &ledger.review_body());
        if root.id != expected {
            return Err(error("coding_research_ledger_content_address_mismatch"));
        }
        Ok(ledger)
    }
}

/// Successful research and exact execution evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodingResearchExecution {
    pub gap_name: String,
    pub procedure_id: String,
    pub output: String,
    pub research_proposal: String,
    pub formalization: String,
    pub cycle: String,
}

/// Resolve one recorded gap through cached research and bounded execution.
///
/// Live network access remains controlled exclusively by
/// `CachedSourceClient::with_online`; the default client is offline.
pub fn research_coding_skill_gap<T: SourceTransport>(
    gap: &mut CodingResearchGap,
    ledger: &mut ResearchedCodingProcedureLedger,
    client: &CachedSourceClient<T>,
    source: &str,
    expected_output: &str,
    approval: &CodingResearchApproval,
) -> Result<CodingResearchExecution, CodingResearchError> {
    let query = gap.next_query.clone();
    let mut cycle = ResearchLearningCycle::new(
        "coding_research_no_procedure",
        ["source_provenance", "workspace_execution"],
        CycleConfig {
            autonomy: AutonomyMode::FullTrust,
            ..CycleConfig::default()
        },
    );
    cycle.begin_unknown(gap.name.clone());
    if let Err(contract_error) = validate_contract() {
        return Err(fail(gap, query, &contract_error.reason, cycle));
    }
    if gap.task.trim().is_empty() || gap.language.trim().is_empty() {
        return Err(fail(
            gap,
            query,
            "coding_research_task_identity_required",
            cycle,
        ));
    }

    let research = match execute_source_research(client, &query, SEARCH_PAGE_LIMIT) {
        Ok(research) => research,
        Err(fetch_error) => {
            return Err(fail(
                gap,
                query,
                &format!("coding_research_fetch_failed:{fetch_error}"),
                cycle,
            ));
        }
    };
    let (source_procedure, capture) = match captured_procedure(&research, gap) {
        Ok(procedure) => procedure,
        Err(reason) => return Err(fail(gap, query, reason, cycle)),
    };
    cycle.record_source(
        capture.source_url(),
        String::from_utf8_lossy(capture.bytes()),
        true,
    );
    let formalization = source_procedure.formalize(&gap.name);
    let formalization_tree = parse_lino(&formalization);
    let formalization_id = root_named(&formalization_tree, "coding_procedure")?
        .id
        .clone();
    let candidate_id = cycle.propose_version(KnowledgeKind::Procedure, formalization.clone());

    let execution = match execute_workspace_rewrite(
        source,
        &source_procedure.pattern,
        &source_procedure.replacement,
    ) {
        Ok(execution) => execution,
        Err(execution_error) => {
            cycle.verify_candidate(&candidate_id, verification_gates(false, approval));
            return Err(fail(
                gap,
                query,
                &format!("coding_research_execution_failed:{execution_error}"),
                cycle,
            ));
        }
    };
    let exact = execution.output == expected_output;
    let approved = approval.granted && !approval.reviewer.trim().is_empty();
    let promoted = cycle.verify_candidate(&candidate_id, verification_gates(exact, approval));
    if !exact {
        return Err(fail(
            gap,
            query,
            "coding_research_execution_verification_failed",
            cycle,
        ));
    }
    if !approved {
        return Err(fail(
            gap,
            query,
            "coding_research_human_approval_required",
            cycle,
        ));
    }
    if !promoted {
        return Err(fail(
            gap,
            query,
            "coding_research_verification_gate_failed",
            cycle,
        ));
    }

    let mut procedure = procedure_from_execution(
        gap,
        &query,
        &source_procedure,
        capture,
        formalization_id,
        &execution,
        approval.reviewer.trim(),
    );
    procedure.id = procedure.expected_id();
    let procedure_id = procedure.id.clone();
    ledger.promote(procedure)?;
    gap.record_success(query, &procedure_id);
    Ok(CodingResearchExecution {
        gap_name: gap.name.clone(),
        procedure_id,
        output: execution.output,
        research_proposal: research.learning_proposal(),
        formalization,
        cycle: cycle.links_notation(),
    })
}

/// Apply an approved source-derived operation to an equivalent held-out input
/// through the same bounded executor used during verification.
pub fn execute_researched_coding_procedure(
    ledger: &ResearchedCodingProcedureLedger,
    gap_name: &str,
    source: &str,
) -> Result<WorkspaceRewriteExecution, CodingResearchError> {
    let procedure = ledger
        .procedure_for(gap_name)
        .ok_or_else(|| error("coding_research_procedure_not_approved"))?;
    validate_procedure(procedure)?;
    execute_workspace_rewrite(source, &procedure.pattern, &procedure.replacement).map_err(
        |execution_error| {
            error(format!(
                "coding_research_execution_failed:{execution_error}"
            ))
        },
    )
}

fn captured_procedure<'a>(
    research: &'a SourceResearchExecution,
    gap: &CodingResearchGap,
) -> Result<(SourceProcedure, &'a SourceCapture), &'static str> {
    for page in &research.pages {
        let Ok(procedure) = SourceProcedure::parse(page.capture.bytes()) else {
            continue;
        };
        if procedure.task != gap.task || !procedure.language.eq_ignore_ascii_case(&gap.language) {
            continue;
        }
        if procedure.operation != WORKSPACE_CHANGE_TASK_FAMILY {
            continue;
        }
        return Ok((procedure, &page.capture));
    }
    Err("coding_research_procedure_not_found")
}

fn verification_gates(
    execution_passed: bool,
    approval: &CodingResearchApproval,
) -> Vec<VerificationGate> {
    vec![
        VerificationGate::immutable("source_provenance", true),
        VerificationGate::immutable("workspace_execution", execution_passed),
        VerificationGate::adaptive(
            "named_human_review",
            approval.granted && !approval.reviewer.trim().is_empty(),
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
fn procedure_from_execution(
    gap: &CodingResearchGap,
    query: &str,
    source: &SourceProcedure,
    capture: &SourceCapture,
    formalization: String,
    execution: &WorkspaceRewriteExecution,
    reviewer: &str,
) -> ResearchedCodingProcedure {
    ResearchedCodingProcedure {
        id: String::new(),
        gap_name: gap.name.clone(),
        task: gap.task.clone(),
        language: gap.language.clone(),
        query: query.to_owned(),
        source_url: capture.source_url().to_owned(),
        source_license: source.license.clone(),
        fetched_at: capture.fetched_at().to_owned(),
        source_sha256: capture.sha256().to_owned(),
        formalization,
        executor: WORKSPACE_CHANGE_TASK_FAMILY.to_owned(),
        pattern: source.pattern.clone(),
        replacement: source.replacement.clone(),
        verified_output_sha256: crate::source_fetch::sha256_hex(execution.output.as_bytes()),
        verification_steps: execution.steps,
        reviewer: reviewer.to_owned(),
    }
}

fn validate_procedure(procedure: &ResearchedCodingProcedure) -> Result<(), CodingResearchError> {
    let expected_gap =
        program_skill_gap::gap_name(Some(&procedure.task), Some(&procedure.language));
    let source = SourceProcedure {
        task: procedure.task.clone(),
        language: procedure.language.clone(),
        license: procedure.source_license.clone(),
        operation: procedure.executor.clone(),
        pattern: procedure.pattern.clone(),
        replacement: procedure.replacement.clone(),
    };
    let expected_formalization = root_named(
        &parse_lino(&source.formalize(&procedure.gap_name)),
        "coding_procedure",
    )?
    .id
    .clone();
    if procedure.id != procedure.expected_id()
        || procedure.gap_name != expected_gap
        || procedure.task.trim().is_empty()
        || procedure.language.trim().is_empty()
        || procedure.query.trim().is_empty()
        || !valid_source_url(&procedure.source_url)
        || !valid_spdx_expression(&procedure.source_license)
        || procedure.fetched_at.parse::<u64>().is_err()
        || !is_sha256(&procedure.source_sha256)
        || procedure.formalization != expected_formalization
        || procedure.executor != WORKSPACE_CHANGE_TASK_FAMILY
        || procedure.pattern.is_empty()
        || procedure.pattern == procedure.replacement
        || !is_sha256(&procedure.verified_output_sha256)
        || procedure.verification_steps == 0
        || procedure.reviewer.trim().is_empty()
    {
        return Err(error("coding_research_procedure_integrity_failed"));
    }
    Ok(())
}

fn valid_source_url(url: &str) -> bool {
    (url.starts_with("https://") || url.starts_with("http://"))
        && !url
            .bytes()
            .any(|byte| byte.is_ascii_whitespace() || byte.is_ascii_control())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_contract() -> Result<(), CodingResearchError> {
    let tree = parse_lino(CODING_RESEARCH_LEARNING_CONTRACT);
    let root = root_named(&tree, "coding_research_learning_contract")?;
    for (field, expected) in [
        ("schema_version", "1"),
        ("gap_source", "program_skill_gap"),
        ("research_boundary", "source_research"),
        ("formalization", "links_notation"),
        ("procedure_origin", "research"),
        ("executor", WORKSPACE_CHANGE_TASK_FAMILY),
        ("verification", "exact_expected_output"),
        ("live_fetch", "opt_in"),
        ("offline_replay", "source_cache"),
        ("failure_effect", "schedule_next_query"),
        ("human_review", "required"),
    ] {
        if required(root, field)? != expected {
            return Err(error(format!("coding_research_contract_invalid_{field}")));
        }
    }
    let provenance = root
        .children
        .iter()
        .filter(|child| child.name == "provenance_field")
        .map(|child| child.id.as_str())
        .collect::<Vec<_>>();
    if provenance
        != [
            "source_url",
            "source_license",
            "fetched_at",
            "source_sha256",
        ]
    {
        return Err(error("coding_research_contract_provenance_incomplete"));
    }
    Ok(())
}

fn fail(
    gap: &mut CodingResearchGap,
    query: String,
    reason: &str,
    mut cycle: ResearchLearningCycle,
) -> CodingResearchError {
    cycle.recover_from_error(reason, vec![RecoveryOption::new("research_next_source")]);
    gap.record_failure(query, reason);
    CodingResearchError {
        reason: reason.to_owned(),
        cycle: cycle.links_notation(),
    }
}

fn error(reason: impl Into<String>) -> CodingResearchError {
    CodingResearchError {
        reason: reason.into(),
        cycle: String::new(),
    }
}

fn root_named<'a>(tree: &'a LinoNode, name: &str) -> Result<&'a LinoNode, CodingResearchError> {
    tree.children
        .iter()
        .find(|node| node.name == name)
        .ok_or_else(|| error(format!("coding_research_missing_{name}")))
}

fn required(node: &LinoNode, name: &str) -> Result<String, CodingResearchError> {
    node.children
        .iter()
        .find(|child| child.name == name)
        .map(|child| child.id.clone())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| error(format!("coding_research_missing_{name}")))
}

fn parse_usize(node: &LinoNode, name: &str) -> Result<usize, CodingResearchError> {
    required(node, name)?
        .parse::<usize>()
        .map_err(|_| error(format!("coding_research_invalid_{name}")))
}
