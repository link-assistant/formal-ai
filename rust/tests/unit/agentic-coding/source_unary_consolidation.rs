//! Native planner custody controls; fixture executables are capability markers and never run.
use formal_ai::agentic_coding::source_unary_consolidation::source_unary_consolidation_request;
use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::{ChatMessage, ToolCall};
use serde_json::{Value, json};
const OWNER: &str = "js/agentic/heldout.mjs";
const TASK: &str = "Consolidate stateless unary callbacks in js/agentic/heldout.mjs using canonical helpers. Preserve callback value behavior and comments.";

#[test]
fn finite_request_rejects_extra_obligations_and_scope_escape() {
    let request = source_unary_consolidation_request(TASK).expect("owned request");
    assert_eq!(request["owner"], OWNER);
    for task in [
        TASK.replace(OWNER, "../heldout.mjs"),
        TASK.replace(OWNER, "js/agentic/../heldout.mjs"),
        format!("{TASK} Delete tests."),
    ] {
        assert!(source_unary_consolidation_request(&task).is_none());
    }
    assert!(
        source_unary_consolidation_request(&format!(
            "\u{feff}{}\u{feff}",
            TASK.replace(' ', "\u{a0}")
        ))
        .is_some()
    );
    assert!(source_unary_consolidation_request(&TASK.replace(' ', "\u{85}")).is_none());
}

