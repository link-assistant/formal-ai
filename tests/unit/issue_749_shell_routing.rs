//! Regression coverage for issue #749 shell-command passthrough.

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::ChatMessage;

fn shell_command(prompt: &str) -> Option<String> {
    let plan = plan_chat_step(&[ChatMessage::user(prompt)], &["exec_command"])?;
    let AgenticPlan::ToolCalls(calls) = plan else {
        return None;
    };
    let arguments: serde_json::Value = serde_json::from_str(&calls[0].arguments).unwrap();
    arguments["command"].as_str().map(str::to_owned)
}

#[test]
fn explicit_shell_forms_pass_the_complete_command_through() {
    for (prompt, expected) in [
        ("execute date", "date"),
        ("run bash: echo hi", "echo hi"),
        ("run the command wc -l foo.txt", "wc -l foo.txt"),
        ("execute ln -s a b", "ln -s a b"),
        ("execute sort foo.txt", "sort foo.txt"),
        (
            "execute arbitrary-tool --alpha beta",
            "arbitrary-tool --alpha beta",
        ),
        ("bash -c 'ls -la'", "bash -c 'ls -la'"),
        ("powershell Get-Location", "powershell Get-Location"),
        ("pwsh -c 'Get-ChildItem'", "pwsh -c 'Get-ChildItem'"),
    ] {
        assert_eq!(shell_command(prompt).as_deref(), Some(expected), "{prompt}");
    }
}

#[test]
fn natural_shell_intents_cover_file_vcs_build_and_search_tasks() {
    for (language, prompt, expected) in [
        ("English", "show current directory", "pwd"),
        (
            "en",
            "create an empty file called note.txt",
            "touch note.txt",
        ),
        ("en", "delete the file old.txt", "rm old.txt"),
        ("en", "copy a.txt to b.txt", "cp a.txt b.txt"),
        ("en", "rename old.txt to new.txt", "mv old.txt new.txt"),
        ("en", "remove the directory build", "rmdir build"),
        ("en", "create a symbolic link from a to b", "ln -s a b"),
        ("en", "show environment variables", "env"),
        ("en", "show metadata for Cargo.toml", "stat Cargo.toml"),
        ("en", "show git log", "git log"),
        ("en", "what changed in git", "git diff"),
        ("en", "commit my changes", "git commit"),
        ("en", "run the tests", "cargo test"),
        ("en", "install dependencies", "cargo fetch"),
        ("en", "build the project", "cargo build"),
        ("ru", "удали файл old.txt", "rm old.txt"),
        ("ru", "скопируй a.txt в b.txt", "cp a.txt b.txt"),
        ("hi", "फ़ाइल old.txt हटाओ", "rm old.txt"),
        ("hi", "प्रोजेक्ट बनाएँ", "cargo build"),
        ("zh", "删除文件 old.txt", "rm old.txt"),
        ("zh", "运行测试", "cargo test"),
    ] {
        assert_eq!(
            shell_command(prompt).as_deref(),
            Some(expected),
            "{language}: {prompt}"
        );
    }
}

#[test]
fn explicit_and_listing_routes_win_over_embedded_semantic_cues() {
    assert_eq!(
        shell_command("execute echo show current directory").as_deref(),
        Some("echo show current directory")
    );
    assert_eq!(
        shell_command("print a directory listing of the current working directory").as_deref(),
        Some("ls")
    );
    assert_eq!(
        shell_command("Run ls to list files here").as_deref(),
        Some("ls")
    );
}

#[test]
fn bare_web_search_imperative_is_not_mistaken_for_find_command() {
    let messages = vec![ChatMessage::user(
        "find information about the 2022 FIFA World Cup winner",
    )];
    if let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &["bash"]) {
        assert!(
            calls.iter().all(|call| call.tool != "bash"),
            "web-search prose must not become a shell call: {calls:?}"
        );
    }
}

#[test]
fn local_search_is_shell_routed_instead_of_web_searched() {
    for prompt in [
        "search for TODO in the code",
        "find FIXME in the repository",
        "grep for error in local files",
        "найди TODO в коде",
        "कोड में TODO खोजें",
        "在代码中搜索 TODO",
    ] {
        let command = shell_command(prompt).expect(prompt);
        assert!(
            command.starts_with("rg --fixed-strings -- '"),
            "{prompt}: {command}"
        );
        assert!(command.ends_with("' ."), "{prompt}: {command}");
    }
}

