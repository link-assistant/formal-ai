//! The `formal-ai translate` subcommand: any-direction translation through
//! the meta pivot (issue #1138, plan 16 L2 first rung).

use std::error::Error;
use std::path::PathBuf;

use formal_ai::meta_translate::{self, SourceRoot, TranslationOutcome};

pub(crate) fn run_translate(
    from: Option<String>,
    to: Option<String>,
    input: Option<PathBuf>,
    list: bool,
) -> Result<(), Box<dyn Error>> {
    if list {
        for (from, to, pending) in meta_translate::directions() {
            println!("{}", meta_translate::describe_leg(from, to, pending));
        }
        return Ok(());
    }
    let (Some(from_name), Some(to_name)) = (&from, &to) else {
        return Err("translate needs --from and --to (or --list to see every direction)".into());
    };
    let Some(from_root) = SourceRoot::parse(from_name) else {
        return Err(format!("unknown --from '{from_name}': expected one of rust|js|ts|meta").into());
    };
    let Some(to_root) = SourceRoot::parse(to_name) else {
        return Err(format!("unknown --to '{to_name}': expected one of rust|js|ts|meta").into());
    };
    if from_root == to_root {
        return Err(format!(
            "{} → {} is not a direction: the roots are the same",
            from_root.name(),
            to_root.name()
        )
        .into());
    }
    if let Some(plan_leaf) = meta_translate::pending_leg(from_root, to_root) {
        return Err(format!(
            "translate {} → {} is pending: plan 16 {plan_leaf} owes it \
             (docs/case-studies/issue-1138/plans/16-js-ts-rust-cycle.md)",
            from_root.name(),
            to_root.name()
        )
        .into());
    }
    let Some(path) = input else {
        return Err(format!(
            "translate {} → {} reads a source file: pass --input PATH",
            from_root.name(),
            to_root.name()
        )
        .into());
    };
    let source = std::fs::read_to_string(&path)?;
    match meta_translate::translate(
        from_root,
        to_root,
        &path.display().to_string(),
        &source,
    ) {
        TranslationOutcome::Rendered { target } => {
            println!("{target}");
            Ok(())
        }
        TranslationOutcome::Pending { plan_leaf } => Err(format!(
            "translate {} → {} is pending: plan 16 {plan_leaf} owes it",
            from_root.name(),
            to_root.name()
        )
        .into()),
    }
}
