//! Issue #1168 R1168-8: the planner's workflow pins agree across runtimes.
//!
//! The agentic planner attaches a CI workflow to a program-contract recipe,
//! and that workflow is where every generated-version pin lands. Its version
//! source is injected (`render_ci_workflow` over a `VersionSet`), so this lane
//! renders it from the offline shipped baseline -- no network, no source
//! cache -- and compares the text with the fixture
//! `rust/tests/web/issue-1168-workflow-render-parity.test.mjs` asserts against
//! `js/agentic/ci_workflow.mjs`.

use formal_ai::agentic_coding::render_ci_workflow;
use formal_ai::event_log::EventLog;
use formal_ai::program_contract;
use formal_ai::version_resolution::VersionSet;

const FIXTURE: &str = include_str!("../fixtures/issue-1168/workflow-render-parity.json");

fn field<'a>(case: &'a serde_json::Value, key: &str) -> &'a str {
    case.get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("every parity case carries `{key}`"))
}

#[test]
fn the_planner_workflow_renders_the_fixture_pins_from_the_injected_baseline() {
    let fixture: serde_json::Value =
        serde_json::from_str(FIXTURE).expect("the parity fixture is JSON");
    let cases = fixture
        .get("cases")
        .and_then(serde_json::Value::as_array)
        .expect("the parity fixture lists cases");
    assert_eq!(cases.len(), 4);
    let versions = VersionSet::baseline().expect("the toolchains seed carries every baseline");
    for case in cases {
        let id = field(case, "id");
        let answer = program_contract::answer(field(case, "prompt"), &mut EventLog::default())
            .unwrap_or_else(|| panic!("{id}: the program contract answers"));
        let recipe = answer
            .execution_recipe
            .unwrap_or_else(|| panic!("{id}: the answer carries a recipe"));
        assert_eq!(recipe.path, field(case, "path"), "{id}");
        assert_eq!(
            render_ci_workflow(&recipe, &versions),
            field(case, "workflow"),
            "{id}"
        );
    }
}
