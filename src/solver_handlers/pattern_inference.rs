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
    infer_grid_patterns, infer_sequence_patterns, Grid, GridPatternReport, GridSymmetry,
    LinkAddress, SequencePattern, SequencePatternReport, SequenceStore, SymbolTable,
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
///
/// The report is rendered in English. Use
/// [`try_pattern_inference_with_response_language`] when a response-language
/// follow-up (issue #556) forces the reply into another seeded language.
pub fn try_pattern_inference(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    try_pattern_inference_with_response_language(prompt, normalized, log, "en")
}

/// Try to answer a pattern-inference request, rendering the human-readable
/// report in `language` (one of `en`, `ru`, `hi`, `zh`).
///
/// The structural analysis is language-neutral — the same sequence or grid
/// classifies identically regardless of `language`; only the prose labels and
/// the next-element phrasing are localized. This is what lets a response-language
/// follow-up ("answer in Russian") replay a prior pattern-inference answer in the
/// requested language instead of leaving it stranded in English.
pub fn try_pattern_inference_with_response_language(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    language: &str,
) -> Option<SymbolicAnswer> {
    let _ = normalized;
    let lowered = prompt.to_lowercase();
    if !INTENT_MARKERS.iter().any(|marker| lowered.contains(marker)) {
        return None;
    }

    if language != "en" {
        log.append("language_to", language.to_owned());
    }

    if let Some(grid) = parse_grid(prompt) {
        return Some(answer_grid(prompt, log, &grid, language));
    }

    let atoms = parse_sequence(prompt)?;
    Some(answer_sequence(prompt, log, &atoms, language))
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

fn answer_sequence(
    prompt: &str,
    log: &mut EventLog,
    atoms: &[Atom],
    language: &str,
) -> SymbolicAnswer {
    let mut store = SequenceStore::new();
    let mut symbols = SymbolTable::new();
    let points = to_points(&mut store, &mut symbols, atoms);
    let report = infer_sequence_patterns(&mut store, &points);

    let rendered: Vec<&str> = atoms.iter().map(|atom| atom.token.as_str()).collect();
    let next = predict_next(atoms, &report.classification);
    let body = sequence_body(language, &rendered.join(" "), &report, next.as_deref());

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

fn answer_grid(
    prompt: &str,
    log: &mut EventLog,
    parsed: &GridParse,
    language: &str,
) -> SymbolicAnswer {
    let mut store = SequenceStore::new();
    let mut symbols = SymbolTable::new();
    // Allocate a real link point for every distinct cell id so the grid's cells
    // are valid addresses the inference pipeline can expand losslessly.
    let cells = to_points(&mut store, &mut symbols, &parsed.atoms);
    let grid = Grid::new(parsed.rows, parsed.cols, cells)
        .expect("cell count matches rows * cols by construction");
    let report = infer_grid_patterns(&mut store, &grid);
    let body = grid_body(language, &report);

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

/// Pick the localized variant for `language`, falling back to English for any
/// language outside the seeded set (`ru`, `hi`, `zh`).
fn loc<'a>(language: &str, en: &'a str, ru: &'a str, hi: &'a str, zh: &'a str) -> &'a str {
    match language {
        "ru" => ru,
        "hi" => hi,
        "zh" => zh,
        _ => en,
    }
}

/// Render the full sequence report in `language`. English is delegated to the
/// substrate's own `summary()` so its wording stays byte-identical; the other
/// seeded languages are rendered from the same structured fields.
fn sequence_body(
    language: &str,
    rendered: &str,
    report: &SequencePatternReport,
    next: Option<&str>,
) -> String {
    if language == "en" {
        let mut body = format!("Sequence: {rendered}\n{}", report.summary());
        if let Some(next) = next {
            let _ = write!(body, "\nMost likely next element: {next}.");
        }
        return body;
    }

    let mut lines: Vec<String> = Vec::new();
    let label = loc(language, "Sequence", "Последовательность", "अनुक्रम", "序列");
    let colon = if language == "zh" { "：" } else { ": " };
    lines.push(format!("{label}{colon}{rendered}"));
    lines.push(sequence_lines(language, report));
    if let Some(next) = next {
        lines.push(format!(
            "{}{next}{}",
            next_prefix(language),
            sentence_end(language)
        ));
    }
    lines.join("\n")
}

/// The structural body lines (count, classification, palindrome, dedup) of a
/// sequence report in a non-English seeded language.
fn sequence_lines(language: &str, report: &SequencePatternReport) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push(count_line(language, report.length, report.distinct));
    lines.push(classification_line(language, &report.classification));
    if report.palindrome && report.length > 1 {
        lines.push(
            loc(
                language,
                "It reads the same forwards and backwards (palindrome).",
                "Читается одинаково слева направо и справа налево (палиндром).",
                "यह आगे और पीछे दोनों तरह से समान पढ़ा जाता है (पैलिंड्रोम)।",
                "它正读和反读相同（回文）。",
            )
            .to_owned(),
        );
    }
    if report.compression.is_compressed() {
        lines.push(compression_line(
            language,
            report.compression.steps.len(),
            report.compression.compression_ratio(),
        ));
    } else {
        lines.push(
            loc(
                language,
                "No repeated adjacent pairs to deduplicate.",
                "Нет повторяющихся соседних пар для дедупликации.",
                "डिडुप्लीकेट करने के लिए कोई दोहराई गई आसन्न जोड़ी नहीं।",
                "没有可去重的重复相邻对。",
            )
            .to_owned(),
        );
    }
    lines.join("\n")
}

fn count_line(language: &str, length: usize, distinct: usize) -> String {
    match language {
        "ru" => format!("Последовательность из {length} элемента(ов), различных: {distinct}."),
        "hi" => format!("{length} तत्व(ों) का अनुक्रम, {distinct} विशिष्ट।"),
        "zh" => format!("包含 {length} 个元素的序列，其中 {distinct} 个不同。"),
        _ => format!("Sequence of {length} element(s), {distinct} distinct."),
    }
}

fn classification_line(language: &str, classification: &SequencePattern) -> String {
    match classification {
        SequencePattern::Empty => loc(
            language,
            "The sequence is empty.",
            "Последовательность пуста.",
            "अनुक्रम खाली है।",
            "序列为空。",
        )
        .to_owned(),
        SequencePattern::Constant => loc(
            language,
            "Every element is identical (constant sequence).",
            "Все элементы одинаковы (постоянная последовательность).",
            "प्रत्येक तत्व समान है (स्थिर अनुक्रम)।",
            "每个元素都相同（常量序列）。",
        )
        .to_owned(),
        SequencePattern::Repetition(pattern) => {
            let (p, r) = (pattern.period, pattern.repetitions);
            match language {
                "ru" => {
                    format!("Это повторение: блок из {p} элемента(ов) повторяется {r} раз(а).")
                }
                "hi" => format!("यह एक पुनरावृत्ति है: {p} तत्व(ों) का खंड {r} बार दोहराया गया।"),
                "zh" => format!("这是一个重复：{p} 个元素的块重复了 {r} 次。"),
                _ => format!("It is a repetition: a block of {p} element(s) repeated {r} times."),
            }
        }
        SequencePattern::Periodic { period } => match language {
            "ru" => format!("Оно периодично с периодом {period}."),
            "hi" => format!("यह {period} अवधि के साथ आवर्ती है।"),
            "zh" => format!("它是周期性的，周期为 {period}。"),
            _ => format!("It is periodic with period {period}."),
        },
        SequencePattern::Aperiodic => loc(
            language,
            "It has no exact repeating period.",
            "У него нет точного повторяющегося периода.",
            "इसका कोई सटीक दोहराव अवधि नहीं है।",
            "它没有精确的重复周期。",
        )
        .to_owned(),
    }
}

fn compression_line(language: &str, pairs: usize, ratio: f64) -> String {
    let pct = ratio * 100.0;
    match language {
        "ru" => format!(
            "Ассоциативная дедупликация заменила {pairs} повторяющихся пар(ы), сжав до {pct:.0}% исходной длины (без потерь)."
        ),
        "hi" => format!(
            "साहचर्य डिडुप्लीकेशन ने {pairs} दोहराई गई जोड़ी(यों) को बदला, मूल लंबाई के {pct:.0}% तक संपीड़ित (हानिरहित)।"
        ),
        "zh" => format!("关联去重替换了 {pairs} 个重复对，压缩到原始长度的 {pct:.0}%（无损）。"),
        _ => format!(
            "Associative deduplication replaced {pairs} repeated pair(s), compressing to {pct:.0}% of the original length (lossless)."
        ),
    }
}

fn next_prefix(language: &str) -> &'static str {
    loc(
        language,
        "Most likely next element: ",
        "Наиболее вероятный следующий элемент: ",
        "सबसे संभावित अगला तत्व: ",
        "最可能的下一个元素：",
    )
}

/// The sentence-terminating punctuation for `language` (Devanagari danda,
/// full-width Chinese stop, or an ASCII period).
fn sentence_end(language: &str) -> &'static str {
    match language {
        "hi" => "।",
        "zh" => "。",
        _ => ".",
    }
}

