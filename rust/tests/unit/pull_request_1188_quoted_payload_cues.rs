//! PR #1188 dogfooding: the single-quoted payloads of a `Replace 'X' with 'Y'`
//! edit request are data, not computer-use cues.
//!
//! A replacement naming a precedence "order" and a built "list" was planned as
//! a computer-use order listing (plan
//! `synthesized-computer_use_resource_orders-computer_use_list_directory`)
//! because `instruction_surface` dropped only double-quoted spans; it now drops
//! every quote pair `text_outside_quoted_segments` reads. Twin of
//! `rust/tests/web/pull-request-1188-quoted-payload-cues.test.mjs`.

use formal_ai::ChatMessage;
use formal_ai::computer_use::{plan_agentic_step, plan_request};

const AGENT_CLI_TOOLS: [&str; 9] = [
    "bash",
    "edit",
    "glob",
    "grep",
    "list",
    "read",
    "webfetch",
    "websearch",
    "write",
];

const EDIT: &str = "Replace 'Partially implemented' with 'Pinned: every handler in precedence \
                    order, every subcommand (also equal to the built `formal-ai --help` list), \
                    and the parts.' in docs/r.md.";

#[test]
fn an_edit_whose_payload_names_orders_and_a_list_is_not_a_computer_use_plan() {
    assert!(plan_request(EDIT).is_none());
    assert!(plan_agentic_step(&[ChatMessage::user(EDIT)], &AGENT_CLI_TOOLS).is_none());
}

#[test]
fn an_explicit_computer_use_count_retains_its_supplied_operands() {
    // Main's 66169b8f requires explicit inputs rather than guessed resources.
    let prompt = "Count the lines in customers.csv and save counts.md.\n```computer-use\n\
                  [{\"primitive\":\"shell.run\",\"arguments\":{\"operation\":\"count_lines\",\
                  \"input\":\"customers.csv\",\"output\":\"counts.md\",\"confirmed\":true}}]\n```";
    let plan = plan_request(prompt).expect("explicit count request");
    assert_eq!(plan.steps.len(), 1);
    assert_eq!(
        plan.steps[0].arguments,
        serde_json::json!({
            "operation": "count_lines", "input": "customers.csv", "output": "counts.md", "confirmed": true,
        })
    );
}

/// G105 (TRANSLATE, T773): a «…» payload that runs over a blank line was read
/// line by line, so neither line closed its pair and the payload's "order" and
/// "count" planned a computer-use count of orders.
const MULTILINE: &str = "Replace «old cell» with «A.\n\norder, count» in r.md.";

#[test]
fn a_quoted_payload_over_several_lines_is_not_a_computer_use_plan() {
    assert!(plan_request(MULTILINE).is_none());
    assert!(plan_agentic_step(&[ChatMessage::user(MULTILINE)], &AGENT_CLI_TOOLS).is_none());
}