#[test]
fn source_text_and_advertised_bash_never_install_an_engine() {
    let plan = plan_chat_step(&[ChatMessage::user(TASK)], &["read", "bash", "edit"])
        .expect("explicit missing capability");
    let AgenticPlan::Final(answer) = plan else {
        panic!("missing installed host must be a gap");
    };
    assert!(!answer.contains("sourceReadbackVerified"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn installed_unary_route_requires_preimage_receipt_and_exact_readback() {
    use formal_ai::agentic_coding::installed_source_commands::configure_installed_source_commands;
    struct Fixture(std::path::PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    const CHILD: &str = "FORMAL_AI_UNARY_CAPABILITY_TEST_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let name = std::thread::current().name().expect("test name").to_owned();
        let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
            .args(["--exact", &name, "--nocapture"])
            .env(CHILD, "1")
            .output()
            .expect("isolated registry control");
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let directory = Fixture(std::env::temp_dir().join(format!(
        "formal-ai-unary-native-control-{}",
        std::process::id()
    )));
    std::fs::create_dir(&directory.0).expect("exclusive fixture");
    let home = directory.0.canonicalize().unwrap();
    std::fs::create_dir(home.join("scripts")).unwrap();
    let node = home.join("node-fixture");
    std::fs::write(&node, b"capability-only fixture, never executed\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&node, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    let cst = home.join("scripts/source-cst-transform-request.mjs");
    let unary = home.join("scripts/consolidate-stateless-unary-request.mjs");
    std::fs::write(&cst, b"// pinned source fixture\n").unwrap();
    std::fs::write(&unary, b"// pinned unary fixture\n").unwrap();
    let digest =
        |path: &std::path::Path| formal_ai::source_fetch::sha256_hex(&std::fs::read(path).unwrap());
    let configuration = json!({"nodeExecutable":node,"resourceHome":home,"nodeSha256":digest(&node),"sourceCstTransformSha256":digest(&cst),"unaryConsolidationSha256":digest(&unary)});
    configure_installed_source_commands(Some(&configuration.to_string()))
        .expect("explicit trusted caller installation");
    let tools = ["read", "bash", "edit"];
    let mut messages = vec![ChatMessage::user(TASK)];
    let AgenticPlan::ToolCalls(calls) = plan_chat_step(&messages, &tools).expect("preimage read")
    else {
        panic!("read required");
    };
    assert_eq!(calls[0].tool, "read");
    let source = "export const result = values.map(item => item.text);\n";
    let read_args = calls[0].arguments.clone();
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "preimage", "read", &read_args,
    )]));
    let mut read = ChatMessage::tool_result("preimage", "read", source);
    read.source_read = Some(json!({"path":OWNER,"success":true,"complete":true,"format":"raw"}));
    messages.push(read);
    let AgenticPlan::ToolCalls(calls) =
        plan_chat_step(&messages, &tools).expect("installed command")
    else {
        panic!("command required");
    };
    assert_eq!(calls[0].tool, "bash");
    let run_args = calls[0].arguments.clone();
    let arguments: Value = serde_json::from_str(&run_args).unwrap();
    let command = arguments["command"].as_str().unwrap();
    let sha = formal_ai::source_fetch::sha256_hex(source.as_bytes());
    assert!(command.contains(&sha));
    assert!(command.contains(unary.to_str().unwrap()));
    assert!(!command.contains(source));
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "operation",
        "bash",
        &run_args,
    )]));
    let request_json = serde_json::to_string(TASK).unwrap();
    let owner_json = serde_json::to_string(OWNER).unwrap();
    let receipt = format!(
        r#"{{"schema":"stateless-unary-request/v1","request":{request_json},"owner":{owner_json},"beforeSha256":"{sha}","afterSha256":"{sha}","files":0,"callbacks":0,"requiresIndependentConsumerChecks":true,"completeSemanticEquivalence":false,"independentBehaviorValidation":"pending","autonomousAuthorshipCredit":0}}"#
    );
    messages.push(ChatMessage::tool_result(
        "operation",
        "bash",
        format!("Output: {receipt}\nExit Code: 0"),
    ));
    let AgenticPlan::ToolCalls(calls) =
        plan_chat_step(&messages, &tools).expect("physical readback")
    else {
        panic!("receipt alone is insufficient");
    };
    assert_eq!(calls[0].tool, "read");
    let before_readback = messages.clone();
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "readback",
        "read",
        &calls[0].arguments,
    )]));
    let mut read = ChatMessage::tool_result("readback", "read", source);
    read.source_read = Some(json!({"path":OWNER,"success":true,"complete":true,"format":"raw"}));
    messages.push(read);
    let AgenticPlan::Final(answer) = plan_chat_step(&messages, &tools).expect("finding") else {
        panic!("readback finding required");
    };
    assert!(answer.contains("\"sourceReadbackVerified\":true"));
    assert!(answer.contains("\"independentBehaviorValidation\":\"pending\""));
    let mut repeated = messages.clone();
    repeated.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "operation-latest",
        "bash",
        &run_args,
    )]));
    repeated.push(ChatMessage::tool_result(
        "operation-latest",
        "bash",
        format!("Output: {receipt}\nExit Code: 0"),
    ));
    let AgenticPlan::ToolCalls(calls) =
        plan_chat_step(&repeated, &tools).expect("fresh latest read required")
    else {
        panic!("an earlier read must not certify a later operation");
    };
    assert_eq!(calls[0].tool, "read");
    let mut wrong = before_readback;
    wrong.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function("other-read","read",r#"{"path":"js/agentic/other.mjs","filePath":"js/agentic/other.mjs","file_path":"js/agentic/other.mjs"}"#)]));
    let mut read = ChatMessage::tool_result("other-read", "read", source);
    read.source_read =
        Some(json!({"path":"js/agentic/other.mjs","success":true,"complete":true,"format":"raw"}));
    wrong.push(read);
    assert!(matches!(
        plan_chat_step(&wrong, &tools),
        Some(AgenticPlan::ToolCalls(_))
    ));
    for payload in [
        format!("{receipt}{{}}"),
        receipt.replacen("{\"schema\":", "{\"schema\":\"wrong\",\"schema\":", 1),
        receipt.replace(&sha, &"0".repeat(64)),
    ] {
        let mut invalid = messages.clone();
        invalid[4] = ChatMessage::tool_result(
            "operation",
            "bash",
            format!("Output: {payload}\nExit Code: 0"),
        );
        let AgenticPlan::Final(answer) =
            plan_chat_step(&invalid, &tools).expect("invalid receipt gap")
        else {
            panic!("receipt must not be accepted");
        };
        assert!(!answer.contains("sourceReadbackVerified"));
    }
    let mut failed = messages.clone();
    failed.last_mut().unwrap().is_error = true;
    let AgenticPlan::Final(answer) = plan_chat_step(&failed, &tools).expect("failed read gap")
    else {
        panic!("failed provider read is not success");
    };
    assert!(!answer.contains("sourceReadbackVerified"));
}
