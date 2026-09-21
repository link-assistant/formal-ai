//! Executable grounding for the deep-formalization recipe (issue #1138, plan
//! 04 L17).
//!
//! Same contract as the coding-discovery recipe: every `function` the recipe
//! names exists as `fn <name>` in the file it names, `order` is contiguous
//! 1..6, and every declared consumer path exists in the tree.

use std::fs;
use std::path::Path;

const RECIPE: &str = "data/meta/formalization-depth-recipe.lino";

fn read(relative: &str) -> String {
    let is_crate_namespace = relative.starts_with("src/")
        || relative.starts_with("tests/")
        || relative.starts_with("examples/");
    let base = if is_crate_namespace {
        Path::new(env!("CARGO_MANIFEST_DIR"))
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("the repository root sits one level above the crate")
    };
    fs::read_to_string(base.join(relative)).unwrap_or_else(|error| panic!("{relative}: {error}"))
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
fn formalization_depth_recipe_is_grounded_in_the_live_formalizer() {
    let recipe = read(RECIPE);
    let records = records(&recipe);
    let header = records.first().expect("meta recipe header");

    assert_eq!(field(header, "record_type"), "meta_recipe");
    assert_eq!(field(header, "topic"), "deep_formalization");
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
        ("sentences", "rust/src/formalization/segment.rs"),
        ("emit_needs", "rust/src/formalization/needs.rs"),
        ("satisfy_needs", "rust/src/formalization/needs.rs"),
        ("concept_from_sense", "rust/src/formalization/concepts.rs"),
        (
            "procedure_from_steps",
            "rust/src/formalization/procedures.rs",
        ),
        (
            "formalize_deeply",
            "rust/src/formalization/concept_links.rs",
        ),
    ];
    let steps = records
        .iter()
        .filter(|record| field(record, "record_type") == "meta_step")
        .collect::<Vec<_>>();
    assert_eq!(steps.len(), expected.len(), "the recipe has six steps");

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
