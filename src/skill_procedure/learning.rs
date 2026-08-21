//! Human-gated, durable vocabulary learning for compiled procedures.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::sync::OnceLock;

use crate::engine::stable_id;
use crate::links_format::push_lino_node;
use crate::seed::{self, ROLE_SKILL_PROCEDURE_STEP_VERB, parser::LinoNode};

use super::{ProcedureCompileError, procedure_lexicon};

/// Reviewable learning generated from an honest named capability gap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureLearningProposal {
    pub id: String,
    pub missing_step: String,
    pub source_span: (usize, usize),
    pub gap: String,
}

impl ProcedureLearningProposal {
    /// Turn a compiler gap into a proposal. A non-procedure decline has no
    /// learning signal and therefore returns `None`.
    #[must_use]
    pub fn from_compile_error(error: &ProcedureCompileError) -> Option<Self> {
        let ProcedureCompileError::UncompilableStep { step, span, gap } = error else {
            return None;
        };
        let id = stable_id(
            "procedure_learning_proposal",
            &format!("{}:{}..{}:{gap}", step.to_lowercase(), span.0, span.1),
        );
        Some(Self {
            id,
            missing_step: step.clone(),
            source_span: *span,
            gap: gap.clone(),
        })
    }

    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::new();
        push_lino_node(&mut out, 0, "procedure_learning_proposal", Some(&self.id));
        push_lino_node(&mut out, 2, "status", Some("human_review_required"));
        push_lino_node(&mut out, 2, "missing_step", Some(&self.missing_step));
        push_lino_node(
            &mut out,
            2,
            "span_start",
            Some(&self.source_span.0.to_string()),
        );
        push_lino_node(
            &mut out,
            2,
            "span_end",
            Some(&self.source_span.1.to_string()),
        );
        push_lino_node(&mut out, 2, "gap", Some(&self.gap));
        out
    }
}

/// One reviewed multilingual surface mapping for a learned capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureLearnedSurface {
    pub language: String,
    pub text: String,
}

/// One observed unsupported surface paired with a seeded paraphrase that
/// already compiles. The learner resolves the paraphrase to a typed operation;
/// callers do not supply a canonical kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureLearningObservation {
    pub language: String,
    pub missing_surface: String,
    pub supported_paraphrase: String,
}

impl ProcedureLearningObservation {
    #[must_use]
    pub fn new(
        language: impl Into<String>,
        missing_surface: impl Into<String>,
        supported_paraphrase: impl Into<String>,
    ) -> Self {
        Self {
            language: language.into(),
            missing_surface: missing_surface.into(),
            supported_paraphrase: supported_paraphrase.into(),
        }
    }
}

/// The content a reviewer may promote after a gap. The canonical kind must
/// already be a typed step meaning; learning adds surfaces, never executable
/// Rust behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureCapabilityLesson {
    pub canonical_kind: String,
    pub surfaces: Vec<ProcedureLearnedSurface>,
}

impl ProcedureCapabilityLesson {
    /// Build a lesson and require parity across every supported language.
    pub fn new<K, I, L, T>(canonical_kind: K, surfaces: I) -> Result<Self, ProcedureLearningError>
    where
        K: Into<String>,
        I: IntoIterator<Item = (L, T)>,
        L: Into<String>,
        T: Into<String>,
    {
        let mut unique = BTreeMap::new();
        for (language, text) in surfaces {
            let language = language.into();
            let text = text.into().trim().to_lowercase();
            if text.is_empty() {
                return Err(ProcedureLearningError::EmptySurface);
            }
            if unique.insert(language.clone(), text).is_some() {
                return Err(ProcedureLearningError::DuplicateLanguage(language));
            }
        }
        let languages = unique.keys().map(String::as_str).collect::<BTreeSet<_>>();
        let required = ["en", "hi", "ru", "zh"]
            .into_iter()
            .collect::<BTreeSet<_>>();
        if languages != required {
            return Err(ProcedureLearningError::MissingLanguageParity);
        }
        Ok(Self {
            canonical_kind: canonical_kind.into(),
            surfaces: unique
                .into_iter()
                .map(|(language, text)| ProcedureLearnedSurface { language, text })
                .collect(),
        })
    }
}

/// An automatically inferred, evidence-bearing mapping that still requires a
/// green regression gate and explicit human approval before promotion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureLearningCandidate {
    pub id: String,
    pub proposal_id: String,
    pub lesson: ProcedureCapabilityLesson,
    pub observations: Vec<ProcedureLearningObservation>,
}

