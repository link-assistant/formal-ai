//! The portable `data/seed/hello-world-programs.lino` knowledge bundle is a
//! hand-mirrored copy of this catalog. These tests lock the two together so
//! a task or template added to the Rust catalog can never silently drift
//! out of the downloadable seed (the divergence that previously left
//! `list_files_arg` out of the seed). Regenerate the seed blocks with
//! `experiments/issue-330-coding-tasks/generate_lino.py` when adding tasks.
use super::*;

/// Escape raw template code the way the `.lino` seed encodes a
/// single-quoted `code '...'` payload: backslash -> `\\`, single-quote ->
/// `\x27`, newline -> `\n`. Mirrors `generate_lino.py`.
fn escape_for_lino(code: &str) -> String {
    code.replace('\\', "\\\\")
        .replace('\'', "\\x27")
        .replace('\n', "\\n")
}

#[test]
fn lino_seed_tasks_line_lists_every_catalog_task() {
    let lino = crate::seed::HELLO_WORLD_PROGRAMS_LINO;
    let tasks_line = lino
        .lines()
        .find(|l| l.trim_start().starts_with("tasks \""))
        .expect("lino seed declares a tasks line");
    let listed: Vec<&str> = tasks_line
        .trim()
        .trim_start_matches("tasks \"")
        .trim_end_matches('"')
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
    let lino = crate::seed::HELLO_WORLD_PROGRAMS_LINO;
    for template in program_templates() {
        let needle = format!("code '{}'", escape_for_lino(template.code));
        assert!(
            lino.contains(&needle),
            "lino seed is missing the {}/{} template (escaped code not found)",
            template.task_slug,
            template.language_slug
        );
    }
}

#[test]
fn lino_seed_has_no_extra_templates() {
    let lino = crate::seed::HELLO_WORLD_PROGRAMS_LINO;
    let seed_templates = lino.matches("\n  code '").count();
    assert_eq!(
        seed_templates,
        program_template_count(),
        "lino seed template count ({seed_templates}) must equal the catalog count ({})",
        program_template_count()
    );
}
