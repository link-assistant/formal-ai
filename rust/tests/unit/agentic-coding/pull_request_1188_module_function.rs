//! PR #1188 dogfooding, T1: `Add a function multiply(a, b) to math.mjs that
//! returns a times b, add a test for it to math.test.mjs, and run node --test`.
//! The function is lowered from the same IR as the Python `def`, through the
//! seeded `javascript` realization of `integer_multiply` and the seeded
//! `javascript_ir_function` template; a node with no JavaScript realization is
//! a named gap. The agentic arm is JavaScript-first
//! (`js/agentic/module_function.mjs`, pinned in
//! `rust/tests/web/pull-request-1188-dogfood.test.mjs`).

use formal_ai::fragment_catalog::FragmentCatalog;
use formal_ai::ir_lowering::lowering_for;
use formal_ai::program_ir::{IrNode, IrType, ProgramIr, ReuseMode};

fn multiply(body: IrNode) -> ProgramIr {
    ProgramIr {
        name: "multiply".to_owned(),
        parameters: vec![
            ("a".to_owned(), IrType::Integer),
            ("b".to_owned(), IrType::Integer),
        ],
        result: IrType::Integer,
        body,
        fragments: vec!["integer_multiply".to_owned()],
        source_urls: Vec::new(),
        source_licenses: Vec::new(),
        reuse: ReuseMode::Verbatim,
    }
}

fn parameter(name: &str) -> IrNode {
    IrNode::Parameter {
        name: name.to_owned(),
        ty: IrType::Integer,
    }
}

#[test]
fn the_multiplication_ir_lowers_to_an_es_module_function() {
    let catalog = FragmentCatalog::bootstrap();
    let ir = multiply(IrNode::Apply {
        fragment: "integer_multiply".to_owned(),
        arguments: vec![parameter("a"), parameter("b")],
    });
    let javascript = lowering_for("javascript")
        .expect("a javascript lowering is registered")
        .lower(&ir, &catalog)
        .expect("integer_multiply has a seeded javascript realization");
    assert_eq!(
        javascript,
        "export function multiply(a, b) {\n  return a * b;\n}\n"
    );
}

#[test]
fn a_node_with_no_javascript_realization_is_a_named_gap() {
    let catalog = FragmentCatalog::bootstrap();
    let ir = multiply(IrNode::Apply {
        fragment: "drop_whitespace".to_owned(),
        arguments: vec![parameter("a")],
    });
    let gap = lowering_for("javascript")
        .expect("a javascript lowering is registered")
        .lower(&ir, &catalog)
        .expect_err("drop_whitespace is realized in Python only");
    assert_eq!(gap.language, "javascript");
    assert_eq!(gap.node, "Apply");
    assert_eq!(
        gap.detail,
        "`drop_whitespace` has no seeded javascript realization"
    );
}

// Round 6: the native twin of the agentic arm
// (`rust/src/agentic_coding/module_function.rs`), driven through the planner
// the way the JavaScript T1 cases drive `planChatStep`.

const ROUTE_TOOLS: [&str; 4] = ["read", "write", "edit", "bash"];
const MATH: &str = "export function add(a, b) {\n  return a + b;\n}\n";
const TEST: &str = "import { test } from 'node:test';\nimport assert from 'node:assert/strict';\nimport { add } from './math.mjs';\n\ntest('add', () => {\n  assert.equal(add(2, 3), 5);\n});\n";
const PROMPT: &str = "Add a function multiply(a, b) to math.mjs that returns a times b, add a test for it to math.test.mjs, and run node --test to confirm it passes.";

/// Run `prompt` over `math.mjs` and `math.test.mjs`; the tools act on an
/// in-memory workspace the way the Agent CLI's do. Returns the tools called
/// and the two files afterwards.
fn drive(prompt: &str) -> (Vec<String>, String, String) {
    use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
    use formal_ai::protocol::{ChatMessage, ToolCall};
    let mut files = std::collections::BTreeMap::from([
        ("math.mjs".to_owned(), MATH.to_owned()),
        ("math.test.mjs".to_owned(), TEST.to_owned()),
    ]);
    let mut messages = vec![ChatMessage::user(prompt)];
    let mut called = Vec::new();
    for index in 0..10 {
        let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &ROUTE_TOOLS) else {
            break;
        };
        let call = calls[0].clone();
        let arguments: serde_json::Value =
            serde_json::from_str(&call.arguments).expect("tool arguments are JSON");
        let path = ["filePath", "file_path", "path"]
            .iter()
            .find_map(|key| arguments[key].as_str())
            .unwrap_or_default()
            .to_owned();
        let result = match call.tool.as_str() {
            "read" => files.get(&path).cloned().unwrap_or_default(),
            "write" => {
                let content = arguments["content"].as_str().unwrap_or_default();
                files.insert(path, content.to_owned());
                String::new()
            }
            "bash" => "# pass 2\n# fail 0\n".to_owned(),
            _ => String::new(),
        };
        called.push(call.tool.clone());
        let id = format!("call_{index}");
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            id.clone(),
            call.tool.clone(),
            call.arguments.clone(),
        )]));
        messages.push(ChatMessage::tool_result(id, call.tool.clone(), result));
    }
    (
        called,
        files.remove("math.mjs").unwrap_or_default(),
        files.remove("math.test.mjs").unwrap_or_default(),
    )
}

