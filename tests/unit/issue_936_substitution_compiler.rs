use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use formal_ai::agentic_coding::{plan_symbolic_command_reroute, AgenticPlan};
use formal_ai::engine::ExecutionRecipe;
use formal_ai::protocol::{ChatMessage, ToolCall};
use formal_ai::{
    compile_substitution_rules, CompiledSubstitutionProgram, ConversationTurn, CrudEvent,
    ProgramPlanCompilationError, SubstitutionCompilationTarget, SubstitutionGraph,
    SubstitutionRuleSet, UniversalSolver,
};

const COUNTER_RULES: &str = r#"
substitution_rules
  id "counter_loop"
  rule "step_0_to_1"
    order "1"
    event "manual"
    when "$machine -> role:counter"
    when "machine:$id -> state:0"
    replace "machine:$id -> state:0"
      with "machine:$id -> state:1"
  rule "step_1_to_2"
    order "2"
    event "manual"
    when "machine:$id -> state:1"
    replace "machine:$id -> state:1"
      with "machine:$id -> state:2"
  rule "step_2_to_3_and_halt"
    order "3"
    event "manual"
    when "machine:$id -> state:2"
    when "machine:$id -> control:run"
    replace "machine:$id -> state:2"
      with "machine:$id -> state:3"
    replace "machine:$id -> control:run"
      with "machine:$id -> control:halt"
"#;

const NON_TERMINATING_RULES: &str = r#"
substitution_rules
  id "non_terminating_program_plan"
  rule "a_to_b"
    order "1"
    event "manual"
    when "request:task -> loop_a"
    replace "request:task -> loop_a"
      with "request:task -> loop_b"
  rule "b_to_a"
    order "2"
    event "manual"
    when "request:task -> loop_b"
    replace "request:task -> loop_b"
      with "request:task -> loop_a"
"#;

const INPUT: &str = concat!(
    "machine:counter\tcontrol:run\n",
    "machine:counter\trole:counter\n",
    "machine:counter\tstate:0\n",
);

fn interpreted_output(rules: &SubstitutionRuleSet) -> String {
    let mut graph = SubstitutionGraph::new()
        .with_link("machine:counter", "control:run")
        .with_link("machine:counter", "role:counter")
        .with_link("machine:counter", "state:0");
    let report = graph.apply_rules(rules, CrudEvent::Manual);
    assert_eq!(report.applied_count(), 3);
    assert!(!report.terminated_by_guard);
    graph
        .links()
        .iter()
        .fold(String::new(), |mut output, link| {
            output.push_str(&link.from);
            output.push('\t');
            output.push_str(&link.to);
            output.push('\n');
            output
        })
}

fn temporary_directory() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should follow the Unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("formal-ai-936-{}-{nonce}", std::process::id()));
    fs::create_dir_all(&path).expect("temporary directory should be created");
    path
}

fn write_artifact(directory: &Path, artifact: &CompiledSubstitutionProgram) -> PathBuf {
    let primary = directory.join(&artifact.primary_file.name);
    fs::write(&primary, &artifact.primary_file.contents).expect("primary file should be written");
    for file in &artifact.supporting_files {
        fs::write(directory.join(&file.name), &file.contents)
            .expect("support file should be written");
    }
    primary
}

