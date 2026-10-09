use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::{ChatMessage, ToolCall};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
const ORIGINAL: &str = "First, create file a.txt with exactly this content «alpha». Second, create file b.txt with exactly this content «beta».";
static SERIAL: AtomicU64 = AtomicU64::new(0);
struct Run {
    root: PathBuf,
    receipts: Vec<(String, String)>,
    writes: usize,
    answer: Option<String>,
}
impl Drop for Run {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
fn run(task: &str, tools: &[&str]) -> Run {
    run_with_sources(task, tools, &[], successful_source_receipt)
}
fn successful_source_receipt(stdout: &str) -> String {
    format!("Output: {stdout}\nExit Code: 0")
}
fn json_source_receipt(stdout: &str) -> String {
    serde_json::json!({"stdout": stdout, "exit_code": 0}).to_string()
}
fn run_with_sources(
    task: &str,
    tools: &[&str],
    sources: &[(&str, &str)],
    receipt: fn(&str) -> String,
) -> Run {
    run_with_receipts(task, tools, sources, receipt, false)
}
fn run_with_receipts(
    task: &str,
    tools: &[&str],
    sources: &[(&str, &str)],
    receipt: fn(&str) -> String,
    inject_digest: bool,
) -> Run {
    let root = std::env::temp_dir().join(format!(
        "formal-ai-obligation-{}-{}",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&root).expect("fixture directory");
    for (path, bytes) in sources {
        let target = root.join(path);
        fs::create_dir_all(target.parent().expect("fixture parent")).expect("parent directory");
        fs::write(target, bytes).expect("fixture preimage");
    }
    let mut run = Run {
        root,
        receipts: Vec::new(),
        writes: 0,
        answer: None,
    };
    let mut messages = vec![ChatMessage::user(task)];
    for turn in 0..8 {
        match plan_chat_step(&messages, tools) {
            Some(AgenticPlan::ToolCalls(calls)) => {
                for (index, call) in calls.into_iter().enumerate() {
                    let args: serde_json::Value =
                        serde_json::from_str(&call.arguments).expect("arguments");
                    let result = if call.tool == "write" {
                        let target = run.root.join(args["path"].as_str().expect("target"));
                        fs::create_dir_all(target.parent().expect("target parent"))
                            .expect("write parent directory");
                        fs::write(target, args["content"].as_str().expect("bytes"))
                            .expect("target write");
                        run.writes += 1;
                        String::new()
                    } else if call.tool == "read" {
                        let path = args["path"].as_str().expect("read target");
                        match fs::read_to_string(run.root.join(path)) {
                            Ok(bytes) => bytes,
                            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                                "Error: ENOENT: no such file or directory".to_owned()
                            }
                            Err(error) => {
                                serde_json::json!({"error": error.to_string()}).to_string()
                            }
                        }
                    } else if call.tool == "bash" {
                        let command = args["command"].as_str().expect("command");
                        let output = Command::new("bash")
                            .args(["-c", command])
                            .current_dir(&run.root)
                            .output()
                            .expect("bounded fixture shell");
                        let stdout = String::from_utf8(output.stdout).expect("UTF8 stdout");
                        let result = if (command.starts_with("cat ")
                            || (inject_digest && command.starts_with("sha256sum -- ")))
                            && output.status.success()
                        {
                            receipt(&stdout)
                        } else {
                            format!(
                                "Output: {stdout}\nExit Code: {}",
                                output.status.code().unwrap_or(1)
                            )
                        };
                        if command.starts_with("cat ")
                            || (inject_digest && command.starts_with("sha256sum -- "))
                        {
                            run.receipts.push((command.to_owned(), stdout));
                        }
                        result
                    } else {
                        panic!("unsupported fixture primitive {}", call.tool);
                    };
                    let identity = format!("step-{turn}-{index}");
                    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                        &identity,
                        &call.tool,
                        call.arguments,
                    )]));
                    messages.push(ChatMessage::tool_result(identity, call.tool, result));
                }
            }
            Some(AgenticPlan::Final(answer)) => {
                run.answer = Some(answer);
                break;
            }
            None => break,
        }
    }
    run
}
fn exact_pair(run: &Run, first: &str, second: &str) {
    assert_eq!(
        fs::read_to_string(run.root.join("a.txt")).expect("first file"),
        first
    );
    assert_eq!(
        fs::read_to_string(run.root.join("b.txt")).expect("second file"),
        second
    );
    assert!(
        run.receipts
            .contains(&("cat a.txt".to_owned(), first.to_owned()))
    );
    assert!(
        run.receipts
            .contains(&("cat b.txt".to_owned(), second.to_owned()))
    );
    assert_eq!(run.writes, 2);
    assert!(run.answer.is_some());
}
#[test]
fn unchanged_original_delivers_both_bytes_and_independent_receipts() {
    exact_pair(&run(ORIGINAL, &["bash", "write"]), "alpha", "beta");
}
#[test]
fn keywords_inside_every_literal_obligation_remain_bytes() {
    let quote = char::from(96);
    let first = format!("When {quote}input{quote} then {quote}output{quote}");
    let second = "rename a.txt to gone.txt and replace alpha with wrong";
    let request = format!(
        "First, create file a.txt with exactly this content «{first}». Second, create file b.txt with exactly this content «{second}»."
    );
    let outcome = run(&request, &["bash", "write"]);
    exact_pair(&outcome, &first, second);
    assert!(!outcome.root.join("gone.txt").exists());
}
#[test]
fn write_only_target_is_retained_without_fabricating_second_artifact() {
    let outcome = run(ORIGINAL, &["write"]);
    assert_eq!(
        fs::read_to_string(outcome.root.join("a.txt")).expect("first target"),
        "alpha"
    );
    assert!(!outcome.root.join("b.txt").exists());
    assert_eq!(outcome.writes, 1);
    assert!(
        !outcome
            .answer
            .as_deref()
            .expect("honest gap")
            .contains("Completed the general change request")
    );
}
#[test]
fn mandatory_unknown_clause_is_a_gap_after_first_verified_artifact() {
    let outcome = run(
        "First, create file a.txt with exactly this content «alpha». Second, derive the missing compiler contract.",
        &["bash", "write"],
    );
    assert_eq!(
        fs::read_to_string(outcome.root.join("a.txt")).expect("first artifact"),
        "alpha"
    );
    assert!(
        outcome
            .receipts
            .contains(&("cat a.txt".to_owned(), "alpha".to_owned()))
    );
    let answer = outcome.answer.as_deref().expect("mandatory gap");
    assert!(answer.contains("no_artifact_in_clause"));
    assert!(!answer.contains("Completed the general change request"));
}

