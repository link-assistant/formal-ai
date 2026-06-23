//! Issue #559 (R334): the evidence pipeline — the unifying audit projection.
//!
//! The meta core records several link artifacts as it works: the problem frame
//! (every detected need), the recursive work-unit tree (the decomposition), the
//! need-satisfaction ledger (one status per need), and the method registry (the
//! catalogue of resolving handlers). On their own each answers a different
//! question. The evidence pipeline *joins* them into one coherent record: for
//! every need it traces the full chain `frame need → work-unit leaf → ledger
//! status → catalogued method`, so "ensure every detected need is addressed"
//! becomes an end-to-end auditable fact rather than four separate projections a
//! reader must reconcile by hand.
//!
//! This is a behavior-preserving projection: it reads the artifacts the loop
//! already produced and emits a trace-only `solution_evidence` event. It changes
//! neither routing nor the answer (R13). Runtime outcome feedback — flipping a
//! satisfied prediction back to blocked when a leaf fails to validate — is layered
//! on in a later phase; this phase pins the *static* chain so the wiring exists.

use crate::event_log::EventLog;
use crate::links_format::format_lino_record;
use crate::meta_frame::{NeedLedger, NeedStatus, ProblemFrame};
use crate::method_registry::MethodRegistry;

/// One need's full evidence chain through the meta core.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceTrail {
    /// The need this trail accounts for.
    pub need_id: String,
    /// The need's source span (provenance).
    pub source_span: String,
    /// The work-unit leaf that resolves the need, when the decomposition reached
    /// one — the link back to the recursive downward pass.
    pub work_unit_id: Option<String>,
    /// The need's resolved status from the ledger.
    pub status: NeedStatus,
    /// The route the resolving leaf carries, when matched.
    pub route: Option<String>,
    /// The catalogued method this trail resolves to, when the route names a
    /// registered handler — the link to the method registry.
    pub method: Option<String>,
    /// Whether the chain is connected end to end: a leaf was reached and the need
    /// carries an explicit, non-pending status.
    pub connected: bool,
}

impl EvidenceTrail {
    #[must_use]
    fn to_links_notation(&self) -> String {
        let mut pairs: Vec<(&str, String)> = vec![
            ("record_type", "evidence_trail".to_owned()),
            ("need_id", self.need_id.clone()),
            ("source_span", self.source_span.clone()),
            ("status", self.status.slug().to_owned()),
            ("connected", self.connected.to_string()),
        ];
        if let Some(unit_id) = &self.work_unit_id {
            pairs.push(("work_unit", unit_id.clone()));
        }
        if let Some(route) = &self.route {
            pairs.push(("route", route.clone()));
        }
        if let Some(method) = &self.method {
            pairs.push(("method", method.clone()));
        }
        format_lino_record(&self.need_id, &pairs)
    }
}

/// The end-to-end evidence that the solve addressed every detected need.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolutionEvidence {
    /// The frame this evidence audits.
    pub frame_id: String,
    /// One trail per detected need, in frame order.
    pub trails: Vec<EvidenceTrail>,
}

impl SolutionEvidence {
    /// Assemble the evidence by joining the frame's needs (via the ledger) to the
    /// work-unit leaves that resolve them and the catalogued methods they route
    /// to.
    #[must_use]
    pub fn assemble(frame: &ProblemFrame, ledger: &NeedLedger, registry: &MethodRegistry) -> Self {
        let trails = ledger
            .rows
            .iter()
            .map(|row| {
                let method = row.route.as_ref().and_then(|route| {
                    registry
                        .methods
                        .iter()
                        .find(|method| &method.name == route)
                        .map(|method| method.name.clone())
                });
                let connected = row.unit_id.is_some() && row.status != NeedStatus::Pending;
                EvidenceTrail {
                    need_id: row.need_id.clone(),
                    source_span: row.source_span.clone(),
                    work_unit_id: row.unit_id.clone(),
                    status: row.status,
                    route: row.route.clone(),
                    method,
                    connected,
                }
            })
            .collect();
        Self {
            frame_id: frame.frame_id.clone(),
            trails,
        }
    }

    /// Every need has a connected chain and a non-pending status — the structural
    /// form of "every detected need is accounted for in the response".
    #[must_use]
    pub fn accounted_for(&self) -> bool {
        !self.trails.is_empty() && self.trails.iter().all(|trail| trail.connected)
    }

    /// Every need is `Satisfied` by a connected chain — a stronger guarantee than
    /// merely accounted-for (which also counts an explicitly blocked need).
    #[must_use]
    pub fn fully_resolved(&self) -> bool {
        !self.trails.is_empty()
            && self
                .trails
                .iter()
                .all(|trail| trail.connected && trail.status == NeedStatus::Satisfied)
    }

    /// Number of trails that resolve to a catalogued method.
    #[must_use]
    pub fn resolved_to_method(&self) -> usize {
        self.trails
            .iter()
            .filter(|trail| trail.method.is_some())
            .count()
    }

    /// Render the evidence and every trail as Links Notation records (R311).
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut pairs: Vec<(&str, String)> = vec![
            ("record_type", "solution_evidence".to_owned()),
            ("frame_id", self.frame_id.clone()),
            ("trail_count", self.trails.len().to_string()),
            ("accounted_for", self.accounted_for().to_string()),
            ("fully_resolved", self.fully_resolved().to_string()),
            ("resolved_to_method", self.resolved_to_method().to_string()),
        ];
        for trail in &self.trails {
            pairs.push(("trail", trail.need_id.clone()));
        }
        let mut out = format_lino_record("solution_evidence", &pairs);
        for trail in &self.trails {
            out.push('\n');
            out.push_str(&trail.to_links_notation());
        }
        out
    }
}

/// Assemble the solution evidence and emit it as a loop event plus its Links
/// Notation trace.
///
/// Trace-only (R334): it appends one `solution_evidence` event (the serialized
/// chain, which enumerates every trail) and a compact
/// `solution_evidence:accounted_for`, so the end-to-end audit is observable in
/// the event log without changing routing or the answer (R13).
pub(crate) fn record_solution_evidence(
    log: &mut EventLog,
    frame: &ProblemFrame,
    ledger: &NeedLedger,
    registry: &MethodRegistry,
) -> SolutionEvidence {
    let evidence = SolutionEvidence::assemble(frame, ledger, registry);
    log.append("solution_evidence", evidence.to_links_notation());
    log.append(
        "solution_evidence:accounted_for",
        evidence.accounted_for().to_string(),
    );
    evidence
}
