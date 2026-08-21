//! The behaviour range of issue #1021, pinned as routing tests.
//!
//! Issue #1021 collects seven reported prompts that Formal AI answered wrongly
//! — from a bare `ls` (#868) to a Laravel request in Russian (#723) — and asks
//! for one pull request that fixes them *by generalization*: "a per-prompt fix
//! that makes one task pass is a regression against that instruction, not
//! progress".
//!
//! Every reported prompt therefore arrives here paired with **held-out
//! paraphrases** that no seed entry was written for verbatim. A fix that only
//! satisfies the reported wording leaves the paraphrases failing, so a green
//! run is evidence of generalization rather than of a memorised seed.

use formal_ai::agentic_coding::mutating_action::verified_recipe;
use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::{ChatMessage, ToolCall, UniversalSolver};

/// The shell command the agentic planner resolves for `prompt`, or `None` when
/// it routes somewhere other than a command execution tool.
///
/// A command that changes the workspace is planned as the verified recipe its
/// seed intent declares (issue #944), so the plan is driven to its end — each
/// step reported as having succeeded — and the mutating action is picked out of
/// it as the one step that is a recipe of its own. A read-only command is a
/// single-step plan and is returned unchanged.
fn shell_command(prompt: &str) -> Option<String> {
    let mut messages = vec![ChatMessage::user(prompt)];
    let mut commands: Vec<String> = Vec::new();
    while commands.len() <= MAX_PLAN_STEPS {
        let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &["exec_command"])
        else {
            break;
        };
        let call = calls.first()?;
        let arguments: serde_json::Value = serde_json::from_str(&call.arguments).unwrap();
        let command = arguments["command"].as_str()?.to_owned();
        let id = format!("call_{}", messages.len());
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            id.clone(),
            call.tool.clone(),
            call.arguments.clone(),
        )]));
        messages.push(ChatMessage::tool_result(
            id,
            &call.tool,
            format!("Command: {command}\nOutput: (empty)\nExit Code: 0"),
        ));
        commands.push(command);
    }
    commands
        .iter()
        .find(|command| verified_recipe(command).is_some())
        .or_else(|| commands.first())
        .cloned()
}

/// A plan longer than this is a runaway, not an answer; the longest recipe the
/// seed declares is six steps.
const MAX_PLAN_STEPS: usize = 12;

/// Issue #866 and #867: *"Execute ls command"* ran `/bin/ls command`, which
/// fails with *cannot access 'command'*. The noun naming the command is part of
/// the request, not of the command line.
#[test]
fn a_command_naming_noun_is_not_an_argument() {
    for (language, prompt) in [
        // Reported.
        ("en", "Execute ls command"),
        // Held out.
        ("en", "Execute the ls command"),
        ("en", "run ls command"),
        ("en", "Run the ls command please"),
        ("ru", "выполни команду ls"),
        ("ru", "запусти ls команду"),
    ] {
        assert_eq!(
            shell_command(prompt).as_deref(),
            Some("ls"),
            "{language}: {prompt}"
        );
    }
}

/// The same rule for commands other than `ls`, so the fix is a rule about
/// command-naming nouns rather than a special case for one binary.
#[test]
fn a_command_naming_noun_is_stripped_for_every_command() {
    for (prompt, expected) in [
        ("execute pwd command", "pwd"),
        ("run whoami command", "whoami"),
        ("execute git status command", "git status"),
        ("run the df -h command", "df -h"),
    ] {
        assert_eq!(shell_command(prompt).as_deref(), Some(expected), "{prompt}");
    }
}

/// Issue #865: *"List me files here"* reached web search. A listing request is
/// composed of a listing verb (or a question word), an object naming what is
/// listed, and a local scope — in any word order.
#[test]
fn a_prose_listing_request_routes_to_ls_in_any_word_order() {
    for (language, prompt) in [
        // Reported.
        ("en", "List me files here"),
        // Held out: word orders and wordings no seed phrase spells out.
        ("en", "list me the files here"),
        ("en", "here, show me files"),
        ("en", "could you enumerate the files in this folder"),
        ("en", "which files do we have here?"),
        ("en", "give me the contents of the current directory"),
        ("ru", "перечисли мне файлы здесь"),
        ("ru", "покажи мне файлы в этой папке"),
        ("hi", "यहाँ फ़ाइलें दिखाओ"),
        // Spanish (es) is registered in `data/seed/languages.lino` but carried
        // no listing vocabulary at all, so every Spanish phrasing fell through
        // to web search the way the reported English one did. Adding the parts
        // to `data/seed/shell-intents.lino` -- verbs, question words, objects,
        // scopes -- is the whole fix: the detector combines parts and is not
        // told which language it is reading, so español routes on the same
        // rule as English, held-out word orders included.
        ("es", "lista los archivos aquí"),
        ("es", "muéstrame los archivos de la carpeta actual"),
        ("es", "aquí, enseña los ficheros"),
        ("es", "¿cuáles archivos hay en este directorio?"),
    ] {
        assert_eq!(
            shell_command(prompt).as_deref(),
            Some("ls"),
            "{language}: {prompt}"
        );
    }
}

