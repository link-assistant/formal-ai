use super::*;

#[test]
fn codex_ephemeral_uses_seeded_responses_provider_and_model_catalog() {
    let dir = tmpdir();
    let home = dir.join("home");
    let bin_dir = dir.join("bin");
    std::fs::create_dir_all(&home).expect("home");
    std::fs::create_dir_all(&bin_dir).expect("bin");
    write_fake_cli(&bin_dir, "codex");
    let capture = dir.join("capture.txt");

    let output = run_with_capture(
        &home,
        &bin_dir,
        &capture,
        &[
            "with",
            "--base-url",
            "http://127.0.0.1:18080",
            "codex",
            "hi",
        ],
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "stderr: {stderr}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Hi, how may I help you?"
    );
    let captured = std::fs::read_to_string(&capture).expect("capture");
    assert!(captured.contains("arg[0]=exec"), "capture:\n{captured}");
    assert!(captured.contains("arg[1]=--skip-git-repo-check"));
    assert!(captured.contains("arg[2]=--sandbox"));
    assert!(captured.contains("arg[3]=read-only"));
    assert!(captured.contains("model_provider=\"formalai\""));
    assert!(captured.contains("model=\"formal-ai\""));
    assert!(captured.contains("wire_api=\"responses\""));
    assert!(
        captured.contains("model_catalog_json=\""),
        "Codex must receive a model catalog path: {captured}"
    );
    assert!(captured.contains("base_url=\"http://127.0.0.1:18080/api/openai/v1\""));
    assert!(captured.contains("FORMAL_AI_API_KEY=formal-ai"));
    assert!(captured.contains("---CODEX_MODEL_CATALOG---"));
    assert!(captured.contains("\"slug\": \"formal-ai\""));
    assert!(captured.contains("\"context_window\": 60000"));
    assert!(captured.contains("\"shell_type\": \"shell_command\""));

    let _ = std::fs::remove_dir_all(&dir);
}
