//! An answer that announces something must show it (issue #1066).
//!
//! The ladder scored a node green when its proof file existed, opened with the
//! pinned marker, and was not empty. Thirty-two of the sixty-three files that
//! passed that check said nothing: the check is mechanical and the answers were
//! hollow. These tests pin the two ways a decomposition answer could come out
//! hollow, each with wording the ladder never uses, because the guard is on the
//! shape of the answer and not on any prompt.

use formal_ai::engine::{FormalAiEngine, SymbolicAnswer};
use formal_ai::meta_frame::AtomicityReason;
use formal_ai::task_decomposition::{decompose_task, stated_task};
use formal_ai::{SolverConfig, UniversalSolver};

/// The lines of an answer that are numbered sub-task entries.
fn numbered_lines(answer: &str) -> Vec<&str> {
    answer
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.split_once(['.', ')'])
                .is_some_and(|(head, _)| !head.is_empty() && head.chars().all(char::is_numeric))
        })
        .collect()
}

#[test]
fn an_answer_that_announces_sub_tasks_never_lists_none() {
    // The recursion can reach a leaf it did not resolve: a need it cannot split
    // into two independently checkable halves and for which it knows no
    // observable completion criterion. That leaf is a root with no children
    // that is nonetheless not reported atomic, so the reply said "these are the
    // sub-tasks" and then listed nothing at all -- a heading with no list.
    // Either the list is made, or the answer says instead why there is none.
    // The colon is the tell: it is the punctuation that promises the list, so an
    // answer that ends on one has announced something it never showed.
    // Both prompts reach the same refusal, shown here in full: a reader has to
    // be able to tell an honest "there is nothing to enumerate" from a heading
    // that promised a list, and only the exact text says which one this is.
    const UNENUMERABLE: &str = "This task cannot be split into two sub-tasks that can be checked independently, and \
         no observable completion criterion is known for it, so there is nothing to \
         enumerate: it is an irreducible single need.";
    for prompt in [
        "Work out whether migrating the billing database divides into smaller pieces.",
        "Split the following into sub-tasks: soothe the reviewer.",
    ] {
        let response = FormalAiEngine.answer(prompt);
        assert_eq!(
            response.answer.trim(),
            UNENUMERABLE,
            "unexpected answer for {prompt:?}"
        );
        let answer = response.answer;
        let answer = answer.trim();
        if numbered_lines(answer).is_empty() {
            assert!(
                !answer.ends_with([':', '\u{ff1a}']),
                "announced sub-tasks and listed none for {prompt:?}: {answer:?}"
            );
            assert!(
                answer.split_whitespace().count() > 4,
                "listed no sub-tasks and did not say why for {prompt:?}: {answer:?}"
            );
        }
    }
}

#[test]
fn a_root_the_recursion_never_split_is_reported_as_such() {
    // Both questions are two views of one recursion, so when the recursion
    // resolved nothing both views owe the reader the same explanation -- and
    // neither may borrow the other's verdict: the task is not atomic in the
    // sense that makes it directly checkable, and it is not split either.
    let decomposition = decompose_task("soothe the reviewer", 4);
    assert_eq!(
        decomposition.unenumerable_reason(),
        Some(AtomicityReason::SingleNeed),
        "the irreducible single need was not reported as one"
    );
}

#[test]
fn a_bound_that_stopped_the_split_before_it_started_is_reported() {
    // A depth bound of zero cuts the recursion before the first split, which
    // leaves the same shape -- a childless root that is not atomic -- for an
    // entirely different reason. A reader who lowered the bound has to be told
    // that the bound is what they are looking at, not the task's nature.
    let decomposition = decompose_task("rewrite the deployment script", 0);
    assert_eq!(
        decomposition.unenumerable_reason(),
        Some(AtomicityReason::DepthBound),
        "the depth bound was not reported as the reason nothing was enumerated"
    );
}

#[test]
fn a_split_that_did_happen_is_still_enumerated() {
    // The guard must cost nothing to the tasks that do split: reporting "there
    // is nothing to enumerate" for a task with sub-tasks would be the same
    // hollowness pointing the other way.
    let decomposition = decompose_task("rewrite the deployment script", 4);
    assert_eq!(decomposition.unenumerable_reason(), None);
    assert!(
        !decomposition.rows().is_empty(),
        "a splittable task enumerated nothing"
    );
}

