//! The portable `data/seed/hello-world-programs.lino` knowledge bundle is a
//! hand-mirrored copy of this catalog. These tests lock the two together so
//! a task or template added to the Rust catalog can never silently drift
//! out of the downloadable seed (the divergence that previously left
//! `list_files_arg` out of the seed). Regenerate the seed blocks with
//! `experiments/issue-330-coding-tasks/generate_lino.py` when adding tasks.
use super::*;
use crate::seed::parser::{parse_lino, LinoNode};

#[test]
fn lino_seed_tasks_line_lists_every_catalog_task() {
    let root = seed_tree();
    let write_program = root
        .children
        .iter()
        .find(|node| node.name == WRITE_PROGRAM_INTENT)
        .expect("lino seed declares a tasks line");
    let listed: Vec<&str> = child_value(write_program, "tasks")
        .expect("write_program seed declares tasks")
        .split('|')
        .collect();
    for task in PROGRAM_TASKS {
        assert!(
            listed.contains(&task.slug),
            "lino tasks line is missing `{}` (has {listed:?})",
            task.slug
        );
    }
}

#[test]
fn lino_seed_mirrors_every_catalog_template() {
    let root = seed_tree();
    for template in program_templates() {
        let seed_code = template_code(&root, template.task_slug, template.language_slug);
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
        .filter(|node| child_value(node, "code").is_some())
        .count();
    assert_eq!(
        seed_templates,
        program_template_count(),
        "lino seed template count ({seed_templates}) must equal the catalog count ({})",
        program_template_count()
    );
}

fn seed_tree() -> LinoNode {
    parse_lino(crate::seed::HELLO_WORLD_PROGRAMS_LINO)
}

fn template_code(root: &LinoNode, task_slug: &str, language_slug: &str) -> Option<String> {
    root.children
        .iter()
        .find(|node| {
            child_value(node, "task") == Some(task_slug)
                && child_value(node, "language") == Some(language_slug)
        })
        .and_then(|node| child_value(node, "code"))
        .map(ToOwned::to_owned)
}

fn child_value<'a>(node: &'a LinoNode, name: &str) -> Option<&'a str> {
    node.children
        .iter()
        .find(|child| child.name == name)
        .map(|child| child.id.as_str())
}
