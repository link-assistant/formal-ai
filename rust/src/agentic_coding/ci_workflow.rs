//! The workflow a work item asks for beside the program (issue #1133).
//!
//! Every Hello World issue Hive Mind dispatches ends with the same requirement:
//! a GitHub Actions workflow that runs the program on every push and pull
//! request. The program's execution recipe already knows the commands that
//! verify it, so the workflow is those commands, nothing more -- the same
//! sequence the harness ran, in CI. It rides along as a supporting file of the
//! recipe, so it is written, committed and pushed with the program.

use crate::engine::{ExecutionRecipe, ExecutionRecipeFile};
use crate::intent_formalization::{ObligationKind, request_demands};

/// Where the workflow is written.
#[allow(clippy::literal_string_with_formatting_args, dead_code)] // not yet read by the work-item steps
pub(super) fn workflow_path() -> String {
    super::work_item_steps::fill("workflow_path", &[])
}

/// Whether the work item asks for a CI workflow, in any seeded language.
///
/// Read from the formalized request (issue #1166 R1166-3): a clause of the
/// obligation graph demands the workflow, so a role word that only appears
/// inside a quoted literal is data the request carries, not a demand.
pub(super) fn requested_in(objective: &str) -> bool {
    request_demands(objective, ObligationKind::CiWorkflow)
}

/// Add the workflow to `recipe` as a supporting file, once.
#[allow(clippy::literal_string_with_formatting_args)]
pub(super) fn attach(recipe: &mut ExecutionRecipe) {
    let name: String = std::path::Path::new(&recipe.path)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("run")
        .chars()
        .map(|value| {
            if value.is_ascii_alphanumeric() || value == '-' || value == '_' {
                value
            } else {
                '-'
            }
        })
        .collect();
    let path = super::work_item_steps::fill("workflow_path_named", &[("{name}", &name)]);
    if recipe.supporting_files.iter().any(|file| file.path == path) {
        return;
    }
    let source = render(recipe);
    recipe
        .supporting_files
        .push(ExecutionRecipeFile { path, source });
}

/// The workflow text over the generation-time version set.
///
/// Every third-party pin the workflow carries -- the `uses:` refs and the
/// toolchain versions -- is resolved at generation time rather than shipped
/// from the template, and wherever a pin came from something other than a
/// live lookup, the workflow says so in a comment (issue #1168).
pub(super) fn render(recipe: &ExecutionRecipe) -> String {
    render_with(
        recipe,
        &crate::version_resolution::VersionSet::for_generation(),
    )
}

/// The workflow text: checkout, then the recipe's commands in order.
///
/// Every pin is filled from the injected `versions`, so a parity lane renders
/// the same workflow in both runtimes from one offline version set (issue
/// #1168 R1168-8); `render` injects the generation-time set.
#[must_use]
#[allow(clippy::literal_string_with_formatting_args)]
pub fn render_with(
    recipe: &ExecutionRecipe,
    versions: &crate::version_resolution::VersionSet,
) -> String {
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
    // Filling versions trims the fragment's final newline; each fragment is a
    // whole line block, so it is restored before the next one is appended.
    out = crate::version_resolution::fill_workflow_versions(&out, versions);
    out.push('\n');
    if let Some(setup) = crate::coding::program_contract::runtime_steps(&recipe.language) {
        out.push_str(&crate::version_resolution::fill_workflow_versions(
            &setup, versions,
        ));
        out.push('\n');
    }
    for command in &recipe.commands {
        out.push_str(&super::work_item_steps::fill(
            "workflow_command_step",
            &[("{command}", command)],
        ));
    }
    out
}
