//! Shared types for the universal proof engine.
//!
//! The engine never returns a flat "I cannot do this" — it always produces a
//! [`Proof`] (proven or disproven) or a [`ProofOutcome::PartialPlan`] that
//! lists the axioms / definitions the user still needs to supply. This keeps
//! the same code path responsible for honest "we tried" output and stops
//! proof-shaped prompts from collapsing back to the unknown-intent fallback.

/// The classical proof techniques the engine can label a discharged proof
/// with. Used purely for presentation (so users see *how* the proof was
/// produced) and for the structured Links Notation trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofMethod {
    /// The two sides reduce to identical numeric / symbolic values.
    DirectCalculation,
    /// Assume the negation, derive a contradiction.
    Contradiction,
    /// Base case + inductive step.
    Induction,
    /// Construct a witness object.
    Construction,
    /// Show the contrapositive instead of the direct implication.
    Contrapositive,
    /// Reduce to a finite case split, decide each case.
    Cases,
    /// Restate a classical / textbook theorem with its standard proof.
    KnownTheorem,
    /// Use a known equivalent formal system as the meta-context (e.g.
    /// Gödel/PA, ZFC, Newtonian mechanics).
    AxiomReduction,
    /// Enumerate every assignment / configuration over a finite domain.
    Tautology,
    /// Delegate to the relative-meta-logic / SMT-style decision procedure.
    DecisionProcedure,
}

impl ProofMethod {
    /// Short English slug used in the Links Notation trace and the diagnostic
    /// banner.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::DirectCalculation => "direct_calculation",
            Self::Contradiction => "contradiction",
            Self::Induction => "induction",
            Self::Construction => "construction",
            Self::Contrapositive => "contrapositive",
            Self::Cases => "cases",
            Self::KnownTheorem => "known_theorem",
            Self::AxiomReduction => "axiom_reduction",
            Self::Tautology => "tautology",
            Self::DecisionProcedure => "decision_procedure",
        }
    }

    /// User-facing label localized to the requested language.
    #[must_use]
    pub fn label(self, language: &str) -> &'static str {
        match (self, language) {
            (Self::DirectCalculation, "ru") => "прямое вычисление",
            (Self::DirectCalculation, "zh") => "直接计算",
            (Self::DirectCalculation, "hi") => "प्रत्यक्ष गणना",
            (Self::DirectCalculation, _) => "direct calculation",
            (Self::Contradiction, "ru") => "от противного",
            (Self::Contradiction, "zh") => "反证法",
            (Self::Contradiction, "hi") => "अंतर्विरोध द्वारा",
            (Self::Contradiction, _) => "proof by contradiction",
            (Self::Induction, "ru") => "по индукции",
            (Self::Induction, "zh") => "数学归纳法",
            (Self::Induction, "hi") => "गणितीय आगमन",
            (Self::Induction, _) => "mathematical induction",
            (Self::Construction, "ru") => "конструктивно",
            (Self::Construction, "zh") => "构造法",
            (Self::Construction, "hi") => "रचनात्मक प्रमाण",
            (Self::Construction, _) => "constructive proof",
            (Self::Contrapositive, "ru") => "от противоположного",
            (Self::Contrapositive, "zh") => "逆否命题",
            (Self::Contrapositive, "hi") => "विपरीतधर्मी",
            (Self::Contrapositive, _) => "contrapositive",
            (Self::Cases, "ru") => "разбор случаев",
            (Self::Cases, "zh") => "分情况讨论",
            (Self::Cases, "hi") => "मामलों का विश्लेषण",
            (Self::Cases, _) => "case analysis",
            (Self::KnownTheorem, "ru") => "известная теорема",
            (Self::KnownTheorem, "zh") => "已知定理",
            (Self::KnownTheorem, "hi") => "ज्ञात प्रमेय",
            (Self::KnownTheorem, _) => "known theorem",
            (Self::AxiomReduction, "ru") => "сведение к аксиоматике",
            (Self::AxiomReduction, "zh") => "公理化归约",
            (Self::AxiomReduction, "hi") => "अभिगृहीतों में निरूपण",
            (Self::AxiomReduction, _) => "axiom-set reduction",
            (Self::Tautology, "ru") => "тавтология",
            (Self::Tautology, "zh") => "重言式",
            (Self::Tautology, "hi") => "तथ्यात्मक",
            (Self::Tautology, _) => "tautology check",
            (Self::DecisionProcedure, "ru") => "процедура разрешения relative-meta-logic / SMT",
            (Self::DecisionProcedure, "zh") => "relative-meta-logic / SMT 判定过程",
            (Self::DecisionProcedure, "hi") => "relative-meta-logic / SMT निर्णय प्रक्रिया",
            (Self::DecisionProcedure, _) => "relative-meta-logic / SMT decision procedure",
        }
    }
}

/// Classification of a single line in a presented proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepKind {
    Hypothesis,
    Definition,
    Axiom,
    Inference,
    SubProof,
    Conclusion,
}

