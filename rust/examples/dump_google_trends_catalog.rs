//! Print the deterministic catalog, or stage the catalog and an actual driver
//! session in a fresh directory supplied as the sole argument.

use formal_ai::agentic_coding::{
    GOOGLE_TRENDS_CATALOG_PATH, GOOGLE_TRENDS_CATALOG_TASK, google_trends_catalog,
    run_agentic_task_with_tools,
};

fn main() {
    let document = google_trends_catalog::render_document();
    assert!(!document.is_empty(), "Google Trends source unavailable");
    let mut arguments = std::env::args_os().skip(1);
    let Some(output) = arguments.next() else {
        print!("{document}");
        return;
    };
    assert!(
        arguments.next().is_none(),
        "one output directory is accepted"
    );
    let output = std::path::PathBuf::from(output);
    std::fs::create_dir(&output).expect("a fresh output directory is required");

    // Only the task and advertised tools are inputs. Never reuse captured
    // outputs, command receipts, or final answers when generating a session.
    let path = "docs/case-studies/issue-498/agent-cli-session-google-trends.json";
    let captured: serde_json::Value =
        serde_json::from_slice(&std::fs::read(path).expect("read pinned session inputs"))
            .expect("parse pinned session inputs");
    assert_eq!(captured["task"].as_str(), Some(GOOGLE_TRENDS_CATALOG_TASK));
    let tools: Vec<&str> = captured["tools_advertised"]
        .as_array()
        .expect("pinned advertised tools")
        .iter()
        .map(|tool| tool.as_str().expect("tool name"))
        .collect();
    let fresh = run_agentic_task_with_tools(GOOGLE_TRENDS_CATALOG_TASK, &tools)
        .expect("execute the native catalog driver");
    assert!(!fresh.hit_turn_cap, "catalog driver reached its turn cap");
    assert_eq!(
        fresh.steps.len(),
        2,
        "one write and one actual verification"
    );
    assert_eq!(fresh.steps[0].tool, "write_file");
    let written: serde_json::Value =
        serde_json::from_str(&fresh.steps[0].arguments).expect("native write arguments");
    assert_eq!(written["path"].as_str(), Some(GOOGLE_TRENDS_CATALOG_PATH));
    assert_eq!(written["content"].as_str(), Some(document.as_str()));
    assert_eq!(fresh.steps[1].tool, "run_command");
    let receipt: serde_json::Value =
        serde_json::from_str(&fresh.steps[1].result).expect("actual command receipt");
    assert_eq!(receipt["exit_code"].as_i64(), Some(0));
    assert_eq!(receipt["complete"].as_bool(), Some(true));
    let session = format!(
        "{}\n",
        serde_json::to_string_pretty(&fresh.session_json()).expect("serialize actual session")
    );
    for (path, content) in [(GOOGLE_TRENDS_CATALOG_PATH, document), (path, session)] {
        let target = output.join(path);
        std::fs::create_dir_all(target.parent().expect("fixture parent"))
            .expect("create fixture parent");
        std::fs::write(target, content).expect("write generated fixture");
        println!("wrote {path}");
    }
}