/// The listing parts have to combine: a verb with no object, or an object with
/// no local scope, is a different request and must keep its own route.
#[test]
fn listing_parts_alone_do_not_make_a_listing_request() {
    assert_ne!(
        shell_command("show me the current directory").as_deref(),
        Some("ls")
    );
    assert_ne!(
        shell_command("list the running processes").as_deref(),
        Some("ls")
    );
}

/// Issue #868: a bare `ls` was answered with a chat refusal instead of being
/// run. A command typed on its own is the whole request; the tokens accepted
/// bare are the ones that need no argument, so nothing is invented for them.
#[test]
fn a_bare_command_is_the_request() {
    for (prompt, expected) in [
        // Reported.
        ("ls", "ls"),
        // Held out: other argument-free tokens, and flags typed with them.
        ("pwd", "pwd"),
        ("whoami", "whoami"),
        ("ls -la", "ls -la"),
        ("df -h", "df -h"),
    ] {
        assert_eq!(shell_command(prompt).as_deref(), Some(expected), "{prompt}");
    }
}

/// A bare token stays a command only while the words around it are not prose:
/// *"find information about Rust"* opens with a command token and is a search.
#[test]
fn a_bare_token_followed_by_prose_is_not_a_command() {
    assert_ne!(
        shell_command("ls of the achievements you have here").as_deref(),
        Some("ls of the achievements you have here")
    );
}

/// Issue #824: *"Move /Users/konard/Desktop/Archive/hive-control-center to
/// ~/Code/Archive/link-assistant"* was refused. Two separate rules stood in the
/// way — the `mv` intent had no cue for a bare *move*, and the operand filter
/// rejected every absolute and home-relative path — so the request matched
/// nothing at all. Both operands are words the user typed, so accepting them is
/// not a widening of what Formal AI may reach on its own.
#[test]
fn a_move_between_absolute_paths_is_performed() {
    for (prompt, expected) in [
        (
            "Move /Users/konard/Desktop/Archive/hive-control-center to ~/Code/Archive/link-assistant",
            "mv /Users/konard/Desktop/Archive/hive-control-center ~/Code/Archive/link-assistant",
        ),
        (
            "move /tmp/report.txt to ~/Documents/report.txt",
            "mv /tmp/report.txt ~/Documents/report.txt",
        ),
        (
            "please move the file /var/log/build.log to ~/archive/build.log",
            "mv /var/log/build.log ~/archive/build.log",
        ),
        (
            "перемести файл /tmp/report.txt в ~/Documents/report.txt",
            "mv /tmp/report.txt ~/Documents/report.txt",
        ),
    ] {
        assert_eq!(
            shell_command(prompt).as_deref(),
            Some(expected),
            "prompt: {prompt}"
        );
    }
}

/// A path the user did not write stays out of reach: a `..` traversal is a way of
/// naming a destination without saying where it is, so it is still refused.
#[test]
fn a_traversing_move_is_not_performed() {
    for prompt in [
        "move ../secrets/key.pem to ~/key.pem",
        "move /tmp/report.txt to ../../elsewhere/report.txt",
    ] {
        assert_eq!(shell_command(prompt), None, "prompt: {prompt}");
    }
}

/// Issues #863 and #862: *"copy stdin to stdout"* is the name of a programming
/// exercise, not a file operation, and a Rosetta Code URL naming that exercise is
/// not one either. Both lowered to `cp` because a bare *copy* cue accepted any two
/// words as its operands. An unanchored cue now needs operands that are written
/// the way paths are written, so neither request reaches the shell.
#[test]
fn a_named_exercise_is_not_a_file_operation() {
    for prompt in [
        "Give me example of how to do copy stdin to stdout in Rust",
        "Execute https://rosettacode.org/wiki/Copy_stdin_to_stdout in Rust",
        "show me how to copy stdin to stdout",
        "how do I copy standard input to standard output in Rust?",
        "write a program that copies stdin to stdout",
        "rename the variable counter to total in this function",
    ] {
        assert_eq!(shell_command(prompt), None, "prompt: {prompt}");
    }
}