impl ProcedureLearningCandidate {
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::new();
        push_lino_node(&mut out, 0, "procedure_learning_candidate", Some(&self.id));
        push_lino_node(&mut out, 2, "status", Some("human_review_required"));
        push_lino_node(&mut out, 2, "proposal_id", Some(&self.proposal_id));
        push_lino_node(
            &mut out,
            2,
            "canonical_kind",
            Some(&self.lesson.canonical_kind),
        );
        for observation in &self.observations {
            push_observation(&mut out, 2, observation);
        }
        out
    }
}

impl ProcedureLearningProposal {
    /// Infer one typed multilingual mapping from successful, already-supported
    /// paraphrases. Conflicting operation meanings fail closed.
    pub fn infer_candidate<I>(
        &self,
        observations: I,
    ) -> Result<ProcedureLearningCandidate, ProcedureLearningError>
    where
        I: IntoIterator<Item = ProcedureLearningObservation>,
    {
        let mut normalized = Vec::new();
        let mut surfaces = Vec::new();
        let mut canonical_kind: Option<String> = None;
        let missing_step = self.missing_step.to_lowercase();
        let mut matches_proposal = false;

        for observation in observations {
            let language = observation.language.trim().to_lowercase();
            let missing_surface = observation.missing_surface.trim().to_lowercase();
            let supported_paraphrase = observation.supported_paraphrase.trim().to_lowercase();
            if missing_surface.is_empty() {
                return Err(ProcedureLearningError::EmptySurface);
            }
            if supported_paraphrase.is_empty() {
                return Err(ProcedureLearningError::EmptySupportedParaphrase);
            }
            let found =
                super::first_procedure_match(&supported_paraphrase, ROLE_SKILL_PROCEDURE_STEP_VERB)
                    .ok_or_else(|| {
                        ProcedureLearningError::UnknownSupportedParaphrase(
                            supported_paraphrase.clone(),
                        )
                    })?;
            if canonical_kind
                .as_ref()
                .is_some_and(|current| current != &found.slug)
            {
                return Err(ProcedureLearningError::ConflictingCandidateKinds);
            }
            canonical_kind = Some(found.slug);
            matches_proposal |=
                missing_step
                    .match_indices(&missing_surface)
                    .any(|(start, matched)| {
                        super::is_standalone(&missing_step, start, start + matched.len())
                    });
            surfaces.push((language.clone(), missing_surface.clone()));
            normalized.push(ProcedureLearningObservation {
                language,
                missing_surface,
                supported_paraphrase,
            });
        }
        if !matches_proposal {
            return Err(ProcedureLearningError::EvidenceDoesNotMatchProposal);
        }
        let canonical_kind = canonical_kind.ok_or(ProcedureLearningError::MissingLanguageParity)?;
        let lesson = ProcedureCapabilityLesson::new(canonical_kind, surfaces)?;
        normalized.sort_by(|left, right| left.language.cmp(&right.language));
        let id = candidate_identity(&self.id, &lesson, &normalized);
        Ok(ProcedureLearningCandidate {
            id,
            proposal_id: self.id.clone(),
            lesson,
            observations: normalized,
        })
    }
}

/// Regression evidence required before a lesson can enter the durable ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureLearningGate {
    pub suite: String,
    pub passed: usize,
    pub failed: usize,
}

impl ProcedureLearningGate {
    #[must_use]
    pub fn passed(suite: impl Into<String>, passed: usize) -> Self {
        Self {
            suite: suite.into(),
            passed,
            failed: 0,
        }
    }

    #[must_use]
    pub fn failed(suite: impl Into<String>, passed: usize, failed: usize) -> Self {
        Self {
            suite: suite.into(),
            passed,
            failed,
        }
    }

    const fn is_green(&self) -> bool {
        self.passed > 0 && self.failed == 0
    }
}

/// Explicit reviewer decision. Automatic gap detection cannot manufacture it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureLearningApproval {
    pub reviewer: String,
    pub granted: bool,
}

impl ProcedureLearningApproval {
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

/// A promoted lesson plus the review evidence that authorized it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovedProcedureLesson {
    pub id: String,
    pub proposal_id: String,
    pub candidate_id: Option<String>,
    pub lesson: ProcedureCapabilityLesson,
    pub candidate_evidence: Vec<ProcedureLearningObservation>,
    pub gate: ProcedureLearningGate,
    pub reviewer: String,
}