/// Render the full grid report in `language`, mirroring the English delegation
/// used by [`sequence_body`].
fn grid_body(language: &str, report: &GridPatternReport) -> String {
    if language == "en" {
        return format!("Grid pattern inference.\n{}", report.summary());
    }

    let heading = loc(
        language,
        "Grid pattern inference.",
        "Инференция паттернов сетки.",
        "ग्रिड पैटर्न अनुमान।",
        "网格模式推断。",
    );
    let (rows, cols) = (report.rows, report.cols);
    let shape = match language {
        "ru" => format!("Сетка {rows}x{cols}."),
        "hi" => format!("ग्रिड {rows}x{cols}।"),
        "zh" => format!("网格 {rows}x{cols}。"),
        _ => format!("Grid {rows}x{cols}."),
    };
    let symmetry = grid_symmetry_line(language, report.symmetries);
    let structure = sequence_lines(language, &report.row_major);
    format!("{heading}\n{shape}\n{symmetry}\n{structure}")
}

fn grid_symmetry_line(language: &str, symmetries: GridSymmetry) -> String {
    let mut labels: Vec<&str> = Vec::new();
    if symmetries.horizontal {
        labels.push(loc(
            language,
            "left-right mirror",
            "зеркало лево-право",
            "बाएँ-दाएँ दर्पण",
            "左右镜像",
        ));
    }
    if symmetries.vertical {
        labels.push(loc(
            language,
            "top-bottom mirror",
            "зеркало верх-низ",
            "ऊपर-नीचे दर्पण",
            "上下镜像",
        ));
    }
    if symmetries.rotational_180 {
        labels.push(loc(
            language,
            "180-degree rotation",
            "поворот на 180 градусов",
            "180-डिग्री घूर्णन",
            "180度旋转",
        ));
    }
    if symmetries.diagonal {
        labels.push(loc(
            language,
            "main-diagonal reflection",
            "отражение по главной диагонали",
            "मुख्य-विकर्ण परावर्तन",
            "主对角线反射",
        ));
    }
    if symmetries.anti_diagonal {
        labels.push(loc(
            language,
            "anti-diagonal reflection",
            "отражение по побочной диагонали",
            "प्रति-विकर्ण परावर्तन",
            "副对角线反射",
        ));
    }
    if labels.is_empty() {
        return loc(
            language,
            "No symmetry detected.",
            "Симметрия не обнаружена.",
            "कोई समरूपता नहीं मिली।",
            "未检测到对称性。",
        )
        .to_owned();
    }
    let separator = if language == "zh" { "、" } else { ", " };
    let joined = labels.join(separator);
    match language {
        "ru" => format!("Симметрично относительно: {joined}."),
        "hi" => format!("इसके अंतर्गत सममित: {joined}।"),
        "zh" => format!("对称于：{joined}。"),
        _ => format!("Symmetric under: {joined}."),
    }
}
