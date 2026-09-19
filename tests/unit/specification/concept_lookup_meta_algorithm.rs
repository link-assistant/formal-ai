//! Executable grounding for the concept-lookup recipe (issue #1138, plan 01 L16).
//!
//! The recipe is a claim about the live source: every `function` it names must
//! exist as `fn <name>` in the file it names, its steps must be contiguous, and
//! every consumer it declares must be a file that exists. A recipe that cannot
//! be checked against the tree is prose, not a meta-algorithm.

use std::fs;
use std::path::Path;

const RECIPE: &str = "data/meta/concept-lookup-recipe.lino";

fn read(relative: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("{relative}: {error}"))
}

fn records(text: &str) -> Vec<String> {
    text.lines()
        .fold(Vec::<String>::new(), |mut records, line| {
            if !line.starts_with(char::is_whitespace) {
                records.push(String::new());
            }
            let record = records.last_mut().expect("record header");
            record.push_str(line);
            record.push('\n');
            records
        })
}

fn field<'a>(record: &'a str, name: &str) -> &'a str {
    record
        .lines()
        .find_map(|line| {
            let line = line.trim();
            line.strip_prefix(name)
                .and_then(|value| value.strip_prefix(' '))
        })
        .unwrap_or_else(|| panic!("record is missing {name}:\n{record}"))
}

#[test]
fn concept_lookup_recipe_is_grounded_in_the_live_retrieval_kernel() {
    let recipe = read(RECIPE);
    let records = records(&recipe);
    let header = records.first().expect("meta recipe header");

    assert_eq!(field(header, "record_type"), "meta_recipe");
    assert_eq!(field(header, "topic"), "live_concept_lookup");
    assert_eq!(field(header, "issue"), "1138");

    for line in header
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("consumer "))
    {
        let path = line.trim_start_matches("consumer ");
        assert!(
            !read(path).is_empty(),
            "the recipe names a consumer that does not exist: {path}"
        );
    }

    let expected = [
        ("unknown_surfaces", "src/concept_lookup.rs"),
        ("select_sources", "src/source_walk.rs"),
        ("walk_sources", "src/source_walk.rs"),
        ("lookup_surface", "src/concept_lookup.rs"),
        ("remember", "src/concept_sense_ledger.rs"),
    ];
    let steps = records
        .iter()
        .filter(|record| field(record, "record_type") == "meta_step")
        .collect::<Vec<_>>();
    assert_eq!(steps.len(), expected.len(), "the recipe has five steps");

    for (index, (function, source)) in expected.into_iter().enumerate() {
        let step = steps[index];
        assert_eq!(field(step, "order"), (index + 1).to_string());
        assert_eq!(field(step, "function"), function);
        assert!(
            read(source).contains(&format!("fn {function}")),
            "{source} must define the function named by recipe step {}",
            index + 1
        );
    }
}
