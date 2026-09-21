//! Issue #707 acceptance: held-out computer-use requests must be *planned*, not
//! recalled.
//!
//! `data/benchmarks/computer-use-generalization.lino` holds twelve requests in
//! four languages that appear nowhere in the recorded corpus. Each one must be
//! refused by the recorded-plan lookup, produced by the schemas auto-learned
//! from that corpus, agree across all four languages on a single plan identity,
//! and execute with every precondition, effect, and postcondition verified.

use std::collections::BTreeSet;
use std::fs;

use formal_ai::computer_use::{
    capability_gap_for_request, plan_for_prompt, plan_request, run_verified_plan, synthesize,
};

const SUITE: &str = "data/benchmarks/computer-use-generalization.lino";

struct Case {
    id: String,
    resource: String,
    operations: Vec<String>,
    prompts: Vec<(String, String)>,
}

fn suite() -> Vec<Case> {
    let text = fs::read_to_string(SUITE).expect("generalization suite");
    let mut cases: Vec<Case> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(id) = trimmed.strip_prefix("case ") {
            cases.push(Case {
                id: id.to_owned(),
                resource: String::new(),
                operations: Vec::new(),
                prompts: Vec::new(),
            });
            continue;
        }
        let Some(case) = cases.last_mut() else {
            continue;
        };
        if let Some(resource) = trimmed.strip_prefix("resource ") {
            resource.clone_into(&mut case.resource);
        } else if let Some(operation) = trimmed.strip_prefix("operation ") {
            case.operations.push(operation.to_owned());
        } else if let Some(rest) = trimmed.strip_prefix("prompt ") {
            let (locale, quoted) = rest.split_once(' ').expect("prompt locale");
            let prompt = quoted.trim().trim_matches('"').to_owned();
            case.prompts.push((locale.to_owned(), prompt));
        }
    }
    cases
}

fn minimum_cases() -> usize {
    let text = fs::read_to_string(SUITE).expect("generalization suite");
    text.lines()
        .find_map(|line| line.trim().strip_prefix("minimum_cases "))
        .map(|value| value.trim().trim_matches('"').to_owned())
        .expect("minimum_cases")
        .parse()
        .expect("numeric minimum_cases")
}

#[test]
fn the_suite_holds_at_least_the_ratcheted_number_of_four_language_cases() {
    let cases = suite();
    assert!(
        cases.len() >= minimum_cases(),
        "suite shrank below its ratchet: {} < {}",
        cases.len(),
        minimum_cases()
    );
    for case in &cases {
        assert_eq!(case.prompts.len(), 4, "{}", case.id);
        assert!(!case.resource.is_empty(), "{}", case.id);
        assert!(!case.operations.is_empty(), "{}", case.id);
    }
}

#[test]
fn every_held_out_prompt_is_absent_from_the_recorded_corpus() {
    for case in suite() {
        for (locale, prompt) in &case.prompts {
            assert!(
                plan_for_prompt(prompt).is_none(),
                "{} [{locale}] is recorded, so it proves nothing about generalization",
                case.id
            );
        }
    }
}

#[test]
fn every_held_out_request_synthesizes_the_expected_resource_and_operations() {
    for case in suite() {
        for (locale, prompt) in &case.prompts {
            let synthesis = synthesize(prompt)
                .unwrap_or_else(|| panic!("{} [{locale}] produced no plan: {prompt}", case.id));
            assert_eq!(synthesis.resource, case.resource, "{} [{locale}]", case.id);
            assert_eq!(
                synthesis.operations, case.operations,
                "{} [{locale}]",
                case.id
            );
        }
    }
}

#[test]
fn the_four_languages_of_a_case_agree_on_one_plan() {
    for case in suite() {
        let identities = case
            .prompts
            .iter()
            .map(|(_, prompt)| {
                let plan = plan_request(prompt).expect("plan");
                (plan.id, plan.steps)
            })
            .collect::<Vec<_>>();
        let (first_id, first_steps) = &identities[0];
        for (id, steps) in &identities[1..] {
            assert_eq!(id, first_id, "{}", case.id);
            assert_eq!(steps, first_steps, "{}", case.id);
        }
    }
}

#[test]
fn every_synthesized_plan_executes_with_every_step_verified() {
    let mut workspaces = BTreeSet::new();
    for case in suite() {
        for (locale, prompt) in &case.prompts {
            let outcome = run_verified_plan(prompt)
                .unwrap_or_else(|error| panic!("{} [{locale}]: {error:?}", case.id));
            assert!(
                outcome.plan.id.starts_with("synthesized-"),
                "{} [{locale}] fell back to a recorded plan",
                case.id
            );
            assert!(outcome.verified, "{} [{locale}]", case.id);
            assert!(
                outcome.steps.len() >= 3,
                "{} [{locale}] has no materialisation, body, and verification",
                case.id
            );
            for step in &outcome.steps {
                assert!(step.verified, "{} [{locale}] {}", case.id, step.step_id);
                assert!(!step.events.is_empty(), "{} [{locale}]", case.id);
                assert!(
                    step.events.iter().all(|event| event.passed),
                    "{} [{locale}] {}",
                    case.id,
                    step.step_id
                );
            }
            workspaces.insert(outcome.workspace);
        }
    }
    assert_eq!(
        workspaces.len(),
        suite().len() * 4,
        "each run must get its own isolated workspace"
    );
}

#[test]
fn out_of_boundary_requests_are_refused_honestly_in_all_four_languages() {
    for (locale, prompt) in [
        (
            "en",
            "Take a screenshot of the rendered customers dashboard",
        ),
        ("ru", "Сделай снимок отрисованной страницы с клиентами"),
        ("hi", "ग्राहकों के rendered page का screenshot लो"),
        ("zh", "截图渲染后的客户页面"),
    ] {
        assert!(
            plan_request(prompt).is_none(),
            "[{locale}] invented a plan for a rendering request"
        );
        let gap = capability_gap_for_request(prompt)
            .unwrap_or_else(|| panic!("[{locale}] gave no named capability gap"));
        assert_eq!(gap.capability, "gui_rendering", "[{locale}]");
        assert_eq!(gap.locale, locale);
        assert!(gap.response.contains("capability_gap"), "[{locale}]");
    }
}

/// A request that *transports* content must not be planned from words inside
/// that content. `order "90"` in a Links Notation payload is data being
/// written, not a request about the orders resource, and `list_files_arg`
/// inside a quoted literal names nothing the speaker asked to list.
#[test]
fn literal_payload_inside_a_request_is_not_read_as_a_computer_use_plan() {
    let task = "Create file data/seed/learned-program-rules.lino containing\n\
                substitution_rules\n  id \"learned_program_plan_rules\"\n  \
                rule \"learned_reverse\"\n    order \"90\"\n    \
                replace \"request:task -> list_files_arg\"\n";
    assert!(
        formal_ai::computer_use::plan_request(task).is_none(),
        "payload tokens must not synthesize a plan"
    );
    assert!(
        formal_ai::computer_use::capability_gap_for_request(task).is_none(),
        "a file-authoring request is not a computer-use capability gap"
    );
}