/// Refusing to run it as a file copy is only half of #863: the exercise then has
/// to be *answered*. The catalog could not express it, because every task in it
/// produced its output from nothing — `output` described a task completely, so a
/// task whose subject is a stream had no way to say what stream. Naming the
/// input the task is defined against (`ProgramTask::input`) is the general
/// change: the fixture travels with the task into the run command the answer
/// prints, so what the reader is told to type is what was checked
/// (`experiments/issue-1021-copy-stdin`).
#[test]
fn a_named_exercise_is_answered_as_a_program() {
    for (natural_language, prompt, program_language) in [
        // Reported (#863), now with the language it names.
        (
            "en",
            "Give me example of how to do copy stdin to stdout in Rust",
            "rust",
        ),
        // Held out: paraphrases, and languages no phrase was written against.
        (
            "en",
            "write a program that copies stdin to stdout in Python",
            "python",
        ),
        (
            "en",
            "Write a Go program that copies standard input to standard output",
            "go",
        ),
        (
            "ru",
            "напиши программу на C, которая копирует стандартный ввод в стандартный вывод",
            "c",
        ),
        ("ru", "скопировать stdin в stdout на php", "php"),
        ("hi", "मानक इनपुट को मानक आउटपुट में कॉपी करें python", "python"),
        ("zh", "用 Java 将标准输入复制到标准输出", "java"),
        ("es", "copiar stdin a stdout en JavaScript", "javascript"),
    ] {
        let response = UniversalSolver::default().solve(prompt);
        assert_eq!(
            response.intent, "write_program",
            "{natural_language}: {prompt}"
        );
        assert!(
            response.evidence_links.iter().any(|link| link
                == &format!("response:write_program:copy_stdin_to_stdout:{program_language}")),
            "{natural_language}: {prompt} -> {:?}",
            response.evidence_links
        );
    }
}

/// A program that reads standard input is not run by naming it: with no
/// redirection the reader gets a process waiting on a terminal, and the
/// "expected output" above it never appears. So the answer prints the fixture
/// piped in, in the execution report *and* in the numbered test steps — the two
/// places a reader copies from — and both come from the same task data.
#[test]
fn the_stdin_answer_prints_the_input_it_was_verified_against() {
    // The whole answer, so a reader can see that the fixture appears in the run
    // command, in the reproduction steps and in the output block, and nowhere
    // it does not belong.
    const EXPECTED: &str = "Here is a minimal Rust copy standard input to standard output program:\n\
         \n\
         ```rust\n\
         use std::io::{self, Read, Write};\n\
         \n\
         fn main() -> io::Result<()> {\n\
         \x20   let mut input = Vec::new();\n\
         \x20   io::stdin().read_to_end(&mut input)?;\n\
         \x20   io::stdout().write_all(&input)\n\
         }\n\
         ```\n\
         \n\
         Execution status: compiled and ran in issue-8 local verification harness (isolated sandbox).\n\
         Check command: `rustc main.rs -o main`\n\
         Run command: `printf 'hello\\nworld\\n' | ./main`\n\
         Output:\n\
         ```text\n\
         hello\n\
         world\n\
         ```\n\
         1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.\n\
         \n\
         How it works:\n\
         The program performs the requested task and prints its result to standard output.\n\
         \n\
         How to test it yourself:\n\
         1. Install the Rust toolchain from https://rustup.rs.\n\
         2. Save the code above to a file named `main.rs`.\n\
         3. Check that it compiles: `rustc main.rs -o main`.\n\
         4. Run it: `printf 'hello\\nworld\\n' | ./main`.\n\
         5. Compare the output with the expected output shown above.";

    let response = UniversalSolver::default().solve("copy stdin to stdout in Rust");
    assert_eq!(response.answer, EXPECTED);
}