#[test]
fn a_listed_sub_task_keeps_the_text_that_says_what_to_do() {
    // A sub-task is composed by putting the task inside a statement about it.
    // When the task was recovered from a question it carried the question mark
    // along, the statement became a question, and the answer's own question
    // policy deleted it as unearned -- leaving `1.  [criterion]`: a numbered
    // list whose every entry had lost its text. The criterion names the check;
    // the text is what a reader would have to do.
    // Each reply is shown in full. The two English prompts differ in their
    // opening because one asks whether the task is atomic and the other asks
    // whether it can be split, and the Russian one is answered in Russian
    // because the surfaces are seeded per language rather than translated.
    for (prompt, expected) in [
        (
            "Is polishing the onboarding copy an atomic task?",
            concat!(
                "No — this task is not atomic. It splits into these sub-tasks, each with a completion criterion you can observe:\n",
                "1. Record independently checkable requirements for Is polishing the onboarding copy an atomic task [requirements_are_independently_checkable]\n",
                "2. Add a regression test that reproduces Is polishing the onboarding copy an atomic task [regression_test_reproduces_failure]\n",
                "3. Implement the smallest general change that satisfies Is polishing the onboarding copy an atomic task [requested_behavior_passes]\n",
                "4. Run the acceptance checks for Is polishing the onboarding copy an atomic task [acceptance_checks_pass]",
            ),
        ),
        (
            "Is the checkout rewrite a task you can split into steps?",
            concat!(
                "Sub-tasks, each with a completion criterion you can observe:\n",
                "1. Record independently checkable requirements for Is the checkout rewrite a task you can split into steps [requirements_are_independently_checkable]\n",
                "2. Add a regression test that reproduces Is the checkout rewrite a task you can split into steps [regression_test_reproduces_failure]\n",
                "3. Implement the smallest general change that satisfies Is the checkout rewrite a task you can split into steps [requested_behavior_passes]\n",
                "4. Run the acceptance checks for Is the checkout rewrite a task you can split into steps [acceptance_checks_pass]",
            ),
        ),
        (
            "Является ли рефакторинг платёжного модуля атомарной задачей?",
            concat!(
                "Нет — задача не атомарная. Она разбивается на подзадачи, у каждой из которых есть наблюдаемый критерий завершения:\n",
                "1. Зафиксируй независимо проверяемые требования для Является ли рефакторинг платёжного модуля атомарной задачей [requirements_are_independently_checkable]\n",
                "2. Добавь регрессионный тест, воспроизводящий Является ли рефакторинг платёжного модуля атомарной задачей [regression_test_reproduces_failure]\n",
                "3. Реализуй минимальное общее изменение, выполняющее Является ли рефакторинг платёжного модуля атомарной задачей [requested_behavior_passes]\n",
                "4. Запусти приёмочные проверки для Является ли рефакторинг платёжного модуля атомарной задачей [acceptance_checks_pass]",
            ),
        ),
    ] {
        let response = FormalAiEngine.answer(prompt);
        assert_eq!(
            response.answer.trim(),
            expected,
            "unexpected answer for {prompt:?}"
        );
        let answer = response.answer;
        let lines = numbered_lines(&answer);
        assert!(
            !lines.is_empty(),
            "no sub-tasks were listed for {prompt:?}: {answer:?}"
        );
        for line in lines {
            let text = line
                .split_once(['.', ')'])
                .map(|(_, rest)| rest)
                .unwrap_or_default();
            let prose = text
                .rsplit_once('[')
                .map_or(text, |(before, _)| before)
                .trim();
            assert!(
                prose.chars().any(char::is_alphabetic),
                "a listed sub-task lost its text for {prompt:?}: {answer:?}"
            );
        }
    }
}

