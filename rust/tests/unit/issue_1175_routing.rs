//! Issue #1175: routing by a single surface word.
//!
//! Two detection sites accepted a prompt because *one word* matched a
//! recognition surface, without asking what the word was doing in the
//! sentence:
//!
//! 1. The terminal-command detector fired whenever a prompt's **first word**
//!    was a seed shell token — and `find`, `make`, `file`, `which`, `head`,
//!    `tail`, `touch`, `kill`, `export` and `cat` are ordinary English words.
//!    "Find the bug: def average(xs): …" and "Make a 3-day itinerary for a
//!    first visit to Rome." were answered with "It looks like you want to run
//!    a terminal command". The leading-token path now requires the rest of
//!    the prompt to parse as command arguments (no question mark, no
//!    sentence-final punctuation on bare tokens, no `": "` outside quotes, no
//!    bare function word that never appears in an argument position).
//!
//! 2. The software-project detector picked its artifact kind from the first
//!    artifact surface **anywhere** in the prompt, so "Write a regular
//!    expression that matches a US ZIP code with an optional 4-digit
//!    extension." became a plan for a TypeScript *extension* — there
//!    `extension` modifies `ZIP code`; the head of what is written is
//!    "regular expression". The artifact kind now comes only from the head
//!    noun of the authoring verb's object phrase.
//!
//! Beyond the three reported probes, the held-out probes below occur in no
//! file under `data/seed/` or `rust/src/`, so a pass cannot come from a
//! memorized cue phrase.

use formal_ai::FormalAiEngine;

/// Probes from the issue report itself: the reported misroutes must stop.
#[test]
fn reported_probes_stop_misrouting() {
    // "Find the bug: …" must not be a terminal command (its first word is a
    // seed shell token, but the rest is a code-review task introduced by a
    // colon).
    let answer =
        FormalAiEngine.answer("Find the bug: def average(xs): return sum(xs) / len(xs) - 1");
    assert_ne!(answer.intent, "agent_suggestion", "answer: {answer:?}");

    // "Make a 3-day itinerary …" must not be a terminal command (natural
    // language after an ordinary-English shell word).
    let answer = FormalAiEngine.answer("Make a 3-day itinerary for a first visit to Rome.");
    assert_ne!(answer.intent, "agent_suggestion", "answer: {answer:?}");
    assert_ne!(answer.intent, "software_project_plan", "answer: {answer:?}");

    // "Write a regular expression … with an optional 4-digit extension." must
    // not become a software-project plan for an extension: `extension` there
    // modifies `ZIP code`, it is not the head of what is written.
    let answer = FormalAiEngine.answer(
        "Write a regular expression that matches a US ZIP code with an optional 4-digit extension.",
    );
    assert_ne!(answer.intent, "software_project_plan", "answer: {answer:?}");
    assert_ne!(answer.intent, "agent_suggestion", "answer: {answer:?}");
}

/// Natural language that merely starts with a shell-token word is not a
/// command line, in every rejection shape the guard knows.
#[test]
fn natural_language_starting_with_a_shell_word_is_not_a_command() {
    for prompt in [
        // language: en — a question is never a command line.
        "Which search engine do you prefer and why?",
        // language: en — prose after an ordinary-English shell word.
        "Head of the department asked me to schedule a review.",
        "Touch base with the team before Friday.",
        "Cat owners know that cats sleep most of the day.",
        "File a complaint about the noise, please.",
    ] {
        let answer = FormalAiEngine.answer(prompt);
        assert_ne!(
            answer.intent, "agent_suggestion",
            "prompt: {prompt:?} answer: {answer:?}"
        );
    }
}