fn run_with_input(command: &mut Command, input: &str) -> String {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("child process should start");
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(input.as_bytes())
        .expect("input should be written");
    let output = child.wait_with_output().expect("child should finish");
    assert!(
        output.status.success(),
        "command failed: {command:?}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("output should be UTF-8")
}

#[test]
fn counter_loop_executes_identically_in_rust_javascript_and_webassembly() {
    let rules = SubstitutionRuleSet::from_links_notation(COUNTER_RULES).expect("valid rules");
    let expected = interpreted_output(&rules);
    assert_eq!(
        expected,
        concat!(
            "machine:counter\tcontrol:halt\n",
            "machine:counter\trole:counter\n",
            "machine:counter\tstate:3\n",
        )
    );
    let directory = temporary_directory();

    let rust = compile_substitution_rules(&rules, SubstitutionCompilationTarget::Rust);
    let rust_source = write_artifact(&directory, &rust);
    let rust_binary = directory.join("counter_loop");
    run_with_input(
        Command::new("rustc")
            .arg("--edition=2021")
            .args(["-D", "warnings"])
            .arg(&rust_source)
            .arg("-o")
            .arg(&rust_binary),
        "",
    );
    assert_eq!(
        run_with_input(&mut Command::new(&rust_binary), INPUT),
        expected
    );

    let javascript = compile_substitution_rules(&rules, SubstitutionCompilationTarget::JavaScript);
    let javascript_source = write_artifact(&directory, &javascript);
    assert!(javascript_source.is_file());
    assert!(javascript
        .primary_file
        .contents
        .contains("substitution semantics remain in Rust/WASM"));
    assert!(!javascript.primary_file.contents.contains("applyRule"));
    let javascript_wasm_source = javascript
        .supporting_files
        .iter()
        .find(|file| file.name.ends_with("_wasm.rs"))
        .map(|file| directory.join(&file.name))
        .expect("JavaScript interop should include its canonical Rust/WASM source");
    let javascript_wasm_binary = directory.join("counter_loop_js.wasm");
    run_with_input(
        Command::new("rustc")
            .arg("--edition=2021")
            .args(["-D", "warnings"])
            .args(["--target", "wasm32-unknown-unknown"])
            .args(["--crate-type", "cdylib"])
            .args(["-C", "panic=abort"])
            .arg(javascript_wasm_source)
            .arg("-o")
            .arg(&javascript_wasm_binary),
        "",
    );
    assert_eq!(
        run_with_input(
            Command::new("node")
                .arg(javascript_source)
                .arg(javascript_wasm_binary),
            INPUT,
        ),
        expected
    );

    let webassembly =
        compile_substitution_rules(&rules, SubstitutionCompilationTarget::WebAssembly);
    let wasm_source = write_artifact(&directory, &webassembly);
    let wasm_binary = directory.join("counter_loop.wasm");
    run_with_input(
        Command::new("rustc")
            .arg("--edition=2021")
            .args(["-D", "warnings"])
            .args(["--target", "wasm32-unknown-unknown"])
            .args(["--crate-type", "cdylib"])
            .args(["-C", "panic=abort"])
            .arg(wasm_source)
            .arg("-o")
            .arg(&wasm_binary),
        "",
    );
    assert_eq!(
        run_with_input(
            Command::new("node")
                .arg(directory.join("run_substitution_wasm.mjs"))
                .arg(wasm_binary),
            INPUT,
        ),
        expected
    );

    for (artifact, target) in [
        (&rust, "rust"),
        (&javascript, "javascript"),
        (&webassembly, "webassembly"),
    ] {
        assert!(artifact.trace.contains("substitution_compilation"));
        assert!(artifact
            .trace
            .contains("verification executable_parity_required"));
        assert!(artifact.trace.contains(target));
    }

    fs::remove_dir_all(directory).expect("temporary directory should be removed");
}

#[test]
fn program_plan_exports_only_after_a_verified_finite_rewrite() {
    let modified = formal_ai::program_plan::lower("list_files", &[String::from("reverse_sort")]);
    let compiled = modified
        .compile(SubstitutionCompilationTarget::Rust)
        .expect("an applied finite program-plan rewrite should be exportable");
    assert_eq!(compiled.ir.id, "program_plan_rules");
    assert_eq!(
        compiled.ir.rules.len(),
        formal_ai::program_plan::rules().rules.len()
    );

    let unchanged = formal_ai::program_plan::lower("list_files", &[]);
    assert_eq!(
        unchanged.compile(SubstitutionCompilationTarget::Rust),
        Err(ProgramPlanCompilationError::NoVerifiedRewrite)
    );

    let looping_rules =
        SubstitutionRuleSet::from_links_notation(NON_TERMINATING_RULES).expect("valid loop");
    let looping = formal_ai::program_plan::lower_with_rules(&looping_rules, "loop_a", &[]);
    assert!(looping.report.terminated_by_guard);
    assert_eq!(
        looping.compile_with_rules(&looping_rules, SubstitutionCompilationTarget::Rust),
        Err(ProgramPlanCompilationError::TerminationGuardReached)
    );
}

struct MultilingualExportCase {
    language: &'static str,
    initial: &'static str,
    follow_up: &'static str,
    target: &'static str,
    primary_file: &'static str,
    localized_intro: &'static str,
}

fn export_code_fence(path: &str) -> &'static str {
    match Path::new(path).extension().and_then(|value| value.to_str()) {
        Some("rs") => "rust",
        Some("mjs") => "javascript",
        Some("tsv") => "text",
        _ => "json",
    }
}

fn exact_export_answer(intro: &str, recipe: &ExecutionRecipe) -> String {
    let mut answer = intro.to_owned();
    for (path, source) in std::iter::once((recipe.path.as_str(), recipe.source.as_str())).chain(
        recipe
            .supporting_files
            .iter()
            .map(|file| (file.path.as_str(), file.source.as_str())),
    ) {
        let _ = write!(
            answer,
            "\n\n`{path}`\n```{}\n{}\n```",
            export_code_fence(path),
            source.trim_end()
        );
    }
    answer.push_str("\n\n```sh\n");
    answer.push_str(&recipe.commands.join("\n"));
    answer.push_str("\n```");
    answer
}