#[test]
fn a_colon_in_a_later_sentence_does_not_become_the_task() {
    // A request states its task and then keeps writing. A colon further down
    // introduces a deadline, an owner, a criterion -- never the work -- and
    // taking the prompt's last colon made that fragment the task. A deadline is
    // an irreducible single need, so a rewrite that splits four ways came back
    // reported as unsplittable, which is the same hollowness as an announced
    // list with no entries: the reply is about something the caller never asked
    // about.
    // The replies are shown in full, and they show the scope of the fix
    // exactly: the enumerated work is the rewrite the caller asked about, and
    // the trailing deadline or owner rides along inside the task text instead
    // of replacing it. Before the fix every entry was about "the end of the
    // quarter" alone.
    for (prompt, subject, expected) in [
        (
            "Break the warehouse restocking rewrite into sub-tasks. Deadline: the end of \
             the quarter.",
            "warehouse restocking",
            concat!(
                "Sub-tasks, each with a completion criterion you can observe:\n",
                "1. Record independently checkable requirements for Break the warehouse restocking rewrite into sub-tasks. Deadline: the end of the quarter [requirements_are_independently_checkable]\n",
                "2. Add a regression test that reproduces Break the warehouse restocking rewrite into sub-tasks. Deadline: the end of the quarter [regression_test_reproduces_failure]\n",
                "3. Implement the smallest general change that satisfies Break the warehouse restocking rewrite into sub-tasks. Deadline: the end of the quarter [requested_behavior_passes]\n",
                "4. Run the acceptance checks for Break the warehouse restocking rewrite into sub-tasks. Deadline: the end of the quarter [acceptance_checks_pass]",
            ),
        ),
        (
            "Split the seat-booking migration into sub-tasks. Owner: the reservations \
             team.",
            "seat-booking migration",
            concat!(
                "Sub-tasks, each with a completion criterion you can observe:\n",
                "1. Record independently checkable requirements for Split the seat-booking migration into sub-tasks. Owner: the reservations team [requirements_are_independently_checkable]\n",
                "2. Add a regression test that reproduces Split the seat-booking migration into sub-tasks. Owner: the reservations team [regression_test_reproduces_failure]\n",
                "3. Implement the smallest general change that satisfies Split the seat-booking migration into sub-tasks. Owner: the reservations team [requested_behavior_passes]\n",
                "4. Run the acceptance checks for Split the seat-booking migration into sub-tasks. Owner: the reservations team [acceptance_checks_pass]",
            ),
        ),
        (
            "Разбей переработку складского учёта на подзадачи. Срок: конец квартала.",
            "складского",
            concat!(
                "Подзадачи, у каждой из которых есть наблюдаемый критерий завершения:\n",
                "1. Зафиксируй независимо проверяемые требования для Разбей переработку складского учёта на подзадачи. Срок: конец квартала [requirements_are_independently_checkable]\n",
                "2. Добавь регрессионный тест, воспроизводящий Разбей переработку складского учёта на подзадачи. Срок: конец квартала [regression_test_reproduces_failure]\n",
                "3. Реализуй минимальное общее изменение, выполняющее Разбей переработку складского учёта на подзадачи. Срок: конец квартала [requested_behavior_passes]\n",
                "4. Запусти приёмочные проверки для Разбей переработку складского учёта на подзадачи. Срок: конец квартала [acceptance_checks_pass]",
            ),
        ),
    ] {
        let response = FormalAiEngine.answer(prompt);
        assert_eq!(
            response.answer.trim(),
            expected,
            "unexpected answer for {prompt:?}"
        );
        let answer = response.answer;
        let lines = numbered_lines(&answer);
        assert!(
            !lines.is_empty(),
            "nothing was enumerated for {prompt:?}: {answer:?}"
        );
        assert!(
            lines.iter().any(|line| line.contains(subject)),
            "the sub-tasks are about something other than {subject:?} for {prompt:?}: \
             {answer:?}"
        );
    }
}

