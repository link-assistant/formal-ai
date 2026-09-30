//! The workflow a work item asks for beside the program (issue #1133).
//!
//! Every Hello World issue Hive Mind dispatches ends with the same requirement:
//! a GitHub Actions workflow that runs the program on every push and pull
//! request. The program's execution recipe already knows the commands that
//! verify it, so the workflow is those commands, nothing more -- the same
//! sequence the harness ran, in CI. It rides along as a supporting file of the
//! recipe, so it is written, committed and pushed with the program.

use crate::engine::{ExecutionRecipe, ExecutionRecipeFile};
use crate::seed;

/// Where the workflow is written.
#[allow(clippy::literal_string_with_formatting_args)]
pub(super) fn workflow_path() -> String {
    super::work_item_steps::fill("workflow_path", &[])
}

/// Whether the work item asks for a CI workflow, in any seeded language.
pub(super) fn requested_in(objective: &str) -> bool {
    seed::lexicon().mentions_role(
        seed::ROLE_CI_WORKFLOW_REQUEST,
        &crate::engine::normalize_prompt(objective),
    )
}

/// Add the workflow to `recipe` as a supporting file, once.
#[allow(clippy::literal_string_with_formatting_args)]
pub(super) fn attach(recipe: &mut ExecutionRecipe) {
    let path = workflow_path();
    if recipe.supporting_files.iter().any(|file| file.path == path) {
        return;
    }
    let source = render(recipe);
    recipe.supporting_files.push(ExecutionRecipeFile { path, source });
}

/// The workflow text: checkout, then the recipe's commands in order.
///
/// Every third-party pin the workflow carries -- the `uses:` refs and the
/// toolchain versions -- is resolved at generation time rather than shipped
/// from the template, and wherever a pin came from something other than a
/// live lookup, the workflow says so in a comment (issue #1168).
#[allow(clippy::literal_string_with_formatting_args)]
pub(super) fn render(recipe: &ExecutionRecipe) -> String {
    let versions = crate::version_resolution::VersionSet::for_generation();
    let mut out = String::new();
    for note in versions.provenance_note() {
        out.push_str("# ");
        out.push_str(&note);
        out.push('\n');
    }
    out.push_str(&super::work_item_steps::fill(
        "workflow_template",
        &[("{path}", &recipe.path)],
    ));
    out = crate::version_resolution::fill_workflow_versions(&out, &versions);
    if let Some(setup) = crate::coding::program_contract::runtime_steps(&recipe.language) {
        out.push_str(&crate::version_resolution::fill_workflow_versions(
            &setup, &versions,
        ));
    }
    for command in &recipe.commands {
        out.push_str(&super::work_item_steps::fill(
            "workflow_command_step",
            &[("{command}", command)],
        ));
    }
    out
}
