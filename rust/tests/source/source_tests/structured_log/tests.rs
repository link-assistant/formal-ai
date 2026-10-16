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