#[test]
fn framing_addressed_to_the_solver_is_not_a_sub_task() {
    // An agent harness states the task, leaves a blank line, and then addresses
    // the solver: how to work, where to leave evidence, what not to claim. That
    // second block is not work of its own, and decomposing it beside the task
    // produced a sub-task made entirely of the framing sentences pasted
    // together -- a numbered line the reader cannot do anything with, which is
    // exactly the hollowness issue #1066 is about. Worded away from the #1028
    // ladder's own prompt so the rule is not the ladder's.
    let prompt = "Break the invoice reconciliation rewrite into sub-tasks.\n\nYou are \
                  worker 7 of 12. Work only in the scratch checkout. Its completion \
                  criterion is: every child reports independently. Leave what you find in \
                  worker-7.md. Do not claim success without evidence.";
    // The whole reply, so the fix is legible: four entries, every one of them
    // about the rewrite, and not one of them about being worker 7 of 12.
    let expected: &str = concat!(
        "Sub-tasks, each with a completion criterion you can observe:\n",
        "1. Record independently checkable requirements for Break the invoice reconciliation rewrite into sub-tasks [requirements_are_independently_checkable]\n",
        "2. Add a regression test that reproduces Break the invoice reconciliation rewrite into sub-tasks [regression_test_reproduces_failure]\n",
        "3. Implement the smallest general change that satisfies Break the invoice reconciliation rewrite into sub-tasks [requested_behavior_passes]\n",
        "4. Run the acceptance checks for Break the invoice reconciliation rewrite into sub-tasks [acceptance_checks_pass]",
    );
    let response = FormalAiEngine.answer(prompt);
    assert_eq!(
        response.answer.trim(),
        expected,
        "unexpected answer for the framed prompt"
    );
    let answer = response.answer;
    let lines = numbered_lines(&answer);
    assert!(
        !lines.is_empty(),
        "nothing was enumerated for the framed prompt: {answer:?}"
    );
    for line in &lines {
        assert!(
            line.contains("invoice reconciliation"),
            "a sub-task is about the framing rather than the task: {line:?} in {answer:?}"
        );
        for framing in [
            "worker 7 of 12",
            "scratch checkout",
            "Do not claim success",
            "worker-7.md",
        ] {
            assert!(
                !line.contains(framing),
                "the framing {framing:?} was enumerated as work: {line:?} in {answer:?}"
            );
        }
    }
}

#[test]
fn a_block_that_asks_survives_and_one_that_does_not_is_dropped() {
    // The block rule is scoped by the caller's own recogniser, so it is pinned
    // here with that recogniser spelled out rather than through a handler.
    let asks = |block: &str| block.to_lowercase().contains("sub-tasks");
    assert_eq!(
        stated_task(
            "Break the invoice reconciliation rewrite into sub-tasks.\n\nYou are worker 7 \
             of 12. Leave what you find in worker-7.md.",
            &asks,
        ),
        "Break the invoice reconciliation rewrite into sub-tasks"
    );
    assert_eq!(
        stated_task(
            "Read the handover memo.\n\nThen tell me what it says.",
            &asks,
        ),
        "Read the handover memo.\n\nThen tell me what it says",
        "when no block asks on its own, none of them is dropped"
    );
    assert_eq!(
        stated_task(
            "Break the invoice reconciliation rewrite into sub-tasks.\n\nThen break the \
             payroll export rewrite into sub-tasks.",
            &asks,
        ),
        "Break the invoice reconciliation rewrite into sub-tasks.\n\nThen break the \
         payroll export rewrite into sub-tasks",
        "a task stated across a blank line keeps both of its halves"
    );
}

#[test]
fn a_calculator_verb_in_the_framing_does_not_claim_the_whole_prompt() {
    // "Solve" is one of the calculator's request cues, and a harness that
    // addresses the solver uses it in its ordinary English sense. The cue was
    // read from where it appeared to the end of the prompt, so the framing --
    // sentences apart, and carrying the digits and the `=` of an unrelated
    // instruction -- became the expression. It could not be evaluated, and the
    // arithmetic reading answered anyway, at a confidence low enough to be
    // obviously wrong yet high enough to displace the decomposition the first
    // sentence actually asked for. A cue states its request in a sentence; it
    // does not own the rest of the document. Worded away from the #1028 ladder
    // so the rule is not the ladder's.
    let prompt = "Is the invoice reconciliation rewrite an atomic task?\n\nYou are worker 7 of \
                  12. Solve only what this worker owns in the scratch checkout. Write the outcome \
                  to worker-7.md with its first line set to worker_id=7.";
    let answer = FormalAiEngine.answer(prompt);
    assert_ne!(
        answer.intent, "calculation_error",
        "the framing was read as arithmetic: {:?}",
        answer.answer
    );
    assert_eq!(
        answer.intent, "task_atomicity",
        "the question about the task was not answered: {:?}",
        answer.answer
    );
    assert!(
        answer.answer.contains("invoice reconciliation"),
        "the answer is not about the task that was asked about: {:?}",
        answer.answer
    );
}