fn added(name: &str, operator: &str) -> String {
    format!("{MATH}\nexport function {name}(a, b) {{\n  return a {operator} b;\n}}\n")
}

fn tested(name: &str, expected: i64) -> String {
    format!(
        "{}\ntest('{name}', () => {{\n  assert.equal({name}(2, 3), {expected});\n}});\n",
        TEST.replace("{ add }", &format!("{{ add, {name} }}"))
    )
}

#[test]
fn the_native_arm_reads_both_writes_both_and_runs_the_stated_command() {
    let (called, module, test) = drive(PROMPT);
    assert_eq!(called, ["read", "read", "write", "write", "bash", "bash"]);
    assert_eq!(module, added("multiply", "*"));
    assert_eq!(test, tested("multiply", 6));
}

#[test]
fn the_native_request_reading_matches_the_javascript_one() {
    use formal_ai::agentic_coding::module_function::{
        ModuleFunctionRequest, module_function_request,
    };
    assert_eq!(
        module_function_request(PROMPT),
        Some(ModuleFunctionRequest {
            name: "multiply".to_owned(),
            parameters: vec!["a".to_owned(), "b".to_owned()],
            at: 15,
            module: "math.mjs".to_owned(),
            test: Some("math.test.mjs".to_owned()),
            language: "javascript".to_owned(),
            clause: "Add a function multiply(a, b) to math.mjs that returns a times b".to_owned(),
            command: Some("node --test".to_owned()),
        })
    );
    assert_eq!(
        module_function_request(
            "Add a function multiply(a, b) to math.txt that returns a times b."
        ),
        None
    );
    assert_eq!(
        module_function_request(
            "Add a function multiply(a, b) to math.mjs that returns a times b, and add a test."
        ),
        None
    );
    for prompt in [
        "Добавь функцию both(a, b) в math.mjs, которая возвращает их сумму, добавь тест для неё в math.test.mjs и запусти node --test.",
        "math.mjs में एक फ़ंक्शन both(a, b) जोड़ो जो उनका योग लौटाता है, math.test.mjs में उसका टेस्ट जोड़ो और node --test चलाओ।",
        "在 math.mjs 中添加一个函数 both(a, b)，返回它们的乘积，在 math.test.mjs 中为它添加测试，然后运行 node --test。",
    ] {
        let request = module_function_request(prompt).unwrap_or_else(|| panic!("{prompt}"));
        assert_eq!(request.command.as_deref(), Some("node --test"), "{prompt}");
    }
}

#[test]
fn a_relation_the_request_names_is_the_expected_value_in_every_seeded_language() {
    for (prompt, operator, expected) in [
        (
            "Add a function both(a, b) to math.mjs that returns their sum, add a test for it to math.test.mjs, and run node --test.",
            "+",
            5,
        ),
        (
            "Add a function both(a, b) to math.mjs that returns their product, add a test for it to math.test.mjs, and run node --test.",
            "*",
            6,
        ),
        (
            "Add a function both(a, b) to math.mjs that returns their difference, add a test for it to math.test.mjs, and run node --test.",
            "-",
            -1,
        ),
        (
            "Добавь функцию both(a, b) в math.mjs, которая возвращает их сумму, добавь тест для неё в math.test.mjs и запусти node --test.",
            "+",
            5,
        ),
        (
            "math.mjs में एक फ़ंक्शन both(a, b) जोड़ो जो उनका अंतर लौटाता है, math.test.mjs में उसका टेस्ट जोड़ो और node --test चलाओ।",
            "-",
            -1,
        ),
        (
            "在 math.mjs 中添加一个函数 both(a, b)，返回它们的乘积，在 math.test.mjs 中为它添加测试，然后运行 node --test。",
            "*",
            6,
        ),
    ] {
        let (called, module, test) = drive(prompt);
        assert_eq!(
            called,
            ["read", "read", "write", "write", "bash", "bash"],
            "{prompt}"
        );
        assert_eq!(module, added("both", operator), "{prompt}");
        assert_eq!(test, tested("both", expected), "{prompt}");
    }
}

#[path = "../../fixtures/observed-callable-contracts.rs"]
mod observed_callable_contracts;
#[path = "../../fixtures/observed-callable-discovery.rs"]
mod observed_callable_discovery;
