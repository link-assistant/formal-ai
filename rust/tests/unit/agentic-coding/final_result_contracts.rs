// Compile the actual private provenance module inside this test-only bridge.
mod planner {
    pub use formal_ai::agentic_coding::AgenticPlan;
}

#[allow(dead_code)]
#[path = "../../../src/agentic_coding/final_result.rs"]
mod final_result;

#[path = "../../fixtures/final-result-provenance.rs"]
mod assertions;
