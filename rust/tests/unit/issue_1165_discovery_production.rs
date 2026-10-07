//! Issue #1165 (E130): discovery on the production path.
//!
//! Covers the rediscoverable procedure cache: the policy seed governs what a
//! row must carry, `content_id` is recomputed so a stored address can never
//! drift, deleting the cache is total and the miss path re-runs
//! `research_coding_skill_gap`, and the embedded `ORACLE_SNAPSHOTS` bootstrap
//! answers only while the policy seed says it is active — the deletable-cache
//! contract applied to the compiled-in tier. The production wiring points the
//! issue names (the `WriteProgram` branch of the solver, whose answer
//! `plan_work_item_execution` executes) read the cache through
//! `cached_write_program`; the six-language online rediscovery run of
//! R1165-7 is still open and will be gated by `FORMAL_AI_LIVE_FETCH`.

use std::path::PathBuf;

use formal_ai::coding_research_learning::{
    CodingResearchApproval, CodingResearchGap, ResearchedCodingProcedureLedger,
};
use formal_ai::discovery_production::{
    CachedOrDiscovered, NO_DOCUMENTATION_CAPTURE, ProcedureCache, RediscoverableRecipe,
    RunContract, all_documentation_captures, bootstrap_cache_active, cached_write_program,
    documentation_captures, documentation_route_active, documented_catalog_programs, fnv1a64,
    grammar_exists, knows_language, miss_research_missing, rediscover_from_documentation,
};
use formal_ai::web_formalize::{PageBlock, formalize_page_with_context};

/// The Rust Book page the Rust Hello World is rediscovered from.
const RUST_BOOK: &str = "https://doc.rust-lang.org/book/ch01-02-hello-world.html";

/// An isolated cache file per test, so a failing test never destroys the
/// committed cache and tests never see each other's rows.
fn isolated_cache(name: &str) -> PathBuf {
    let directory = std::env::temp_dir()
        .join("formal-ai-issue-1165")
        .join(name.to_string().replace("::", "-"));
    let _unused = std::fs::remove_dir_all(&directory);
    directory
}

fn kotlin_hello_recipe() -> RediscoverableRecipe {
    RediscoverableRecipe {
        language: String::from("kotlin"),
        task: String::from("hello_world"),
        rediscovery_query: String::from(
            "kotlin hello_world verified coding procedure SPDX license",
        ),
        rediscovery_source: String::from("http://helloworldcollection.de/#Kotlin"),
        entry: String::from("fun main() {\n    println(\"Hello, World!\")\n}"),
        verified_output: String::from("Hello, World!"),
        content_id: 0,
    }
}

/// A row without a rediscovery query and source is memorization, not a cache
/// entry: `store` refuses it (R1165-3).
#[test]
fn store_rejects_procedure_without_rediscovery_fields() {
    let mut cache = ProcedureCache::load_at(&isolated_cache("reject"));
    let mut recipe = kotlin_hello_recipe();
    recipe.rediscovery_query.clear();
    recipe.rediscovery_source.clear();
    let error = cache.store(recipe).expect_err("policy must refuse the row");
    assert!(
        error.contains("rediscovery"),
        "the refusal names the missing policy fields: {error}"
    );
    assert_eq!(cache.recipes(), []);
}

/// The happy path: store, reload from disk, look up case-insensitively.
#[test]
fn store_lookup_reload_roundtrip() {
    let path = isolated_cache("roundtrip");
    let mut cache = ProcedureCache::load_at(&path);
    cache
        .store(kotlin_hello_recipe())
        .expect("policy-complete row stores");
    let reloaded = ProcedureCache::load_at(&path);
    let recipe = reloaded
        .lookup("Kotlin", "hello_world")
        .expect("lookup is case-insensitive on the language");
    assert_eq!(recipe.entry, kotlin_hello_recipe().entry);
    assert_eq!(recipe.verified_output, "Hello, World!");
    assert_eq!(
        recipe.content_id,
        RediscoverableRecipe::content_address(&recipe.entry),
        "the stored address matches the program"
    );
}