#[test]
fn closed_fence_enumeration_payload_has_one_real_exact_receipt() {
    let quote = char::from(96).to_string().repeat(12);
    let body = "λ🙂 Alpha.\nNext beta.\nWhen «input» then «output».\nAlso list behavior rules.\n";
    let request =
        format!("Set the contents of a.txt to exactly this content:\n{quote}text\n{body}{quote}");
    let outcome = run(&request, &["bash", "write"]);
    assert_eq!(
        fs::read_to_string(outcome.root.join("a.txt")).expect("literal bytes"),
        body
    );
    assert_eq!(outcome.writes, 1);
    assert_eq!(
        outcome.receipts,
        vec![("cat a.txt".to_owned(), body.to_owned())]
    );
    assert!(outcome.answer.is_some());
}
#[test]
fn outer_literal_artifacts_keep_inner_enumeration_payloads() {
    let first = "λ🙂 Alpha. Next beta.";
    let second = "Gamma. Then delta.";
    let request = format!(
        "First, create a.txt with exactly this content «{first}». Second, create b.txt with exactly this content «{second}»."
    );
    exact_pair(&run(&request, &["bash", "write"]), first, second);
}

#[test]
fn source_readback_retains_unicode_terminal_newline_and_typed_json_stdout() {
    let source = "λ🙂 old old\n";
    for receipt in [
        successful_source_receipt as fn(&str) -> String,
        json_source_receipt,
    ] {
        let outcome = run_with_sources(
            "In f.txt replace every «old» with «new».",
            &["read", "write", "bash"],
            &[("f.txt", source)],
            receipt,
        );
        assert_eq!(
            fs::read_to_string(outcome.root.join("f.txt")).expect("target"),
            "λ🙂 new new\n"
        );
        assert_eq!(
            outcome.receipts,
            vec![("cat f.txt".to_owned(), "λ🙂 new new\n".to_owned())]
        );
        let answer = outcome.answer.as_deref().expect("final answer");
        assert!(!answer.contains("Verification failed"));
    }
}
#[test]
fn typed_failed_readback_cannot_certify_correct_physical_source_write() {
    fn failed_receipt(stdout: &str) -> String {
        serde_json::json!({"stdout": stdout, "exit_code": 1}).to_string()
    }
    fn denied_receipt(stdout: &str) -> String {
        serde_json::json!({"stdout": stdout, "exit_code": 0, "is_error": true, "error": "denied"})
            .to_string()
    }
    fn bare_receipt(_stdout: &str) -> String {
        // Bare source bytes remain neither a successful byte receipt nor a digest.
        "new new\n".to_owned()
    }
    for receipt in [
        failed_receipt as fn(&str) -> String,
        denied_receipt,
        bare_receipt,
    ] {
        let outcome = run_with_receipts(
            "In f.txt replace every «old» with «new».",
            &["read", "write", "bash"],
            &[("f.txt", "old old\n")],
            receipt,
            true,
        );
        assert_eq!(
            fs::read_to_string(outcome.root.join("f.txt")).expect("target"),
            "new new\n"
        );
        assert!(
            outcome
                .answer
                .as_deref()
                .expect("final answer")
                .contains("Verification failed")
        );
    }
}
#[test]
fn structured_source_insertion_accepts_actual_exact_receipt() {
    let source = "const ITEMS: &[&str] = &[\"a\", \"b\"];\n";
    let outcome = run_with_sources(
        "In f.rs add «c» to the list ITEMS alongside «a» and «b».",
        &["read", "write", "bash"],
        &[("f.rs", source)],
        json_source_receipt,
    );
    let expected = "const ITEMS: &[&str] = &[\"a\", \"b\", \"c\"];\n";
    assert_eq!(
        fs::read_to_string(outcome.root.join("f.rs")).expect("target"),
        expected
    );
    assert_eq!(
        outcome.receipts,
        vec![("cat f.rs".to_owned(), expected.to_owned())]
    );
    assert!(
        !outcome
            .answer
            .as_deref()
            .expect("final answer")
            .contains("Verification failed")
    );
}
#[test]
fn composite_source_and_registration_have_independent_exact_receipts() {
    let outcome = run_with_sources(
        "Create f.rs containing a public Rust function named held_out_value returning 29 and register the module in lib.rs.",
        &["read", "write", "bash"],
        &[("lib.rs", "pub mod base;\n")],
        successful_source_receipt,
    );
    let source = "pub fn held_out_value() -> i64 {\n    29\n}\n";
    let registration = "pub mod base;\npub mod f;\n";
    assert_eq!(
        fs::read_to_string(outcome.root.join("f.rs")).expect("source"),
        source
    );
    assert_eq!(
        fs::read_to_string(outcome.root.join("lib.rs")).expect("registration"),
        registration
    );
    assert_eq!(
        outcome.receipts,
        vec![
            ("cat f.rs".to_owned(), source.to_owned()),
            ("cat lib.rs".to_owned(), registration.to_owned())
        ]
    );
    assert_eq!(outcome.writes, 2);
    assert!(
        !outcome
            .answer
            .as_deref()
            .expect("final answer")
            .contains("Verification failed")
    );
}

