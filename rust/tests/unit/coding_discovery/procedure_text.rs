//! Issue #1138, plan 04 L9 and plan 02 L10 — retrieved page → ordered step list.
//!
//! One "ordered step with provenance" record serves the whole tree, and the HTML
//! handling is the existing `how_to_guide::extract` path rather than a second
//! extractor. Two rules keep the result honest: fewer than
//! `MIN_PROCEDURE_STEPS` items is not a procedure, and a share-alike capture may
//! contribute its shape but never its verbatim text.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use formal_ai::procedure_text::{MIN_PROCEDURE_STEPS, StepShape, reuse_mode, steps_from_capture};
use formal_ai::program_ir::ReuseMode;
use formal_ai::seed::source_record;
use formal_ai::source_fetch::{CachedSourceClient, FetchError, SourceCapture, SourceTransport};
use formal_ai::source_walk::LookupBounds;

static TEMP_IDS: AtomicUsize = AtomicUsize::new(0);

const ALGORITHM_URL: &str = "https://en.wikipedia.org/wiki/Lipogram";
const ONE_STEP_URL: &str = "https://en.wikipedia.org/wiki/Isogram";

const ORDERED_LIST: &str = concat!(
    "<html><body><h2>Algorithm</h2><ol>",
    "<li>Read the text.</li>",
    "<li>Drop spacing and punctuation.</li>",
    "<li>Confirm the forbidden letter never appears.</li>",
    "</ol></body></html>"
);

const ONE_INSTRUCTION: &str = "<html><body><ol><li>Read the text.</li></ol></body></html>";

/// Serves the two committed page shapes and refuses everything else, so no test
/// can silently reach the network.
#[derive(Clone, Copy, Default)]
struct PageTransport;

impl SourceTransport for PageTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        match url {
            ALGORITHM_URL => Ok(ORDERED_LIST.as_bytes().to_vec()),
            ONE_STEP_URL => Ok(ONE_INSTRUCTION.as_bytes().to_vec()),
            other => Err(FetchError::Transport(format!("fixture_missing:{other}"))),
        }
    }
}

fn temp_cache(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "formal-ai-issue-1138-steps-{label}-{}-{}",
        std::process::id(),
        TEMP_IDS.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = fs::remove_dir_all(&path);
    path
}

fn capture(url: &str, label: &str) -> SourceCapture {
    CachedSourceClient::new(temp_cache(label), PageTransport)
        .with_online(true)
        .fetch(url)
        .expect("the committed page")
}

#[test]
fn ordered_list_capture_becomes_a_provenance_bearing_step_list() {
    let source = source_record("wikipedia").expect("the registry declares wikipedia");
    let (shape, steps) = steps_from_capture(
        &capture(ALGORITHM_URL, "ordered"),
        &source,
        &LookupBounds::default(),
    )
    .expect("three ordered items are a procedure");

    assert_eq!(shape, StepShape::OrderedList);
    assert_eq!(steps.len(), 3);
    assert_eq!(steps[0].ordinal, 1);
    assert_eq!(steps[0].text, "Read the text.");
    assert_eq!(steps[2].text, "Confirm the forbidden letter never appears.");
    for step in &steps {
        assert_eq!(step.source_id, "wikipedia");
        assert_eq!(step.source_url, ALGORITHM_URL);
        assert_eq!(step.sha256.len(), 64);
        assert!(!step.license_name.is_empty() && !step.license_url.is_empty());
    }
}

#[test]
fn a_capture_with_one_instruction_is_not_a_procedure() {
    assert_eq!(MIN_PROCEDURE_STEPS, 2);
    let source = source_record("wikipedia").expect("the registry declares wikipedia");
    assert_eq!(
        steps_from_capture(
            &capture(ONE_STEP_URL, "single"),
            &source,
            &LookupBounds::default(),
        ),
        None,
        "a single instruction is a guess, not a recovered procedure"
    );
}

#[test]
fn share_alike_capture_is_marked_shape_only() {
    let share_alike = source_record("wikipedia").expect("the registry declares wikipedia");
    assert!(
        share_alike.license_name.contains("BY-SA") || share_alike.license_name.contains("GFDL"),
        "this case is about a share-alike source: {}",
        share_alike.license_name
    );
    let (_, steps) = steps_from_capture(
        &capture(ALGORITHM_URL, "share-alike"),
        &share_alike,
        &LookupBounds::default(),
    )
    .expect("three ordered items are a procedure");

    assert_eq!(
        reuse_mode(&steps[0]),
        ReuseMode::ShapeOnly,
        "a share-alike capture contributes an abstract step, never verbatim text"
    );
}