#[test]
fn a_calculator_verb_does_not_claim_the_rest_of_its_paragraph() {
    // Bounding the cue by a blank line alone is the same defect one step in:
    // the cue was read to the end of its *paragraph*, so every later sentence in
    // it joined the expression. Framing written as three blocks instead of two
    // put an unrelated `worker_id=7` inside the slice, the arithmetic reading
    // failed to evaluate it, and it answered in place of the question the first
    // block asked. A sentence bound and a paragraph bound both apply, and the
    // nearer of the two is the one that holds.
    let prompt = "Is the warehouse restocking rewrite an atomic task?\n\nYou are worker 7 of \
                  12. Solve only what this worker owns in the scratch checkout. Write the \
                  outcome to worker-7.md with its first line set to worker_id=7.\n\nAsk the \
                  coordinator when anything is unclear.";
    let answer = FormalAiEngine.answer(prompt);
    assert_ne!(
        answer.intent, "calculation_error",
        "the framing was read as arithmetic: {:?}",
        answer.answer
    );
    assert_eq!(
        answer.intent, "task_atomicity",
        "the question about the task was not answered: {:?}",
        answer.answer
    );
    assert!(
        answer.answer.contains("warehouse restocking"),
        "the answer is not about the task that was asked about: {:?}",
        answer.answer
    );
}

#[test]
fn a_colon_the_asking_sentence_owns_still_introduces_the_task() {
    // The scoping may not cost the shape it exists for. Issue #847's own
    // prompts introduce the task with a colon, and that colon belongs to the
    // sentence that asks, so it still wins over the whole prompt. The asking
    // sentence is whichever one the caller's own recogniser accepts; here that
    // is spelled out, so the rule is pinned without a handler in the way.
    let asks = |sentence: &str| sentence.to_lowercase().contains("sub-tasks");
    assert_eq!(
        stated_task(
            "Split this task into sub-tasks: add a paths-ignore filter to the release \
             workflow.",
            &asks,
        ),
        "add a paths-ignore filter to the release workflow"
    );
    assert_eq!(
        stated_task(
            "Break the warehouse restocking rewrite into sub-tasks. Deadline: the end of \
             the quarter.",
            &asks,
        ),
        "Break the warehouse restocking rewrite into sub-tasks. Deadline: the end of the \
         quarter"
    );
    assert_eq!(
        stated_task("Split this into steps", &asks),
        "Split this into steps",
        "a bare task with no colon and no quotation is the task"
    );
}

#[test]
fn an_answer_is_never_a_heading_with_no_list() {
    // The handler that composes a reply knows why it has nothing to enumerate
    // and says so; this is the backstop underneath it, for the callers that
    // deliver an answer somewhere a reader will later find it. A reply that
    // stops on the colon introducing its list is a heading with nothing under
    // it, and it passes every mechanical check a harness makes -- a file that
    // says nothing is still a non-empty file.
    for text in [
        "This task divides into the following sub-tasks:",
        "Задача делится на следующие подзадачи:",
        // Chinese and Japanese introduce a list with the full-width colon, so a
        // guard that only read ASCII would hold for four supported languages
        // and not the fifth.
        "该任务可分解为以下子任务：",
    ] {
        let answer = announcing(text);
        assert!(
            answer.announces_a_list_it_does_not_make(),
            "an answer that promised a list and made none was not recognised: {text:?}"
        );
    }
    for text in [
        "This task divides into two sub-tasks:\n1. Write the migration.\n2. Verify it.",
        "It is an irreducible single need.",
    ] {
        let answer = announcing(text);
        assert!(
            !answer.announces_a_list_it_does_not_make(),
            "an answer that kept its promise was refused: {text:?}"
        );
    }
}