#[test]
fn structured_stdout_cannot_certify_a_pretty_printed_source_representation() {
    fn object_receipt(_stdout: &str) -> String {
        let value = serde_json::json!({"a": 1});
        serde_json::json!({"stdout": value, "exit_code": 0}).to_string()
    }
    let expected = "{\n  \"a\": 1\n}";
    let request = format!("In f.json replace «old» with «{expected}».");
    let outcome = run_with_receipts(
        &request,
        &["read", "write", "bash"],
        &[("f.json", "old")],
        object_receipt,
        true,
    );
    assert_eq!(
        fs::read_to_string(outcome.root.join("f.json")).expect("actual authored bytes"),
        expected
    );
    assert_eq!(
        outcome.receipts,
        vec![(
            "sha256sum -- f.json".to_owned(),
            format!("{}  f.json\n", formal_ai::sha256_hex(expected.as_bytes())),
        )]
    );
    assert!(
        outcome
            .answer
            .as_deref()
            .expect("final answer")
            .contains("Verification failed")
    );
}

#[test]
fn unchanged_l21_preserves_source_and_both_independent_records() {
    let task = include_str!("l21-original-task.txt");
    let source = include_str!("l21-original-source.txt");
    let target = "rust/src/solver_handler_how_synthesis.rs";
    let outcome = run_with_sources(
        task,
        &["read", "write", "bash"],
        &[(target, source)],
        successful_source_receipt,
    );
    assert_eq!(
        fs::read_to_string(outcome.root.join(target)).expect("changed source"),
        source.replace(
            "FORMAL_AI_SOURCE_CACHE_DIR",
            "FORMAL_AI_HOW_SOURCE_CACHE_DIR"
        )
    );
    let effect = fs::read_to_string(
        outcome
            .root
            .join("agent-ladder-effects/node-2.1.2.1.1.lino"),
    )
    .expect("independent effect");
    for field in ["node_path=2.1.2.1.1", "node_depth=5", "node_kind=leaf"] {
        assert!(effect.lines().any(|line| line == field));
    }
    let result = effect
        .lines()
        .find_map(|line| line.strip_prefix("result="))
        .expect("result field");
    assert!(result.split_whitespace().count() >= 4);
    assert!(result.contains("FORMAL_AI_HOW_SOURCE_CACHE_DIR"));
    let proof = fs::read_to_string(outcome.root.join(".agent-ladder/node-2.1.2.1.1-proof.md"))
        .expect("independent proof");
    assert_eq!(proof.lines().next(), Some("node_path=2.1.2.1.1"));
    assert!(proof.contains("FORMAL_AI_HOW_SOURCE_CACHE_DIR"));
    assert!(outcome.receipts.contains(&(
        "cat agent-ladder-effects/node-2.1.2.1.1.lino".to_owned(),
        effect
    )));
    assert!(outcome.receipts.contains(&(
        "cat .agent-ladder/node-2.1.2.1.1-proof.md".to_owned(),
        proof
    )));
    assert_eq!(outcome.writes, 3);
    assert!(outcome.answer.is_some());
}

