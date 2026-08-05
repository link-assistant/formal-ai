//! Regression coverage for issue #462: Russian Spider-Man film release-order
//! prompts should resolve to a grounded fact lookup.
//!
//! Since issue #892 the answer is rendered from the dated Wikidata snapshot
//! rather than from a frozen sentence, so the expectations here are stated in
//! terms of that snapshot: the ten films the issue reported must still come out
//! in theatrical release order, in the language the question was asked in, and
//! films the snapshot dates after today must stay out of the numbered list.

use formal_ai::{release_timeline, ConversationTurn, SymbolicAnswer, UniversalSolver};

const REPORTED_PROMPT: &str = "Перечисли фильмы про человека-паука в порядке выхода на экран?";

const RELEASED_TITLE_ROLE_FILMS: [&str; 10] = [
    "Spider-Man (2002)",
    "Spider-Man 2 (2004)",
    "Spider-Man 3 (2007)",
    "The Amazing Spider-Man (2012)",
    "The Amazing Spider-Man 2 (2014)",
    "Spider-Man: Homecoming (2017)",
    "Spider-Man: Into the Spider-Verse (2018)",
    "Spider-Man: Far From Home (2019)",
    "Spider-Man: No Way Home (2021)",
    "Spider-Man: Across the Spider-Verse (2023)",
];

#[test]
fn reported_russian_spider_man_release_order_prompt_is_fact_lookup() {
    let solver = UniversalSolver::default();
    let history = [
        ConversationTurn::user("На php не получится написать?"),
        ConversationTurn::assistant("Вот минимальная программа Hello World на языке PHP."),
    ];

    let response = solver.solve_with_history(REPORTED_PROMPT, &history);

    assert_spider_man_release_order(&response, REPORTED_PROMPT, "ru");
}

#[test]
fn spider_man_release_order_variants_route_to_same_fact() {
    let solver = UniversalSolver::default();

    for (prompt, language) in [
        ("List Spider-Man films in release order.", "en"),
        ("Назови фильмы о человеке-пауке по порядку выхода.", "ru"),
        (
            "Перечисли фильмы про человека паука в порядке выхода.",
            "ru",
        ),
    ] {
        let response = solver.solve(prompt);
        assert_spider_man_release_order(&response, prompt, language);
    }
}

#[test]
fn english_answers_keep_the_reported_film_list_in_release_order() {
    let solver = UniversalSolver::default();
    let response = solver.solve("List Spider-Man films in release order.");

    assert_ordered(
        &response,
        "List Spider-Man films in release order.",
        |index| RELEASED_TITLE_ROLE_FILMS.get(index).copied(),
    );
}

fn assert_spider_man_release_order(response: &SymbolicAnswer, prompt: &str, language: &str) {
    assert_eq!(
        response.intent, "fact_lookup",
        "{prompt:?} should route to fact_lookup, got {} -> {}",
        response.intent, response.answer
    );
    assert!(
        response
            .thinking_steps
            .iter()
            .any(|step| step.source_event == "fact_lookup:hit"
                && step.detail == "fact_spider_man_films_release_order"),
        "{prompt:?} should select the Spider-Man release-order fact, got {:?}",
        response.thinking_steps
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "wikidata:Q2307877"),
        "{prompt:?} should keep the Spider-Man Wikidata anchor, got {:?}",
        response.evidence_links
    );

    let today = formal_ai::external_benchmarks::today_utc();
    let rendered = release_timeline::render("spider_man_title_role_films", language, &today)
        .expect("the Spider-Man timeline should render");

    assert_ordered(response, prompt, |index| {
        rendered
            .released
            .get(index)
            .map(|entry| entry.title_for(language))
    });
    assert!(
        !response
            .answer
            .contains(&format!("{}. ", rendered.released.len() + 1)),
        "{prompt:?} should number exactly the released films, got: {}",
        response.answer
    );
    for entry in &rendered.announced {
        assert!(
            entry.release_date.is_empty() || entry.release_date.as_str() > today.as_str(),
            "{prompt:?} should only hold back films dated after {today}, got {:?}",
            entry.release_date
        );
    }
}

/// Assert that the answer mentions every expected title, in order.
fn assert_ordered<'a>(
    response: &SymbolicAnswer,
    prompt: &str,
    expected: impl Fn(usize) -> Option<&'a str>,
) {
    let mut previous_index = None;
    let mut index = 0;
    while let Some(film) = expected(index) {
        let found = response.answer.find(film).unwrap_or_else(|| {
            panic!(
                "{prompt:?} answer should contain {film:?}, got: {}",
                response.answer
            )
        });
        if let Some(previous) = previous_index {
            assert!(
                previous < found,
                "{prompt:?} should keep theatrical release order, got: {}",
                response.answer
            );
        }
        previous_index = Some(found);
        index += 1;
    }
}
