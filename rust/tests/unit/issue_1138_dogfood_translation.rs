//! Plan 16 L3 (issue #1138): the dogfood loop. `./ts` is generated from
//! `./js` by `formal-ai translate --from js --to ts --write`, so the committed
//! tree is a check — regenerating must reproduce it byte for byte, and a rule
//! that stops carrying must refuse by name instead of drifting quietly. A
//! mismatch's fix is the source or the translator, never a hand-edit of the
//! generated tree.

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::es_meta::{self, ProjectionTarget, SourceLanguage};
use formal_ai::meta_translate::{self, SourceRoot, TranslationOutcome};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate")
        .to_path_buf()
}

fn collect_owned(directory: &Path, extension: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_owned(&path, extension, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some(extension) {
            out.push(path);
        }
    }
    out.sort();
}

/// The dogfood check itself: the committed `ts` tree is exactly what the
/// translator regenerates from the committed `js` tree — same file set, same
/// bytes — or the mismatch is named per file.
#[test]
fn the_committed_ts_tree_is_what_the_translator_regenerates() {
    let root = root();
    let mut sources = Vec::new();
    collect_owned(&root.join("js"), "js", &mut sources);
    assert!(
        sources.len() >= 60,
        "the committed js corpus is the loop's input: {count} files",
        count = sources.len()
    );
    let mut committed = Vec::new();
    collect_owned(&root.join("ts"), "ts", &mut committed);
    let mut expected = Vec::new();
    for source in &sources {
        let repo_relative = source
            .strip_prefix(&root)
            .expect("the source lives under the repository root")
            .to_string_lossy()
            .replace('\\', "/");
        let target = meta_translate::write_target(
            SourceRoot::JavaScript,
            SourceRoot::TypeScript,
            &repo_relative,
        )
        .unwrap_or_else(|error| panic!("{repo_relative} must map: {error:?}"));
        let text = fs::read_to_string(source).expect("the committed source is readable");
        match meta_translate::translate(
            SourceRoot::JavaScript,
            SourceRoot::TypeScript,
            &repo_relative,
            &text,
        ) {
            TranslationOutcome::Rendered {
                target: rendered, ..
            } => {
                let committed_text = fs::read_to_string(root.join(&target))
                    .unwrap_or_else(|error| panic!("the committed {target} is readable: {error}"));
                assert_eq!(
                    rendered, committed_text,
                    "{target} drifted: regenerate it, never hand-edit it"
                );
            }
            other => panic!("{repo_relative} must carry to ts, got {other:?}"),
        }
        expected.push(target);
    }
    expected.sort();
    let mut committed_relative = committed
        .iter()
        .map(|path| {
            path.strip_prefix(&root)
                .expect("the committed tree lives under the repository root")
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect::<Vec<_>>();
    committed_relative.sort();
    assert_eq!(
        committed_relative, expected,
        "the committed ts tree is exactly the mapped file set"
    );
}

/// A corrupted projection rule refuses by name: drop the identifier class's
/// ts carry from the seed text and the renderer — which follows the data, not
/// a copy of it — refuses every identifier-carrying source naming the class.
#[test]
fn a_corrupted_projection_rule_fails_naming_the_construct() {
    let corrupted = formal_ai::seed::LANGUAGE_PROJECTION_LINO.replace(
        "  token_class identifier\n    carries_to js\n    carries_to ts\n",
        "  token_class identifier\n    carries_to js\n",
    );
    assert_ne!(
        corrupted,
        formal_ai::seed::LANGUAGE_PROJECTION_LINO,
        "the corruption must bite: the seed carries identifier to ts"
    );
    let rules = es_meta::projection_rules_from(&corrupted);
    let document = es_meta::extract(
        "probe.js",
        SourceLanguage::JavaScript,
        "export const probe = 1;\n",
    )
    .expect("the probe source is plain JavaScript");
    let rendered = es_meta::render_source_under(&document, ProjectionTarget::TypeScript, &rules);
    assert!(
        rendered.output.is_none(),
        "a corrupted rule must not render: {rendered:?}"
    );
    assert!(
        rendered
            .report
            .refused
            .iter()
            .any(|refusal| refusal.construct == "token_class identifier"),
        "the refusal names the corrupted class: {:?}",
        rendered.report.refused
    );
    // The committed seed still carries it — the loop's green path is the
    // seed's decision, restated every run.
    let live = es_meta::render_source(&document, ProjectionTarget::TypeScript);
    assert!(
        live.output.is_some(),
        "the committed seed carries: {live:?}"
    );
}
