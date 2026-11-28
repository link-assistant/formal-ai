//! Wrapper diagnostic flags must remain separate from forwarded client flags.

use super::*;

#[test]
fn with_formal_ai_wrapper_and_client_verbosity_have_separate_scopes() {
    for wrapper_verbose in [false, true] {
        let dir = tmpdir();
        let home = dir.join("home");
        let bin_dir = dir.join("bin");
        std::fs::create_dir_all(&home).expect("home");
        std::fs::create_dir_all(&bin_dir).expect("bin");
        write_fake_cli(&bin_dir, "agent");
        let capture = dir.join("capture.txt");
        let port = unused_loopback_port();
        let mut command = Command::new(env!("CARGO_BIN_EXE_formal-ai"));
        fixture_env::clear_client_environment(&mut command);
        command.arg("with");
        if wrapper_verbose {
            command.arg("--verbose");
        }
        let output = command
            .args([
                "--port",
                &port.to_string(),
                "agent",
                "-p",
                "hi",
                "--verbose",
            ])
            .env("HOME", &home)
            .env("PATH", path_with_fake_clis(&bin_dir))
            .env("FORMAL_AI_CAPTURE", &capture)
            .output()
            .expect("run wrapper with a verbose client");
        assert!(
            output.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        if wrapper_verbose {
            assert!(
                structured_stderr::messages(&output.stderr)
                    .contains("temporary server in agent mode")
            );
        } else {
            assert!(
                output.stderr.is_empty(),
                "client verbosity must not enable wrapper logs: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let captured = std::fs::read_to_string(&capture).expect("capture");
        assert!(captured_args(&captured).contains(&"--verbose"));
        assert!(
            TcpStream::connect(("127.0.0.1", port)).is_err(),
            "temporary server still listening"
        );
        std::fs::remove_dir_all(&dir).expect("remove fixture");
    }
}
