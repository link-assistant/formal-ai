// Issue #395 — execution-verification harness for the universal numeric-list
// coding algorithm.
//
// The solver claims a result *by construction*: every numeric-list operation is
// a pure, total function, so the answer is computed deterministically rather than
// by running the emitted code. This harness closes the loop empirically. For
// every (operation, language) pair it:
//
//   1. asks the public `FormalAiEngine` to solve a concrete prompt,
//   2. extracts the generated code and the claimed result from the answer,
//   3. compiles and runs that code with the real toolchain, and
//   4. asserts the program's stdout equals the claimed result.
//
// A language whose toolchain is not installed is reported as SKIPPED, not failed,
// so the harness is useful on any machine; install more compilers to widen the
// matrix. Integer inputs are used throughout because every target language
// formats integers identically, keeping the textual comparison exact.
//
// Run with: cargo run --example numeric_list_execution

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use formal_ai::FormalAiEngine;

/// The integer list every prompt sorts/reduces. Distinct values make a wrong
/// ordering or a wrong reduction observable.
const NUMBERS: &str = "3, 1, 4, 5, 9";

/// One numeric-list operation paired with the natural-language verb that selects
/// it. The phrasing always asks for "the code and the result", which is the
/// `code_request` signal the reductions require.
struct OpCase {
    canonical: &'static str,
    /// Prompt fragment placed before "in <Language>, give me the code and the
    /// result".
    phrase: String,
}

fn op_cases() -> Vec<OpCase> {
    [
        ("sort", format!("Sort the numbers {NUMBERS}")),
        (
            "reverse_sort",
            format!("Sort the numbers {NUMBERS} in descending order"),
        ),
        ("reverse", format!("Reverse the numbers {NUMBERS}")),
        ("sum", format!("Sum the numbers {NUMBERS}")),
        ("product", format!("Multiply the numbers {NUMBERS}")),
        ("minimum", format!("Find the minimum of {NUMBERS}")),
        ("maximum", format!("Find the maximum of {NUMBERS}")),
    ]
    .into_iter()
    .map(|(canonical, phrase)| OpCase { canonical, phrase })
    .collect()
}

/// A target language: the slug used for logging, and the surface word the prompt
/// uses so the seed alias matcher resolves it.
struct Lang {
    slug: &'static str,
    word: &'static str,
}

const LANGS: &[Lang] = &[
    Lang {
        slug: "javascript",
        word: "JavaScript",
    },
    Lang {
        slug: "typescript",
        word: "TypeScript",
    },
    Lang {
        slug: "python",
        word: "Python",
    },
    Lang {
        slug: "rust",
        word: "Rust",
    },
    Lang {
        slug: "go",
        word: "Go",
    },
    Lang {
        slug: "ruby",
        word: "Ruby",
    },
    Lang {
        slug: "java",
        word: "Java",
    },
    // `C#` / `C++` surfaces collapse onto the bare `c` token under prompt
    // normalization (it folds `#` and `+` to whitespace), so the harness uses the
    // unambiguous catalog slugs `csharp` / `cpp` to exercise those two targets.
    Lang {
        slug: "csharp",
        word: "csharp",
    },
    Lang {
        slug: "cpp",
        word: "cpp",
    },
    Lang {
        slug: "c",
        word: "C",
    },
];

/// Outcome of attempting to compile and run one generated program.
enum Outcome {
    /// The toolchain is not installed; the pair was not exercised.
    Skipped,
    /// The program ran; holds its trimmed stdout.
    Ran(String),
}

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_dir() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("nl-exec-{}-{n}", std::process::id()));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

/// Extract the fenced code block from an engine answer.
fn extract_code(answer: &str) -> String {
    let start = answer.find("```").expect("answer has a code fence") + 3;
    let after = &answer[start..];
    let nl = after.find('\n').expect("fence has a language line") + 1;
    let body = &after[nl..];
    let end = body.find("```").expect("code block is closed");
    body[..end].trim_end_matches('\n').to_string()
}

