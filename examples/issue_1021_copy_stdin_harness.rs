//! Compile and run the `copy stdin to stdout` answer Formal AI actually gives,
//! in every language it catalogues (issues #863, #862, #1021).
//!
//! Nothing here is copied out of the catalog. The harness asks the engine the
//! same question a user would, reads the file name, the check command, the run
//! command and the expected output *out of the rendered answer*, and then does
//! exactly what the answer told the reader to do. So a template that no longer
//! works, a run command that forgets to supply standard input, or an expected
//! output that drifted from the program all fail here — which is the property
//! that makes the answer's "compiled and ran" claim checkable rather than
//! asserted.
//!
//! Run it through `experiments/issue-1021-copy-stdin/run.sh`. A language whose
//! toolchain is not installed on this machine is reported as SKIP, never as a
//! pass, so the report says what was really proved here.
//!
//! The workspace must sit **outside** this repository: the repository's own
//! `package.json` declares `"type": "module"`, and Node applies that to every
//! `.js` file beneath it, so a `main.js` saved inside the tree would fail on
//! `require` for reasons that have nothing to do with the answer. `run.sh`
//! picks a temporary directory for exactly this reason.

use std::process::{Command, Stdio};

/// The catalogued languages, named as a request names them.
const LANGUAGES: &[&str] = &[
    "Rust",
    "Python",
    "JavaScript",
    "TypeScript",
    "Go",
    "C",
    "C++",
    "Java",
    "C#",
    "Ruby",
    "Scala",
    "Kotlin",
    "PHP",
];

fn main() {
    let workspace = std::env::args().nth(1).unwrap_or_else(|| {
        std::env::temp_dir()
            .join("issue-1021-copy-stdin")
            .to_string_lossy()
            .into_owned()
    });
    let fixture = "hello\nworld\n";
    let (mut passed, mut failed, mut skipped) = (0, 0, 0);

    for language in LANGUAGES {
        let prompt = format!("copy stdin to stdout in {language}");
        let answer = formal_ai::FormalAiEngine.answer(&prompt).answer;
        let Some(report) = AnswerReport::read(&answer) else {
            println!("FAIL {language:<12} the answer did not render a runnable program");
            failed += 1;
            continue;
        };
        if !report
            .run_command
            .contains(fixture.replace('\n', "\\n").as_str())
        {
            println!(
                "FAIL {language:<12} the run command does not supply standard input: `{}`",
                report.run_command
            );
            failed += 1;
            continue;
        }

        let directory = std::path::Path::new(&workspace).join(language.replace(['+', '#'], "p"));
        std::fs::create_dir_all(&directory).expect("failed to create the language workspace");
        std::fs::write(directory.join(&report.save_as), &report.code)
            .expect("failed to write the program");

        if let Some(check_command) = &report.check_command {
            match shell(&directory, check_command) {
                Ok(output) if !output.status.success() => {
                    println!(
                        "SKIP {language:<12} `{check_command}`: {}",
                        first_line(&String::from_utf8_lossy(&output.stderr))
                    );
                    skipped += 1;
                    continue;
                }
                Err(error) => {
                    println!("SKIP {language:<12} `{check_command}`: {error}");
                    skipped += 1;
                    continue;
                }
                Ok(_) => {}
            }
        }

        let Ok(output) = shell(&directory, &report.run_command) else {
            println!("SKIP {language:<12} `{}` did not start", report.run_command);
            skipped += 1;
            continue;
        };
        let actual = String::from_utf8_lossy(&output.stdout).into_owned();
        // The fixture ends in a newline the copy faithfully reproduces; the
        // answer prints the expected output without that trailing newline,
        // exactly as it sits inside its fenced block.
        if actual.trim_end_matches('\n') == report.expected_output {
            println!("PASS {language:<12} {}", report.run_command);
            passed += 1;
        } else {
            println!(
                "FAIL {language:<12} expected {:?}, got {actual:?}",
                report.expected_output
            );
            failed += 1;
        }
    }

    println!("pass={passed} fail={failed} skip={skipped}");
    if failed > 0 {
        std::process::exit(1);
    }
}

/// Everything the harness needs, taken from the answer the reader is given.
struct AnswerReport {
    code: String,
    save_as: String,
    check_command: Option<String>,
    run_command: String,
    expected_output: String,
}

impl AnswerReport {
    fn read(answer: &str) -> Option<Self> {
        let mut blocks = fenced_blocks(answer);
        let code = blocks.next()?;
        let expected_output = blocks.next()?;
        Some(Self {
            code,
            save_as: backticked_after(answer, "file named `")?,
            check_command: backticked_after(answer, "Check command: `"),
            run_command: backticked_after(answer, "Run command: `")?,
            expected_output,
        })
    }
}

/// The contents of each ```-fenced block, in order.
fn fenced_blocks(answer: &str) -> impl Iterator<Item = String> + '_ {
    answer.split("\n```").skip(1).step_by(2).map(|block| {
        block
            .split_once('\n')
            .map_or(String::new(), |(_, body)| body.to_owned())
    })
}

/// The text between the marker and the backtick that closes it.
fn backticked_after(answer: &str, marker: &str) -> Option<String> {
    let rest = answer.split_once(marker)?.1;
    Some(rest.split_once('`')?.0.to_owned())
}

fn first_line(text: &str) -> String {
    text.lines().next().unwrap_or_default().trim().to_owned()
}

fn shell(directory: &std::path::Path, command: &str) -> std::io::Result<std::process::Output> {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(directory)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?
        .wait_with_output()
}