/// `content_id` is the FNV-1a of the program bytes and is recomputed on
/// store, so a caller cannot drift a stored address from its program.
#[test]
fn content_id_is_fnv1a_of_entry_and_recomputed_on_store() {
    let recipe = kotlin_hello_recipe();
    assert_eq!(
        RediscoverableRecipe::content_address(&recipe.entry),
        fnv1a64(recipe.entry.as_bytes())
    );
    let mut cache = ProcedureCache::load_at(&isolated_cache("content-id"));
    let mut poisoned = kotlin_hello_recipe();
    poisoned.content_id = u64::MAX;
    cache.store(poisoned).expect("stores");
    assert_eq!(
        cache
            .lookup("kotlin", "hello_world")
            .expect("row")
            .content_id,
        fnv1a64(recipe.entry.as_bytes()),
        "store recomputes the address from the entry"
    );

    // A file row whose stored address disagrees with its program is dropped
    // on load: it is corruption, not a cache entry, and the miss path
    // rediscovers.
    let path = isolated_cache("content-id-drift");
    let mut drifted = ProcedureCache::load_at(&path);
    drifted.store(kotlin_hello_recipe()).expect("stores");
    let text = std::fs::read_to_string(&path).expect("cache file");
    std::fs::write(&path, text.replace("content_id \"0x", "content_id \"0xf")).expect("drift");
    assert!(
        ProcedureCache::load_at(&path)
            .lookup("kotlin", "hello_world")
            .is_none(),
        "a drifted address must not answer"
    );
}

/// R1165-7's first half, offline: `delete_all` empties the cache file
/// completely — the whole cache is one deletable artifact.
#[test]
fn delete_all_empties_the_cache_file() {
    let path = isolated_cache("delete-all");
    let mut cache = ProcedureCache::load_at(&path);
    cache.store(kotlin_hello_recipe()).expect("stores");
    cache.delete_all();
    assert_eq!(cache.recipes(), []);
    let reloaded = ProcedureCache::load_at(&path);
    assert!(
        reloaded.lookup("kotlin", "hello_world").is_none(),
        "no row survives deletion"
    );
    assert!(
        std::fs::read_to_string(&path)
            .expect("the file survives as an empty cache")
            .contains("coding_procedure_cache"),
        "the empty cache keeps its document head"
    );
}

/// The miss path routes through `research_coding_skill_gap` and stores the
/// verified procedure with its rediscovery query (R1165-1), on the same
/// offline capture-transport idiom issue #919 uses: the search endpoint
/// answers with one documented page, research verifies the procedure against
/// it, and no network is touched.
#[test]
fn miss_path_runs_research_and_stores_the_verified_procedure() {
    use formal_ai::{CachedSourceClient, FetchError, SourceTransport};

    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct ServingCount {
        requests: Arc<AtomicUsize>,
    }
    impl SourceTransport for ServingCount {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            self.requests.fetch_add(1, Ordering::SeqCst);
            if url.starts_with("https://api.duckduckgo.com/") {
                return Ok(
                    br#"{"AbstractURL":"https://research.invalid/ruby-count","Heading":"Ruby iteration guide","AbstractText":"Use upto to emit each integer in an inclusive range."}"#
                        .to_vec(),
                );
            }
            if url == "https://research.invalid/ruby-count" {
                return Ok(
                    b"Formal AI coding procedure\nSPDX-License-Identifier: CC0-1.0\nTask: count_to_three\nLanguage: ruby\nOperation: verified_workspace_rewrite\nPattern: __COUNT_TO_THREE__\nReplacement: 1.upto(3) { |number| puts number }\n"
                        .to_vec(),
                );
            }
            Err(FetchError::Transport(format!("fixture_missing:{url}")))
        }
    }

    let requests = Arc::new(AtomicUsize::new(0));

    let path = isolated_cache("miss-path");
    let mut cache = ProcedureCache::load_at(&path);
    let transport = ServingCount {
        requests: Arc::clone(&requests),
    };
    // The capture store lives beside the procedure cache file, never under
    // it (the cache path is a file), and the client is online over the
    // fixture transport -- the issue #919 idiom -- since an offline client
    // answers only from captures already on disk.
    let client =
        CachedSourceClient::new(isolated_cache("miss-path-sources"), transport).with_online(true);
    let candidate = "def main\n  __COUNT_TO_THREE__\nend\n";
    let verified = "def main\n  1.upto(3) { |number| puts number }\nend\n";
    let mut gap = CodingResearchGap::for_program_task("count_to_three", "ruby");
    let mut ledger = ResearchedCodingProcedureLedger::new();
    let answer = formal_ai::discovery_production::cached_or_research(
        &mut cache,
        &mut gap,
        &mut ledger,
        &client,
        candidate,
        verified,
        &CodingResearchApproval::granted("pull_request_review"),
        "https://research.invalid/ruby-count",
    )
    .expect("research verifies the candidate");
    assert!(
        !answer.was_cached(),
        "an empty cache must route to research"
    );
    let recipe = answer.recipe();
    assert_eq!(
        recipe.entry, verified,
        "the row reuses the verified produced program"
    );
    assert_eq!(recipe.verified_output, verified);
    assert_eq!(
        recipe.rediscovery_source,
        "https://research.invalid/ruby-count"
    );
    assert!(
        !recipe.rediscovery_query.is_empty(),
        "the row names where a rediscovery starts"
    );
    assert!(
        cache.lookup("ruby", "count_to_three").is_some(),
        "the verified procedure is stored"
    );

    // The second ask is a cache hit: no research round, the row answers.
    let mut gap = CodingResearchGap::for_program_task("count_to_three", "ruby");
    let mut ledger = ResearchedCodingProcedureLedger::new();
    let answer = formal_ai::discovery_production::cached_or_research(
        &mut cache,
        &mut gap,
        &mut ledger,
        &client,
        candidate,
        verified,
        &CodingResearchApproval::granted("pull_request_review"),
        "https://research.invalid/ruby-count",
    )
    .expect("lookup answers");
    assert!(matches!(answer, CachedOrDiscovered::Cached(_)));
    assert!(
        requests.load(Ordering::SeqCst) < 6,
        "a cache hit must not re-run the research round"
    );
}