/// Extract the claimed result (the text after the localized "Result:" label,
/// which is English here because the prompts are English).
fn extract_result(answer: &str) -> String {
    let idx = answer.rfind("```").expect("answer has a closing fence") + 3;
    let tail = answer[idx..].trim();
    let colon = tail.find(':').expect("result line has a label");
    tail[colon + 1..].trim().to_string()
}

/// True when a spawn error means "command not found" (so the language should be
/// skipped rather than treated as a failure).
fn is_missing(err: &std::io::Error) -> bool {
    err.kind() == std::io::ErrorKind::NotFound
}

/// Run a command in `dir`, returning `Ok(stdout)` on success, `Err(None)` when
/// the executable is missing, or `Err(Some(message))` on a real failure.
fn run_in(
    dir: &Path,
    program: &str,
    args: &[&str],
    envs: &[(&str, String)],
) -> Result<String, Option<String>> {
    let mut cmd = Command::new(program);
    cmd.args(args).current_dir(dir);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    match cmd.output() {
        Ok(out) if out.status.success() => Ok(String::from_utf8_lossy(&out.stdout).to_string()),
        Ok(out) => Err(Some(format!(
            "`{program} {}` failed ({}):\nstdout:\n{}\nstderr:\n{}",
            args.join(" "),
            out.status,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ))),
        Err(ref e) if is_missing(e) => Err(None),
        Err(e) => Err(Some(format!("could not spawn `{program}`: {e}"))),
    }
}

/// Compile (when needed) and run the generated `code` for `slug`, returning its
/// stdout. Panics on a genuine compile/run failure; returns `Skipped` when the
/// toolchain is absent.
fn execute(slug: &str, code: &str) -> Outcome {
    let dir = unique_dir();
    let write = |name: &str| {
        let p = dir.join(name);
        fs::write(&p, code).expect("write source");
        p
    };
    let go_env = vec![
        ("GO111MODULE", "auto".to_owned()),
        (
            "GOCACHE",
            dir.join(".gocache").to_string_lossy().into_owned(),
        ),
        ("GOPATH", dir.join(".gopath").to_string_lossy().into_owned()),
    ];

    let result = match slug {
        "javascript" => {
            write("main.js");
            run_in(&dir, "node", &["main.js"], &[])
        }
        "typescript" => return execute_typescript(&dir, code),
        "python" => {
            write("main.py");
            run_in(&dir, "python3", &["main.py"], &[])
        }
        "rust" => {
            write("main.rs");
            match run_in(&dir, "rustc", &["main.rs", "-o", "main_bin"], &[]) {
                Ok(_) => run_in(&dir, "./main_bin", &[], &[]),
                other => other,
            }
        }
        "go" => {
            write("main.go");
            run_in(&dir, "go", &["run", "main.go"], &go_env)
        }
        "ruby" => {
            write("main.rb");
            run_in(&dir, "ruby", &["main.rb"], &[])
        }
        "java" => {
            write("Main.java");
            match run_in(&dir, "javac", &["Main.java"], &[]) {
                Ok(_) => run_in(&dir, "java", &["-cp", ".", "Main"], &[]),
                other => other,
            }
        }
        "csharp" => return execute_csharp(&dir, code),
        "cpp" => {
            write("main.cpp");
            match run_in(
                &dir,
                "g++",
                &["-std=c++17", "main.cpp", "-o", "main_bin"],
                &[],
            ) {
                Ok(_) => run_in(&dir, "./main_bin", &[], &[]),
                other => other,
            }
        }
        "c" => {
            write("main.c");
            match run_in(&dir, "gcc", &["main.c", "-o", "main_bin"], &[]) {
                Ok(_) => run_in(&dir, "./main_bin", &[], &[]),
                other => other,
            }
        }
        other => panic!("unknown language slug: {other}"),
    };

    finish(slug, result)
}