#[test]
fn verified_program_plan_exports_are_seeded_in_four_languages() {
    let solver = UniversalSolver::default();
    let cases = [
        MultilingualExportCase {
            language: "English",
            initial: "Write me a Rust program that lists the files in the current directory",
            follow_up:
                "Sort the results in reverse order and export the substitution rule to Rust",
            target: "rust",
            primary_file: "program_plan_rules.rs",
            localized_intro: "The verified substitution-rule program is ready for rust. Save each named file exactly as shown. The input format is one `from<TAB>to` link per line; then run the commands below. Primary file: program_plan_rules.rs.",
        },
        MultilingualExportCase {
            language: "Russian",
            initial:
                "Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории",
            follow_up: "Сделай сортировку результатов в обратном порядке и экспортируй правило подстановки в JavaScript",
            target: "javascript",
            primary_file: "program_plan_rules.mjs",
            localized_intro: "Проверенная программа правил подстановки готова для javascript. Сохраните каждый файл точно под указанным именем. Формат входных данных: одна связь `from<TAB>to` на строку; затем выполните команды ниже. Основной файл: program_plan_rules.mjs.",
        },
        MultilingualExportCase {
            language: "Hindi",
            initial: "Rust में फ़ाइलों की सूची दिखाने वाला प्रोग्राम लिखो",
            follow_up: "परिणामों को उल्टे क्रम में क्रमबद्ध करो और प्रतिस्थापन नियम को WebAssembly में निर्यात करो",
            target: "webassembly",
            primary_file: "program_plan_rules_wasm.rs",
            localized_intro: "सत्यापित प्रतिस्थापन-नियम प्रोग्राम webassembly के लिए तैयार है। हर फ़ाइल को दिखाए गए नाम से सहेजें। इनपुट प्रारूप में हर पंक्ति पर एक `from<TAB>to` लिंक होता है; फिर नीचे दिए आदेश चलाएँ। मुख्य फ़ाइल: program_plan_rules_wasm.rs।",
        },
        MultilingualExportCase {
            language: "Chinese",
            initial: "用 Rust 编写一个列出当前目录中文件的程序",
            follow_up: "把结果按相反顺序排序，并将替换规则导出为 Rust",
            target: "rust",
            primary_file: "program_plan_rules.rs",
            localized_intro: "经过验证的替换规则程序已生成，可用于 rust。请按所示名称保存每个文件。输入格式为每行一个 `from<TAB>to` 链接；然后运行下面的命令。主文件：program_plan_rules.rs。",
        },
    ];

    for case in cases {
        let initial = solver.solve(case.initial);
        assert_eq!(initial.intent, "write_program", "{} setup", case.language);
        let history = [
            ConversationTurn::user(case.initial),
            ConversationTurn::assistant(initial.answer),
        ];
        let exported = solver.solve_with_history(case.follow_up, &history);
        assert_eq!(
            exported.intent, "substitution_rule_export",
            "{} export routed incorrectly: {}\n{}",
            case.language, exported.intent, exported.links_notation
        );
        assert!(
            exported.answer.starts_with(case.localized_intro),
            "{} response was not localized: {}",
            case.language,
            exported.answer
        );
        assert!(exported.answer.contains(case.primary_file));
        assert!(exported.links_notation.contains("rule_verification"));
        assert!(exported.links_notation.contains("status passed"));
        assert!(exported
            .links_notation
            .contains("verification executable_parity_required"));
        let recipe = exported
            .execution_recipe
            .expect("export should carry an executable primary artifact");
        assert_eq!(
            exported.answer,
            exact_export_answer(case.localized_intro, &recipe),
            "{} exact export transcript changed",
            case.language
        );
        assert_eq!(recipe.language, case.target);
        assert_eq!(recipe.path, case.primary_file);
        assert!(!recipe.source.is_empty());
        assert!(
            recipe
                .supporting_files
                .iter()
                .any(|file| file.path.ends_with(".substitution-ir.json")),
            "{} export omitted its IR support file",
            case.language
        );
        assert_eq!(
            recipe
                .supporting_files
                .iter()
                .find(|file| file.path == "input.tsv")
                .map(|file| file.source.as_str()),
            Some("request:modifier\treverse_sort\nrequest:task\tlist_files\n"),
            "{} export omitted the executable plan input",
            case.language
        );
        if case.target != "rust" {
            assert!(
                recipe.supporting_files.iter().any(|file| {
                    file.path.ends_with("_wasm.rs")
                        || Path::new(&file.path)
                            .extension()
                            .is_some_and(|value| value.eq_ignore_ascii_case("mjs"))
                }),
                "{} export omitted an interop support file",
                case.language
            );
        }
        assert!(!recipe.commands.is_empty());
    }
}