impl StepKind {
    /// Localized label printed in front of the step.
    #[must_use]
    pub fn label(self, language: &str) -> &'static str {
        match (self, language) {
            (Self::Hypothesis, "ru") => "Гипотеза",
            (Self::Hypothesis, "zh") => "前提",
            (Self::Hypothesis, "hi") => "परिकल्पना",
            (Self::Hypothesis, _) => "Hypothesis",
            (Self::Definition, "ru") => "Определение",
            (Self::Definition, "zh") => "定义",
            (Self::Definition, "hi") => "परिभाषा",
            (Self::Definition, _) => "Definition",
            (Self::Axiom, "ru") => "Аксиома",
            (Self::Axiom, "zh") => "公理",
            (Self::Axiom, "hi") => "अभिगृहीत",
            (Self::Axiom, _) => "Axiom",
            (Self::Inference, "ru") => "Вывод",
            (Self::Inference, "zh") => "推理",
            (Self::Inference, "hi") => "निष्कर्षण",
            (Self::Inference, _) => "Inference",
            (Self::SubProof, "ru") => "Подкаравасьное",
            (Self::SubProof, "zh") => "子证明",
            (Self::SubProof, "hi") => "उप-प्रमाण",
            (Self::SubProof, _) => "Sub-proof",
            (Self::Conclusion, "ru") => "Заключение",
            (Self::Conclusion, "zh") => "结论",
            (Self::Conclusion, "hi") => "निष्कर्ष",
            (Self::Conclusion, _) => "Conclusion",
        }
    }
}

/// A single line in a rendered proof. The text is already user-ready; the
/// kind controls how it's prefixed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofStep {
    pub kind: StepKind,
    pub text: String,
}

/// A complete (discharged) proof — the statement that was proven, every
/// intermediate step, and a closing conclusion that ends with a "∎" mark in
/// English (and the equivalent in other languages).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proof {
    pub statement: String,
    pub steps: Vec<ProofStep>,
    pub conclusion: String,
    pub method: ProofMethod,
}

/// What the proof engine produced for a given claim. Every variant carries
/// enough information that the surface presenter can format an honest reply
/// instead of falling back to a refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofOutcome {
    /// The claim is true; here is the proof.
    Proven { proof: Proof },
    /// The claim is false; here is the counterexample.
    Disproven {
        counterexample: String,
        method: ProofMethod,
        /// The discharge of `¬claim` itself, when the engine can write it
        /// out (e.g. arithmetic).
        partial_proof: Option<Proof>,
    },
    /// The engine has a proof plan but is blocked on a missing input (the
    /// user has not yet specified an axiom set, a definition, or a context).
    /// The plan is still surfaced verbatim so the user sees real progress.
    PartialPlan {
        plan: Vec<ProofStep>,
        missing_inputs: Vec<String>,
        method: ProofMethod,
    },
    /// The claim looks ill-formed or the engine cannot extract a checkable
    /// proposition from it. Still better than a flat refusal because we tell
    /// the user *why* we cannot continue.
    Inconclusive { reason: String },
}

impl ProofOutcome {
    /// English slug for the Links Notation trace event.
    #[must_use]
    pub const fn status_slug(&self) -> &'static str {
        match self {
            Self::Proven { .. } => "proven",
            Self::Disproven { .. } => "disproven",
            Self::PartialPlan { .. } => "partial_plan",
            Self::Inconclusive { .. } => "inconclusive",
        }
    }

    /// Method label for the trace (where applicable).
    #[must_use]
    pub const fn method(&self) -> Option<ProofMethod> {
        match self {
            Self::Proven { proof } => Some(proof.method),
            Self::Disproven { method, .. } | Self::PartialPlan { method, .. } => Some(*method),
            Self::Inconclusive { .. } => None,
        }
    }
}

/// Configuration the proof engine reads to decide *how* to present a proof.
///
/// The two sliders mirror the JS front-end (`src/web/app.js`) and the surface
/// [`crate::solver::SolverConfig`]:
///
/// * `guess_probability` (0.0..1.0): high values mean "be confident, show how
///   you interpreted the prompt, translate it into the formal system, and
///   execute the proof". Low values mean "stay literal, only execute what is
///   unambiguous".
/// * `follow_up_probability` (0.0..1.0): high values mean "after presenting
///   what you have, ask the user the questions you still need answered so the
///   final research execution is unambiguous". Low values keep the response
///   action-only.
///
/// The two sliders are independent: setting both high produces an
/// interpretation header *and* a clarifying-questions footer. Setting both low
/// reduces the response to just the proof body.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProofRenderConfig {
    pub guess_probability: f32,
    pub follow_up_probability: f32,
}

impl Default for ProofRenderConfig {
    fn default() -> Self {
        Self {
            guess_probability: 0.8,
            follow_up_probability: 0.75,
        }
    }
}

impl ProofRenderConfig {
    /// True when the configuration asks the engine to surface how it
    /// interpreted the prompt (translation to the formal system).
    #[must_use]
    pub fn show_interpretation(self) -> bool {
        self.guess_probability >= 0.6
    }

    /// True when the configuration asks the engine to add follow-up
    /// clarification questions instead of (or in addition to) executing.
    #[must_use]
    pub fn ask_follow_ups(self) -> bool {
        self.follow_up_probability >= 0.5
    }

    /// True when the configuration explicitly asks the engine to be terse
    /// (low guess *and* low follow-up). Used to drop the deep-reasoning block
    /// from the rendered proof.
    #[must_use]
    pub fn is_terse(self) -> bool {
        self.guess_probability < 0.4 && self.follow_up_probability < 0.5
    }
}