/// A grammar exists for the languages the CST seed carries and not for ones
/// it does not — the first half of the R1165-4 `knows_language` reading.
#[test]
fn grammar_exists_reads_the_cst_seed() {
    for language in ["kotlin", "php", "swift", "scala", "rust", "r"] {
        assert!(grammar_exists(language), "{language} carries a CST grammar");
    }
    assert!(!grammar_exists("latin-vulgate"));
}

/// `knows_language` is "a grammar exists and discovery found a procedure":
/// kotlin has both a grammar and a bootstrap-recorded discovery, rust has a
/// grammar and a program the documentation captures rediscover (R1165-4),
/// pascal has a grammar but no procedure, and a grammarless language never
/// answers.
#[cfg(feature = "meta-language")]
#[test]
fn knows_language_requires_grammar_and_procedure() {
    assert!(
        knows_language("kotlin"),
        "grammar plus a bootstrap-recorded discovery answers"
    );
    assert!(
        knows_language("rust"),
        "grammar plus a documentation-rediscovered program answers"
    );
    assert!(
        !knows_language("pascal"),
        "a grammar with no recorded procedure is a rediscovery away, not knowledge"
    );
    assert!(
        !knows_language("latin-vulgate"),
        "no grammar means no amount of caching answers"
    );
}

/// The bootstrap tier obeys the policy seed: while `bootstrap` carries
/// `active "true"` the embedded snapshots are the record of past discovery.
/// Flipping the seed is the deletion path the issue's wiring completes.
///
/// The Hello World snapshots the documentation route reproduces are retired
/// (Kotlin, PHP, Swift, Lua); the Kotlin factorial still answers from its
/// Rosetta Code snapshot, and Kotlin stays known through its captures.
#[test]
fn bootstrap_tier_is_governed_by_the_policy_seed() {
    use formal_ai::knowledge::CodingOracle;
    assert!(
        bootstrap_cache_active(),
        "the committed policy keeps the bootstrap active until the wiring retires it"
    );
    assert!(CodingOracle::knows_language("kotlin"));
    assert!(CodingOracle::lookup("factorial", "kotlin").is_some());
    let retired = ["kotlin", "php", "swift", "lua"]
        .into_iter()
        .filter(|language| CodingOracle::lookup("hello_world", language).is_some())
        .collect::<Vec<_>>();
    assert_eq!(retired, Vec::<&str>::new(), "retired Hello World snapshots");
}

