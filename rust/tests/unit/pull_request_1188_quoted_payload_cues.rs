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
fn an_unquoted_computer_use_request_still_plans() {
    let prompt = "Count the sub-tasks of the customer import rewrite and save the result in \
                  `counts.md`.";
    assert!(plan_request(prompt).is_some());
}
