use super::*;
#[test]
fn dialog_numeric_extension_preserves_native_semantics() {
    let value = json!({"float":1.0, "negative_zero":-0.0, "integer":9_007_199_254_740_993_u64});
    let line = encode_dialog(&value).expect("encode numeric extensions");
    assert!(line.contains("(float 64 1.0)"));
    assert!(line.contains("(integer 9007199254740993)"));
    assert_eq!(decode_dialog(&line).expect("decode native numbers"), value);
    assert!(decode(&line).is_err());
}
#[test]
fn typed_boundary_retains_nested_types_and_rejects_ambiguous_records() {
    let value = json!({"text":"line\n%()\" é", "numeric_text":"1", "number":1, "empty":[], "null":null, "flag":false});
    assert_eq!(
        decode(&encode(&value).expect("encode")).expect("decode"),
        value
    );
    for line in [
        "(formal-ai-log-v1 (object (field x (number 1)) (field x (number 2))))",
        "(formal-ai-log-v1 (number 01))",
        "(formal-ai-log-v1 (string %4a))",
        "(formal-ai-log-v1 (array)) trailing",
    ] {
        assert!(decode(line).is_err(), "must refuse {line}");
    }
}
#[test]
fn control_evidence_ignores_diagnostic_filters() {
    const CHILD: &str = "FORMAL_AI_CONTROL_TEST_CHILD";
    if crate::solver_config::flag_enabled(CHILD) {
        configure(
            crate::solver_config::flag_enabled("FORMAL_AI_CONTROL_TEST_SILENT"),
            false,
        );
        let evaluated = std::cell::Cell::new(false);
        event(
            "debug",
            "control_fixture",
            || {
                evaluated.set(true);
                "diagnostic_probe".into()
            },
            || json!({}),
        );
        assert_eq!(evaluated.get(), enabled("debug"));
        let session =
            json!({"id":"s".repeat(1_048_577), "resume_command":"agent --resume fixture"});
        control(
            "orchestration_session",
            "control_fixture",
            &format!("{ORCHESTRATION_SESSION_PREFIX}{session}"),
            &json!({}),
        );
        return;
    }
    let qualified = concat!(
        module_path!(),
        "::control_evidence_ignores_diagnostic_filters"
    );
    let (_, test_name) = qualified
        .split_once("::")
        .expect("crate-qualified test name");
    for (silent, env_silent, level, diagnostics) in [
        ("0", "0", "debug", 1),
        ("1", "0", "trace", 0),
        ("0", "1", "trace", 0),
        ("0", "0", "error", 0),
        ("0", "0", "silent", 0),
    ] {
        let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
            .args(["--exact", test_name, "--nocapture"])
            .env(CHILD, "1")
            .env("FORMAL_AI_CONTROL_TEST_SILENT", silent)
            .env("FORMAL_AI_SILENT", env_silent)
            .env("FORMAL_AI_LOG_LEVEL", level)
            .output()
            .expect("isolated logging fixture");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .any(|line| line == "running 1 test"),
            "child must run exactly the requested test"
        );
        let stderr = String::from_utf8(output.stderr).expect("typed UTF-8 stderr");
        let records: Vec<_> = stderr
            .lines()
            .map(|line| {
                decode_control(line)
                    .or_else(|_| decode(line))
                    .expect("actual canonical stderr record")
            })
            .collect();
        assert!(
            stderr
                .lines()
                .filter(|line| decode_control(line).is_ok())
                .all(|line| decode(line).is_err()),
            "ordinary diagnostic record cap stays in force"
        );
        let controls: Vec<_> = records
            .iter()
            .filter(|record| record["schema"] == "formal-ai-control-v1")
            .collect();
        assert_eq!(controls.len(), 1);
        assert_eq!(controls[0]["kind"], "orchestration_session");
        let message = controls[0]["message"].as_str().expect("control message");
        let session: Value = serde_json::from_str(
            message
                .strip_prefix("formal-ai: orchestration-session-json:")
                .expect("original machine prefix"),
        )
        .expect("original session JSON");
        assert_eq!(session["id"].as_str().expect("id").len(), 1_048_577);
        assert_eq!(session["resume_command"], "agent --resume fixture");
        assert_eq!(
            records
                .iter()
                .filter(|record| record["schema"] == "formal-ai-diagnostic-v1")
                .count(),
            diagnostics
        );
    }
}

