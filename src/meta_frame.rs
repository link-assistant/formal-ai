//! Issue #559 Phase 1A: an explicit, link-serializable problem frame.
//!
//! The universal solver already formalizes every prompt into an
//! [`IntentFormalization`](crate::intent_formalization::IntentFormalization)
//! meaning record (root requirement R157). Issue #559 generalizes the solver
//! away from one prompt → one handler intent toward one prompt → a frame of
//! *every* detected need. This module makes that frame first-class and
//! link-native, without changing routing or answers: a [`ProblemFrame`] wraps
//! the formalization, enumerates each [`Need`] found in the prompt (R7), and is
//! serialized to Links Notation via
//! [`format_lino_record`](crate::links_format::format_lino_record) (R311). It is
//! emitted as a solver loop event so the meaning record is observable, but the
//! existing dispatch still decides the answer. Later phases build the recursive
//! `WorkUnit` trace, the need-satisfaction ledger, and the method registry on
//! top of this frame.
//!
//! Vocabulary follows `docs/case-studies/issue-559/alignment.md`: `ProblemFrame`
//! = the formalized impulse made explicit; `Need` = a detected requirement or
//! question (root requirement R158); the frame is the data the recursive core
//! decomposes (see `docs/case-studies/issue-559/recursive-core.md`).

use crate::engine::stable_id;
use crate::event_log::EventLog;
use crate::intent_formalization::{formalize_intent, IntentFormalization, IntentKind};
use crate::links_format::format_lino_record;

/// Lifecycle status of a single detected [`Need`].
///
/// Phase 1A records every need as [`NeedStatus::Pending`]: the frame is a
/// trace-only projection that does not yet resolve needs. Phase 2 introduces the
/// need-satisfaction ledger (root requirement R333) that flips each status to
/// `Satisfied`, `Deferred`, `Blocked`, or `Rejected` from the answer projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeedStatus {
    /// Detected but not yet resolved (the only status produced in Phase 1A).
    Pending,
    /// A work unit produced a validated result for this need.
    Satisfied,
    /// Intentionally postponed (e.g. out of scope this turn).
    Deferred,
    /// No method or evidence is available; recorded rather than hidden.
    Blocked,
    /// Deliberately not done, with a reason.
    Rejected,
}

impl NeedStatus {
    /// Stable slug used in the Links Notation trace.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Satisfied => "satisfied",
            Self::Deferred => "deferred",
            Self::Blocked => "blocked",
            Self::Rejected => "rejected",
        }
    }
}

/// A single question, requirement, task, or statement detected inside a prompt.
///
/// A `Need` reuses the canonical [`IntentKind`] classification rather than
/// introducing a parallel taxonomy, so a need is always one of the intent kinds
/// the engine already understands. The `source_span` is the exact substring of
/// the prompt the need was detected in, preserving provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Need {
    /// Content-addressed identifier, stable for a given frame and span.
    pub need_id: String,
    /// The substring of the prompt this need was detected in.
    pub source_span: String,
    /// The canonical intent kind classifying this need.
    pub kind: IntentKind,
    /// The routing slug the span formalizes to, when one is recognized.
    pub route: Option<String>,
    /// Lifecycle status (always [`NeedStatus::Pending`] in Phase 1A).
    pub status: NeedStatus,
}

impl Need {
    #[must_use]
    fn to_links_notation(&self) -> String {
        let mut pairs: Vec<(&str, String)> = vec![
            ("record_type", "problem_need".to_owned()),
            ("need_id", self.need_id.clone()),
            ("source_span", self.source_span.clone()),
            ("kind", self.kind.slug().to_owned()),
            ("status", self.status.slug().to_owned()),
        ];
        if let Some(route) = &self.route {
            pairs.push(("route", route.clone()));
        }
        format_lino_record(&self.need_id, &pairs)
    }
}

/// The explicit meaning record for one prompt: the formalized impulse plus every
/// detected [`Need`].
///
/// A frame is a projection of the existing [`IntentFormalization`]; it never
/// changes routing in Phase 1A. It exists so the solver loop can emit a single,
/// link-serializable object that names what the user asked for in full, which is
/// the foundation the recursive core and the need ledger build on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProblemFrame {
    /// Content-addressed identifier for the frame.
    pub frame_id: String,
    /// The impulse this frame formalizes.
    pub impulse_id: String,
    /// Detected language slug.
    pub language: String,
    /// The whole-prompt intent kind (the frame's primary classification).
    pub kind: IntentKind,
    /// The whole-prompt routing slug, when recognized.
    pub route: Option<String>,
    /// Every need detected in the prompt, in source order.
    pub needs: Vec<Need>,
}

