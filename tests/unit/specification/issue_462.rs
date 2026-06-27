//! Regression coverage for issue #462: Russian Spider-Man film release-order
//! prompts should resolve to a grounded fact lookup.

use formal_ai::{ConversationTurn, SymbolicAnswer, UniversalSolver};

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

    assert_spider_man_release_order(&response, REPORTED_PROMPT);
}

#[test]
fn spider_man_release_order_variants_route_to_same_fact() {
    let solver = UniversalSolver::default();

    for prompt in [
        "List Spider-Man films in release order.",
        "Назови фильмы о человеке-пауке по порядку выхода.",
        "Перечисли фильмы про человека паука в порядке выхода.",
    ] {
        let response = solver.solve(prompt);
        assert_spider_man_release_order(&response, prompt);
    }
}

fn assert_spider_man_release_order(response: &SymbolicAnswer, prompt: &str) {
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

    let mut previous_index = None;
    for film in RELEASED_TITLE_ROLE_FILMS {
        let index = response.answer.find(film).unwrap_or_else(|| {
            panic!(
                "{prompt:?} answer should contain {film:?}, got: {}",
                response.answer
            )
        });
        if let Some(previous) = previous_index {
            assert!(
                previous < index,
                "{prompt:?} should keep theatrical release order, got: {}",
                response.answer
            );
        }
        previous_index = Some(index);
    }

    assert!(
        !response.answer.contains("Brand New Day"),
        "{prompt:?} should not include unreleased future films, got: {}",
        response.answer
    );
}