/// R1165-1/R1165-2: the solver's `WriteProgram` branch reuses a verified
/// cache row for an unmodified request, and keeps the catalog template when
/// the prompt customised it or the cache has no row for the pair.
#[test]
fn write_program_reuses_the_cached_procedure_only_for_an_unmodified_request() {
    let mut cache = ProcedureCache::load_at(&isolated_cache("write-program"));
    cache
        .store(kotlin_hello_recipe())
        .expect("policy-complete row stores");
    let template = "fun main() {\n    println(\"Hello, world!\")\n}\n";
    let customised = "fun main() {\n    println(\"Hi\")\n}\n";
    let answers = [
        ("kotlin", template),
        ("kotlin", customised),
        ("pascal", template),
    ]
    .map(|(language, rendered)| {
        cached_write_program(&cache, language, "hello_world", template, rendered)
            .map(|recipe| recipe.entry.clone())
    });
    assert_eq!(answers, [Some(kotlin_hello_recipe().entry), None, None]);
}

/// R1165-1/R1165-2: a solve-time cache miss names what research lacks.
///
/// The policy seed's `miss_route` says research needs an expected output and
/// a reviewer's approval while a solve carries only the expected output, so
/// the solver's `WriteProgram` branch records the miss with the approval it
/// lacks instead of answering as if research had run.
///
/// Python counting to three has no documentation capture (its output is
/// not one printed literal), so its miss is the research miss.
#[test]
fn miss_route_names_what_research_lacks_on_a_solve() {
    assert_eq!(miss_research_missing(), ["reviewer_approval"]);
    let response =
        formal_ai::UniversalSolver::default().solve("Write a Python program that counts to three");
    assert!(
        response.links_notation.contains(
            "procedure_cache outcome=miss language=python task=count_to_three \
             research_missing=reviewer_approval"
        ),
        "{}",
        response.links_notation
    );
}

/// The catalog writer binds a one-line quoted literal, never a supplied code
/// block: a Markdown fence reads as backtick quotes, and taking its body as
/// the replacement nested the supplied program inside the generated output
/// call (the e1164 parity failure). The unmodified request keeps its program
/// and records its cache miss.
#[test]
fn a_supplied_code_fence_is_not_bound_as_the_hello_world_literal() {
    let response = formal_ai::UniversalSolver::default()
        .solve("write me hello world program in Rust, like this one:\n```rust\nfn main() {}\n```");
    // The program is asserted through the execution recipe, exactly; the
    // answer text around it reports host-dependent execution status.
    let recipe = response
        .execution_recipe
        .expect("a program answer carries its execution recipe");
    assert_eq!(
        recipe.source, "fn main() {\n    println!(\"Hello, world!\");\n}",
        "the fence body is not an output literal"
    );
}

/// A page query over a supplied page whose example is a Hello World program is
/// a question about the page, not a program request (issues #1163 R10, #1164
/// R1164-11): the native solver answers with the example's parts, as the
/// browser worker does.
#[test]
fn a_code_example_page_query_is_not_a_program_request() {
    let response = formal_ai::UniversalSolver::default().solve(
        "The parts of the code example:\nThe program below prints a greeting.\n\n```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```\n\nCompile it with this line.\n\nrustc hello.rs",
    );
    assert_eq!(
        response.intent, "code_example_decomposition",
        "{}",
        response.answer
    );
    assert_eq!(
        response.answer,
        "entry_point main\noutput_operation println!\nstring_literal Hello, world!"
    );
}

/// The run contract of a catalog row: the file it is saved as and the
/// commands that check and run it.
const fn contract(
    save_as: &'static str,
    commands: &'static [&'static str],
) -> RunContract<'static> {
    RunContract { save_as, commands }
}

