//! The architect's vision is kept in the repository, and nothing contradicts it.
//!
//! `ARCHITECT-VISION.md` is the standing guideline, recorded in the architect's
//! own words. These tests pin the two things that made it necessary: a vision
//! that lived only in issue threads had to be repeated, and documents drifted
//! into framings the architect never asked for (the kernel/non-kernel split and
//! the rule that Rust "may only shrink").

use std::fs;
use std::path::Path;

fn read(relative: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("{relative} must be readable: {error}"))
}

/// The guideline exists and states each part of the vision it records.
#[test]
fn the_guideline_is_kept_in_the_repository() {
    let vision = read("ARCHITECT-VISION.md");
    for clause in [
        // The goal, in the architect's words.
        "The goal of the project is to produce the meta algorithm",
        // Code held in the meta language, emitted into languages.
        "full representation of rust code",
        "translatable by our own Formal AI system into any programming language",
        // Approval, in three states.
        "by default not approved",
        "approve it forever",
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
            "ARCHITECT-VISION.md must record the architect's words: {clause:?}"
        );
    }
}

/// The guideline is reachable from what a contributor or an agent opens first.
#[test]
fn the_guideline_is_linked_from_the_entry_points() {
    for entry_point in ["README.md", "CONTRIBUTING.md"] {
        assert!(
            read(entry_point).contains("ARCHITECT-VISION.md"),
            "{entry_point} must point at the standing guideline"
        );
    }
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
    let requirements = read("REQUIREMENTS.md");
    assert!(
        requirements.contains("| R1085-1 | Superseded, and the kernel/non-kernel split with it."),
        "R1085-1 must record that it is superseded rather than silently disappear"
    );
}