#[test]
fn failed_record_observation_blocks_the_other_independent_delivery() {
    fn wrong_receipt(_: &str) -> String {
        serde_json::json!({"stdout": "wrong", "exit_code": 0}).to_string()
    }
    fn nonzero_receipt(stdout: &str) -> String {
        serde_json::json!({"stdout": stdout, "exit_code": 1}).to_string()
    }
    for receipt in [wrong_receipt as fn(&str) -> String, nonzero_receipt] {
        let source = include_str!("l21-original-source.txt");
        let target = "rust/src/solver_handler_how_synthesis.rs";
        let outcome = run_with_sources(
            include_str!("l21-original-task.txt"),
            &["read", "write", "bash"],
            &[(target, source)],
            receipt,
        );
        assert_eq!(
            fs::read_to_string(outcome.root.join(target)).expect("actual source effect"),
            source.replace(
                "FORMAL_AI_SOURCE_CACHE_DIR",
                "FORMAL_AI_HOW_SOURCE_CACHE_DIR"
            )
        );
        assert!(
            outcome
                .root
                .join(".agent-ladder/node-2.1.2.1.1-proof.md")
                .exists()
        );
        assert!(
            !outcome
                .root
                .join("agent-ladder-effects/node-2.1.2.1.1.lino")
                .exists()
        );
        assert_eq!(outcome.writes, 2);
        assert_eq!(outcome.receipts.len(), 1);
        assert!(
            outcome
                .answer
                .as_deref()
                .expect("honest failure")
                .contains("failed")
        );
    }
}

#[test]
fn native_original_memory_request_does_not_panic_or_author_literal_bytes() {
    let task = include_str!("qwen-memory-original-task.txt");
    let outcome = run(task, &["write"]);
    assert_eq!(outcome.writes, 0);
}

#[test]
fn literal_bindings_use_local_actions_with_exact_unicode_content() {
    for (task, expected) in [
        (
            "Choose where to write records.\nCreate file x.txt containing «hello».",
            "hello",
        ),
        (
            "Escribe otra cosa.\nCrea el archivo x.txt con el contenido «hola».",
            "hola",
        ),
        (
            "Напиши что-нибудь.\nСоздай файл x.txt с содержимым «привет».",
            "привет",
        ),
        (
            "Choose where to write records.\n创建 x.txt 内容为 «你好»。",
            "你好",
        ),
        ("नोट लिखो।\nबनाओ x.txt ठीक इसी सामग्री के साथ «नमस्ते»।", "नमस्ते"),
        (
            "Choose where to write records.\nx.txt में «नमस्ते» लिखो",
            "नमस्ते",
        ),
        (
            "Note İ\u{212a}𐐷 😀 café.\nCreate file x.txt containing «hello».",
            "hello",
        ),
        (
            "Read policy.md.\nNew file: x.txt, contents: «hello»",
            "hello",
        ),
    ] {
        let outcome = run(task, &["write"]);
        assert_eq!(outcome.writes, 1, "{task}");
        assert_eq!(
            fs::read_to_string(outcome.root.join("x.txt")).expect("observed target"),
            expected,
            "{task}"
        );
    }
}

#[test]
fn unrelated_write_actions_never_license_read_targets() {
    for task in [
        "Choose where to write records.\nRead file x.txt with care.",
        "Write a report elsewhere.\nRead file x.txt containing hello.",
        "Read file x.txt then write it containing hello",
        "The instruction says «write».\nRead file x.txt with care.",
    ] {
        let outcome = run(task, &["write"]);
        assert_eq!(outcome.writes, 0, "{task}");
        assert!(!outcome.root.join("x.txt").exists(), "{task}");
    }
}

