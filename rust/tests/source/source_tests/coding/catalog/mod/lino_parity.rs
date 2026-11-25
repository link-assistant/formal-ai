//! The portable `data/seed/hello-world-programs.lino` knowledge bundle is a
//! hand-mirrored copy of this catalog. These tests lock the two together so
//! a task or template added to the Rust catalog can never silently drift
//! out of the downloadable seed (the divergence that previously left
//! `list_files_arg` out of the seed). Regenerate the seed blocks with
//! `experiments/issue-330-coding-tasks/generate_lino.py` when adding tasks.
use super::*;
use crate::seed::parser::{LinoNode, parse_lino};

#[test]
fn lino_seed_tasks_line_lists_every_catalog_task() {
    let root = seed_tree();
    let write_program = root
        .children
        .iter()
        .find(|node| node.name == WRITE_PROGRAM_INTENT)
        .expect("lino seed declares a tasks line");
    let listed = crate::seed::parser::split_pipe_list(
        child_value(write_program, "tasks").expect("write_program seed declares tasks"),
    );
    for task in PROGRAM_TASKS {
        assert!(
            listed.iter().any(|slug| slug == task.slug),
            "lino tasks line is missing `{}` (has {listed:?})",
            task.slug
        );
    }
}

/// The browser worker installs its task table from the seed's `task_<slug>`
/// rows (issue #921 R921-8), so each row must carry the catalog task's label
/// and any output fixed by the task.
#[test]
fn lino_seed_task_rows_mirror_every_catalog_task() {
    let root = seed_tree();
    for task in PROGRAM_TASKS {
        let row = root
            .children
            .iter()
            .find(|node| node.name == format!("task_{}", task.slug))
            .unwrap_or_else(|| panic!("lino seed has no task_{} row", task.slug));
        assert_eq!(child_value(row, "task"), Some(task.slug));
        assert_eq!(child_value(row, "label"), Some(task.label), "{}", task.slug);
        let runtime_input =
            task.slug.starts_with("list_files") || task.slug == "copy_stdin_to_stdout";
        assert_eq!(
            child_value(row, "output"),
            (!runtime_input && !task.output.is_empty()).then_some(task.output),
            "{}",
            task.slug
        );
        assert_eq!(child_value(row, "input"), None, "{}", task.slug);
    }
}

/// A seed row whose `program_source` is the documentation route stores no
/// program (issue #1165 R1165-4): the solver rediscovers it from
/// `data/seed/coding-documentation-captures.lino`, and the unit test
/// `retired_seed_programs_are_reproduced_by_the_documentation` holds the
/// compiled template equal to the rediscovered program instead.
const DOCUMENTATION_ROUTE: &str = "documentation_route";

#[test]
fn lino_seed_mirrors_every_catalog_template() {
    let root = seed_tree();
    for template in program_templates() {
        if program_source(&root, template.task_slug, template.language_slug)
            == Some(DOCUMENTATION_ROUTE)
        {
            continue;
        }
        let seed_code = template_code(&root, template.task_slug, template.language_slug)
            .or_else(|| rendered_template_code(&root, template.task_slug, template.language_slug));
        assert_eq!(
            seed_code.as_deref(),
            Some(template.code),
            "lino seed is missing the {}/{} template (escaped code not found)",
            template.task_slug,
            template.language_slug
        );
    }
}

#[test]
fn lino_seed_has_no_extra_templates() {
    let root = seed_tree();
    let seed_templates = root
        .children
        .iter()
        .filter(|node| {
            child_value(node, "code").is_some()
                || child_value(node, "program_source") == Some(DOCUMENTATION_ROUTE)
        })
        .count();
    let composed = program_templates()
        .filter(|template| {
            template_row(&root, template.task_slug, template.language_slug).is_none()
                && rendered_template_code(&root, template.task_slug, template.language_slug)
                    .is_some()
        })
        .count();
    assert_eq!(
        seed_templates + composed,
        program_template_count(),
        "lino seed template count ({seed_templates} rows + {composed} composed by a \
         print_renderer) must equal the catalog count ({})",
        program_template_count()
    );
}

/// The program the seed composes for a print-literal task (`operation
/// print_literal`, such as `hello_world`) from the language's
/// `print_renderer` row. Since commit 10a6f2855 the seed keeps no
/// `template_hello_world_laravel` row: the production catalog renders the
/// Laravel hello world from `print_renderer laravel` and the task's `output`
/// (`literal_output_program` in `rust/src/coding/catalog/mod.rs`), so the
/// parity check renders it the same way instead of looking for a row
/// (issue #1639).
fn rendered_template_code(root: &LinoNode, task_slug: &str, language_slug: &str) -> Option<String> {
    let task = root
        .children
        .iter()
        .find(|node| node.name == format!("task_{task_slug}"))?;
    if child_value(task, "operation") != Some("print_literal") {
        return None;
    }
    let output = child_value(task, "output")?;
    let renderer = root
        .children
        .iter()
        .find(|node| node.name == WRITE_PROGRAM_INTENT)?
        .children
        .iter()
        .find(|node| node.name == "print_renderer" && node.id == language_slug)?;
    let style = child_value(renderer, "literal").unwrap_or_default();
    let mut literal = match style {
        "shell" => format!("'{}'", output.replace('\'', "'\\''")),
        "php_single" => format!("'{}'", output.replace('\\', "\\\\").replace('\'', "\\'")),
        _ => {
            let mut quoted = String::from("\"");
            for character in output.chars() {
                match character {
                    '\t' => quoted.push_str("\\t"),
                    '\\' | '"' => {
                        quoted.push('\\');
                        quoted.push(character);
                    }
                    '$' if style == "dollar" => quoted.push_str("\\$"),
                    '#' if style == "ruby" => quoted.push_str("\\#"),
                    _ => quoted.push(character),
                }
            }
            quoted.push('"');
            quoted
        }
    };
    if style == "rust_format" && output.contains(['{', '}']) {
        literal = format!("\"{{}}\", {literal}");
    }
    Some(child_value(renderer, "syntax")?.replace("{literal}", &literal))
}

fn seed_tree() -> LinoNode {
    parse_lino(crate::seed::HELLO_WORLD_PROGRAMS_LINO)
}

fn template_row<'a>(
    root: &'a LinoNode,
    task_slug: &str,
    language_slug: &str,
) -> Option<&'a LinoNode> {
    root.children.iter().find(|node| {
        child_value(node, "task") == Some(task_slug)
            && child_value(node, "language") == Some(language_slug)
    })
}

fn template_code(root: &LinoNode, task_slug: &str, language_slug: &str) -> Option<String> {
    template_row(root, task_slug, language_slug)
        .and_then(|node| child_value(node, "code"))
        .map(ToOwned::to_owned)
}

fn program_source<'a>(root: &'a LinoNode, task_slug: &str, language_slug: &str) -> Option<&'a str> {
    template_row(root, task_slug, language_slug)
        .and_then(|node| child_value(node, "program_source"))
}

fn child_value<'a>(node: &'a LinoNode, name: &str) -> Option<&'a str> {
    node.children
        .iter()
        .find(|child| child.name == name)
        .map(|child| child.id.as_str())
}
