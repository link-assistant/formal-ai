//! Executable grounding for the dynamic coding-discovery recipe (#710).

use std::fs;
use std::path::Path;

const RECIPE: &str = "data/meta/coding-discovery-recipe.lino";

fn read(relative: &str) -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("the repository root sits one level above the crate")
            .join(relative),
    )
    .unwrap_or_else(|error| panic!("{relative}: {error}"))
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
fn coding_discovery_recipe_is_grounded_in_the_live_pipeline() {
    let recipe = read(RECIPE);
    let records = recipe
        .lines()
        .fold(Vec::<String>::new(), |mut records, line| {
            if !line.starts_with(char::is_whitespace) {
                records.push(String::new());
            }
            let record = records.last_mut().expect("record header");
            record.push_str(line);
            record.push('\n');
            records
        });
    let header = records.first().expect("meta recipe header");
    assert_eq!(field(header, "record_type"), "meta_recipe");
    assert_eq!(field(header, "topic"), "dynamic_coding_discovery");
    assert_eq!(field(header, "issue"), "710");

    for source_field in ["source"] {
        for line in header
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with(&format!("{source_field} ")))
        {
            let path = line.trim_start_matches(&format!("{source_field} "));
            assert!(
                !read(path).is_empty(),
                "recipe cites an empty source: {path}"
            );
        }
    }

    // Issue #1138, plan 01 L16: `understand` is a step of its own, between
    // recognising the request and discovering parts for it. Before the live
    // concept lookup there was nothing for it to name: a word the seed did not
    // contain was skipped, so the recipe would have claimed a step the code
    // never ran. It is added here, with the leaf that made the coding path run
    // it (plan 01 L10), and not before.
    let expected = [
        ("recognise", "rust/src/coding/task_spec.rs"),
        ("lookup_surface", "rust/src/concept_lookup.rs"),
        ("discover", "rust/src/coding/concept_discovery.rs"),
        ("compose", "rust/src/coding/composition.rs"),
        ("run_command", "rust/src/agent.rs"),
        ("remember", "rust/src/coding/discovered_procedures.rs"),
    ];
    let steps = records
        .iter()
        .filter(|record| field(record, "record_type") == "meta_step")
        .collect::<Vec<_>>();
    assert_eq!(steps.len(), expected.len());

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