/// R1165-1: rediscovery from the documentation captures.
///
/// Each captured page's example is decomposed, the task's expected output is
/// bound into its literal, and the program is verified before it is
/// returned: the Rust Book, go.dev and kotlinlang pages yield their programs
/// (kotlinlang's command-line page beats the tour, whose example carries a
/// comment line), the Scala book's `object hello` binds its own name in
/// place of the catalog's `Main` (its `entry_container` keyword), a program
/// that declares no name the contract can bind is refused, and a pair with no
/// capture is the research miss.
#[cfg(feature = "meta-language")]
#[test]
fn documentation_rediscovery_recomposes_the_captured_example_and_checks_the_run_contract() {
    assert!(documentation_route_active());
    let rust = rediscover_from_documentation(
        "rust",
        "hello_world",
        "Hello, world!",
        contract("main.rs", &["rustc main.rs -o main", "./main"]),
    )
    .expect("the Rust Book page yields a program");
    assert_eq!(
        rust.entry,
        "fn main() {\n    println!(\"Hello, world!\");\n}"
    );
    assert_eq!(rust.rediscovery_source, RUST_BOOK);
    assert_eq!(rust.rediscovery_query, "Rust hello world program");
    assert_eq!(rust.verified_output, "Hello, world!");
    assert_eq!(rust.content_id, fnv1a64(rust.entry.as_bytes()));
    let go = rediscover_from_documentation(
        "go",
        "hello_world",
        "Hello, world!",
        contract("main.go", &["go run main.go"]),
    )
    .expect("go.dev yields a program");
    assert_eq!(
        go.entry,
        "package main\n\nimport \"fmt\"\n\nfunc main() {\n    fmt.Println(\"Hello, world!\")\n}"
    );
    assert_eq!(
        go.rediscovery_source,
        "https://go.dev/doc/tutorial/getting-started"
    );
    let kotlin_contract = contract(
        "Main.kt",
        &[
            "kotlinc Main.kt -include-runtime -d Main.jar",
            "java -jar Main.jar",
        ],
    );
    assert_eq!(documentation_captures("kotlin", "hello_world").len(), 2);
    let kotlin =
        rediscover_from_documentation("kotlin", "hello_world", "Hello, world!", kotlin_contract)
            .expect("kotlinlang yields a program");
    assert_eq!(
        kotlin.entry,
        "fun main() {\n    println(\"Hello, world!\")\n}"
    );
    assert_eq!(
        kotlin.rediscovery_source,
        "https://kotlinlang.org/docs/command-line.html"
    );
    let greeting =
        rediscover_from_documentation("kotlin", "hello_world", "Hi there", kotlin_contract)
            .expect("the bound literal is the requirement's");
    assert_eq!(greeting.entry, "fun main() {\n    println(\"Hi there\")\n}");
    let scala = rediscover_from_documentation(
        "scala",
        "hello_world",
        "Hello, world!",
        contract("Main.scala", &["scalac Main.scala", "scala Main"]),
    )
    .expect("the Scala book's object binds its own name");
    assert_eq!(
        scala.entry,
        "object hello {\n  def main(args: Array[String]) = {\n    println(\"Hello, world!\")\n  }\n}"
    );
    assert_eq!(
        rediscover_from_documentation(
            "kotlin",
            "hello_world",
            "Hello, world!",
            contract("Main.scala", &["scalac Main.scala", "scala Main"]),
        ),
        Err(String::from("run_contract:Main")),
        "a program declaring no name the contract can bind is refused"
    );
    assert_eq!(
        rediscover_from_documentation(
            "python",
            "count_to_three",
            "1\n2\n3",
            contract("main.py", &["python3 main.py"]),
        ),
        Err(NO_DOCUMENTATION_CAPTURE.to_owned()),
        "a task no page was captured for is the research miss"
    );
    assert_eq!(
        rediscover_from_documentation(
            "rust",
            "hello_world",
            "1\n2\n3",
            contract("main.rs", &["./main"]),
        ),
        Err(String::from("no_single_line_output")),
        "a multi-line output is not a printed literal"
    );
}

/// R1165-1/R1165-2: the solver's `WriteProgram` miss answers from the
/// rediscovered program.
///
/// The derivation records `outcome=discovered` with the page and the
/// program's content address, and the execution recipe the work-item
/// executor runs carries the same program.
#[cfg(feature = "meta-language")]
#[test]
fn write_program_miss_answers_from_the_documentation() {
    let program = "fn main() {\n    println!(\"Hello, world!\");\n}";
    let response =
        formal_ai::UniversalSolver::default().solve("write me hello world program in Rust");
    let event = format!(
        "procedure_cache outcome=discovered language=rust task=hello_world \
         rediscovery_source={RUST_BOOK} content_id=0x{:016x}",
        fnv1a64(program.as_bytes())
    );
    assert!(
        response.links_notation.contains(&event),
        "{}",
        response.links_notation
    );
    let recipe = response
        .execution_recipe
        .expect("a program answer carries its execution recipe");
    assert_eq!(recipe.source, program);
}