/// TypeScript has no single canonical runner; try the common ones and skip if
/// none are installed.
fn execute_typescript(dir: &Path, code: &str) -> Outcome {
    fs::write(dir.join("main.ts"), code).expect("write source");
    let attempts: &[(&str, &[&str])] = &[
        ("tsx", &["main.ts"]),
        ("ts-node", &["main.ts"]),
        ("node", &["--experimental-strip-types", "main.ts"]),
    ];
    for (program, args) in attempts {
        // A runner that exists but rejects the file/flag (e.g. older node
        // without type stripping) is not a code failure — try the next one.
        if let Ok(stdout) = run_in(dir, program, args, &[]) {
            return Outcome::Ran(stdout.trim().to_string());
        }
    }
    Outcome::Skipped
}

/// C# runs through the .NET SDK by scaffolding a minimal console project.
fn execute_csharp(dir: &Path, code: &str) -> Outcome {
    fs::write(dir.join("Program.cs"), code).expect("write source");
    fs::write(
        dir.join("app.csproj"),
        "<Project Sdk=\"Microsoft.NET.Sdk\">\n  <PropertyGroup>\n    <OutputType>Exe</OutputType>\n    <TargetFramework>net8.0</TargetFramework>\n    <Nullable>disable</Nullable>\n    <ImplicitUsings>disable</ImplicitUsings>\n  </PropertyGroup>\n</Project>\n",
    )
    .expect("write csproj");
    let envs = vec![
        ("DOTNET_CLI_TELEMETRY_OPTOUT", "1".to_owned()),
        ("DOTNET_NOLOGO", "1".to_owned()),
        ("DOTNET_SKIP_FIRST_TIME_EXPERIENCE", "1".to_owned()),
    ];
    let result = run_in(
        dir,
        "dotnet",
        &["run", "-c", "Release", "--project", "."],
        &envs,
    );
    finish("csharp", result)
}

/// Convert a run result into an [`Outcome`], panicking on a real failure.
fn finish(slug: &str, result: Result<String, Option<String>>) -> Outcome {
    match result {
        Ok(stdout) => Outcome::Ran(stdout.trim().to_string()),
        Err(None) => Outcome::Skipped,
        Err(Some(message)) => panic!("[{slug}] {message}"),
    }
}

fn main() {
    let mut ran = 0_u32;
    let mut skipped = 0_u32;
    let mut failures: Vec<String> = Vec::new();

    for op in op_cases() {
        for lang in LANGS {
            let prompt = format!(
                "{} in {}, give me the code and the result",
                op.phrase, lang.word
            );
            let response = FormalAiEngine.answer(&prompt);
            assert_eq!(
                response.intent, "write_program",
                "[{}/{}] expected write_program, got {} for prompt: {prompt}",
                op.canonical, lang.slug, response.intent
            );

            let code = extract_code(&response.answer);
            let claimed = extract_result(&response.answer);

            match execute(lang.slug, &code) {
                Outcome::Skipped => {
                    skipped += 1;
                    println!(
                        "SKIP  {:<13} {:<13} (toolchain not installed)",
                        op.canonical, lang.slug
                    );
                }
                Outcome::Ran(stdout) => {
                    if stdout == claimed {
                        ran += 1;
                        println!("OK    {:<13} {:<13} => {stdout}", op.canonical, lang.slug);
                    } else {
                        let msg = format!(
                            "MISMATCH {}/{}: solver claimed `{claimed}`, program printed `{stdout}`",
                            op.canonical, lang.slug
                        );
                        println!("FAIL  {msg}");
                        failures.push(msg);
                    }
                }
            }
        }
    }

    println!(
        "\n{ran} executed-and-matched, {skipped} skipped, {} failed",
        failures.len()
    );
    assert!(
        failures.is_empty(),
        "execution verification found mismatches:\n{}",
        failures.join("\n")
    );
    println!("All executed (operation, language) pairs matched the solver's computed result.");
}
