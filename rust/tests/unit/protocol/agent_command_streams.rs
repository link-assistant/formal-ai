//! Actual public workspace commands preserve simultaneous output beyond pipe capacity.
use formal_ai::agent::{AgentWorkspace, AgentWorkspaceConfig};
use std::time::Duration;

#[test]
fn public_commands_drain_both_large_streams_and_preserve_nonzero_status() {
    let root = std::env::temp_dir().join("formal-ai-command-streams-public");
    for code in [0, 7] {
        let configuration = AgentWorkspaceConfig {
            base_dir: root.clone(),
            time_budget: Duration::from_secs(2),
        };
        let mut workspace =
            AgentWorkspace::for_prompt(&format!("command-streams-{code}"), &configuration).unwrap();
        let program = format!(
            "import sys\nsys.stdout.buffer.write(('λ\\n'*65536).encode('utf-8'))\n\
             sys.stdout.buffer.flush()\nsys.stderr.buffer.write(('🙂\\r\\n'*65536).encode('utf-8'))\n\
             sys.stderr.buffer.flush()\nsys.exit({code})\n"
        );
        workspace.create_file("streams.py", &program);
        workspace.run_command("python3 streams.py");
        let output = workspace
            .last_command_result()
            .expect("actual subprocess result");
        assert!(
            !output.timed_out,
            "status={:?}; stdout={} bytes; stderr={} bytes",
            output.status_code,
            output.stdout.len(),
            output.stderr.len()
        );
        assert_eq!(output.status_code, Some(code));
        assert_eq!(output.stdout, "λ\n".repeat(65536));
        assert_eq!(output.stderr, "🙂\r\n".repeat(65536));
        std::fs::remove_dir_all(workspace.root()).unwrap();
    }
}