/// And a task that reads no input keeps the plain command it always had, so the
/// fixture is a property of the task rather than a new decoration on every
/// answer.
#[test]
fn a_task_that_reads_no_input_keeps_its_plain_run_command() {
    // The same answer shape as above with the pipe absent everywhere, which is
    // the whole claim: the fixture belongs to the task, not to the renderer.
    const EXPECTED: &str = "Here is a minimal Rust hello world program:\n\
         \n\
         ```rust\n\
         fn main() {\n\
         \x20   println!(\"Hello, world!\");\n\
         }\n\
         ```\n\
         \n\
         Execution status: compiled and ran in issue-8 local verification harness (isolated sandbox).\n\
         Check command: `rustc main.rs -o main`\n\
         Run command: `./main`\n\
         Output:\n\
         ```text\n\
         Hello, world!\n\
         ```\n\
         1 iteration completed under the 1 minute execution budget; no timeout reduction was needed.\n\
         \n\
         How it works:\n\
         The program prints the text `Hello, world!` to standard output and then exits.\n\
         \n\
         How to test it yourself:\n\
         1. Install the Rust toolchain from https://rustup.rs.\n\
         2. Save the code above to a file named `main.rs`.\n\
         3. Check that it compiles: `rustc main.rs -o main`.\n\
         4. Run it: `./main`.\n\
         5. Compare the output with the expected output shown above.";

    let response = UniversalSolver::default().solve("write me hello world program in Rust");
    assert_eq!(response.answer, EXPECTED);
    assert!(!response.answer.contains("printf"), "{}", response.answer);
}

/// The same guard must not cost the file operations that were already routed: a
/// cue that names the object it acts on (*"remove the directory build"*) still
/// accepts a plain name as its operand.
#[test]
fn a_cue_that_names_its_object_still_takes_a_plain_name() {
    for (prompt, expected) in [
        ("remove the directory build", "rmdir build"),
        ("delete the file old.txt", "rm old.txt"),
        ("copy a.txt to b.txt", "cp a.txt b.txt"),
        ("rename old.txt to new.txt", "mv old.txt new.txt"),
        ("create a symbolic link from a to b", "ln -s a b"),
    ] {
        assert_eq!(
            shell_command(prompt).as_deref(),
            Some(expected),
            "prompt: {prompt}"
        );
    }
}

/// The intent `UniversalSolver` resolves for `prompt`.
fn intent(prompt: &str) -> String {
    UniversalSolver::default().solve(prompt).intent
}

/// Issue #723: *"напиши мне код на PHP Laravel"* — "write me PHP Laravel code" —
/// was answered with the unspecified-request refusal, because PHP was one of the
/// few languages the catalog did not template. Cataloguing PHP stopped the
/// refusal, but it answered a request for Laravel with plain PHP: the catalog
/// had a single axis where the sentence names two things, so `laravel` could
/// only be carried as an alias surface of `php`.
///
/// The generalization widens that axis instead of adding a rule about Laravel.
/// A catalog row is an *implementation target*, and a target may be a framework
/// of another target (`framework_of`). What belongs to the language — its
/// grammar, its composable idioms — is read through
/// `ProgramLanguage::base_language`, so a framework inherits it without
/// restating it; what the request actually asked for — the template, the file
/// to save it in, the command that runs it — is the framework's own. A
/// framework nobody catalogued still answers in the language it is written in,
/// which is the honest answer rather than a refusal.
#[test]
fn a_framework_named_coding_request_is_answered_in_that_framework() {
    for (language, prompt, expected) in [
        // Reported.
        (
            "ru",
            "напиши мне код на PHP Laravel",
            "write_script_laravel",
        ),
        // Held out: the same request in the other project languages.
        ("en", "write me PHP Laravel code", "write_script_laravel"),
        ("hi", "PHP Laravel में कोड लिखें", "write_script_laravel"),
        ("zh", "用 PHP Laravel 写代码", "write_script_laravel"),
        // Held out: a second PHP framework nobody catalogued, and the bare
        // language. Both answer in PHP — an uncatalogued framework falls back
        // to the language it is written in rather than to a refusal, and the
        // Laravel row does not capture requests that never named it.
        ("en", "write me PHP Symfony code", "write_script_php"),
        ("en", "write me some PHP code", "write_script_php"),
        ("hi", "PHP में कोड लिखें", "write_script_php"),
        ("zh", "用 PHP 写代码", "write_script_php"),
        // Controls: the routes this request should have taken all along keep
        // taking it, so PHP joined a rule rather than acquiring one.
        ("en", "write me Ruby on Rails code", "write_script_ruby"),
        (
            "ru",
            "напиши мне код на Python Django",
            "write_script_python",
        ),
    ] {
        assert_eq!(intent(prompt), expected, "{language}: {prompt}");
    }
}