/// R1165-6: a documented class or object name binds the file the answer
/// saves and every command, and the derivation records where each command
/// shown comes from.
///
/// Oracle's tutorial declares `class HelloWorldApp` and states `javac
/// HelloWorldApp.java` and `java HelloWorldApp`, so both commands are the
/// page's; the Scala book's `object hello` binds the catalog's commands,
/// whose source stays the catalog. php.net's `echo` prints no trailing
/// newline, and the derivation records that deviation.
#[cfg(feature = "meta-language")]
#[test]
fn a_documented_program_binds_its_run_contract_and_records_each_command_source() {
    const ORACLE: &str = "https://docs.oracle.com/javase/tutorial/getStarted/cupojava/unix.html";
    let cases = [
        (
            "Java",
            "HelloWorldApp.java",
            ["javac HelloWorldApp.java", "java HelloWorldApp"],
            [ORACLE, ORACLE],
        ),
        (
            "Scala",
            "hello.scala",
            ["scalac hello.scala", "scala hello"],
            ["catalog", "catalog"],
        ),
    ];
    for (name, path, commands, sources) in cases {
        let response = formal_ai::UniversalSolver::default()
            .solve(&format!("write me hello world program in {name}"));
        let language = name.to_ascii_lowercase();
        for ((role, command), source) in ["check", "run"].iter().zip(commands).zip(sources) {
            let event = format!(
                "command_source language={language} task=hello_world role={role} \
                 source={source} command={command}"
            );
            assert!(
                response.links_notation.contains(&event),
                "{event}: {}",
                response.links_notation
            );
        }
        let recipe = response
            .execution_recipe
            .unwrap_or_else(|| panic!("{name}: a program answer carries its recipe"));
        assert_eq!(recipe.path, path, "{name}");
        assert_eq!(recipe.commands, commands, "{name}");
    }
    let php = formal_ai::UniversalSolver::default().solve("write me hello world program in PHP");
    assert!(
        php.links_notation.contains(
            "documentation_deviation language=php task=hello_world trailing_newline=absent"
        ),
        "{}",
        php.links_notation
    );
}

/// A rediscovered row stored in a runtime cache is reused as a hit: the
/// cache answers the next unmodified request with the same program.
#[cfg(feature = "meta-language")]
#[test]
fn a_rediscovered_row_is_stored_and_reused() {
    let row = rediscover_from_documentation(
        "go",
        "hello_world",
        "Hello, world!",
        contract("main.go", &["go run main.go"]),
    )
    .expect("go.dev yields a program");
    let path = isolated_cache("rediscovered").join("cache.lino");
    let mut cache = ProcedureCache::load_at(&path);
    cache
        .store(row.clone())
        .expect("a rediscovered row is policy-complete");
    let reloaded = ProcedureCache::load_at(&path);
    assert_eq!(
        cached_write_program(&reloaded, "go", "hello_world", &row.entry, &row.entry),
        Some(&row)
    );
}

/// R1165-1: the documentation captures are pre-cached source data.
///
/// Each capture names a byte-for-byte fixture and its SHA-256, and its block
/// rows are exactly the code blocks the page formalizer reads from those
/// bytes, so no program in the seed was written by hand. The browser twin is
/// `rust/tests/web/issue-1165-documentation-captures.test.mjs`.
#[test]
fn documentation_captures_are_the_formalized_fixtures() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    let captures = all_documentation_captures();
    assert_eq!(captures.len(), 16);
    for capture in captures {
        let bytes = std::fs::read(root.join(&capture.fixture))
            .unwrap_or_else(|error| panic!("{}: {error}", capture.fixture));
        assert_eq!(
            formal_ai::sha256_hex(&bytes),
            capture.sha256,
            "{}",
            capture.fixture
        );
        let blocks: Vec<(String, String)> =
            formalize_page_with_context(&bytes, Some(&capture.mime), Some(&capture.url))
                .1
                .into_iter()
                .filter_map(|(_, block)| match block {
                    PageBlock::CodeBlock { language, text } => Some((language, text)),
                    _ => None,
                })
                .collect();
        assert_eq!(blocks, capture.blocks, "{}", capture.url);
    }
}

