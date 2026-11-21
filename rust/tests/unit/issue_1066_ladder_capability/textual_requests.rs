use super::*;

pub(super) fn textual_rendering_language_is_not_a_gui_capability_request() {
    // `rendered` also describes text produced by a reader, serializer, or
    // template. Treating that adjective alone as a visual request made a
    // composite ladder node refuse to inspect two ordinary text files. Keep a
    // text-rendering example for every registry language so removing the
    // ambiguous GUI cue cannot become an English-only repair.
    let cases = [
        (
            "en",
            "Do not copy tool-rendered line numbers or file wrappers while reading text records.",
        ),
        (
            "ru",
            "Проверь отрисованную текстовую запись и опиши её текстовый формат.",
        ),
        ("hi", "rendered पाठ रिकॉर्ड जाँचें और उसका पाठ प्रारूप बताएँ।"),
        ("zh", "检查渲染后的文本记录并报告文本格式。"),
        (
            "es",
            "Inspecciona el texto renderizado y describe su formato textual.",
        ),
    ];
    for &(language, prompt) in &cases {
        assert!(
            formal_ai::computer_use::capability_gap_for_request(prompt).is_none(),
            "{language}: text-only wording was mistaken for a GUI capability gap: {prompt:?}",
        );
        assert!(
            final_answer(prompt).is_none_or(|answer| !answer.contains("gui_rendering")),
            "{language}: the agentic router refused a text-only task as visual: {prompt:?}",
        );
    }

    let registered = formal_ai::language::registered_languages()
        .into_iter()
        .map(formal_ai::Language::slug)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        cases
            .map(|(language, _)| language)
            .into_iter()
            .collect::<BTreeSet<_>>(),
        registered,
        "the textual-rendering matrix must track every registered language",
    );
}

pub(super) fn a_file_the_request_asks_for_is_never_opened_for_reading() {
    // A request can ask for work and name where the answer goes, and constrain
    // how that file has to open. *First line* is a reading cue, so read the
    // prompt as one span and the cue captures the delivery path: the run opened
    // the file it was asked to create, got "No such file or directory", and
    // recorded that error as its evidence. Whatever cue accompanies it, a path
    // the request states as a write destination is not a file to read.
    for (prompt, target) in [
        (
            "Decide whether rewriting the deployment script is a single atomic task. \
             Leave the outcome in `triage/deploy.md`. The first line must be exactly \
             `triage=deploy`.",
            "triage/deploy.md",
        ),
        (
            "Judge whether the billing database migration is a problem you can break \
             down. Record the finding in `triage/billing.md`. The first line must be \
             exactly `triage=billing`.",
            "triage/billing.md",
        ),
        // A request can also be delivery and nothing else. Both prompts above
        // ask a question about a task, so the task-structure route answers them
        // and the read planner is never consulted -- which leaves the guard
        // unexercised by them. These two state only where the answer goes and
        // how it opens, so the read planner is exactly what decides them, and
        // the guard is what it decides with.
        (
            "Leave observable evidence in `audit/ledger-check.md`. The first line must \
             be exactly `audit=ledger-check`.",
            "audit/ledger-check.md",
        ),
        (
            "Put the outcome in `runs/nightly-42.md`. The first line must be exactly \
             `run=nightly-42`.",
            "runs/nightly-42.md",
        ),
    ] {
        let reads = planned_reads(prompt);
        assert!(
            !reads.iter().any(|path| path == target),
            "opened the file it was asked to write for {prompt:?}: {reads:?}"
        );
    }
}