#[test]
fn completed_statement_cue_cannot_authorize_another_target_token() {
    for task in [
        "Write file.\nx.txt containing «hello».",
        "Write a file.\nfolder/note-α.txt containing «hello».",
        "Создай файл.\nзаметка.txt с содержимым «привет».",
        "Crea el archivo.\nnota.txt con el contenido «hola».",
    ] {
        let outcome = run(task, &["write"]);
        assert_eq!(outcome.writes, 0, "{task}");
    }
    let outcome = run(
        "Write file folder/note-α.txt containing «hello».",
        &["write"],
    );
    assert_eq!(outcome.writes, 1);
    assert_eq!(
        fs::read_to_string(outcome.root.join("folder/note-α.txt")).expect("same-statement target"),
        "hello"
    );
}

#[test]
fn unicode_case_mapping_preserves_original_unquoted_literal_bytes() {
    for prefix in ["İİ", "İİİ", "İK𐐷 😀 café", "KK 😀 中文"] {
        let task = format!("Note {prefix}.\nCreate file x.txt containing hello.");
        let outcome = run(&task, &["write"]);
        assert_eq!(outcome.writes, 1, "{task}");
        assert_eq!(
            fs::read_to_string(outcome.root.join("x.txt")).expect("original target bytes"),
            "hello.",
            "{task}"
        );
    }
}

#[test]
fn unicode_prefixes_preserve_multilingual_literals_and_sentence_marks() {
    use formal_ai::agentic_coding::general_planner::compose_general_change_plan;
    for (instruction, expected) in [
        ("Create file x.txt containing «hello».", "hello"),
        ("Создай файл x.txt с содержимым «привет».", "привет"),
        ("Crea el archivo x.txt con el contenido «hola».", "hola"),
        ("创建 x.txt 内容为 «你好»。", "你好"),
        ("बनाओ x.txt ठीक इसी सामग्री के साथ «नमस्ते»।", "नमस्ते"),
        (
            "Create file `policy/retention.md` containing Logs are kept for ninety days; backups are kept for a year.",
            "Logs are kept for ninety days; backups are kept for a year.",
        ),
        ("Create a file new.txt containing 'hello'.", "hello"),
    ] {
        let task = format!("Note İİK𐐷 😀 café.\n{instruction}");
        let plan = compose_general_change_plan(&task).expect("literal operand");
        assert_eq!(plan.content, expected, "{task}");
    }
}

#[test]
fn objective_boundaries_remain_original_after_case_mapping() {
    use formal_ai::agentic_coding::general_planner::objective_text;
    assert_eq!(
        objective_text("Note İİK😀.\nTask: Create file x.txt containing hello"),
        "Create file x.txt containing hello"
    );
    let task = "Note İİK😀.\nCreate file x.txt containing «Task: preserve this». ";
    assert_eq!(objective_text(task), task);
}

#[test]
fn unicode_case_mapping_does_not_promote_unrelated_write_owners() {
    use formal_ai::agentic_coding::general_planner::compose_general_change_plan;
    for task in [
        "Note İİK😀.\nWrite file.\nx.txt containing «hello».",
        "Note İİK😀.\nThe instruction says «write».\nRead file x.txt containing hello.",
        "Note İİK😀.\nProduce a legitimate release with no fabricated evidence.\nAdding a bypass flag to check-self-development-release.rs is not acceptable.",
    ] {
        assert!(compose_general_change_plan(task).is_none(), "{task}");
    }
}

#[test]
fn unicode_offsets_preserve_seeded_circumfix_closers() {
    use formal_ai::agentic_coding::general_planner::compose_general_change_plan;
    for marker in ["जिसमें Gemfile.lock हो", "把 Gemfile.lock 写入"] {
        let task = format!("Note İİK 😀.\nCreate file x.txt {marker}");
        assert_eq!(
            compose_general_change_plan(&task)
                .expect("closed content lead")
                .content,
            "Gemfile.lock",
            "{task}"
        );
    }
}

#[test]
fn full_planner_fallback_never_flattens_completed_file_cues() {
    for task in [
        "Write file. folder/note.txt containing «hello».",
        "Note İİK😀.\nWrite file.\nx.txt containing «hello».",
        "The instruction says «Write file x.txt containing hello».\nRead x.txt with care.",
    ] {
        let outcome = run(task, &["write"]);
        assert_eq!(outcome.writes, 0, "{task}");
    }
}