/// Append-only, serializable vocabulary growth consumed by the compiler.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProcedureCapabilityLedger {
    pub lessons: Vec<ApprovedProcedureLesson>,
}

impl ProcedureCapabilityLedger {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            lessons: Vec::new(),
        }
    }

    /// Promote only an existing typed operation with green tests and explicit
    /// human approval.
    pub fn promote(
        &mut self,
        proposal: &ProcedureLearningProposal,
        lesson: ProcedureCapabilityLesson,
        gate: ProcedureLearningGate,
        approval: ProcedureLearningApproval,
    ) -> Result<(), ProcedureLearningError> {
        self.promote_lesson(&proposal.id, None, lesson, Vec::new(), gate, approval)
    }

    /// Promote an automatically inferred mapping while preserving the
    /// paraphrase evidence that selected its canonical operation.
    pub fn promote_candidate(
        &mut self,
        candidate: &ProcedureLearningCandidate,
        gate: ProcedureLearningGate,
        approval: ProcedureLearningApproval,
    ) -> Result<(), ProcedureLearningError> {
        let expected = candidate_identity(
            &candidate.proposal_id,
            &candidate.lesson,
            &candidate.observations,
        );
        if candidate.id != expected || candidate.observations.is_empty() {
            return Err(ProcedureLearningError::InvalidCandidate(
                candidate.id.clone(),
            ));
        }
        validate_candidate_evidence(&candidate.lesson, &candidate.observations)?;
        self.promote_lesson(
            &candidate.proposal_id,
            Some(candidate.id.clone()),
            candidate.lesson.clone(),
            candidate.observations.clone(),
            gate,
            approval,
        )
    }

    fn promote_lesson(
        &mut self,
        proposal_id: &str,
        candidate_id: Option<String>,
        lesson: ProcedureCapabilityLesson,
        candidate_evidence: Vec<ProcedureLearningObservation>,
        gate: ProcedureLearningGate,
        approval: ProcedureLearningApproval,
    ) -> Result<(), ProcedureLearningError> {
        if !gate.is_green() {
            return Err(ProcedureLearningError::RegressionGateFailed);
        }
        if !approval.granted || approval.reviewer.trim().is_empty() {
            return Err(ProcedureLearningError::HumanApprovalRequired);
        }
        let is_typed_kind = procedure_lexicon()
            .meanings_with_role(ROLE_SKILL_PROCEDURE_STEP_VERB)
            .any(|meaning| meaning.slug == lesson.canonical_kind);
        if !is_typed_kind {
            return Err(ProcedureLearningError::UnknownCanonicalKind(
                lesson.canonical_kind,
            ));
        }
        if self
            .lessons
            .iter()
            .any(|entry| entry.proposal_id == proposal_id)
        {
            return Err(ProcedureLearningError::DuplicateProposal(
                proposal_id.to_owned(),
            ));
        }
        let id = stable_id(
            "approved_procedure_lesson",
            &format!(
                "{}:{}:{}",
                proposal_id, lesson.canonical_kind, approval.reviewer
            ),
        );
        self.lessons.push(ApprovedProcedureLesson {
            id,
            proposal_id: proposal_id.to_owned(),
            candidate_id,
            lesson,
            candidate_evidence,
            gate,
            reviewer: approval.reviewer,
        });
        Ok(())
    }

    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::new();
        push_lino_node(&mut out, 0, "procedure_capability_ledger", None);
        push_lino_node(&mut out, 2, "schema_version", Some("2"));
        push_lino_node(&mut out, 2, "human_gated", Some("true"));
        for entry in &self.lessons {
            push_lino_node(&mut out, 2, "lesson", Some(&entry.id));
            push_lino_node(&mut out, 4, "proposal_id", Some(&entry.proposal_id));
            if let Some(candidate_id) = &entry.candidate_id {
                push_lino_node(&mut out, 4, "candidate_id", Some(candidate_id));
            }
            push_lino_node(
                &mut out,
                4,
                "canonical_kind",
                Some(&entry.lesson.canonical_kind),
            );
            push_lino_node(&mut out, 4, "suite", Some(&entry.gate.suite));
            push_lino_node(&mut out, 4, "passed", Some(&entry.gate.passed.to_string()));
            push_lino_node(&mut out, 4, "failed", Some("0"));
            push_lino_node(&mut out, 4, "reviewer", Some(&entry.reviewer));
            for surface in &entry.lesson.surfaces {
                push_lino_node(&mut out, 4, "surface", None);
                push_lino_node(&mut out, 6, "language", Some(&surface.language));
                push_lino_node(&mut out, 6, "text", Some(&surface.text));
            }
            for observation in &entry.candidate_evidence {
                push_observation(&mut out, 4, observation);
            }
        }
        out
    }

    /// Restore only a fully reviewed, green ledger.
    pub fn from_links_notation(text: &str) -> Result<Self, ProcedureLearningError> {
        let document = seed::parser::parse_lino(text);
        let root = document
            .children
            .iter()
            .find(|node| node.name == "procedure_capability_ledger")
            .ok_or_else(|| {
                ProcedureLearningError::InvalidLedger(
                    "missing procedure_capability_ledger root".to_owned(),
                )
            })?;
        if !matches!(root.find_child_value("schema_version"), "1" | "2") {
            return Err(ProcedureLearningError::InvalidLedger(
                "unsupported procedure capability ledger schema".to_owned(),
            ));
        }
        if root.find_child_value("human_gated") != "true" {
            return Err(ProcedureLearningError::InvalidLedger(
                "ledger is not marked human_gated".to_owned(),
            ));
        }
        let mut ledger = Self::new();
        let mut proposal_ids = BTreeSet::new();
        for node in root.children.iter().filter(|node| node.name == "lesson") {
            let canonical_kind = required_child(node, "canonical_kind")?;
            let proposal_id = required_child(node, "proposal_id")?;
            let candidate_id = node
                .children
                .iter()
                .find(|child| child.name == "candidate_id")
                .map(|child| child.id.clone())
                .filter(|value| !value.is_empty());
            let reviewer = required_child(node, "reviewer")?;
            let suite = required_child(node, "suite")?;
            let passed = parse_usize(node, "passed")?;
            let failed = parse_usize(node, "failed")?;
            if failed != 0 || passed == 0 || reviewer.trim().is_empty() {
                return Err(ProcedureLearningError::InvalidLedger(format!(
                    "lesson_review_evidence_not_green:{}",
                    node.id
                )));
            }
            let surfaces = node
                .children
                .iter()
                .filter(|child| child.name == "surface")
                .map(|surface| {
                    Ok((
                        required_child(surface, "language")?,
                        required_child(surface, "text")?,
                    ))
                })
                .collect::<Result<Vec<_>, ProcedureLearningError>>()?;
            let lesson = ProcedureCapabilityLesson::new(canonical_kind, surfaces)?;
            let candidate_evidence = node
                .children
                .iter()
                .filter(|child| child.name == "candidate_evidence")
                .map(|evidence| {
                    Ok(ProcedureLearningObservation {
                        language: required_child(evidence, "language")?,
                        missing_surface: required_child(evidence, "missing_surface")?,
                        supported_paraphrase: required_child(evidence, "supported_paraphrase")?,
                    })
                })
                .collect::<Result<Vec<_>, ProcedureLearningError>>()?;
            match &candidate_id {
                Some(id) => {
                    validate_candidate_evidence(&lesson, &candidate_evidence)?;
                    let expected = candidate_identity(&proposal_id, &lesson, &candidate_evidence);
                    if id != &expected {
                        return Err(ProcedureLearningError::InvalidCandidate(id.clone()));
                    }
                }
                None if !candidate_evidence.is_empty() => {
                    return Err(ProcedureLearningError::InvalidLedger(format!(
                        "candidate_evidence_without_identity:{}",
                        node.id
                    )));
                }
                None => {}
            }
            let is_typed_kind = procedure_lexicon()
                .meanings_with_role(ROLE_SKILL_PROCEDURE_STEP_VERB)
                .any(|meaning| meaning.slug == lesson.canonical_kind);
            if !is_typed_kind {
                return Err(ProcedureLearningError::UnknownCanonicalKind(
                    lesson.canonical_kind,
                ));
            }
            if !proposal_ids.insert(proposal_id.clone()) {
                return Err(ProcedureLearningError::DuplicateProposal(proposal_id));
            }
            let expected_id = stable_id(
                "approved_procedure_lesson",
                &format!("{}:{}:{}", proposal_id, lesson.canonical_kind, reviewer),
            );
            if node.id != expected_id {
                return Err(ProcedureLearningError::InvalidLedger(format!(
                    "lesson_identity_failure:{}",
                    node.id
                )));
            }
            ledger.lessons.push(ApprovedProcedureLesson {
                id: expected_id,
                proposal_id,
                candidate_id,
                lesson,
                candidate_evidence,
                gate: ProcedureLearningGate {
                    suite,
                    passed,
                    failed,
                },
                reviewer,
            });
        }
        Ok(ledger)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcedureLearningError {
    EmptySurface,
    EmptySupportedParaphrase,
    DuplicateLanguage(String),
    MissingLanguageParity,
    UnknownSupportedParaphrase(String),
    ConflictingCandidateKinds,
    EvidenceDoesNotMatchProposal,
    InvalidCandidate(String),
    RegressionGateFailed,
    HumanApprovalRequired,
    UnknownCanonicalKind(String),
    DuplicateProposal(String),
    InvalidLedger(String),
}

impl fmt::Display for ProcedureLearningError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for ProcedureLearningError {}

fn candidate_identity(
    proposal_id: &str,
    lesson: &ProcedureCapabilityLesson,
    observations: &[ProcedureLearningObservation],
) -> String {
    let evidence = observations
        .iter()
        .map(|observation| {
            format!(
                "{}:{}=>{}",
                observation.language,
                observation.missing_surface.to_lowercase(),
                observation.supported_paraphrase.to_lowercase()
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    stable_id(
        "procedure_learning_candidate",
        &format!("{proposal_id}:{}:{evidence}", lesson.canonical_kind),
    )
}

fn validate_candidate_evidence(
    lesson: &ProcedureCapabilityLesson,
    observations: &[ProcedureLearningObservation],
) -> Result<(), ProcedureLearningError> {
    if observations.len() != lesson.surfaces.len() {
        return Err(ProcedureLearningError::InvalidCandidate(
            "candidate_evidence_surface_mismatch".to_owned(),
        ));
    }
    for surface in &lesson.surfaces {
        let observation = observations
            .iter()
            .find(|observation| observation.language == surface.language)
            .ok_or_else(|| {
                ProcedureLearningError::InvalidCandidate(format!(
                    "missing_evidence_language:{}",
                    surface.language
                ))
            })?;
        if observation.missing_surface.to_lowercase() != surface.text {
            return Err(ProcedureLearningError::InvalidCandidate(format!(
                "surface_evidence_mismatch:{}",
                surface.language
            )));
        }
        let found = super::first_procedure_match(
            &observation.supported_paraphrase.to_lowercase(),
            ROLE_SKILL_PROCEDURE_STEP_VERB,
        )
        .ok_or_else(|| {
            ProcedureLearningError::UnknownSupportedParaphrase(
                observation.supported_paraphrase.clone(),
            )
        })?;
        if found.slug != lesson.canonical_kind {
            return Err(ProcedureLearningError::ConflictingCandidateKinds);
        }
    }
    Ok(())
}

fn push_observation(out: &mut String, indent: usize, observation: &ProcedureLearningObservation) {
    push_lino_node(out, indent, "candidate_evidence", None);
    push_lino_node(out, indent + 2, "language", Some(&observation.language));
    push_lino_node(
        out,
        indent + 2,
        "missing_surface",
        Some(&observation.missing_surface),
    );
    push_lino_node(
        out,
        indent + 2,
        "supported_paraphrase",
        Some(&observation.supported_paraphrase),
    );
}

fn required_child(node: &LinoNode, name: &str) -> Result<String, ProcedureLearningError> {
    let value = node.find_child_value(name);
    if value.is_empty() {
        Err(ProcedureLearningError::InvalidLedger(format!(
            "missing_field:{name}:{}",
            node.name
        )))
    } else {
        Ok(value.to_owned())
    }
}

fn parse_usize(node: &LinoNode, name: &str) -> Result<usize, ProcedureLearningError> {
    required_child(node, name)?
        .parse()
        .map_err(|_| ProcedureLearningError::InvalidLedger(format!("invalid_field:{name}")))
}

pub(super) fn default_capability_ledger() -> &'static ProcedureCapabilityLedger {
    static LEDGER: OnceLock<ProcedureCapabilityLedger> = OnceLock::new();
    LEDGER.get_or_init(|| {
        ProcedureCapabilityLedger::from_links_notation(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/meta/procedure-capability-ledger.lino"
        )))
        .expect("embedded procedure capability ledger must validate")
    })
}
