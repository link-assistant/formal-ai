//! Regression coverage for issue #892: the Spider-Man release-order answer is
//! computed from a source-backed, timestamped snapshot instead of a frozen
//! sentence.
//!
//! The three cases the issue asks for are covered against fixed days, so they
//! never depend on when the suite runs:
//!
//!   * *current* — a day inside the freshness window lists every already
//!     released work in date order and nothing else,
//!   * *stale cache* — a day past the freshness window still answers, but says
//!     the snapshot may be missing later releases,
//!   * *future release* — a work dated after the asking day is reported as
//!     announced, and starts counting as released once that day arrives.

use formal_ai::seed::{parse_release_timelines, release_timelines};
use formal_ai::{release_timeline, UniversalSolver};

/// A two-film timeline: one long released, one announced for 2030.
const FIXTURE: &str = r#"release_timelines
  phrasing en
    released-heading "Released {subject} in release order:"
    released-item "{position}. {title} ({year})"
    announced-heading "Announced but not yet released:"
    announced-item "{title} ({date})"
    undated-item "{title} (no announced date)"
    item-separator "; "
    section-end "."
    provenance-note "Source: {source}, snapshot taken {retrieved}."
    stale-note "Source: {source}, snapshot taken {retrieved}; titles released after that date may be missing."
  timeline demo
    source-label "Wikidata Query Service"
    retrieved-at 2026-08-04
    fresh-for-days 180
    localized en
      subject "demo films"
    entry Q2
      release-date 2020-02-02
      localized en
        title "Second"
    entry Q1
      release-date 2010-01-01
      localized en
        title "First"
    entry Q3
      release-date 2030-03-03
      localized en
        title "Third"
    entry Q4
      localized en
        title "Undated"
"#;

#[test]
fn current_snapshot_lists_released_works_in_date_order() {
    let registry = parse_release_timelines(FIXTURE);
    let rendered = release_timeline::render_from(&registry, "demo", "en", "2026-09-01")
        .expect("the demo timeline should render");

    assert!(!rendered.stale, "one month is inside the 180-day window");
    assert_eq!(
        rendered
            .released
            .iter()
            .map(|entry| entry.qid.as_str())
            .collect::<Vec<_>>(),
        ["Q1", "Q2"],
        "released works are ordered by release date, not by snapshot order"
    );
    assert_eq!(
        rendered.text,
        "Released demo films in release order: 1. First (2010); 2. Second (2020). \
         Announced but not yet released: Third (2030-03-03); Undated (no announced date). \
         Source: Wikidata Query Service, snapshot taken 2026-08-04."
    );
}

#[test]
fn stale_snapshot_still_answers_but_flags_the_gap() {
    let registry = parse_release_timelines(FIXTURE);
    let fresh = release_timeline::render_from(&registry, "demo", "en", "2027-01-31")
        .expect("the demo timeline should render");
    let stale = release_timeline::render_from(&registry, "demo", "en", "2027-02-01")
        .expect("the demo timeline should render");

    assert!(!fresh.stale, "day 180 is still inside the window");
    assert!(stale.stale, "day 181 is past the window");
    assert!(
        stale.text.contains("may be missing"),
        "a stale snapshot warns about later releases, got: {}",
        stale.text
    );
    assert!(
        stale.text.contains("2026-08-04"),
        "a stale snapshot still names the day it was taken, got: {}",
        stale.text
    );
    assert_eq!(
        fresh.released, stale.released,
        "staleness changes the wording, never the classification"
    );
}

#[test]
fn a_future_release_is_announced_until_its_release_day() {
    let registry = parse_release_timelines(FIXTURE);

    let before = release_timeline::render_from(&registry, "demo", "en", "2030-03-02")
        .expect("the demo timeline should render");
    assert!(
        before
            .announced
            .iter()
            .any(|entry| entry.qid == "Q3" && entry.title_for("en") == "Third"),
        "a film dated tomorrow is announced, got: {}",
        before.text
    );
    assert!(
        !before.released.iter().any(|entry| entry.qid == "Q3"),
        "a film dated tomorrow is not released, got: {}",
        before.text
    );

    let on_release_day = release_timeline::render_from(&registry, "demo", "en", "2030-03-03")
        .expect("the demo timeline should render");
    assert_eq!(
        on_release_day
            .released
            .iter()
            .map(|entry| entry.qid.as_str())
            .collect::<Vec<_>>(),
        ["Q1", "Q2", "Q3"],
        "the film counts as released on its release day, got: {}",
        on_release_day.text
    );
    assert!(
        on_release_day
            .announced
            .iter()
            .all(|entry| entry.release_date.is_empty()),
        "only the undated film stays announced, got: {}",
        on_release_day.text
    );
}

