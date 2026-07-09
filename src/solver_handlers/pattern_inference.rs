//! Pattern inference over 1D sequences and 2D grids (issue #531).
//!
//! This is the solver-facing surface for the link-native sequence substrate in
//! [`crate::sequences`]. When a prompt asks what pattern a concrete sequence or
//! grid exhibits — "find the pattern in 1 2 1 2 1 2", "what comes next in 2 4 6
//! 8", "is A B B A a palindrome", or a newline-separated grid — this handler
//! parses the atoms into links, runs the full inference pipeline (associative
//! deduplication plus 1D/2D structure detection), and reports the structure it
//! found together with a next-element prediction where one is well-defined.
//!
//! The handler is deliberately data-gated: it only fires when the prompt both
//! mentions pattern-inference intent *and* carries a parseable run of at least
//! three atoms. A bare definitional question like "what is a pattern?" carries
//! no data, returns [`None`] here, and falls through to the concept lookup.

use std::collections::HashMap;
use std::fmt::Write as _;

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::sequences::{
    infer_grid_patterns, infer_sequence_patterns, Grid, LinkAddress, SequencePattern,
    SequenceStore, SymbolTable,
};
use crate::solver_handlers::finalize_simple;

/// Words that signal the user wants structural pattern inference, not a
/// definition. At least one must appear for the handler to consider the prompt.
const INTENT_MARKERS: &[&str] = &[
    "pattern",
    "sequence",
    "palindrome",
    "symmetr",
    "periodic",
    "repeat",
    "what comes next",
    "comes next",
    "next number",
    "next term",
    "next in",
    "continue the",
    "continue this",
];

/// Whether the prompt both signals pattern-inference intent *and* carries a
/// parseable sequence or grid.
///
/// This mirrors the gate in [`try_pattern_inference`] so the intent formalizer
/// can rank this handler ahead of the concept lookup when a concrete sequence or
/// grid is present. A bare "what is the pattern?" carries no data, so this
/// returns `false` and the prompt still routes to the concept lookup. Keeping the
/// predicate next to the parser means routing and execution share one gate.
#[must_use]
pub fn looks_like_pattern_inference(prompt: &str) -> bool {
    let lowered = prompt.to_lowercase();
    if !INTENT_MARKERS.iter().any(|marker| lowered.contains(marker)) {
        return false;
    }
    parse_grid(prompt).is_some() || parse_sequence(prompt).is_some()
}

/// Try to answer a concrete pattern-inference request over a sequence or grid.
pub fn try_pattern_inference(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let _ = normalized;
    let lowered = prompt.to_lowercase();
    if !INTENT_MARKERS.iter().any(|marker| lowered.contains(marker)) {
        return None;
    }

    if let Some(grid) = parse_grid(prompt) {
        return Some(answer_grid(prompt, log, &grid));
    }

    let atoms = parse_sequence(prompt)?;
    Some(answer_sequence(prompt, log, &atoms))
}

/// An atom parsed from the prompt: its display token and a stable id used to
/// deduplicate equal atoms into the same link point.
#[derive(Clone)]
struct Atom {
    token: String,
    id: u64,
}

/// Assign each distinct token a stable, first-seen id so equal atoms map to the
/// same point while the original spelling is preserved for the explanation.
fn intern(tokens: &[String]) -> Vec<Atom> {
    let mut ids: HashMap<String, u64> = HashMap::new();
    tokens
        .iter()
        .map(|token| {
            let next = ids.len() as u64;
            let id = *ids.entry(token.clone()).or_insert(next);
            Atom {
                token: token.clone(),
                id,
            }
        })
        .collect()
}

/// Whether a cleaned token is a usable sequence atom: a run of digits or a
/// single *uppercase* letter (`A`, `B`, …). These cover numeric sequences and
/// the letter alphabets used in classic pattern puzzles. Requiring uppercase for
/// single letters keeps English stop words that happen to be one letter ("a",
/// "i") from being swept into a sequence of prose.
fn is_atom_token(token: &str) -> bool {
    if token.is_empty() {
        return false;
    }
    if token.chars().all(|ch| ch.is_ascii_digit()) {
        return true;
    }
    token.len() == 1 && token.chars().all(|ch| ch.is_ascii_uppercase())
}

/// Strip surrounding punctuation and whitespace so `"2,"` and `"(A)"` normalise
/// to the bare atom.
fn clean_token(raw: &str) -> String {
    raw.trim_matches(|ch: char| !ch.is_alphanumeric())
        .to_owned()
}

/// Extract the longest contiguous run of atom tokens (length >= 3) from a line.
///
/// Scanning for the *longest contiguous* run keeps scattered numbers in the
/// surrounding prose (an issue number, a year) from polluting the sequence: the
/// data payload is the one place several atoms sit next to each other.
fn longest_atom_run(line: &str) -> Vec<String> {
    let mut best: Vec<String> = Vec::new();
    let mut current: Vec<String> = Vec::new();
    for raw in line.split_whitespace() {
        let token = clean_token(raw);
        if is_atom_token(&token) {
            current.push(token);
        } else if current.len() > best.len() {
            best = std::mem::take(&mut current);
        } else {
            current.clear();
        }
    }
    if current.len() > best.len() {
        best = current;
    }
    if best.len() >= 3 {
        best
    } else {
        Vec::new()
    }
}