/// Asking for code is not only *writing* it. Probing paraphrases of the #723
/// prompt found that every asking verb other than *write* — "I need", "I want",
/// "give me", and their Russian, Hindi and Chinese equivalents — fell through
/// the coding path into a web search, in every language, for every language.
/// The verb family is seed data (`request` in `data/seed/meanings.lino`), so
/// naming it once fixes the whole family rather than the reported sentence.
#[test]
fn asking_for_code_is_a_coding_request_whatever_the_asking_verb() {
    for (language, prompt, expected) in [
        ("ru", "мне нужен код на пхп", "write_script_php"),
        ("ru", "дай мне код на php", "write_script_php"),
        ("ru", "мне нужен код на python", "write_script_python"),
        ("en", "I need PHP code", "write_script_php"),
        ("en", "I want PHP code", "write_script_php"),
        ("en", "give me python code", "write_script_python"),
        ("hi", "मुझे php कोड चाहिए", "write_script_php"),
        ("zh", "我需要 php 代码", "write_script_php"),
    ] {
        assert_eq!(intent(prompt), expected, "{language}: {prompt}");
    }
}

/// The asking verbs must not swallow the requests that merely contain them: the
/// coding route needs a code artifact *and* a language, so a question about a
/// language, or a search for something written in one, keeps its own route.
#[test]
fn an_asking_verb_alone_is_not_a_coding_request() {
    for prompt in [
        "I need information about Rust",
        "I need to find a python tutorial",
        "give me the code of this repository",
        "I need a code review",
        // Held out, and the reason the language-less rule is a subtraction:
        // each of these names a *subject* beyond the artefact, so something is
        // left over once the verb, the artefact and the function words go.
        "дай мне код этого репозитория",
        "I need the code of this file",
        "give me a code example for sorting",
    ] {
        assert!(
            !intent(prompt).starts_with("write_"),
            "prompt: {prompt} routed to {}",
            intent(prompt)
        );
    }
}

/// A coding request that names no language at all is still a coding request.
///
/// `мне нужен код` — "I need code" — carries the asking verb and the artefact
/// and nothing else, and it reached a **web search**: the router only accepted a
/// program request when a language resolved, so the shortest possible coding
/// request was the one it could not see. The rule that fixes it is a
/// subtraction, not a phrase list: once the authoring verb, the code artefact
/// and the closed-class function words (`request_function_word` in
/// `data/seed/meanings.lino`) are taken out of the prompt, a bare code request
/// has nothing left. That makes the answer the honest dead end
/// (`program_skill_gap::Shape::RequestUnspecified`), which names the two
/// missing parameters and asks for them, instead of a search for the words.
#[test]
fn a_coding_request_naming_no_language_is_a_coding_request() {
    for (language, prompt) in [
        // Reported shape: the #723 prompt with its language dropped.
        ("ru", "мне нужен код"),
        // Held out: the same request through every other asking verb and every
        // supported language, none of them written down as a phrase anywhere.
        ("ru", "дай мне код"),
        ("ru", "напиши мне код"),
        ("ru", "мне нужна программа"),
        ("en", "I need code"),
        ("en", "give me code"),
        ("en", "I want code"),
        ("en", "write me a program"),
        ("hi", "मुझे कोड चाहिए"),
        ("zh", "我需要代码"),
        ("zh", "给我代码"),
        ("es", "necesito código"),
    ] {
        assert_eq!(
            intent(prompt),
            "write_program_request_unspecified",
            "{language}: {prompt}"
        );
    }
}

/// A one-letter language alias must not match inside an accented word.
///
/// Found while covering the request above. `contains_token` asked
/// `is_ascii_alphanumeric` of the character following a match to decide whether
/// it was a word boundary, and `ó` is not ASCII — so the `c` of the Spanish
/// `código` read as an isolated token and every Spanish request that mentions
/// code named the language C. Word boundaries are a property of letters, not of
/// ASCII, which is what the fix says; the scripts written without word spaces
/// stay boundaries, as `contains_cjk` and `contains_devanagari` already had it.
///
/// Restoring the ASCII test fails this with `write_script_c`.
#[test]
fn a_one_letter_alias_does_not_match_inside_an_accented_word() {
    for prompt in ["escribe código", "necesito código", "dame código"] {
        assert_eq!(
            intent(prompt),
            "write_program_request_unspecified",
            "the c of código is not the language C: {prompt}"
        );
    }
    // The boundary is not simply switched off: an accented word that really
    // does name a language still resolves, and so does a bare `c`.
    assert_eq!(intent("escribe código en Python"), "write_script_python");
    assert_eq!(
        intent("write me a C program that counts to three"),
        "write_program"
    );
}

