use super::*;

pub(super) fn external_agent_fixture_process() {
    let Ok(mode) = std::env::var(FIXTURE_ENV) else {
        return;
    };
    match mode.as_str() {
        "success" => {
            fs::write("README.md", "fixture change\n").unwrap();
            println!(
                "{}",
                std::env::var("FORMAL_AI_ISSUE_703_OUTPUT")
                    .unwrap_or_else(|_| "fixture_stdout".to_string())
            );
            eprintln!("fixture_stderr");
        }
        "native_session" => {
            println!(
                "{}",
                std::env::var("FORMAL_AI_ISSUE_703_OUTPUT")
                    .unwrap_or_else(|_| "fixture_stdout".to_string())
            );
            eprintln!(
                "formal-ai: orchestration-session-json:{{\"id\":\"ses_issue_703\",\"resume_command\":\"agent --resume ses_issue_703\"}}"
            );
        }
        "mismatched_native_session" => {
            eprintln!(
                "formal-ai: orchestration-session-json:{{\"id\":\"ses_fresh_703\",\"resume_command\":\"agent --resume ses_fresh_703\"}}"
            );
        }
        "delayed_success" => {
            std::thread::sleep(Duration::from_millis(150));
            fs::write("README.md", "fixture change\n").unwrap();
        }
        "coordinated_success" => {
            let started = PathBuf::from(std::env::var_os(FIXTURE_STARTED_ENV).unwrap());
            let release = PathBuf::from(std::env::var_os(FIXTURE_RELEASE_ENV).unwrap());
            fs::write(started, "started\n").unwrap();
            let deadline = Instant::now() + Duration::from_secs(10);
            while !release.exists() {
                assert!(
                    Instant::now() < deadline,
                    "fixture release was not signalled"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
            fs::write("README.md", "fixture change\n").unwrap();
        }
        "timeout" => std::thread::sleep(Duration::from_millis(250)),
        #[cfg(unix)]
        "descendant_timeout" => {
            // The descendant proves it is alive by writing a file, but only
            // after a delay far longer than any plausible lateness in the
            // timeout that is supposed to kill it, and far longer than the
            // window the test waits for it to disappear. Issue #1021: the
            // previous 150 ms delay made that assertion a race with the
            // runner's scheduler, and a loaded macOS runner won it (CI/CD
            // Pipeline run 32272689475, job 96137354605).
            let mut descendant = Command::new("sh")
                .args(["-c", "sleep 20; printf escaped > descendant-survived"])
                .spawn()
                .unwrap();
            // Record the pid before waiting: it lets the test ask the kernel
            // whether the descendant outlived the group kill instead of
            // inferring termination from a file that may simply not have been
            // written yet.
            fs::write("descendant-pid", descendant.id().to_string()).unwrap();
            std::thread::sleep(Duration::from_secs(30));
            // Unreachable while the behaviour under test holds -- the timeout
            // kills this fixture long before the sleep above returns. It is
            // here so the child is reaped on the paths where the timeout does
            // not arrive, which is what `clippy::zombie_processes` asks for.
            descendant.wait().unwrap();
        }
        "failed" => std::process::exit(7),
        "failed_change" => {
            fs::write("README.md", "failed agent change\n").unwrap();
            std::process::exit(7);
        }
        "assert_pwd" => {
            assert_eq!(
                std::env::var_os("PWD").map(PathBuf::from),
                Some(std::env::current_dir().unwrap())
            );
        }
        _ => panic!("unknown fixture mode"),
    }
}

pub(super) fn seed_registry_pins_each_clients_exact_native_resume_contract() {
    let expected = [
        (
            "agent",
            "agent --resume session-703 --no-fork",
            &["--resume", "{session_id}", "--no-fork"][..],
        ),
        (
            "claude",
            "claude --resume session-703",
            &["--resume", "{session_id}"][..],
        ),
        (
            "codex",
            "codex exec resume session-703",
            &["resume", "{session_id}"][..],
        ),
        (
            "gemini",
            "gemini --resume session-703",
            &["--resume", "{session_id}"][..],
        ),
        (
            "opencode",
            "opencode --session session-703",
            &["--session", "{session_id}"][..],
        ),
        (
            "qwen",
            "qwen --resume session-703",
            &["--resume", "{session_id}"][..],
        ),
    ];
    let integrations = formal_ai::seed::client_integrations();

    for (cli, display, argv) in expected {
        let integration = integrations
            .iter()
            .find(|integration| integration.id == cli)
            .unwrap_or_else(|| panic!("missing {cli} adapter"));
        assert_eq!(
            integration
                .invocation
                .resume_command
                .replace("{session_id}", "session-703"),
            display
        );
        assert_eq!(
            integration
                .invocation
                .resume_args
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            argv
        );
    }
}
