use super::*;

#[test]
fn the_preamble_sorts_first_then_issues_by_number_then_doctrine() {
    let mut names = vec![
        "doctrine-compiled-logic.md",
        "issue-0991-how-to-synthesis.md",
        "preamble-requirements-for-issue-1.md",
        "issue-0016-multilingual.md",
        "issue-0016-follow-up.md",
    ];
    names.sort_by_key(|name| shard_order(name).expect("named correctly"));
    assert_eq!(
        names,
        [
            "preamble-requirements-for-issue-1.md",
            "issue-0016-follow-up.md",
            "issue-0016-multilingual.md",
            "issue-0991-how-to-synthesis.md",
            "doctrine-compiled-logic.md",
        ]
    );
}

#[test]
fn issue_numbers_order_numerically_not_lexicographically() {
    assert!(shard_order("issue-0016-a.md") < shard_order("issue-0117-a.md"));
    assert!(shard_order("issue-0991-a.md") < shard_order("issue-1000-a.md"));
}

#[test]
fn an_unprefixed_shard_name_is_rejected() {
    assert_eq!(shard_order("current-scope-boundary.md"), None);
    assert_eq!(shard_order("issue-0991-notes.txt"), None);
}

#[test]
fn a_heading_names_the_first_issue_it_mentions() {
    assert_eq!(
        heading_issue("## Issue #16 Follow-Up: Universal Data/Seed (PR #17 reopen)"),
        Some(16)
    );
    assert_eq!(
        heading_issue("## Issue #398 PR Review Standards (comment 4663407299)"),
        Some(398)
    );
    assert_eq!(heading_issue("## Current Scope Boundary"), None);
}

#[test]
fn a_shard_name_is_derived_from_its_heading() {
    assert_eq!(
        shard_name("## Issue #991 Dynamic Multi-Source How-To Synthesis"),
        "issue-0991-dynamic-multi-source-how-to-synthesis.md"
    );
    assert_eq!(
        shard_name("## Standing Doctrine: Compiled Logic (2026-08-04)"),
        "doctrine-standing-doctrine-compiled-logic-2026-08-04.md"
    );
}

#[test]
fn splitting_keeps_the_preamble_apart_from_the_sections() {
    let (preamble, sections) = sections("# Title\n\nIntro.\n\n## One\n\na\n\n## Two\n\nb\n");
    assert_eq!(preamble, "# Title\n\nIntro.\n\n");
    assert_eq!(sections, ["## One\n\na\n\n", "## Two\n\nb\n"]);
}

/// One area that holds every shard.
fn everything() -> Vec<Area> {
    parse_areas("requirement-areas\n  area all\n    title \"All\"\n    match \"\"\n")
}

#[test]
fn areas_are_read_in_order_with_their_titles_and_fragments() {
    let areas = parse_areas(
        "# comment\nrequirement-areas\n  area doctrine\n    title \"Doctrine\"\n    \
         match \"doctrine-\"\n    match \"preamble-\"\n  area rest\n    title \"Rest\"\n    \
         match \"\"\n",
    );
    assert_eq!(
        areas,
        vec![
            Area {
                name: "doctrine".to_string(),
                title: "Doctrine".to_string(),
                matches: vec!["doctrine-".to_string(), "preamble-".to_string()],
            },
            Area {
                name: "rest".to_string(),
                title: "Rest".to_string(),
                matches: vec![String::new()],
            },
        ]
    );
}

#[test]
fn assembly_separates_sections_by_exactly_one_blank_line() {
    let directory = std::env::temp_dir().join("assemble-requirements-blank-lines");
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).expect("temp dir");
    fs::write(directory.join("preamble-a.md"), "# Title\n\nIntro.\n").expect("write");
    fs::write(directory.join("issue-0002-b.md"), "## Issue #2\n\nb\n\n\n").expect("write");
    let shards = ordered_shards(&directory).expect("ordered");
    let areas = everything();
    let assembly = assemble(&shards, &areas).expect("assembled");
    assert_eq!(assembly.parts.len(), 1);
    assert_eq!(assembly.parts[0].0, "all.md");
    assert_eq!(
        assembly.parts[0].1,
        format!(
            "{}# Title\n\nIntro.\n\n## Issue #2\n\nb\n",
            part_header(&areas[0])
        )
    );
    assert!(
        assembly
            .index
            .contains("1. [All](docs/requirements/assembled/all.md)\n")
    );
    assert!(assembly.index.contains("   - Title\n   - Issue #2\n"));
    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn a_part_header_is_exactly_its_counted_lines() {
    assert_eq!(
        part_header(&everything()[0]).lines().count(),
        PART_HEADER_LINES
    );
}

#[test]
fn a_shard_joins_the_first_area_that_matches_its_name() {
    let areas = parse_areas(
        "requirement-areas\n  area doctrine\n    title \"Doctrine\"\n    match \"doctrine-\"\n  \
         area coding\n    title \"Coding\"\n    match \"coding\"\n  area rest\n    \
         title \"Rest\"\n    match \"\"\n",
    );
    let sizes = vec![
        ("issue-0001-coding.md".to_string(), 5),
        ("issue-0002-facts.md".to_string(), 5),
        ("doctrine-a.md".to_string(), 5),
    ];
    assert_eq!(
        group(&sizes, &areas).expect("grouped"),
        vec![(0, vec![2]), (1, vec![0]), (2, vec![1])]
    );
}

