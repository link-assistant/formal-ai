//! Internal final-result provenance; public plans retain their text projection.

use super::planner::AgenticPlan;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FinalDisposition {
    Finding,
    Clarification,
    Gap,
    Failure,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FinalResult {
    pub(super) text: String,
    pub(super) disposition: FinalDisposition,
    pub(super) origin: String,
}

pub(super) struct ResolvedPlan {
    pub(super) plan: AgenticPlan,
    pub(super) result: Option<FinalResult>,
}

impl ResolvedPlan {
    pub(super) fn new(plan: AgenticPlan, result: Option<FinalResult>) -> Self {
        let result = match &plan {
            AgenticPlan::ToolCalls(_) => None,
            AgenticPlan::Final(text) => Some(
                result
                    .filter(|result| result.text == *text)
                    .unwrap_or_else(|| FinalResult {
                        text: text.clone(),
                        disposition: FinalDisposition::Unknown,
                        origin: String::new(),
                    }),
            ),
        };
        Self { plan, result }
    }

    pub(super) fn can_deliver(&self) -> bool {
        self.result.as_ref().is_some_and(|result| {
            result.disposition == FinalDisposition::Finding && !result.origin.is_empty()
        })
    }

    pub(super) fn into_plan(self, result: &mut Option<FinalResult>) -> AgenticPlan {
        *result = self.result;
        self.plan
    }
}

/// Capture metadata only for the final plan actually selected by its route.
pub(super) fn record(
    plan: AgenticPlan,
    disposition: FinalDisposition,
    origin: &str,
    result: &mut Option<FinalResult>,
) -> AgenticPlan {
    if let AgenticPlan::Final(text) = &plan {
        *result = Some(FinalResult {
            text: text.clone(),
            disposition,
            origin: origin.to_owned(),
        });
    }
    plan
}

#[cfg(test)]
mod tests {
    use super::{FinalDisposition, FinalResult, ResolvedPlan};
    use crate::agentic_coding::AgenticPlan;

    #[test]
    fn certificate_belongs_only_to_selected_final_bytes() {
        let observed = FinalResult {
            text: "observed bytes".to_owned(),
            disposition: FinalDisposition::Finding,
            origin: "fixture_observed".to_owned(),
        };
        let selected = ResolvedPlan::new(
            AgenticPlan::Final(observed.text.clone()),
            Some(observed.clone()),
        );
        assert!(selected.can_deliver());
        let other = ResolvedPlan::new(
            AgenticPlan::Final("different bytes".to_owned()),
            Some(observed.clone()),
        );
        assert!(!other.can_deliver());
        assert_eq!(
            other.result.as_ref().map(|result| result.disposition),
            Some(FinalDisposition::Unknown)
        );
        let tool = ResolvedPlan::new(AgenticPlan::ToolCalls(Vec::new()), Some(observed));
        assert!(tool.result.is_none());
    }
}