#[test]
fn agentic_recipe_writes_every_export_file_before_execution() {
    let solver = UniversalSolver::default();
    let initial_prompt = "Write me a Rust program that lists the files in the current directory";
    let initial = solver.solve(initial_prompt);
    let formal_ai::SymbolicAnswer {
        answer: initial_answer,
        ..
    } = initial;
    let history = [
        ConversationTurn::user(initial_prompt),
        ConversationTurn::assistant(initial_answer),
    ];
    let follow_up =
        "Sort the results in reverse order and export the substitution rule to JavaScript";
    let answer = solver.solve_with_history(follow_up, &history);
    let recipe = answer.execution_recipe.as_ref().expect("export recipe");
    let expected_paths = std::iter::once(recipe.path.as_str())
        .chain(
            recipe
                .supporting_files
                .iter()
                .map(|file| file.path.as_str()),
        )
        .collect::<Vec<_>>();
    let mut messages = vec![ChatMessage::user(follow_up)];

    for (index, expected_path) in expected_paths.iter().enumerate() {
        let plan = plan_symbolic_command_reroute(&messages, &["write", "bash"], &answer)
            .expect("recipe should plan a write");
        let AgenticPlan::ToolCalls(calls) = plan else {
            panic!("file {expected_path} was not planned");
        };
        let [call] = calls.as_slice() else {
            panic!("one file should be written at a time");
        };
        assert_eq!(call.tool, "write");
        let arguments: serde_json::Value =
            serde_json::from_str(&call.arguments).expect("valid write arguments");
        assert_eq!(arguments["filePath"], *expected_path);
        let tool_call = ToolCall::function(format!("write-{index}"), "write", &call.arguments);
        messages.push(ChatMessage::assistant_tool_calls(vec![tool_call.clone()]));
        messages.push(ChatMessage::tool_result(
            &tool_call.id,
            "write",
            format!("Wrote {expected_path}"),
        ));
    }

    for (index, expected_command) in recipe.commands.iter().enumerate() {
        let plan = plan_symbolic_command_reroute(&messages, &["write", "bash"], &answer)
            .expect("recipe should plan a command");
        let AgenticPlan::ToolCalls(calls) = plan else {
            panic!("command {expected_command} was not planned");
        };
        let [call] = calls.as_slice() else {
            panic!("one command should run at a time");
        };
        assert_eq!(call.tool, "bash");
        let arguments: serde_json::Value =
            serde_json::from_str(&call.arguments).expect("valid command arguments");
        assert_eq!(arguments["command"], *expected_command);
        let tool_call = ToolCall::function(format!("run-{index}"), "bash", &call.arguments);
        messages.push(ChatMessage::assistant_tool_calls(vec![tool_call.clone()]));
        messages.push(ChatMessage::tool_result(
            &tool_call.id,
            "bash",
            "Exit Code: 0",
        ));
    }

    assert!(matches!(
        plan_symbolic_command_reroute(&messages, &["write", "bash"], &answer),
        Some(AgenticPlan::Final(_))
    ));
}

#[test]
fn live_agent_cli_export_and_self_authored_contract_are_preserved() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let case = root.join("docs/case-studies/issue-936");
    let live_agent = fs::read_to_string(case.join("agent-cli-export-e2e/agent-cli.log"))
        .expect("live resumed Agent CLI trace");
    let live_server = fs::read_to_string(case.join("agent-cli-export-e2e/formal-ai.log"))
        .expect("live resumed formal-ai trace");

    assert!(live_agent.contains("ses_ff77cb103ffe3hhhGqf2qquEkR"));
    assert!(live_agent.contains("request:task\\tlist_files_reverse_sort"));
    assert!(live_server.contains("node program_plan_rules.mjs program_plan_rules.wasm < input.tsv"));
    for artifact in [
        "program_plan_rules.mjs",
        "program_plan_rules_wasm.rs",
        "program_plan_rules.substitution-ir.json",
        "input.tsv",
    ] {
        assert!(
            case.join("agent-cli-export-e2e").join(artifact).is_file(),
            "live export evidence omitted {artifact}"
        );
    }

    let authored =
        fs::read(case.join("self-hosting-authorship/substitution-compiler-contract.lino"))
            .expect("Agent CLI authored compiler contract");
    let canonical = fs::read(root.join("data/meta/substitution-compiler-contract.lino"))
        .expect("canonical compiler contract");
    assert_eq!(authored, canonical);
    assert!(
        fs::read_to_string(case.join("self-hosting-authorship/agent-cli.log"))
            .expect("self-authorship Agent CLI trace")
            .contains("ses_ff77c472cffej9Hmz346niSMgQ")
    );

    let workflow =
        fs::read_to_string(root.join(".github/workflows/release.yml")).expect("release workflow");
    assert!(workflow.contains("experiments/agent_cli_e2e/run_issue_936.sh"));
}