#[test]
fn an_area_too_large_for_one_part_is_rejected() {
    let half = PART_LINE_LIMIT / 2;
    let sizes = vec![
        ("issue-0001-a.md".to_string(), half),
        ("issue-0002-a.md".to_string(), half),
    ];
    let error = group(&sizes, &everything()).expect_err("too large");
    assert!(error.contains("the all area"), "{error}");
}

#[test]
fn a_shard_in_no_area_is_rejected() {
    let areas = parse_areas("requirement-areas\n  area doctrine\n    match \"doctrine-\"\n");
    let sizes = vec![("issue-0001-a.md".to_string(), 5)];
    let error = group(&sizes, &areas).expect_err("no area");
    assert!(error.contains("issue-0001-a.md"), "{error}");
}

#[test]
fn a_shard_link_is_rebased_into_the_parts_directory() {
    for (shard, part) in [
        ("../upload-memory.md", "../../upload-memory.md"),
        (
            "../../scripts/close-total.py",
            "../../../scripts/close-total.py",
        ),
        ("issue-0709-fusion.md", "../issue-0709-fusion.md"),
        ("https://example.org/a.md", "https://example.org/a.md"),
        ("#anchor", "#anchor"),
    ] {
        assert_eq!(
            shard_to_part_relative(shard),
            part,
            "{shard} lands in a part as {part}"
        );
    }
}

#[test]
fn a_shard_link_is_rebased_to_the_repository_root_and_back() {
    for (shard, root) in [
        ("../upload-memory.md", "docs/upload-memory.md"),
        (
            "../design/coverage-ratchet.md",
            "docs/design/coverage-ratchet.md",
        ),
        (
            "../requirements-traceability.md",
            "docs/requirements-traceability.md",
        ),
        ("../../scripts/close-total.py", "scripts/close-total.py"),
        (
            "issue-0709-fusion.md",
            "docs/requirements/issue-0709-fusion.md",
        ),
    ] {
        assert_eq!(to_root_relative(shard), root, "{shard} assembles to {root}");
        assert_eq!(to_shard_relative(root), shard, "{root} shards to {shard}");
    }
}

#[test]
fn a_link_to_the_shard_directory_itself_stays_a_link() {
    assert_eq!(to_shard_relative(SHARDS), ".");
}

#[test]
fn a_link_that_means_the_same_thing_everywhere_is_left_alone() {
    for target in [
        "https://example.org/a.md",
        "#an-anchor",
        "/docs/absolute.md",
        "mailto:someone@example.org",
    ] {
        assert_eq!(to_root_relative(target), target);
        assert_eq!(to_shard_relative(target), target);
    }
}

#[test]
fn rewriting_touches_the_target_and_nothing_else() {
    let shard = "See [`docs/upload-memory.md`](../upload-memory.md) and \
                 [the spec](https://example.org) (a note) — cost: $1 (approx).\n";
    let root = "See [`docs/upload-memory.md`](docs/upload-memory.md) and \
                [the spec](https://example.org) (a note) — cost: $1 (approx).\n";
    assert_eq!(rewrite_links(shard, &to_root_relative), root);
    assert_eq!(rewrite_links(root, &to_shard_relative), shard);
}

#[test]
fn a_link_title_survives_rewriting() {
    assert_eq!(
        rewrite_links("[a](../b.md \"Title\")", &to_root_relative),
        "[a](docs/b.md \"Title\")"
    );
}

#[test]
fn a_shard_link_that_resolves_only_from_the_repository_root_fails() {
    let shards = [PathBuf::from("docs/requirements/issue-0018-memory.md")];
    let read = |_path: &Path| "## Issue #18\n\nSee [it](docs/upload-memory.md).\n".to_string();
    let exists = |path: &Path| path == Path::new("docs/upload-memory.md");
    let failures = link_violations(&shards, &read, &exists);
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert!(
        failures[0].contains("docs/upload-memory.md"),
        "{failures:?}"
    );
}

#[test]
fn a_shard_link_that_resolves_from_the_shard_passes() {
    let shards = [PathBuf::from("docs/requirements/issue-0018-memory.md")];
    let read = |_path: &Path| "## Issue #18\n\nSee [it](../upload-memory.md#top).\n".to_string();
    let exists = |path: &Path| path == Path::new("docs/requirements/../upload-memory.md");
    assert!(link_violations(&shards, &read, &exists).is_empty());
}

#[test]
fn a_shard_named_for_the_wrong_issue_fails() {
    let shards = [PathBuf::from("docs/requirements/issue-0991-copy.md")];
    let read = |_path: &Path| "## Issue #709 Multi-Source Search Fusion\n".to_string();
    let failures = shard_violations(&shards, &read);
    assert!(
        failures
            .iter()
            .any(|failure| failure.contains("issue-0709-")),
        "{failures:?}"
    );
}

#[test]
fn a_shard_named_for_its_own_issue_passes() {
    let shards = [PathBuf::from("docs/requirements/issue-0709-fusion.md")];
    let read = |_path: &Path| "## Issue #709 Multi-Source Search Fusion\n".to_string();
    assert!(shard_violations(&shards, &read).is_empty());
}