#[test]
fn the_refusal_to_enumerate_is_written_in_the_language_that_asked() {
    // An honest refusal answered in the wrong language is hollow in a second
    // way: the reader is told something true in words they did not ask in.
    // `localized_response` falls back to English for a language the seed has no
    // record for, so a missing translation does not fail loudly -- it answers a
    // Spanish speaker in English, which is why this pins the exact sentence per
    // language instead of asserting that some text came back. Both reasons a
    // decomposition can enumerate nothing are checked, because they are
    // separate seed records and either one could be the untranslated one.
    let cases = [
        (
            "en",
            "This task cannot be split into two sub-tasks that can be checked independently, and \
             no observable completion criterion is known for it, so there is nothing to \
             enumerate: it is an irreducible single need.",
            "The decomposition depth bound was reached before this task was split even once, so \
             there is nothing to enumerate: raise the bound to see its sub-tasks.",
        ),
        (
            "ru",
            "Эту задачу нельзя разбить на две подзадачи, которые можно проверить по отдельности, \
             и наблюдаемый критерий завершения для неё неизвестен, поэтому перечислять нечего: \
             это неделимая единичная потребность.",
            "Предел глубины разбиения был достигнут до того, как задачу разбили хотя бы один раз, \
             поэтому перечислять нечего: увеличь предел, чтобы увидеть подзадачи.",
        ),
        (
            "hi",
            "इस कार्य को ऐसे दो उपकार्यों में नहीं बाँटा जा सकता जिन्हें अलग-अलग जाँचा जा सके, और इसके लिए कोई देखा जा \
             सकने वाला पूर्णता मानदंड ज्ञात नहीं है, इसलिए गिनाने के लिए कुछ नहीं है: यह एक अविभाज्य एकल आवश्यकता है।",
            "इस कार्य को एक बार भी बाँटे जाने से पहले ही विभाजन गहराई की सीमा आ गई, इसलिए गिनाने के लिए कुछ नहीं \
             है: उपकार्य देखने के लिए सीमा बढ़ाएँ।",
        ),
        (
            "zh",
            "这个任务无法拆分成两个可以各自独立检查的子任务，也没有已知的可观察完成标准，因此没有可列举的内容：它是一个不可再分的单一需求。",
            "在这个任务被拆分哪怕一次之前就已达到拆分深度上限，因此没有可列举的内容：提高上限即可看到它的子任务。",
        ),
        (
            "es",
            "Esta tarea no puede dividirse en dos subtareas que puedan comprobarse por separado, y \
             no se conoce ningún criterio de finalización observable para ella, así que no hay nada \
             que enumerar: es una necesidad única indivisible.",
            "Se alcanzó el límite de profundidad de descomposición antes de que esta tarea se \
             dividiera siquiera una vez, así que no hay nada que enumerar: sube el límite para ver \
             sus subtareas.",
        ),
    ];
    for (language, single_need, depth_bound) in cases {
        let answer = refusal_in(
            language,
            4,
            "Split the following into sub-tasks: charm the auditors.",
        );
        assert_eq!(answer, single_need, "single need in {language}");
        let answer = refusal_in(
            language,
            0,
            "Split the following into sub-tasks: build the analytics dashboard.",
        );
        assert_eq!(answer, depth_bound, "depth bound in {language}");
    }
}

/// The answer `prompt` gets from a client asking in `language`, with the
/// recursion bounded at `depth`.
fn refusal_in(language: &'static str, depth: u8, prompt: &str) -> String {
    let solver = UniversalSolver::new(SolverConfig {
        forced_response_language: Some(language),
        max_decomposition_depth: depth,
        offline: true,
        compute_budget: 0,
        ..SolverConfig::default()
    });
    solver.solve(prompt).answer.trim().to_owned()
}

/// An answer carrying exactly `text`, for testing the shape of what it says.
fn announcing(text: &str) -> SymbolicAnswer {
    SymbolicAnswer {
        intent: "task_decomposition".to_owned(),
        answer: text.to_owned(),
        confidence: 1.0,
        evidence_links: Vec::new(),
        thinking_steps: Vec::new(),
        links_notation: String::new(),
        execution_recipe: None,
    }
}