impl ProblemFrame {
    /// Build a frame from an already-computed [`IntentFormalization`].
    ///
    /// This reuses the canonical formalization for the whole-prompt
    /// classification and, when the prompt contains more than one segment,
    /// re-runs the same [`formalize_intent`] classifier on each segment so a
    /// multi-part prompt yields multiple typed needs (R7). When the prompt is a
    /// single segment, the frame carries exactly one need that mirrors the
    /// frame's own classification, so single-intent prompts behave identically
    /// to today (R13).
    #[must_use]
    pub fn from_formalization(formalization: &IntentFormalization) -> Self {
        let frame_id = stable_id("problem_frame", &formalization.impulse_id);
        let segments = segment_needs(&formalization.source_text);
        let needs = if segments.len() <= 1 {
            vec![Need {
                need_id: need_id_for(&frame_id, 0, &formalization.source_text),
                source_span: formalization.source_text.clone(),
                kind: formalization.kind,
                route: formalization.route.clone(),
                status: NeedStatus::Pending,
            }]
        } else {
            segments
                .iter()
                .enumerate()
                .map(|(index, span)| {
                    let segment = formalize_intent(span, &formalization.language, None);
                    Need {
                        need_id: need_id_for(&frame_id, index, span),
                        source_span: span.clone(),
                        kind: segment.kind,
                        route: segment.route,
                        status: NeedStatus::Pending,
                    }
                })
                .collect()
        };

        Self {
            frame_id,
            impulse_id: formalization.impulse_id.clone(),
            language: formalization.language.clone(),
            kind: formalization.kind,
            route: formalization.route.clone(),
            needs,
        }
    }

    /// Number of detected needs.
    #[must_use]
    pub const fn need_count(&self) -> usize {
        self.needs.len()
    }

    /// Render the frame and its needs as Links Notation records.
    ///
    /// The frame record references each need by id; every need is its own
    /// record, mirroring the seed's one-record-per-concept convention.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut pairs: Vec<(&str, String)> = vec![
            ("record_type", "problem_frame".to_owned()),
            ("frame_id", self.frame_id.clone()),
            ("impulse_id", self.impulse_id.clone()),
            ("language", self.language.clone()),
            ("kind", self.kind.slug().to_owned()),
            ("need_count", self.needs.len().to_string()),
        ];
        if let Some(route) = &self.route {
            pairs.push(("route", route.clone()));
        }
        for need in &self.needs {
            pairs.push(("need", need.need_id.clone()));
        }
        let mut out = format_lino_record(&self.frame_id, &pairs);
        for need in &self.needs {
            out.push('\n');
            out.push_str(&need.to_links_notation());
        }
        out
    }
}

#[must_use]
fn need_id_for(frame_id: &str, index: usize, span: &str) -> String {
    stable_id("problem_need", &format!("{frame_id}:{index}:{span}"))
}

/// Split a prompt into the spans that each carry one need.
///
/// Segmentation is deliberately structural and language-neutral: it splits on
/// sentence terminators (`?`, `!`, `.`, and their CJK forms) and then on the
/// same coordinating triggers the existing decomposition step uses
/// (`record_decomposition` in `src/solver_helpers.rs`). It introduces no new
/// per-language phrase list, so it does not regress the no-hardcoded-natural-
/// language discipline. A period flanked by non-whitespace (e.g. `3.14`) is kept
/// inside its segment so decimals are never broken apart.
#[must_use]
fn segment_needs(text: &str) -> Vec<String> {
    let mut segments = Vec::new();
    for sentence in split_sentences(text) {
        for clause in split_clauses(&sentence) {
            let trimmed = clause.trim();
            if !trimmed.is_empty() {
                segments.push(trimmed.to_owned());
            }
        }
    }
    segments
}

/// Split on sentence terminators, keeping the terminator attached so a trailing
/// `?` is still visible to the question classifier.
#[must_use]
fn split_sentences(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut sentences = Vec::new();
    let mut current = String::new();
    for (index, &ch) in chars.iter().enumerate() {
        current.push(ch);
        let strong_terminator = matches!(ch, '?' | '!' | '。' | '！' | '？');
        let period_boundary =
            ch == '.' && chars.get(index + 1).is_none_or(|next| next.is_whitespace());
        if strong_terminator || period_boundary {
            push_trimmed(&mut sentences, &current);
            current.clear();
        }
    }
    push_trimmed(&mut sentences, &current);
    sentences
}

/// Split a sentence on coordinating triggers, mirroring the existing shallow
/// decomposition so the frame agrees with `record_decomposition`.
#[must_use]
fn split_clauses(sentence: &str) -> Vec<String> {
    sentence
        .split([',', ';'])
        .flat_map(|chunk| chunk.split(" and "))
        .flat_map(|chunk| chunk.split(" with "))
        .map(|chunk| chunk.trim().to_owned())
        .filter(|chunk| !chunk.is_empty())
        .collect()
}

fn push_trimmed(out: &mut Vec<String>, candidate: &str) {
    let trimmed = candidate.trim();
    if !trimmed.is_empty() {
        out.push(trimmed.to_owned());
    }
}

/// Build a [`ProblemFrame`] from the formalization and emit it as a loop event
/// plus its Links Notation trace.
///
/// This is trace-only: it appends a `problem_frame` event (the serialized frame)
/// and per-need `problem_frame:need` events, so the meaning record is observable
/// in the event log without altering routing or the final answer (R13).
pub(crate) fn record_problem_frame(
    log: &mut EventLog,
    formalization: &IntentFormalization,
) -> ProblemFrame {
    let frame = ProblemFrame::from_formalization(formalization);
    log.append("problem_frame", frame.to_links_notation());
    log.append("problem_frame:need_count", frame.needs.len().to_string());
    for need in &frame.needs {
        log.append(
            "problem_frame:need",
            format!("{} {}", need.kind.slug(), need.source_span),
        );
    }
    frame
}