/// Parse a 1D sequence from the prompt, if one is present.
fn parse_sequence(prompt: &str) -> Option<Vec<Atom>> {
    let tokens = longest_atom_run(prompt);
    if tokens.is_empty() {
        return None;
    }
    Some(intern(&tokens))
}

/// A grid parsed from the prompt: its shape plus the interned atoms in
/// row-major order.
struct GridParse {
    rows: usize,
    cols: usize,
    atoms: Vec<Atom>,
}

/// Parse a 2D grid: two or more lines that each contribute the same number
/// (>= 2) of atom tokens. Returns [`None`] when the prompt is not grid-shaped.
fn parse_grid(prompt: &str) -> Option<GridParse> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    for line in prompt.lines() {
        let tokens: Vec<String> = line
            .split_whitespace()
            .map(clean_token)
            .filter(|token| is_atom_token(token))
            .collect();
        if !tokens.is_empty() {
            rows.push(tokens);
        }
    }
    if rows.len() < 2 {
        return None;
    }
    let cols = rows[0].len();
    if cols < 2 || rows.iter().any(|row| row.len() != cols) {
        return None;
    }
    let flat: Vec<String> = rows.iter().flatten().cloned().collect();
    Some(GridParse {
        rows: rows.len(),
        cols,
        atoms: intern(&flat),
    })
}

/// Materialise atoms as deduplicated points and return their link addresses.
fn to_points(
    store: &mut SequenceStore,
    symbols: &mut SymbolTable,
    atoms: &[Atom],
) -> Vec<LinkAddress> {
    atoms
        .iter()
        .map(|atom| symbols.scalar(store, atom.id))
        .collect()
}

/// Predict the next atom of a sequence when its structure makes one well-defined
/// (constant, exact repetition, or a bare period). Returns the token to append.
fn predict_next(atoms: &[Atom], classification: &SequencePattern) -> Option<String> {
    let len = atoms.len();
    match classification {
        SequencePattern::Constant => atoms.first().map(|atom| atom.token.clone()),
        SequencePattern::Repetition(pattern) => Some(atoms[len % pattern.period].token.clone()),
        SequencePattern::Periodic { period } => Some(atoms[len - period].token.clone()),
        SequencePattern::Empty | SequencePattern::Aperiodic => None,
    }
}

fn answer_sequence(prompt: &str, log: &mut EventLog, atoms: &[Atom]) -> SymbolicAnswer {
    let mut store = SequenceStore::new();
    let mut symbols = SymbolTable::new();
    let points = to_points(&mut store, &mut symbols, atoms);
    let report = infer_sequence_patterns(&mut store, &points);

    let rendered: Vec<&str> = atoms.iter().map(|atom| atom.token.as_str()).collect();
    let mut body = format!("Sequence: {}\n{}", rendered.join(" "), report.summary());
    if let Some(next) = predict_next(atoms, &report.classification) {
        let _ = write!(body, "\nMost likely next element: {next}.");
    }

    log.append("pattern_inference:kind", "sequence".to_owned());
    log.append("pattern_inference:length", report.length.to_string());
    log.append("pattern_inference:distinct", report.distinct.to_string());
    log.append(
        "pattern_inference:compression_ratio",
        format!("{:.2}", report.compression.compression_ratio()),
    );
    let confidence = if report.has_structure() { 0.85 } else { 0.6 };
    finalize_simple(
        prompt,
        log,
        "pattern_inference",
        "response:pattern_inference",
        &body,
        confidence,
    )
}

fn answer_grid(prompt: &str, log: &mut EventLog, parsed: &GridParse) -> SymbolicAnswer {
    let mut store = SequenceStore::new();
    let mut symbols = SymbolTable::new();
    // Allocate a real link point for every distinct cell id so the grid's cells
    // are valid addresses the inference pipeline can expand losslessly.
    let cells = to_points(&mut store, &mut symbols, &parsed.atoms);
    let grid = Grid::new(parsed.rows, parsed.cols, cells)
        .expect("cell count matches rows * cols by construction");
    let report = infer_grid_patterns(&mut store, &grid);
    let body = format!("Grid pattern inference.\n{}", report.summary());

    log.append("pattern_inference:kind", "grid".to_owned());
    log.append(
        "pattern_inference:dimensions",
        format!("{}x{}", report.rows, report.cols),
    );
    log.append(
        "pattern_inference:symmetries",
        report.invariant_transforms.len().to_string(),
    );
    let confidence = if report.has_structure() { 0.85 } else { 0.6 };
    finalize_simple(
        prompt,
        log,
        "pattern_inference",
        "response:pattern_inference",
        &body,
        confidence,
    )
}
