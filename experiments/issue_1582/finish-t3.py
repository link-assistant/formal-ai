from pathlib import Path

p=Path('rust/tests/integration/with_formal_ai/issue_1582.rs')
s=p.read_text()+'''
#[test]
fn t3_claude_subprocess_inherits_the_actual_context_and_output_limits() {
    let dir = tmpdir();
    let home = dir.join("home");
    let bin = dir.join("bin");
    std::fs::create_dir_all(&home).expect("home");
    std::fs::create_dir_all(&bin).expect("bin");
    write_fake_cli(&bin, "t3");
    let capture = dir.join("capture");
    let output = run_with_capture(&home, &bin, &capture, &["with", "t3code", "serve"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let captured = std::fs::read_to_string(capture).expect("capture");
    for (variable, minimum) in [("CLAUDE_CODE_MAX_CONTEXT_TOKENS=", 200_000), ("CLAUDE_CODE_MAX_OUTPUT_TOKENS=", 0)] {
        let value = captured.lines().find_map(|line| line.strip_prefix(variable))
            .expect("override").parse::<u64>().expect("numeric override");
        assert!(value > minimum, "{variable}{value}");
    }
    std::fs::remove_dir_all(dir).expect("cleanup");
}
'''
p.write_text(s)
p=Path('docs/case-studies/issue-1582/README.md')
s=p.read_text().replace('Read-only transport does not grant', 'T3 Code also launches Claude Code, so its ephemeral environment and persistent\nAnthropic profile receive the same documented context/output overrides.\n[Its provider documentation](https://github.com/pingdotgg/t3code/blob/main/docs/user/providers-claude.md)\nconfirms that integration; the pinned documentation and catalog source are\nretained under `raw-data/research/`. Compaction settings are not substituted for\na model context window. A fake-T3 regression checks both numeric overrides;\nan installed T3 UI session has not been exercised here.\n\nRead-only transport does not grant')
p.write_text(s)
p=Path('experiments/issue_1582/pull-request.md')
s=p.read_text().replace('Claude gets named read permissions and its documented context/output environment overrides.', 'Claude gets named read permissions and its documented context/output environment overrides; the T3 Claude path receives the same overrides.')
p.write_text(s)
