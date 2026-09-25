//! The `formal-ai translate` subcommand: any-direction translation through
//! the meta pivot (issue #1138, plan 16 L2 first rung).

use std::error::Error;
use std::path::PathBuf;

use formal_ai::meta_translate::{self, SourceRoot, TranslationOutcome};
use formal_ai::render_response;

/// Render one `translate_*` intent from the seed.
///
/// The intent id is the fallback so a missing record stays visible on
/// stderr instead of silently rendering nothing.
fn cli_text(intent: &str, values: &[(&str, &str)]) -> String {
    render_response(intent, "en", values).unwrap_or_else(|| intent.to_string())
}

pub fn run_translate(
    from: Option<&str>,
    to: Option<&str>,
    input: Option<PathBuf>,
    write: bool,
    list: bool,
) -> Result<(), Box<dyn Error>> {
    if list {
        for (from, to, pending) in meta_translate::directions() {
            println!("{}", meta_translate::describe_leg(from, to, pending));
        }
        return Ok(());
    }
    let (Some(from_name), Some(to_name)) = (from, to) else {
        return Err(cli_text("translate_needs_from_to", &[]).into());
    };
    let Some(from_root) = SourceRoot::parse(from_name) else {
        return Err(cli_text("translate_unknown_from", &[("from", from_name)]).into());
    };
    let Some(to_root) = SourceRoot::parse(to_name) else {
        return Err(cli_text("translate_unknown_to", &[("to", to_name)]).into());
    };
    let roots = [("from", from_root.name()), ("to", to_root.name())];
    if from_root == to_root {
        return Err(cli_text("translate_same_roots", &roots).into());
    }
    if let Some(plan_leaf) = meta_translate::pending_leg(from_root, to_root) {
        return Err(cli_text(
            "translate_pending_docs",
            &[
                ("from", from_root.name()),
                ("to", to_root.name()),
                ("leaf", plan_leaf),
            ],
        )
        .into());
    }
    if write {
        let root = std::env::current_dir()?;
        let report = match input.as_deref().and_then(|path| path.to_str()) {
            // `--input` carries a path relative to the working directory,
            // which is the repository root the sibling mapping is stated on.
            Some(input) => formal_ai::translate_write::write_one(
                from_root,
                to_root,
                &root,
                &input.replace('\\', "/"),
            ),
            None => formal_ai::translate_write::write_tree(from_root, to_root, &root),
        };
        let (intent, values) = report.intent();
        let values = values
            .iter()
            .map(|(key, value)| (*key, value.as_str()))
            .collect::<Vec<_>>();
        let rendered = cli_text(intent, &values);
        return if report.is_failure() {
            Err(rendered.into())
        } else {
            println!("{rendered}");
            Ok(())
        };
    }
    let Some(path) = input else {
        return Err(cli_text("translate_needs_input", &roots).into());
    };
    let source = std::fs::read_to_string(&path)?;
    match meta_translate::translate(from_root, to_root, &path.display().to_string(), &source) {
        TranslationOutcome::Rendered { target, .. } => {
            println!("{target}");
            Ok(())
        }
        TranslationOutcome::Refused { refusals } => {
            let constructs = refusals
                .iter()
                .map(|refusal| refusal.construct.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            Err(cli_text(
                "translate_refused",
                &[
                    ("from", from_root.name()),
                    ("to", to_root.name()),
                    ("constructs", &constructs),
                ],
            )
            .into())
        }
        TranslationOutcome::Pending { plan_leaf } => Err(cli_text(
            "translate_pending",
            &[
                ("from", from_root.name()),
                ("to", to_root.name()),
                ("leaf", plan_leaf),
            ],
        )
        .into()),
        TranslationOutcome::Invalid { reason } => Err(reason.into()),
    }
}
