use super::runner::{AgentSession, AgentTarget, CorrectionRequest};
use crate::client_contract_learning::{ClientContractObservation, DeliveryMode};
use crate::language;
use crate::relative_meta_logic::SourceTier;
use crate::summarization::{
    SourcedStatement, SummarizationConfig, SummarizationMode, deduplicate, deformalize, formalize,
    rank, recheck, summarize,
};
use crate::translation::formalize_prompt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

const SOURCES_PLACEHOLDER: &str = concat!("{", "sources", "}");
const PROBABILITY_PLACEHOLDER: &str = concat!("{", "probability", "}");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentSynthesisSource {
    pub cli: String,
    pub session_sha256: String,
    pub detected_language: String,
    pub meta_language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentSynthesisClaim {
    pub id: String,
    pub text: String,
    pub sources: Vec<String>,
    pub denied_by: Vec<String>,
    pub probability: f64,
    pub importance: u8,
    pub verdict: String,
    pub presented: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentSynthesisContradiction {
    pub asserted: String,
    pub denied: String,
    pub terms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerifiedTranslation {
    pub text: String,
    pub language: String,
    pub session_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentSynthesisReport {
    pub schema: String,
    pub target_language: String,
    pub final_language: String,
    pub fact_check_scope: String,
    pub sources: Vec<AgentSynthesisSource>,
    pub claims: Vec<AgentSynthesisClaim>,
    pub contradictions: Vec<AgentSynthesisContradiction>,
    pub corrections: Vec<CorrectionRequest>,
    pub summary: String,
    pub final_answer: String,
    pub translation_required: bool,
    pub translation: Option<VerifiedTranslation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentSynthesisError {
    MissingSources,
    UnsupportedLanguage(String),
    SessionDigest(String),
    TranslationLanguageMismatch { expected: String, actual: String },
}

impl fmt::Display for AgentSynthesisError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSources => formatter.write_str("missing_agent_sources"),
            Self::UnsupportedLanguage(language) => {
                write!(formatter, "unsupported_response_language:{language}")
            }
            Self::SessionDigest(error) => write!(formatter, "session_digest:{error}"),
            Self::TranslationLanguageMismatch { expected, actual } => {
                write!(
                    formatter,
                    "translation_language_mismatch:{expected}:{actual}"
                )
            }
        }
    }
}

impl std::error::Error for AgentSynthesisError {}

/// Merge agent output as sourced statements, rank it by evidence, and run the
/// repository's conservative cross-source preflight before presentation.
///
/// This deliberately names its scope `cross_agent_evidence_preflight`: model
/// agreement is not external proof. Callers needing factual guarantees must
/// gather primary captures and use the production fact-checking pipeline.
pub fn synthesize_sessions(
    sessions: &[AgentSession],
    target_language: &str,
) -> Result<AgentSynthesisReport, AgentSynthesisError> {
    if sessions.is_empty() {
        return Err(AgentSynthesisError::MissingSources);
    }
    if language::from_slug(target_language).is_none() {
        return Err(AgentSynthesisError::UnsupportedLanguage(
            target_language.to_string(),
        ));
    }

    let mut sources = Vec::new();
    let mut observations = Vec::new();
    for (index, session) in sessions.iter().enumerate() {
        let source = format!("{}:{index}", session.cli);
        let result = extract_agent_result(&session.stdout);
        let detected = language::detect(&result).slug().to_string();
        let meta_language = formalize_prompt(&result, &detected).to_links_notation();
        let digest = super::runner::session_sha256(session)
            .map_err(|error| AgentSynthesisError::SessionDigest(error.to_string()))?;
        sources.push(AgentSynthesisSource {
            cli: session.cli.clone(),
            session_sha256: digest,
            detected_language: detected,
            meta_language,
        });
        observations.extend(formalize(&result).into_iter().map(|statement| {
            SourcedStatement::new(statement, source.clone(), SourceTier::OriginalFirstParty)
        }));
    }

    let dedup = deduplicate(&observations);
    let ranked = rank(&dedup);
    let checked = recheck(&ranked);
    let claims = checked
        .checked
        .iter()
        .map(|item| AgentSynthesisClaim {
            id: item.ranked.statement.id.clone(),
            text: item.text().to_string(),
            sources: item
                .ranked
                .statement
                .sources()
                .into_iter()
                .map(str::to_string)
                .collect(),
            denied_by: item.ranked.denied_by.clone(),
            probability: item.ranked.probability.get(),
            importance: item.ranked.score.weight,
            verdict: item.verdict.slug().to_string(),
            presented: item.verdict.is_presentable(),
        })
        .collect::<Vec<_>>();
    let surviving = checked
        .survivors()
        .into_iter()
        .map(|item| item.ranked.statement.representative.clone())
        .collect::<Vec<_>>();
    let summary = deformalize(&summarize(
        &surviving,
        &SummarizationConfig {
            mode: SummarizationMode::Standard,
            language: target_language.to_string(),
            ..SummarizationConfig::default()
        },
    ));
    let final_language = language::detect(&summary).slug().to_string();
    let translation_required = !summary.is_empty() && final_language != target_language;
    let contradictions = dedup
        .contradictions
        .iter()
        .map(|contradiction| AgentSynthesisContradiction {
            asserted: contradiction.asserted.clone(),
            denied: contradiction.denied.clone(),
            terms: contradiction.terms.clone(),
        })
        .collect();
    let corrections = correction_requests(sessions, &claims);

    Ok(AgentSynthesisReport {
        schema: "formal-ai-agent-synthesis-v1".to_string(),
        target_language: target_language.to_string(),
        final_language,
        fact_check_scope: "cross_agent_evidence_preflight".to_string(),
        sources,
        claims,
        contradictions,
        corrections,
        final_answer: summary.clone(),
        summary,
        translation_required,
        translation: None,
    })
}

/// Extract the last assistant answer from common JSON/JSONL agent streams.
///
/// The complete stdout remains in [`AgentSession`] for replay. Synthesis uses
/// only the answer-bearing event so client diagnostics and tool events do not
/// become factual claims. Plain-text clients pass through unchanged.
#[must_use]
pub fn extract_agent_result(stdout: &str) -> String {
    let mut best: Option<(u8, String)> = None;
    let stream = serde_json::Deserializer::from_str(stdout).into_iter::<Value>();
    let mut complete_stream = true;
    for value in stream {
        let Ok(value) = value else {
            complete_stream = false;
            break;
        };
        update_best_candidate(&mut best, &value);
    }
    if complete_stream && let Some((_, text)) = best.take() {
        return text;
    }

    // Some clients (and process supervisors around them) place diagnostics
    // before or after JSONL events. Recover only complete JSON values so those
    // diagnostics remain in the replay record without becoming claims.
    for line in stdout.lines() {
        update_embedded_json_candidates(&mut best, line);
    }
    best.map_or_else(|| stdout.trim().to_string(), |(_, text)| text)
}

/// Recover complete JSON values even when a process supervisor writes its own
/// status text on the same line. Rust's test harness does this for captured
/// `--nocapture` output (`test name ... {event} ok`), and agent wrappers may use
/// the same shape. Starting only at JSON container delimiters keeps ordinary
/// prose out of the candidate stream while allowing trailing supervisor text.
fn update_embedded_json_candidates(best: &mut Option<(u8, String)>, line: &str) {
    for (offset, _) in line
        .char_indices()
        .filter(|(_, character)| matches!(character, '{' | '['))
    {
        let mut stream = serde_json::Deserializer::from_str(&line[offset..]).into_iter::<Value>();
        if let Some(Ok(value)) = stream.next() {
            update_best_candidate(best, &value);
        }
    }
}

fn update_best_candidate(best: &mut Option<(u8, String)>, value: &Value) {
    if let Some(candidate) = result_candidate(value)
        && best
            .as_ref()
            .is_none_or(|(priority, _)| candidate.0 >= *priority)
    {
        *best = Some(candidate);
    }
}

fn result_candidate(value: &Value) -> Option<(u8, String)> {
    let event_type = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let role = value
        .get("role")
        .and_then(Value::as_str)
        .or_else(|| value.pointer("/message/role").and_then(Value::as_str));

    if value.pointer("/item/type").and_then(Value::as_str) == Some("agent_message") {
        return text_at(value, "/item/text").map(|text| (4, text));
    }
    if role == Some("assistant") {
        return content_text(
            value
                .get("content")
                .or_else(|| value.pointer("/message/content")),
        )
        .map(|text| (4, text));
    }
    if event_type == "result" {
        return ["result", "output", "text"]
            .iter()
            .find_map(|key| value.get(*key).and_then(Value::as_str))
            .filter(|text| !text.trim().is_empty())
            .map(|text| (3, text.trim().to_string()));
    }
    if matches!(event_type, "text" | "assistant") {
        return text_at(value, "/part/text")
            .or_else(|| text_at(value, "/text"))
            .map(|text| (2, text));
    }
    None
}

fn text_at(value: &Value, pointer: &str) -> Option<String> {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

fn content_text(value: Option<&Value>) -> Option<String> {
    let value = value?;
    if let Some(text) = value.as_str() {
        return (!text.trim().is_empty()).then(|| text.trim().to_string());
    }
    let parts = value
        .as_array()?
        .iter()
        .filter(|part| {
            part.get("type")
                .and_then(Value::as_str)
                .is_none_or(|kind| kind == "text")
        })
        .filter_map(|part| part.get("text").and_then(Value::as_str))
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>();
    (!parts.is_empty()).then(|| parts.join("\n"))
}

fn correction_requests(
    sessions: &[AgentSession],
    claims: &[AgentSynthesisClaim],
) -> Vec<CorrectionRequest> {
    let mut requests = Vec::new();
    for claim in claims
        .iter()
        .filter(|claim| !claim.presented && !claim.denied_by.is_empty())
    {
        for source in &claim.sources {
            let Some((cli, index)) = source.rsplit_once(':') else {
                continue;
            };
            let Ok(index) = index.parse::<usize>() else {
                continue;
            };
            let Some(session) = sessions.get(index) else {
                continue;
            };
            let Ok(source_session_sha256) = super::runner::session_sha256(session) else {
                continue;
            };
            let evidence_language = language::detect(&claim.text).slug();
            let evidence = crate::seed::localized_response(
                "orchestration_cross_agent_denial",
                evidence_language,
            )
            .unwrap_or_else(|| "orchestration_cross_agent_denial".to_string())
            .replace(SOURCES_PLACEHOLDER, &claim.denied_by.join(","))
            .replace(
                PROBABILITY_PLACEHOLDER,
                &format!("{:.6}", claim.probability),
            );
            requests.push(CorrectionRequest {
                cli: cli.to_string(),
                claim: claim.text.clone(),
                evidence,
                source_session_sha256,
            });
        }
    }
    requests
}

/// Accept translated bytes only when their detected language matches the
/// requested language, and retain the translating session digest.
pub fn apply_verified_translation(
    report: &mut AgentSynthesisReport,
    text: &str,
    translator_session_sha256: &str,
) -> Result<(), AgentSynthesisError> {
    let actual = language::detect(text).slug().to_string();
    if actual != report.target_language {
        return Err(AgentSynthesisError::TranslationLanguageMismatch {
            expected: report.target_language.clone(),
            actual,
        });
    }
    report.final_answer = text.to_string();
    report.final_language.clone_from(&actual);
    report.translation_required = false;
    report.translation = Some(VerifiedTranslation {
        text: text.to_string(),
        language: actual,
        session_sha256: translator_session_sha256.to_string(),
    });
    Ok(())
}

/// Project one real orchestration session into the existing proposal-only
/// client-contract learner.
#[must_use]
pub fn observe_orchestration_session(
    session: &AgentSession,
    evidence: impl Into<String>,
) -> ClientContractObservation {
    let mut observation = ClientContractObservation::new(
        &session.cli,
        "agent_orchestration",
        &session.task,
        DeliveryMode::InBand,
        std::iter::empty::<String>(),
        evidence,
    );
    observation.observed_contract.insert(
        "orchestration_target".to_string(),
        vec![target_slug(session.target).to_string()],
    );
    observation.observed_contract.insert(
        "orchestration_program".to_string(),
        vec![session.program.clone()],
    );
    if session.native_session.is_some() {
        observation
            .observed_contract
            .insert("native_resume".to_string(), vec!["true".to_string()]);
    }
    observation
}

const fn target_slug(target: AgentTarget) -> &'static str {
    match target {
        AgentTarget::FormalAi => "formal_ai",
        AgentTarget::Vendor => "vendor",
    }
}