#[test]
fn an_undated_work_is_never_reported_as_released() {
    let registry = parse_release_timelines(FIXTURE);
    let rendered = release_timeline::render_from(&registry, "demo", "en", "2999-12-31")
        .expect("the demo timeline should render");

    assert_eq!(
        rendered
            .announced
            .iter()
            .map(|entry| entry.qid.as_str())
            .collect::<Vec<_>>(),
        ["Q4"],
        "a work with no announced date can never become released"
    );
}

#[test]
fn the_seeded_spider_man_timeline_is_grounded_and_ordered() {
    let timeline = release_timelines()
        .timeline("spider_man_title_role_films")
        .expect("the Spider-Man timeline should be registered");

    assert_eq!(timeline.subject_qid, "Q2307877");
    assert_eq!(timeline.source_url, "https://query.wikidata.org/sparql");
    assert_eq!(
        timeline.sha256.len(),
        64,
        "the snapshot digest should be a full SHA-256, got {:?}",
        timeline.sha256
    );
    assert!(
        timeline.entries.len() >= 12,
        "the query returns every title-role film, got {}",
        timeline.entries.len()
    );
    for entry in &timeline.entries {
        assert!(
            entry.qid.starts_with('Q'),
            "every film keeps its Wikidata anchor, got {:?}",
            entry.qid
        );
        assert!(
            !entry.title_for("en").is_empty(),
            "{} should carry an English title",
            entry.qid
        );
    }

    // The films the issue lists, in the order the answer must keep.
    let rendered = release_timeline::render("spider_man_title_role_films", "en", "2026-08-04")
        .expect("the Spider-Man timeline should render");
    let ordered: Vec<&str> = rendered
        .released
        .iter()
        .map(|entry| entry.title_for("en"))
        .collect();
    assert_eq!(
        &ordered[..10],
        &[
            "Spider-Man",
            "Spider-Man 2",
            "Spider-Man 3",
            "The Amazing Spider-Man",
            "The Amazing Spider-Man 2",
            "Spider-Man: Homecoming",
            "Spider-Man: Into the Spider-Verse",
            "Spider-Man: Far From Home",
            "Spider-Man: No Way Home",
            "Spider-Man: Across the Spider-Verse",
        ],
        "the ten released films keep theatrical release order"
    );
}

/// Every registered language answers in its own words. The renderer falls back
/// to English when a language has no wording, which would silently hand a Hindi,
/// Chinese or Spanish reader an English sentence — so the seed must carry a
/// phrasing block, a subject and localized titles for every language the
/// registry declares, and each language's wording must be its own.
#[test]
fn every_registered_language_gets_its_own_release_timeline_wording() {
    let registry = release_timelines();
    let timeline = registry
        .timeline("spider_man_title_role_films")
        .expect("the Spider-Man timeline should be registered");
    let languages: Vec<&str> = formal_ai::language::registered_languages()
        .iter()
        .map(|language| language.slug())
        .collect();
    assert!(
        ["en", "ru", "hi", "zh", "es"]
            .iter()
            .all(|expected| languages.contains(expected)),
        "the registry should still declare en, ru, hi, zh and es, got {languages:?}"
    );

    let mut headings: Vec<(&str, &str)> = Vec::new();
    for language in &languages {
        let phrasing = registry
            .phrasing_for(language)
            .expect("English wording always exists");
        assert_eq!(
            phrasing.language, *language,
            "{language} answers fall back to {} wording",
            phrasing.language
        );
        assert!(
            timeline
                .subjects
                .iter()
                .any(|(candidate, _)| candidate == language),
            "the timeline subject is missing in {language}"
        );
        headings.push((language, phrasing.released_heading.as_str()));

        let rendered =
            release_timeline::render("spider_man_title_role_films", language, "2026-08-04")
                .expect("every registered language should render");
        assert!(
            rendered.text.starts_with(
                phrasing
                    .released_heading
                    .split("{subject}")
                    .next()
                    .unwrap_or_default()
            ),
            "the {language} answer should open with the {language} heading, got: {}",
            rendered.text
        );
        for entry in &timeline.entries {
            assert!(
                entry
                    .titles
                    .iter()
                    .any(|(candidate, _)| candidate == language),
                "{} has no {language} title",
                entry.qid
            );
        }
    }

    for (left, left_heading) in &headings {
        for (right, right_heading) in &headings {
            assert!(
                left == right || left_heading != right_heading,
                "the {left} and {right} headings are the same text: {left_heading:?}"
            );
        }
    }
}

