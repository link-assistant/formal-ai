//! Native request/custody controls. Full engine execution is a separate installed
//! Node bundle acceptance obligation; fixture executables here are never run.
use formal_ai::agentic_coding::source_cst_transform::source_cst_transform_request;
use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::{ChatMessage, ToolCall};

const TASK: &str = "Rename parameter value of function increment to input in src/count.ts";
#[test]
fn qualified_parameter_request_preserves_names_and_rejects_scope_escape() {
    let request = source_cst_transform_request(TASK).expect("qualified request");
    assert_eq!(
        (
            request.binding.as_str(),
            request.declaration.as_str(),
            request.target.as_str(),
            request.owner.as_str()
        ),
        ("value", "increment", "input", "src/count.ts")
    );
    for task in [
        "Rename value to input",
        "Rename parameter value of function increment to input in ../count.ts",
        "Rename parameter value of function increment to input in /count.ts",
        "Rename parameter value of function increment to input in a//count.ts",
        "Rename parameter value of function increment to input in a/./count.ts",
        "Rename parameter value of function increment to input in count.json",
    ] {
        assert!(source_cst_transform_request(task).is_none(), "{task}");
    }
    assert!(source_cst_transform_request(&format!("{}{}", " ".repeat(4096), TASK)).is_none());
    assert!(
        source_cst_transform_request(&format!(
            "\u{feff}{}\u{feff}",
            TASK.replace(' ', "\u{00a0}")
        ))
        .is_some()
    );
    assert!(source_cst_transform_request(&TASK.replace(' ', "\u{0085}")).is_none());
}

#[test]
fn missing_installed_engine_is_terminal_not_a_global_rename() {
    let plan =
        plan_chat_step(&[ChatMessage::user(TASK)], &["read", "bash", "edit"]).expect("route");
    assert_eq!(
        plan,
        AgenticPlan::Final(
            "The installed host does not provide the complete CST transformation command."
                .to_owned()
        )
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn installed_route_binds_read_preimage_and_rejects_changed_capability() {
    use formal_ai::agentic_coding::installed_source_commands::{
        configure_installed_source_commands, installed_source_command,
    };
    struct Fixture(std::path::PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    const CHILD: &str = "FORMAL_AI_CST_CAPABILITY_TEST_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let name = std::thread::current().name().expect("test name").to_owned();
        let result = std::process::Command::new(std::env::current_exe().expect("test executable"))
            .args(["--exact", &name, "--nocapture"])
            .env(CHILD, "1")
            .output()
            .expect("isolated registry control");
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        return;
    }
    let directory = Fixture(std::env::temp_dir().join(format!(
        "formal-ai-installed-source-control-{}",
        std::process::id()
    )));
    std::fs::create_dir(&directory.0).expect("exclusive fixture directory");
    let home = directory.0.canonicalize().expect("canonical home");
    std::fs::create_dir(home.join("scripts")).expect("scripts");
    let node = home.join("node-{task}'fixture");
    std::fs::write(&node, b"capability-only fixture, never executed\n")
        .expect("fixture executable");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&node, std::fs::Permissions::from_mode(0o700))
            .expect("executable permission");
    }
    let transform = home.join("scripts/source-cst-transform-request.mjs");
    let unary = home.join("scripts/consolidate-stateless-unary-request.mjs");
    std::fs::write(&transform, b"// pinned transform fixture\n").expect("transform");
    std::fs::write(&unary, b"// pinned unary fixture\n").expect("unary");
    let digest = |path: &std::path::Path| {
        formal_ai::source_fetch::sha256_hex(&std::fs::read(path).expect("fixture bytes"))
    };
    let configuration = serde_json::json!({"nodeExecutable":node,"resourceHome":home,"nodeSha256":digest(&node),"sourceCstTransformSha256":digest(&transform),"unaryConsolidationSha256":digest(&unary)});
    let mut invalid = configuration.clone();
    invalid["nodeSha256"] = serde_json::json!("0".repeat(64));
    assert!(configure_installed_source_commands(Some(&invalid.to_string())).is_err());
    configure_installed_source_commands(Some(&configuration.to_string()))
        .expect("trusted caller capability");
    assert!(installed_source_command("task-chosen-script").is_none());
    let tools = ["read", "bash", "edit"];
    let mut messages = vec![ChatMessage::user(TASK)];
    let AgenticPlan::ToolCalls(calls) = plan_chat_step(&messages, &tools).expect("owner read")
    else {
        panic!("expected owner read");
    };
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].tool, "read");
    let arguments: serde_json::Value =
        serde_json::from_str(&calls[0].arguments).expect("arguments");
    assert_eq!(arguments["path"], "src/count.ts");
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "owner-read",
        &calls[0].tool,
        &calls[0].arguments,
    )]));
    let source = "function increment(value: number) { return value + 1; }\n";
    let mut read = ChatMessage::tool_result("owner-read", "read", source);
    read.source_read = Some(
        serde_json::json!({"path":"src/count.ts","success":true,"complete":true,"format":"raw"}),
    );
    messages.push(read);
    let AgenticPlan::ToolCalls(calls) =
        plan_chat_step(&messages, &tools).expect("installed engine")
    else {
        panic!("expected installed engine command");
    };
    assert_eq!(calls[0].tool, "bash");
    let arguments: serde_json::Value =
        serde_json::from_str(&calls[0].arguments).expect("run arguments");
    let command = arguments["command"].as_str().expect("command");
    assert!(command.contains(transform.to_str().expect("script path")));
    assert!(command.contains(&formal_ai::source_fetch::sha256_hex(source.as_bytes())));
    let quote = |value: &str| format!("'{}'", value.replace('\'', "'\\''"));
    let expected_command = format!(
        "{} {} --request {} --expected {}",
        quote(node.to_str().expect("node path")),
        quote(transform.to_str().expect("script path")),
        quote(TASK),
        formal_ai::source_fetch::sha256_hex(source.as_bytes())
    );
    assert_eq!(command, expected_command);
    assert!(!command.contains(source));
    let mut wrong = messages.clone();
    wrong[1] = ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "owner-read",
        "read",
        "{\"path\":\"src/other.ts\"}",
    )]);
    assert!(matches!(
        plan_chat_step(&wrong, &tools),
        Some(AgenticPlan::Final(_))
    ));
    let mut incomplete = messages.clone();
    incomplete[2].source_read.as_mut().expect("metadata")["complete"] = serde_json::json!(false);
    assert!(matches!(
        plan_chat_step(&incomplete, &tools),
        Some(AgenticPlan::Final(_))
    ));
    std::fs::write(&transform, b"// changed after configuration\n").expect("replace script");
    assert!(installed_source_command("source-cst-transform").is_none());
    assert!(matches!(
        plan_chat_step(&messages, &tools),
        Some(AgenticPlan::Final(_))
    ));
}