#[test]
fn diagnostic_metadata_preserves_small_values_and_bounds_wide_percent_encoded_fields() {
    let ordinary =
        json!({"tool":"inspect","state":"pending","repeat_limit":3,"__proto__":"own data"});
    assert_eq!(bounded_diagnostic_metadata(&ordinary), ordinary);
    let content = "😀é%()\n".repeat(20000);
    let mut metadata = ordinary;
    metadata["arguments"] = json!(content);
    let bounded = bounded_diagnostic_metadata(&metadata);
    assert_eq!(bounded["tool"], "inspect");
    assert_eq!(bounded["__proto__"], "own data");
    assert_eq!(bounded["arguments"]["bytes"], content.len());
    assert_eq!(
        bounded["arguments"]["sha256"],
        crate::source_fetch::hex_lower(&Sha256::digest(content.as_bytes()))
    );
    assert!(
        bounded["arguments"]["preview"]
            .as_str()
            .expect("UTF-8 preview")
            .len()
            <= DIAGNOSTIC_PREVIEW_BYTES
    );
    let mut wide = serde_json::Map::new();
    wide.insert("tool".into(), json!("inspect"));
    for index in 0..1500 {
        wide.insert(format!("{}{index}", "😀".repeat(50)), json!("é".repeat(80)));
    }
    let bounded = bounded_diagnostic_metadata(&Value::Object(wide));
    assert_eq!(bounded["fields"]["tool"], "inspect");
    assert!(bounded["omitted_fields"].as_u64().expect("omission count") > 0);
    let line = encode(&bounded).expect("bounded encoding");
    assert!(line.len() <= DIAGNOSTIC_METADATA_BYTES);
    assert_eq!(decode(&line).expect("default decoder"), bounded);
}

#[test]
fn actual_diagnostic_producer_bounds_metadata_after_redaction() {
    const CHILD: &str = "FORMAL_AI_BOUNDED_METADATA_CHILD";
    const SECRET: &str = "private-metadata-secret";
    if crate::solver_config::flag_enabled(CHILD) {
        configure(false, false);
        event(
            "debug",
            "planner",
            || "tool_call_bound".into(),
            || json!({"tool":"inspect","state":"pending","repeat_limit":3,"password":SECRET,"arguments":format!("prefix {SECRET} 😀é ").repeat(20000)}),
        );
        return;
    }
    let qualified = concat!(
        module_path!(),
        "::actual_diagnostic_producer_bounds_metadata_after_redaction"
    );
    let (_, test_name) = qualified.split_once("::").expect("qualified native test");
    let output =
        std::process::Command::new(std::env::current_exe().expect("native test executable"))
            .args(["--exact", test_name, "--nocapture"])
            .env(CHILD, "1")
            .env("GH_TOKEN", SECRET)
            .env("FORMAL_AI_SILENT", "0")
            .env("FORMAL_AI_LOG_LEVEL", "debug")
            .output()
            .expect("isolated actual diagnostic producer");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 diagnostic");
    let records: Vec<_> = stderr
        .lines()
        .map(|line| {
            assert!(line.len() < 1024 * 1024);
            decode(line).expect("unchanged default diagnostic reader")
        })
        .collect();
    assert_eq!(records.len(), 1);
    let metadata = &records[0]["metadata"];
    assert_eq!(metadata["tool"], "inspect");
    assert_eq!(metadata["repeat_limit"], 3);
    assert_eq!(metadata["password"], "[REDACTED]");
    let safe = "prefix [REDACTED] 😀é ".repeat(20000);
    assert_eq!(metadata["arguments"]["bytes"], safe.len());
    assert_eq!(
        metadata["arguments"]["sha256"],
        crate::source_fetch::hex_lower(&Sha256::digest(safe.as_bytes()))
    );
    assert!(!stderr.contains(SECRET));
}