/// The seed must be a faithful transcription of the checked-in snapshot, not a
/// hand-typed list that happens to look like one: every date is re-read from the
/// SPARQL answer, every title from the entity cache, and the recorded digest is
/// recomputed from the snapshot bytes.
#[test]
fn the_seeded_timeline_is_a_transcription_of_the_checked_in_cache() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let timeline = release_timelines()
        .timeline("spider_man_title_role_films")
        .expect("the Spider-Man timeline should be registered");

    let snapshot = std::fs::read(root.join(&timeline.cache_file))
        .unwrap_or_else(|error| panic!("{} should be checked in: {error}", timeline.cache_file));
    assert_eq!(
        formal_ai::sha256_hex(&snapshot),
        timeline.sha256,
        "{} records the digest of a different snapshot",
        timeline.cache_file
    );
    assert!(
        root.join(&timeline.query_file).exists(),
        "{} should ship the query the snapshot came from",
        timeline.query_file
    );

    let document: serde_json::Value =
        serde_json::from_slice(&snapshot).expect("the snapshot should be a SPARQL JSON answer");
    let mut expected: Vec<(String, String)> = document["results"]["bindings"]
        .as_array()
        .expect("the SPARQL answer should carry bindings")
        .iter()
        .map(|binding| {
            let qid = binding["film"]["value"]
                .as_str()
                .expect("every row names a film")
                .rsplit('/')
                .next()
                .expect("an entity URI ends in its id")
                .to_owned();
            let date = binding["firstRelease"]["value"]
                .as_str()
                .unwrap_or_default()
                .split('T')
                .next()
                .unwrap_or_default()
                .to_owned();
            (qid, date)
        })
        .collect();
    expected.sort_by(|left, right| {
        (left.1.is_empty(), &left.1, &left.0).cmp(&(right.1.is_empty(), &right.1, &right.0))
    });

    assert_eq!(
        timeline
            .entries
            .iter()
            .map(|entry| (entry.qid.clone(), entry.release_date.clone()))
            .collect::<Vec<_>>(),
        expected,
        "the seeded entries should be exactly the films and dates the snapshot returned"
    );

    for entry in &timeline.entries {
        let path = root.join(format!("data/cache/wikidata/entity/{}.json", entry.qid));
        let cached: serde_json::Value = serde_json::from_slice(
            &std::fs::read(&path)
                .unwrap_or_else(|error| panic!("{} should be cached: {error}", entry.qid)),
        )
        .expect("a cached entity should be JSON");
        for (language, title) in &entry.titles {
            assert_eq!(
                cached["entities"][&entry.qid]["labels"][language]["value"]
                    .as_str()
                    .unwrap_or_default(),
                title,
                "the {language} title of {} should be its cached Wikidata label",
                entry.qid
            );
        }
    }
}

#[test]
fn spider_man_answers_come_from_the_snapshot_not_from_a_sentence() {
    let solver = UniversalSolver::default();
    let response = solver.solve("List Spider-Man films in release order.");

    let timeline = release_timelines()
        .timeline("spider_man_title_role_films")
        .expect("the Spider-Man timeline should be registered");
    for expected in [
        "release_timeline:spider_man_title_role_films".to_owned(),
        format!("release_timeline:snapshot:{}", timeline.retrieved_at),
        format!("release_timeline:sha256:{}", timeline.sha256),
        format!("source:{}", timeline.source_url),
    ] {
        assert!(
            response.evidence_links.contains(&expected),
            "the answer should record {expected:?}, got {:?}",
            response.evidence_links
        );
    }
    assert!(
        response
            .thinking_steps
            .iter()
            .any(|step| step.source_event == "release_timeline:hit"),
        "the answer should show the timeline lookup in its reasoning, got {:?}",
        response.thinking_steps
    );
    assert!(
        response.answer.contains("Wikidata Query Service"),
        "the answer should name its source, got: {}",
        response.answer
    );
    // Whatever today is, the rendered snapshot is what the solver replies with.
    let today = formal_ai::external_benchmarks::today_utc();
    let expected = release_timeline::render("spider_man_title_role_films", "en", &today)
        .expect("the Spider-Man timeline should render");
    assert_eq!(response.answer, expected.text);
}
