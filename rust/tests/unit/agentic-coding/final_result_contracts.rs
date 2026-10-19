// Compile the actual private provenance module inside this test-only bridge.
mod planner {
    pub use formal_ai::agentic_coding::{AgenticPlan, planner::trace_route_with};
}

#[allow(dead_code)]
#[path = "../../../src/agentic_coding/final_result.rs"]
mod final_result;

#[path = "../../fixtures/final-result-provenance.rs"]
mod assertions;

#[test]
fn route_evidence_is_produced_only_when_request_tracing_is_enabled() {
    for (flag, expected_calls) in [("false", 0), ("true", 1)] {
        temp_env::with_var("FORMAL_AI_TRACE_REQUESTS", Some(flag), || {
            let calls = std::cell::Cell::new(0);
            formal_ai::agentic_coding::planner::trace_route_with("empty-test-evidence", || {
                calls.set(calls.get() + 1);
                None::<&str>
            });
            assert_eq!(calls.get(), expected_calls);
        });
    }
}