/// R1165-4: the seed programs retired to the documentation route are
/// reproduced by it, and the catalog compiles none of them.
///
/// `data/seed/hello-world-programs.lino` stores no program for these Hello
/// World rows (their `program_source` is the documentation route), and the
/// Rust catalog no longer compiles a template for them: its table takes the
/// program the documentation captures yield when it is first read, so the
/// solver's answer and execution recipe carry exactly that program.
#[cfg(feature = "meta-language")]
#[test]
fn retired_seed_programs_are_reproduced_by_the_documentation() {
    let retired: Vec<(String, String, String)> = documented_catalog_programs()
        .into_iter()
        .map(|program| {
            let documented = program
                .rediscovered
                .unwrap_or_else(|reason| panic!("{}: {reason}", program.language));
            (program.language, program.task, documented.recipe.entry)
        })
        .collect();
    let expected: Vec<(String, String, String)> = RETIRED_PROGRAMS
        .iter()
        .map(|(language, program, _)| {
            (
                (*language).to_owned(),
                String::from("hello_world"),
                (*program).to_owned(),
            )
        })
        .collect();
    assert_eq!(retired, expected);
    for (language, program, prompt_name) in RETIRED_PROGRAMS {
        let response = formal_ai::UniversalSolver::default()
            .solve(&format!("write me hello world program in {prompt_name}"));
        let recipe = response
            .execution_recipe
            .unwrap_or_else(|| panic!("{language}: a program answer carries its recipe"));
        assert_eq!(recipe.source, program, "{language}");
    }
}

/// `(language, program the documentation yields, name a prompt uses)` for
/// every Hello World row retired to the documentation route, in seed order.
const RETIRED_PROGRAMS: &[(&str, &str, &str)] = &[
    (
        "rust",
        "fn main() {\n    println!(\"Hello, world!\");\n}",
        "Rust",
    ),
    ("python", "print('Hello, world!')", "Python"),
    (
        "javascript",
        "console.log(\"Hello, world!\");",
        "JavaScript",
    ),
    (
        "typescript",
        "// Greets the world.\nconsole.log(\"Hello, world!\");",
        "TypeScript",
    ),
    (
        "go",
        "package main\n\nimport \"fmt\"\n\nfunc main() {\n    fmt.Println(\"Hello, world!\")\n}",
        "Go",
    ),
    (
        "c",
        "// crt_puts.c\n// This program uses puts to write a string to stdout.\n\n#include <stdio.h>\n\nint main( void )\n{\n   puts( \"Hello, world!\" );\n}",
        "C",
    ),
    (
        "cpp",
        "#include <iostream>\n\nint main()\n{\n    std::cout << \"Hello, world!\" << std::endl;\n    return 0;\n}",
        "C++",
    ),
    (
        "java",
        "/**\n * The HelloWorldApp class implements an application that\n * simply prints \"Hello World!\" to standard output.\n */\nclass HelloWorldApp {\n    public static void main(String[] args) {\n        System.out.println(\"Hello, world!\"); // Display the string.\n    }\n}",
        "Java",
    ),
    ("csharp", "Console.WriteLine(\"Hello, world!\");", "C#"),
    (
        "ruby",
        "# The famous Hello World\n# Program is trivial in\n# Ruby. Superfluous:\n#\n# * A \"main\" method\n# * Newline\n# * Semicolons\n#\n# Here is the Code:\n\nputs \"Hello, world!\"",
        "Ruby",
    ),
    (
        "scala",
        "object hello {\n  def main(args: Array[String]) = {\n    println(\"Hello, world!\")\n  }\n}",
        "Scala",
    ),
    (
        "kotlin",
        "fun main() {\n    println(\"Hello, world!\")\n}",
        "Kotlin",
    ),
    ("php", "<?php\n\necho \"Hello, world!\";\n\n?>", "PHP"),
];