/// A seed shell token followed by argument-shaped words stays a terminal
/// command, including the shapes the guard could have over-rejected: an
/// all-dots path argument, quoted prose, and file names with dots.
#[test]
fn argument_shaped_prompts_after_a_shell_token_stay_terminal_commands() {
    // The seeded agent-suggestion template ends its third paragraph with a
    // permission question ("Switch to Agent mode … so I can run `{command}`?")
    // that `question_necessity::enforce_questions` refuses and strips from the
    // sentence start to the `?`, leaving the paragraph's leading space. A
    // stop inside the inline command (`find . -name …`) is code, not a
    // sentence end, so the whole question goes there too.
    for (prompt, command, expected) in [
        (
            "ls ~",
            "ls ~",
            "It looks like you want to run a terminal command: `ls ~`.\n\nRunning shell commands requires Agent mode. In Chat mode I only reason about your request and do not execute commands.\n\n Use the mode radio in the toolbar to pick \"Agent\" (or \"Full Auto\" to run commands automatically).",
        ),
        (
            "find . -name '*.log' -size +10M",
            "find . -name '*.log' -size +10M",
            "It looks like you want to run a terminal command: `find . -name '*.log' -size +10M`.\n\nRunning shell commands requires Agent mode. In Chat mode I only reason about your request and do not execute commands.\n\n Use the mode radio in the toolbar to pick \"Agent\" (or \"Full Auto\" to run commands automatically).",
        ),
        (
            "make test",
            "make test",
            "It looks like you want to run a terminal command: `make test`.\n\nRunning shell commands requires Agent mode. In Chat mode I only reason about your request and do not execute commands.\n\n Use the mode radio in the toolbar to pick \"Agent\" (or \"Full Auto\" to run commands automatically).",
        ),
        (
            "head -n 5 main.rs",
            "head -n 5 main.rs",
            "It looks like you want to run a terminal command: `head -n 5 main.rs`.\n\nRunning shell commands requires Agent mode. In Chat mode I only reason about your request and do not execute commands.\n\n Use the mode radio in the toolbar to pick \"Agent\" (or \"Full Auto\" to run commands automatically).",
        ),
        (
            "export FOO=bar",
            "export FOO=bar",
            "It looks like you want to run a terminal command: `export FOO=bar`.\n\nRunning shell commands requires Agent mode. In Chat mode I only reason about your request and do not execute commands.\n\n Use the mode radio in the toolbar to pick \"Agent\" (or \"Full Auto\" to run commands automatically).",
        ),
        (
            "touch newfile.txt",
            "touch newfile.txt",
            "It looks like you want to run a terminal command: `touch newfile.txt`.\n\nRunning shell commands requires Agent mode. In Chat mode I only reason about your request and do not execute commands.\n\n Use the mode radio in the toolbar to pick \"Agent\" (or \"Full Auto\" to run commands automatically).",
        ),
        // Quoted words are data, not prose: the natural-language markers
        // inside the quotes must not reject the command line.
        (
            "git commit -m 'fix the parser bug'",
            "git commit -m 'fix the parser bug'",
            "It looks like you want to run a terminal command: `git commit -m 'fix the parser bug'`.\n\nRunning shell commands requires Agent mode. In Chat mode I only reason about your request and do not execute commands.\n\n Use the mode radio in the toolbar to pick \"Agent\" (or \"Full Auto\" to run commands automatically).",
        ),
    ] {
        let answer = FormalAiEngine.answer(prompt);
        assert_eq!(
            answer.intent, "agent_suggestion",
            "prompt: {prompt:?} answer: {answer:?}"
        );
        assert_eq!(answer.answer, expected, "prompt: {prompt:?}");
        assert!(
            answer.answer.contains(command),
            "prompt {prompt:?} should name the command: {}",
            answer.answer
        );
    }
}

/// An artifact word that is not the head of the authoring verb's object
/// phrase must not claim the software-project frame.
#[test]
fn incidental_artifact_words_do_not_claim_the_software_project_frame() {
    for prompt in [
        // The reported shape with the prepositional phrase fronted: the
        // sentence-final gate must keep the pre-verbal fallback from
        // rescuing the incidental "extension".
        "For a US ZIP code with an optional 4-digit extension, write a regular expression.",
        // The object head is a study plan, not the later "extensions".
        "Create a study plan that covers sorting algorithms and their extensions into graphs.",
        // The object head is a birdhouse; there is no artifact at all.
        "Build a wooden birdhouse for the garden this weekend.",
        // "dashboard" modifies "launch date"; the object head is a flyer.
        "Design a flyer that mentions our new dashboard launch date.",
        // language: ru — the reported shape in Russian: the object head is
        // «регулярное выражение», not the trailing «расширение».
        "Напиши регулярное выражение для почтового индекса США с необязательным 4-значным расширением.",
    ] {
        let answer = FormalAiEngine.answer(prompt);
        assert_ne!(
            answer.intent, "software_project_plan",
            "prompt: {prompt:?} answer: {answer:?}"
        );
        assert_ne!(
            answer.intent, "agent_suggestion",
            "prompt: {prompt:?} answer: {answer:?}"
        );
    }
}

/// A request whose object phrase *is* headed by an artifact keeps the
/// software-project frame, in every language the lexicon covers — including
/// subject-object-verb order (Hindi) and unspaced CJK.
#[test]
fn artifact_headed_requests_still_claim_the_software_project_frame() {
    for (language, prompt) in [
        // language: en
        (
            "English (en)",
            "Write a browser extension that blocks ads on news sites.",
        ),
        // language: ru
        (
            "Russian (ru)",
            "Создай расширение для браузера, чтобы блокировать рекламу.",
        ),
        // language: hi — the object precedes the verb.
        ("Hindi (hi)", "एक ब्राउज़र एक्सटेंशन बनाओ"),
        // language: zh — no inter-word spaces.
        ("Chinese (zh)", "创建一个浏览器扩展"),
    ] {
        let answer = FormalAiEngine.answer(prompt);
        assert_eq!(
            answer.intent, "software_project_plan",
            "{language}: {prompt}: {answer:?}"
        );
        assert_ne!(answer.intent, "unknown", "{language}: {prompt}");
    }
}
