//! The architect's notes are kept in the repository, the vision is updated from
//! them, and nothing contradicts it.
//!
//! The architect asked to "keep track of architect's (me) notes on the system
//! progress", and separately that the vision itself stay in `VISION.md`:
//!
//! > it is not `ARCHITECT-VISION.md` the name is wrong, it is my notes, and I
//! > explicitly asked about that. Vision is in `VISION.md` it is still my
//! > vision, yet it must be updated with all the latests notes. And better to
//! > have a folder for arhictect-notes in docs, so we can list them in
//! > chronological order.
//!
//! So there is one vision document and one folder of dated notes. These tests
//! pin both, and the two drifts that made them necessary: a vision that lived
//! only in issue threads had to be repeated, and documents drifted into framings
//! the architect never asked for (the division of `src` into a privileged part
//! and the rest, and the rule that Rust "may only shrink").

use std::fs;
use std::path::{Path, PathBuf};

fn read(relative: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("{relative} must be readable: {error}"))
}

/// The vision document states each part of the vision, in the architect's own
/// words. There is one of these, and it is `VISION.md`.
#[test]
fn the_vision_is_kept_in_the_repository() {
    assert!(
        !Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("ARCHITECT-VISION.md")
            .exists(),
        "the vision belongs in VISION.md; a second vision document competes with it"
    );
    // Quoted blocks are line-wrapped, so a clause can be split across lines.
    // Compare on collapsed whitespace with the quote markers removed, which
    // makes the assertion about the words rather than the column width.
    let vision = read("VISION.md")
        .lines()
        .map(|line| line.trim_start().trim_start_matches('>').trim())
        .collect::<Vec<_>>()
        .join(" ");
    let vision = vision.split_whitespace().collect::<Vec<_>>().join(" ");
    for clause in [
        // The goal, in the architect's words.
        "The goal of the project is to produce the meta algorithm",
        // Code held in the meta language, emitted into languages.
        "full representation of rust code",
        "translatable by our own Formal AI system into any programming language",
        // Approval, in three states.
        "by default not approved for safety without user permission",
        "we can ask him to aprove it forever",
        // Deduplication as the route to algorithms.
        "natural deduplicator",
        "we can infer algorithms from them purely algorithmically",
        // No task is rated.
        "nothing is hard task",
        "recursively simple",
        // The practical aim.
        "hello world application in top 10-20 languages",
    ] {
        assert!(
            vision.contains(clause),
            "VISION.md must record the architect's words: {clause:?}"
        );
    }
}

/// The vision is reachable from what a contributor or an agent opens first.
#[test]
fn the_vision_is_linked_from_the_entry_points() {
    for entry_point in ["README.md", "CONTRIBUTING.md"] {
        assert!(
            read(entry_point).contains("VISION.md"),
            "{entry_point} must point at the vision"
        );
    }
}

/// The notes live in one folder, dated so they list in chronological order, and
/// the vision points at them.
#[test]
fn the_notes_are_kept_in_chronological_order() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/architect-notes");
    assert!(dir.is_dir(), "docs/architect-notes/ must exist");

    let mut notes: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("the notes folder is readable")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
        .filter(|path| path.file_name().is_some_and(|name| name != "README.md"))
        .collect();
    assert!(notes.len() >= 3, "expected several notes, found {notes:?}");

    // A leading ISO date is what makes the directory listing chronological.
    for note in &notes {
        let name = note
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        let date: Vec<&str> = name.splitn(4, '-').take(3).collect();
        assert!(
            date.len() == 3
                && date[0].len() == 4
                && date[0].chars().all(|c| c.is_ascii_digit())
                && date[1].len() == 2
                && date[2].len() == 2,
            "{name} must begin with an ISO date so the notes list in order"
        );
    }

    notes.sort();
    let index = read("docs/architect-notes/README.md");
    for note in &notes {
        let name = note
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        assert!(
            index.contains(name),
            "{name} is not listed in docs/architect-notes/README.md"
        );
    }

    assert!(
        read("VISION.md").contains("docs/architect-notes/"),
        "VISION.md must say where the notes it is updated from live"
    );
}

/// Rust is one emission target of the meta-language representation, so no
/// document may state that Rust outside a named kernel may only shrink.
#[test]
fn no_document_demands_that_rust_only_shrinks() {
    for document in [
        "VISION.md",
        "GOALS.md",
        "ROADMAP.md",
        "REQUIREMENTS.md",
        "docs/requirements-traceability.md",
    ] {
        let text = read(document);
        assert!(
            !text.contains("Rust outside it may only shrink"),
            "{document} contradicts the vision: Rust is an emission target, and a count of \
             its lines measures an output rather than the system"
        );
        assert!(
            !text.contains("Keep the Rust kernel named and shrinking"),
            "{document} contradicts the vision: there is no kernel/non-kernel division"
        );
    }
}

/// No task is rated before it is split.
#[test]
fn the_goals_do_not_rate_a_task_before_splitting_it() {
    let goals = read("GOALS.md");
    assert!(
        !goals.contains("Split hard tasks"),
        "GOALS.md must not rate a task as hard: complex tasks are composed of simple tasks, \
         so the task is split rather than judged"
    );
    assert!(
        goals.contains("No task is rated hard, complex or ambitious before it is split"),
        "GOALS.md must state that a task is split rather than rated"
    );
}

/// The withdrawal is recorded where the requirement was claimed as delivered,
/// so the history stays honest about what was built and then withdrawn.
#[test]
fn the_withdrawn_requirement_says_so_where_it_was_claimed() {
    // `REQUIREMENTS.md` is assembled from `docs/requirements/` by
    // `scripts/assemble-requirements.rs`, so the shard is asserted too: an edit
    // to the assembled file alone is overwritten by the next regeneration.
    for document in [
        "REQUIREMENTS.md",
        "docs/requirements/issue-1085-the-links-network-is-not-the-system-that-reasons.md",
    ] {
        let text = read(document);
        let row = text
            .lines()
            .find(|line| line.starts_with("| R1085-1 |"))
            .unwrap_or_else(|| panic!("{document} no longer carries an R1085-1 row"));
        // The substance, not one phrasing: the row has to say it is superseded
        // and to name what was superseded with it.
        assert!(
            row.contains("Superseded") || row.contains("superseded"),
            "{document}: R1085-1 must record that it is superseded rather than \
             silently disappear, but reads: {row}"
        );
        assert!(
            row.contains("kernel"),
            "{document}: R1085-1 must say the kernel/non-kernel split went with it, \
             but reads: {row}"
        );
    }
}