#[test]
fn explicit_passthrough_covers_the_full_command_taxonomy() {
    for command in [
        "pwd",
        "cd workspace",
        "ls -la",
        "whoami",
        "id",
        "date",
        "uname -a",
        "env",
        "which cargo",
        "stat Cargo.toml",
        "file Cargo.toml",
        "touch note.txt",
        "mkdir build",
        "rmdir build",
        "rm note.txt",
        "cp a.txt b.txt",
        "mv a.txt b.txt",
        "ln -s a b",
        "chmod 644 note.txt",
        "chown user note.txt",
        "cat note.txt",
        "head -n 5 note.txt",
        "tail -n 5 note.txt",
        "wc -l note.txt",
        "sort note.txt",
        "uniq note.txt",
        "cut -d: -f1 note.txt",
        "tr a-z A-Z",
        "sed s/a/b/ note.txt",
        "awk '{print $1}' note.txt",
        "grep TODO note.txt",
        "find . -name '*.rs'",
        "du -sh .",
        "df -h",
        "curl https://example.com",
        "wget https://example.com",
        "ping localhost",
        "git status",
        "git log -1",
        "git diff",
        "git add note.txt",
        "git commit -m test",
        "git branch",
        "git checkout main",
        "git push",
        "git pull",
        "npm test",
        "pnpm test",
        "yarn test",
        "bun test",
        "cargo test",
        "pip install demo",
        "poetry install",
        "make test",
        "go test ./...",
        "mvn test",
        "gradle test",
        "ps aux",
        "kill 1234",
    ] {
        let prompt = format!("execute {command}");
        assert_eq!(shell_command(&prompt).as_deref(), Some(command), "{prompt}");
    }
}

#[test]
fn passthrough_prefixes_have_language_parity() {
    for (language, prompt, expected) in [
        ("en", "execute date", "date"),
        ("en", "run env", "env"),
        ("en", "run the command stat file.txt", "stat file.txt"),
        ("en", "run shell: wc -l file.txt", "wc -l file.txt"),
        ("en", "run bash: echo hello world", "echo hello world"),
        ("en", "run powershell: Get-Date", "Get-Date"),
        ("ru", "выполни date", "date"),
        ("ru", "запусти env", "env"),
        ("ru", "выполни команду stat file.txt", "stat file.txt"),
        ("ru", "запусти команду wc -l file.txt", "wc -l file.txt"),
        ("ru", "запусти bash: echo привет мир", "echo привет мир"),
        ("ru", "запусти powershell: Get-Date", "Get-Date"),
        ("hi", "चलाओ date", "date"),
        ("hi", "चलाएँ env", "env"),
        ("hi", "कमांड चलाओ stat file.txt", "stat file.txt"),
        ("hi", "निष्पादित wc -l file.txt", "wc -l file.txt"),
        ("hi", "bash चलाओ: echo नमस्ते", "echo नमस्ते"),
        ("hi", "powershell चलाओ: Get-Date", "Get-Date"),
        ("zh", "执行 date", "date"),
        ("zh", "运行 env", "env"),
        ("zh", "执行命令 stat file.txt", "stat file.txt"),
        ("zh", "运行命令 wc -l file.txt", "wc -l file.txt"),
        ("zh", "运行 bash: echo 你好", "echo 你好"),
        ("zh", "运行 powershell: Get-Date", "Get-Date"),
    ] {
        assert_eq!(
            shell_command(prompt).as_deref(),
            Some(expected),
            "{language}: {prompt}"
        );
    }
}

#[test]
fn opencode_outer_prompt_quotes_do_not_hide_the_command() {
    assert_eq!(
        shell_command("\"execute echo ISSUE749_OPENCODE_TWO_WORDS SECOND_ARGUMENT\"").as_deref(),
        Some("echo ISSUE749_OPENCODE_TWO_WORDS SECOND_ARGUMENT"),
    );
}

#[test]
fn committed_agent_cli_session_is_byte_reproducible() {
    const TASK: &str = "execute printf 'issue-749-driver=passed\\n'";
    let committed =
        include_str!("../../docs/case-studies/issue-749/agent-cli-evidence/session.json");
    let fresh = formal_ai::agentic_coding::run_agentic_task(TASK).expect("offline replay");
    assert_eq!(
        committed.trim(),
        serde_json::to_string_pretty(&fresh.session_json())
            .expect("session JSON")
            .trim()
    );
}