/// The dead end has to be an answer, not a shrug: it names both parameters it
/// is missing and asks for them, in the language the request arrived in.
#[test]
fn the_languageless_coding_request_is_answered_in_its_own_language() {
    let english = UniversalSolver::default().solve("I need code");
    assert_eq!(
        english.intent, "write_program_request_unspecified",
        "{}",
        english.answer
    );
    assert!(english.answer.contains("language"), "{}", english.answer);

    let russian = UniversalSolver::default().solve("мне нужен код");
    assert_eq!(
        russian.intent, "write_program_request_unspecified",
        "{}",
        russian.answer
    );
    assert!(russian.answer.contains("язык"), "{}", russian.answer);
}

/// PHP graduating from the coding oracle to the catalog is what makes the #723
/// answer a real one: the templates are `php -l`-checked and executed by the
/// issue-8 harness, so the answer carries the verified execution status the
/// other catalogued languages carry rather than a borrowed claim.
#[test]
fn php_is_answered_from_the_catalog_like_every_catalogued_language() {
    let response = UniversalSolver::default().solve("write a hello world program in php");
    assert_eq!(response.intent, "write_program", "{}", response.answer);
    assert!(response.answer.contains("```php"), "{}", response.answer);
    assert!(response.answer.contains("<?php"), "{}", response.answer);
    assert!(
        response.answer.contains("compiled and ran"),
        "{}",
        response.answer
    );

    // Held out: a catalogued task, and a numeric-list task the universal
    // composer derives from `data/seed/coding-idioms.lino` rather than from a
    // template — both must reach PHP now that the language is catalogued.
    let fizzbuzz = UniversalSolver::default().solve("write a fizzbuzz program in php");
    assert_eq!(fizzbuzz.intent, "write_program", "{}", fizzbuzz.answer);
    assert!(fizzbuzz.answer.contains("```php"), "{}", fizzbuzz.answer);

    let sorted =
        UniversalSolver::default().solve("Sort the numbers 3, 1, 2 in PHP, give me the code");
    assert_eq!(sorted.intent, "write_program", "{}", sorted.answer);
    assert!(
        sorted.answer.contains("sort($numbers);"),
        "{}",
        sorted.answer
    );
    assert!(
        sorted.answer.contains("Result: 1, 2, 3"),
        "{}",
        sorted.answer
    );
}

/// The minimal-script route renders one thing: the hello-world template for the
/// language it recognizes. It was claiming any prompt that merely *mentioned*
/// code, because `data/meta/cue-lexicon.lino` hoists `handler:write_script` onto
/// the front of method selection for the bare tokens "script" and "code" — ahead
/// of the task routes `data/seed/handler-precedence.lino` ranks above it
/// (`numeric_list` is line 29, `write_script` line 43). So "sort these numbers in
/// Python, write me the code" answered with a Python *hello world*: a named task
/// silently replaced by a greeting.
///
/// This predates the asking verbs above — "write me the code" hit it too — and it
/// is fixed where it is wrong rather than where it was noticed: the route now
/// declines a prompt whose task it cannot render, which is a property of the
/// route, not of a phrase. The paraphrases below hold out three operations, four
/// languages of implementation, three asking verbs and two prompt languages.
#[test]
fn a_named_task_is_not_answered_with_a_minimal_script() {
    for prompt in [
        "Sort the numbers 3, 1, 2 in PHP, write me the code",
        "Sort the numbers 3, 1, 2 in PHP, give me the code",
        "Sort the numbers 3, 1, 2 in Python, I need the code",
        "Reverse the numbers 1, 2, 3 in Ruby, give me the code",
        "Sum the numbers 3, 5, 6 in JavaScript, write me the code",
        "отсортируй числа 3, 1, 2 на python, дай мне код",
    ] {
        let response = UniversalSolver::default().solve(prompt);
        assert_eq!(
            response.intent, "write_program",
            "prompt: {prompt} — answer was: {}",
            response.answer
        );
    }

    // Control: with no task of its own to answer, the same asking verbs keep
    // reaching the minimal script. The route lost the prompts it cannot serve,
    // not the ones it exists for.
    for (prompt, expected) in [
        ("write me some code in Python", "write_script_python"),
        ("give me python code", "write_script_python"),
        ("дай мне код на php", "write_script_php"),
    ] {
        assert_eq!(intent(prompt), expected, "prompt: {prompt}");
    }
}
