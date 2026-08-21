#![no_std]
#![allow(clippy::missing_safety_doc)]

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};

#[path = "../../../language.rs"]
#[allow(dead_code)]
mod language;

#[path = "../../../arithmetic.rs"]
#[allow(dead_code)]
mod arithmetic;

#[path = "../../../web_engine_core.rs"]
#[allow(dead_code)]
mod web_engine_core;

#[path = "../../../proof_program/core.rs"]
mod proof_program_core;

#[path = "../../../seed/parser.rs"]
#[allow(dead_code)]
mod seed_parser;

#[path = "../../../web_search_core.rs"]
mod web_search_core;

#[path = "../../../web_search_fusion_core.rs"]
mod web_search_fusion_core;

#[path = "../../../search_fusion_grammar.rs"]
mod search_fusion_grammar;

#[path = "../../../memory_query_language/mod.rs"]
#[allow(dead_code, unused_imports)]
mod memory_query_language;

mod formal_statement_worker;
mod memory_query_worker;
mod proof_translation_worker;

use web_engine_core::{
    assess_arithmetic_claim, detect_language, evaluate_arithmetic_expression,
    matches_intent_route_payload, normalize_prompt, select_unknown_opener, stable_id,
    ArithmeticClaimAssessment, ArithmeticClaimOutcome,
};
use web_search_core::{
    build_request_evidence, default_search_plan_ids, parse_rrf_input, reciprocal_rank_fusion,
    serialize_rrf_output, WEB_SEARCH_CONCURRENCY_PER_CATEGORY, WEB_SEARCH_PROVIDER_LIMIT,
    WEB_SEARCH_PROVIDER_REGISTRY, WEB_SEARCH_RRF_K,
};
use web_search_fusion_core::fuse_statement_search_payload;

const GREETING: u32 = 1;
const WRITE_PROGRAM: u32 = 2;
const IDENTITY: u32 = 8;
const UNKNOWN: u32 = 0;
const INPUT_CAPACITY: usize = 65_536;
const OUTPUT_CAPACITY: usize = 65_536;

// Static byte buffers used by the JS↔WASM byte-buffer protocol.
//
// `INPUT` holds the prompt for `classify` and the tab-delimited RRF rows for
// `web_search_fuse`. `OUTPUT` receives the evidence / plan / fused payload
// the JS side decodes into UTF-8.
static mut INPUT: [u8; INPUT_CAPACITY] = [0; INPUT_CAPACITY];
static mut OUTPUT: [u8; OUTPUT_CAPACITY] = [0; OUTPUT_CAPACITY];

// === Bump allocator ===
//
// Issue #133 wants the symbolic core in Rust→WASM. The web_search_core module
// uses `alloc::String` and `alloc::Vec`, so the no_std worker needs a global
// allocator. We use a single 2 MiB heap with an `AtomicUsize` offset: every
// WASM entry point calls `reset_bump()` first so the heap rolls back between
// calls and no per-allocation deallocation logic is required. Statement
// fusion accepts up to 24 bounded source passages; allocator measurements in
// `experiments/issue709_wasm_heap.rs` keep that provider-limit workload below
// this capacity with headroom for percent-decoding and JSON rendering.
const BUMP_HEAP_SIZE: usize = 2_097_152;

struct BumpHeap {
    buffer: UnsafeCell<[u8; BUMP_HEAP_SIZE]>,
}

unsafe impl Sync for BumpHeap {}

static BUMP_HEAP: BumpHeap = BumpHeap {
    buffer: UnsafeCell::new([0; BUMP_HEAP_SIZE]),
};
static BUMP_OFFSET: AtomicUsize = AtomicUsize::new(0);

struct BumpAllocator;

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align().max(1);
        let size = layout.size();
        let base = BUMP_HEAP.buffer.get() as usize;
        loop {
            let current = BUMP_OFFSET.load(Ordering::Relaxed);
            let aligned_addr = (base + current + align - 1) & !(align - 1);
            let next_offset = aligned_addr - base + size;
            if next_offset > BUMP_HEAP_SIZE {
                return core::ptr::null_mut();
            }
            if BUMP_OFFSET
                .compare_exchange(current, next_offset, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                return aligned_addr as *mut u8;
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator — `reset_bump()` reclaims everything before each call.
    }
}

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator;

fn reset_bump() {
    BUMP_OFFSET.store(0, Ordering::Release);
}

// === Classic prompt classifier (pre-existing API) ===

#[unsafe(no_mangle)]
pub extern "C" fn input_ptr() -> *mut u8 {
    core::ptr::addr_of_mut!(INPUT).cast::<u8>()
}

#[unsafe(no_mangle)]
pub extern "C" fn output_ptr() -> *mut u8 {
    core::ptr::addr_of_mut!(OUTPUT).cast::<u8>()
}

#[unsafe(no_mangle)]
pub extern "C" fn input_capacity() -> usize {
    INPUT_CAPACITY
}

#[unsafe(no_mangle)]
pub extern "C" fn output_capacity() -> usize {
    OUTPUT_CAPACITY
}

#[unsafe(no_mangle)]
pub extern "C" fn classify(length: usize) -> u32 {
    let length = min(length, INPUT_CAPACITY);
    let input =
        unsafe { core::slice::from_raw_parts(core::ptr::addr_of!(INPUT).cast::<u8>(), length) };

    if is_exact_greeting(input) {
        GREETING
    } else if is_identity_question(input) {
        IDENTITY
    } else if contains_word(input, b"hello") && contains_word(input, b"world") {
        WRITE_PROGRAM
    } else {
        UNKNOWN
    }
}

fn is_exact_greeting(input: &[u8]) -> bool {
    let trimmed = trim_ascii(input);
    ascii_eq(trimmed, b"hi") || ascii_eq(trimmed, b"hello") || ascii_eq(trimmed, b"hey")
}

fn is_identity_question(input: &[u8]) -> bool {
    (contains_word(input, b"who") && contains_word(input, b"you"))
        || (contains_word(input, b"what") && contains_word(input, b"you"))
        || ((contains_word(input, b"who") || contains_word(input, b"what"))
            && contains_word(input, b"formal")
            && contains_word(input, b"ai"))
        || (contains_word(input, b"tell") && contains_word(input, b"yourself"))
        || (contains_word(input, b"introduce") && contains_word(input, b"yourself"))
}

fn contains_word(input: &[u8], word: &[u8]) -> bool {
    let mut index = 0;
    while index < input.len() {
        while index < input.len() && !is_ascii_alphanumeric(input[index]) {
            index += 1;
        }

        let start = index;
        while index < input.len() && is_ascii_alphanumeric(input[index]) {
            index += 1;
        }

        if start < index && ascii_eq(&input[start..index], word) {
            return true;
        }
    }

    false
}

fn trim_ascii(input: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = input.len();

    while start < end && !is_ascii_alphanumeric(input[start]) {
        start += 1;
    }
    while end > start && !is_ascii_alphanumeric(input[end - 1]) {
        end -= 1;
    }

    &input[start..end]
}

fn ascii_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }

    let mut index = 0;
    while index < left.len() {
        if to_ascii_lower(left[index]) != right[index] {
            return false;
        }
        index += 1;
    }

    true
}

const fn is_ascii_alphanumeric(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
}

const fn to_ascii_lower(byte: u8) -> u8 {
    if byte.is_ascii_uppercase() {
        byte + 32
    } else {
        byte
    }
}

const fn min(left: usize, right: usize) -> usize {
    if left < right {
        left
    } else {
        right
    }
}

// === Web search core exports ===
//
// Every export consumes/produces UTF-8 bytes via the `INPUT` and `OUTPUT`
// buffers, returning the number of bytes written to `OUTPUT`. JS decodes the
// bytes with `TextDecoder` and parses the line/tab-delimited shape produced by
// `web_search_core::*` helpers. This keeps the WASM↔JS boundary free of any
// allocator imports (`malloc`, `free`, `dlmalloc`, …).

#[unsafe(no_mangle)]
pub extern "C" fn web_search_rrf_k() -> u32 {
    WEB_SEARCH_RRF_K
}

#[unsafe(no_mangle)]
pub extern "C" fn web_search_concurrency_per_category() -> u32 {
    WEB_SEARCH_CONCURRENCY_PER_CATEGORY
}

#[unsafe(no_mangle)]
pub extern "C" fn web_search_provider_limit() -> u32 {
    WEB_SEARCH_PROVIDER_LIMIT
}

#[unsafe(no_mangle)]
pub extern "C" fn web_search_registry_len() -> u32 {
    WEB_SEARCH_PROVIDER_REGISTRY.len() as u32
}

/// Write the canonical default plan ids to `OUTPUT`, one per line.
///
/// Returns the number of bytes written.
#[unsafe(no_mangle)]
pub extern "C" fn web_search_plan() -> usize {
    reset_bump();
    let ids = default_search_plan_ids();
    let mut buffer = String::new();
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            buffer.push('\n');
        }
        buffer.push_str(id);
    }
    write_output(buffer.as_bytes())
}

/// Write the multi-line `web_search:*` evidence prefix for a given
/// (query, language) pair to `OUTPUT`.
///
/// `INPUT` must contain `query\nlanguage` (the language line may be empty).
#[unsafe(no_mangle)]
pub extern "C" fn web_search_request_evidence(input_length: usize) -> usize {
    reset_bump();
    let bytes = unsafe {
        core::slice::from_raw_parts(
            core::ptr::addr_of!(INPUT).cast::<u8>(),
            min(input_length, INPUT_CAPACITY),
        )
    };
    let Ok(text) = core::str::from_utf8(bytes) else {
        return 0;
    };
    let mut parts = text.splitn(2, '\n');
    let query = parts.next().unwrap_or("");
    let language = parts.next().unwrap_or("");
    let lines = build_request_evidence(query, language);
    let mut buffer = String::new();
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            buffer.push('\n');
        }
        buffer.push_str(line);
    }
    write_output(buffer.as_bytes())
}

/// Fuse a flat list of `provider_id\trank\turl\ttitle\texcerpt` rows
/// (one per `INPUT` line) into the RRF-ranked `OUTPUT` block produced by
/// `web_search_core::serialize_rrf_output`.
#[unsafe(no_mangle)]
pub extern "C" fn web_search_fuse(input_length: usize) -> usize {
    reset_bump();
    let bytes = unsafe {
        core::slice::from_raw_parts(
            core::ptr::addr_of!(INPUT).cast::<u8>(),
            min(input_length, INPUT_CAPACITY),
        )
    };
    let Ok(text) = core::str::from_utf8(bytes) else {
        return 0;
    };
    let entries = parse_rrf_input(text);
    let fused = reciprocal_rank_fusion(&entries, WEB_SEARCH_RRF_K);
    let serialized = serialize_rrf_output(&fused);
    write_output(serialized.as_bytes())
}

/// Formalize, merge, rank, and render captured search excerpts as statements.
#[unsafe(no_mangle)]
pub extern "C" fn web_search_statement_fusion(input_length: usize) -> usize {
    reset_bump();
    let bytes = unsafe {
        core::slice::from_raw_parts(
            core::ptr::addr_of!(INPUT).cast::<u8>(),
            min(input_length, INPUT_CAPACITY),
        )
    };
    let Ok(text) = core::str::from_utf8(bytes) else {
        return 0;
    };
    let serialized = fuse_statement_search_payload(text);
    write_output(serialized.as_bytes())
}

// === Engine-core exports (R194 deep port) ===
//
// `engine_normalize_prompt`, `engine_detect_language`, and
// `engine_evaluate_arithmetic` are the canonical Rust implementations of the
// non-UI primitives the JS worker used to own (`normalizePrompt`,
// `detectLanguage`, `evaluateArithmetic`). The JS side now delegates to these
// exports and only keeps a minimal fallback for the offline `js fallback`
// mode. This eliminates the parallel logic the user flagged in PR feedback
// 4489651616.

/// Normalize a prompt to the same lowercase/whitespace-stripped form the JS
/// worker used to produce. `INPUT` contains the raw prompt bytes; on return
/// `OUTPUT` carries the normalized UTF-8 bytes.
#[unsafe(no_mangle)]
pub extern "C" fn engine_normalize_prompt(input_length: usize) -> usize {
    reset_bump();
    let bytes = unsafe {
        core::slice::from_raw_parts(
            core::ptr::addr_of!(INPUT).cast::<u8>(),
            min(input_length, INPUT_CAPACITY),
        )
    };
    let Ok(text) = core::str::from_utf8(bytes) else {
        return 0;
    };
    let normalized = normalize_prompt(text);
    write_output(normalized.as_bytes())
}

/// Detect the dominant language of the prompt held in `INPUT`. Writes a
/// 2-letter slug (`en`, `ru`, `hi`, `zh`, or `unknown`) to `OUTPUT`.
#[unsafe(no_mangle)]
pub extern "C" fn engine_detect_language(input_length: usize) -> usize {
    reset_bump();
    let bytes = unsafe {
        core::slice::from_raw_parts(
            core::ptr::addr_of!(INPUT).cast::<u8>(),
            min(input_length, INPUT_CAPACITY),
        )
    };
    let text = core::str::from_utf8(bytes).unwrap_or("");
    // Issue #706: the slug is the language's identity in the registry, so the
    // bridge forwards it instead of re-listing the languages in Rust.
    let slug: &'static str = detect_language(text).slug();
    write_output(slug.as_bytes())
}

/// Evaluate an arithmetic expression. `INPUT` holds the raw expression bytes;
/// on success `OUTPUT` carries the formatted decimal result. On failure the
/// payload is `ERR:<reason>` so JS can render the failure in its native UI
/// without duplicating the parser. Returns the number of bytes written.
#[unsafe(no_mangle)]
pub extern "C" fn engine_evaluate_arithmetic(input_length: usize) -> usize {
    reset_bump();
    let bytes = unsafe {
        core::slice::from_raw_parts(
            core::ptr::addr_of!(INPUT).cast::<u8>(),
            min(input_length, INPUT_CAPACITY),
        )
    };
    let Ok(text) = core::str::from_utf8(bytes) else {
        return write_output(b"ERR:unparseable");
    };
    match evaluate_arithmetic_expression(text) {
        Ok(value) => write_output(value.as_bytes()),
        Err(message) => {
            let mut buffer = String::with_capacity(message.len() + 4);
            buffer.push_str("ERR:");
            buffer.push_str(&message);
            write_output(buffer.as_bytes())
        }
    }
}

/// Assess an arithmetic equality/inequality for the current-dialog fact
/// checker. `OUTPUT` is a JSON object consumed by the UI-glue worker, or empty
/// when the statement is outside the arithmetic decision procedure.
#[unsafe(no_mangle)]
pub extern "C" fn engine_assess_arithmetic_claim(input_length: usize) -> usize {
    reset_bump();
    let bytes = unsafe {
        core::slice::from_raw_parts(
            core::ptr::addr_of!(INPUT).cast::<u8>(),
            min(input_length, INPUT_CAPACITY),
        )
    };
    let Ok(text) = core::str::from_utf8(bytes) else {
        return 0;
    };
    let Some(assessment) = assess_arithmetic_claim(text) else {
        return 0;
    };
    write_output(serialize_arithmetic_assessment(&assessment).as_bytes())
}

/// Audit prior user turns in the current dialogue. `INPUT` contains five
/// percent-encoded seed templates followed by percent-encoded user turns, one
/// item per line. `OUTPUT` is the complete worker-answer JSON object.
#[unsafe(no_mangle)]
pub extern "C" fn engine_fact_check_dialogue(input_length: usize) -> usize {
    reset_bump();
    let bytes = unsafe {
        core::slice::from_raw_parts(
            core::ptr::addr_of!(INPUT).cast::<u8>(),
            min(input_length, INPUT_CAPACITY),
        )
    };
    let Ok(text) = core::str::from_utf8(bytes) else {
        return 0;
    };
    let decoded = text.lines().map(decode_uri_component).collect::<Vec<_>>();
    if decoded.len() < 5 {
        return 0;
    }
    let templates = FactCheckTemplates {
        audit: &decoded[0],
        statement: &decoded[1],
        statement_counterexample: &decoded[2],
        arithmetic_counterexample: &decoded[3],
        no_statements: &decoded[4],
    };
    let answer = build_dialogue_fact_check(&decoded[5..], templates);
    write_output(answer.as_bytes())
}

/// Parse and execute an exact SQL/GraphQL memory query in the shared Rust
/// query core. The line-oriented input is URI-encoded by the browser bridge;
/// the output is either an answer JSON object or empty for a non-query prompt.
#[unsafe(no_mangle)]
pub extern "C" fn engine_memory_query(input_length: usize) -> usize {
    reset_bump();
    let bytes = unsafe {
        core::slice::from_raw_parts(
            core::ptr::addr_of!(INPUT).cast::<u8>(),
            min(input_length, INPUT_CAPACITY),
        )
    };
    let Ok(payload) = core::str::from_utf8(bytes) else {
        return 0;
    };
    let answer = memory_query_worker::answer(payload);
    write_output(answer.as_bytes())
}

/// Parse a formal proof once and project it into a seed-defined executable
/// program. The three URI-encoded input lines are statement, target, and the
/// proof-program template seed; `OUTPUT` is a complete worker-answer object.
#[unsafe(no_mangle)]
pub extern "C" fn engine_translate_formal_proof(input_length: usize) -> usize {
    reset_bump();
    let bytes = unsafe {
        core::slice::from_raw_parts(
            core::ptr::addr_of!(INPUT).cast::<u8>(),
            min(input_length, INPUT_CAPACITY),
        )
    };
    let Ok(payload) = core::str::from_utf8(bytes) else {
        return 0;
    };
    write_output(proof_translation_worker::answer(payload).as_bytes())
}

/// Project a semantic statement between a seeded natural language and formal
/// concrete syntax. The line-oriented adapter returns a complete answer JSON.
#[unsafe(no_mangle)]
pub extern "C" fn engine_translate_formal_statement(input_length: usize) -> usize {
    reset_bump();
    let bytes = unsafe {
        core::slice::from_raw_parts(
            core::ptr::addr_of!(INPUT).cast::<u8>(),
            min(input_length, INPUT_CAPACITY),
        )
    };
    let Ok(payload) = core::str::from_utf8(bytes) else {
        return 0;
    };
    write_output(formal_statement_worker::answer(payload).as_bytes())
}

struct FactCheckTemplates<'a> {
    audit: &'a str,
    statement: &'a str,
    statement_counterexample: &'a str,
    arithmetic_counterexample: &'a str,
    no_statements: &'a str,
}

struct DialogueStatement {
    id: String,
    text: String,
    probability: &'static str,
    basis: &'static str,
    outcome: &'static str,
    counterexample: String,
}

fn build_dialogue_fact_check(turns: &[String], templates: FactCheckTemplates<'_>) -> String {
    let mut statements = Vec::new();
    for text in turns
        .iter()
        .map(|turn| turn.trim())
        .filter(|turn| !turn.is_empty())
    {
        if !looks_declarative(text) {
            continue;
        }
        let id = stable_id("world_statement", text);
        if statements
            .iter()
            .any(|item: &DialogueStatement| item.id == id)
        {
            continue;
        }
        statements.push(assess_dialogue_statement(id, text, &templates));
    }
    statements.sort_by(|left, right| left.id.cmp(&right.id));

    let summary = if statements.is_empty() {
        templates.no_statements.to_string()
    } else {
        statements
            .iter()
            .map(|statement| render_dialogue_statement(statement, &templates))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let content = fill_template(
        templates.audit,
        &[
            ("count", statements.len().to_string()),
            ("formal_system", String::from("current")),
            ("summary", summary),
        ],
    );
    serialize_dialogue_answer(&content, &statements)
}

fn looks_declarative(text: &str) -> bool {
    let normalized = normalize_prompt(text);
    let words = normalized.split_whitespace().collect::<Vec<_>>();
    words.len() > 1
        || words.first().is_some_and(|word| {
            word.chars().count() > 1
                && word.chars().any(|ch| {
                    let code = ch as u32;
                    (0x3400..=0x9fff).contains(&code) || (0xf900..=0xfaff).contains(&code)
                })
        })
}

fn assess_dialogue_statement(
    id: String,
    text: &str,
    templates: &FactCheckTemplates<'_>,
) -> DialogueStatement {
    let assessment = assess_arithmetic_claim(text);
    let (probability, basis, outcome, counterexample) = match assessment {
        Some(value) if value.outcome == ArithmeticClaimOutcome::Unrefuted => {
            ("1.000000", "evidence_weighted", "unrefuted", String::new())
        }
        Some(value) if value.outcome == ArithmeticClaimOutcome::Refuted => (
            "0.000000",
            "evidence_weighted",
            "refuted",
            fill_template(
                templates.arithmetic_counterexample,
                &[
                    ("left", value.left_expression),
                    ("left_value", value.left_value),
                    ("right", value.right_expression),
                    ("right_value", value.right_value),
                    ("relation", value.relation.to_string()),
                ],
            ),
        ),
        _ => ("0.600000", "prior_only", "inconclusive", String::new()),
    };
    DialogueStatement {
        id,
        text: text.to_string(),
        probability,
        basis,
        outcome,
        counterexample,
    }
}

fn render_dialogue_statement(
    statement: &DialogueStatement,
    templates: &FactCheckTemplates<'_>,
) -> String {
    let template = if statement.counterexample.is_empty() {
        templates.statement
    } else {
        templates.statement_counterexample
    };
    fill_template(
        template,
        &[
            ("statement", statement.text.clone()),
            ("probability", statement.probability.to_string()),
            ("basis", statement.basis.to_string()),
            ("counterexample", statement.counterexample.clone()),
        ],
    )
}

fn fill_template(template: &str, replacements: &[(&str, String)]) -> String {
    let mut rendered = template.to_string();
    for (name, value) in replacements {
        rendered = rendered.replace(&format!("{{{name}}}"), value);
    }
    rendered
}

fn serialize_dialogue_answer(content: &str, statements: &[DialogueStatement]) -> String {
    let mut output = String::from("{\"intent\":\"fact_check_current_dialogue\",\"content\":");
    push_json_string(&mut output, content);
    output.push_str(",\"confidence\":1,\"evidence\":[");
    for (index, statement) in statements.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        let mut evidence = format!("fact_check:statement:{}", statement.id);
        evidence.push(' ');
        evidence.push_str(statement.probability);
        evidence.push(' ');
        evidence.push_str(statement.basis);
        push_json_string(
            &mut output,
            &evidence,
        );
    }
    if !statements.is_empty() {
        output.push(',');
    }
    for (index, evidence) in [
        format!(
            "fact_check:audit:scope=current_dialogue;statements={}",
            statements.len()
        ),
        String::from("fact_check:scope:current_dialogue"),
        format!(
            "fact_check:formal_system:{}",
            stable_id(
                "formal_system",
                "name:current;universe:;interpretation:;"
            )
        ),
    ]
    .iter()
    .enumerate()
    {
        if index > 0 {
            output.push(',');
        }
        push_json_string(&mut output, evidence);
    }
    output.push_str("],\"trace\":[");
    for (index, statement) in statements.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_json_string(
            &mut output,
            &format!(
                "fact_check:refutation:{}:disprove_statement:{}",
                statement.id, statement.outcome
            ),
        );
    }
    output.push_str("]}");
    output
}

fn decode_uri_component(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let (Some(high), Some(low)) =
                (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
            {
                decoded.push((high << 4) | low);
                index += 3;
                continue;
            }
        }
        decoded.push(bytes[index]);
        index += 1;
    }
    String::from_utf8(decoded).unwrap_or_default()
}

const fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn serialize_arithmetic_assessment(assessment: &ArithmeticClaimAssessment) -> String {
    let outcome = match assessment.outcome {
        ArithmeticClaimOutcome::Unrefuted => "unrefuted",
        ArithmeticClaimOutcome::Refuted => "refuted",
        ArithmeticClaimOutcome::Inconclusive => "inconclusive",
    };
    let mut output = String::from("{\"outcome\":");
    push_json_string(&mut output, outcome);
    for (name, value) in [
        ("left", assessment.left_expression.as_str()),
        ("left_value", assessment.left_value.as_str()),
        ("right", assessment.right_expression.as_str()),
        ("right_value", assessment.right_value.as_str()),
        ("relation", assessment.relation),
    ] {
        output.push(',');
        push_json_string(&mut output, name);
        output.push(':');
        push_json_string(&mut output, value);
    }
    output.push('}');
    output
}

fn push_json_string(output: &mut String, value: &str) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    output.push('"');
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            value if value <= '\u{1f}' => {
                let code = value as usize;
                output.push_str("\\u00");
                output.push(HEX[code >> 4] as char);
                output.push(HEX[code & 0x0f] as char);
            }
            value => output.push(value),
        }
    }
    output.push('"');
}

/// Build a stable FNV-1a id. `INPUT` contains `prefix\ntext`; `OUTPUT`
/// receives `prefix_<hash>`.
#[unsafe(no_mangle)]
pub extern "C" fn engine_stable_id(input_length: usize) -> usize {
    reset_bump();
    let bytes = unsafe {
        core::slice::from_raw_parts(
            core::ptr::addr_of!(INPUT).cast::<u8>(),
            min(input_length, INPUT_CAPACITY),
        )
    };
    let Ok(text) = core::str::from_utf8(bytes) else {
        return 0;
    };
    let mut parts = text.splitn(2, '\n');
    let prefix = parts.next().unwrap_or("");
    let value = parts.next().unwrap_or("");
    let id = stable_id(prefix, value);
    write_output(id.as_bytes())
}

/// Select the deterministic unknown-answer opener. `INPUT` contains
/// `language\nprompt`; `OUTPUT` receives the opener text.
#[unsafe(no_mangle)]
pub extern "C" fn engine_select_unknown_opener(input_length: usize) -> usize {
    reset_bump();
    let bytes = unsafe {
        core::slice::from_raw_parts(
            core::ptr::addr_of!(INPUT).cast::<u8>(),
            min(input_length, INPUT_CAPACITY),
        )
    };
    let Ok(text) = core::str::from_utf8(bytes) else {
        return 0;
    };
    let mut parts = text.splitn(2, '\n');
    let language = parts.next().unwrap_or("");
    let prompt = parts.next().unwrap_or("");
    write_output(select_unknown_opener(prompt, language).as_bytes())
}

/// Return 1 when the serialized route payload matches, else 0.
#[unsafe(no_mangle)]
pub extern "C" fn engine_match_intent_route(input_length: usize) -> u32 {
    reset_bump();
    let bytes = unsafe {
        core::slice::from_raw_parts(
            core::ptr::addr_of!(INPUT).cast::<u8>(),
            min(input_length, INPUT_CAPACITY),
        )
    };
    let Ok(text) = core::str::from_utf8(bytes) else {
        return 0;
    };
    u32::from(matches_intent_route_payload(text))
}

/// Write the registry as `id\tlabel\tcategory\tcors_readable\tdefault\n…`.
#[unsafe(no_mangle)]
pub extern "C" fn web_search_registry_dump() -> usize {
    reset_bump();
    let mut buffer = String::new();
    for (index, spec) in WEB_SEARCH_PROVIDER_REGISTRY.iter().enumerate() {
        if index > 0 {
            buffer.push('\n');
        }
        buffer.push_str(spec.id);
        buffer.push('\t');
        buffer.push_str(spec.label);
        buffer.push('\t');
        buffer.push_str(spec.category.slug());
        buffer.push('\t');
        buffer.push(if spec.cors_readable { '1' } else { '0' });
        buffer.push('\t');
        buffer.push(if spec.default_for_category { '1' } else { '0' });
    }
    write_output(buffer.as_bytes())
}

fn write_output(bytes: &[u8]) -> usize {
    let written = min(bytes.len(), OUTPUT_CAPACITY);
    unsafe {
        let dst = core::ptr::addr_of_mut!(OUTPUT).cast::<u8>();
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), dst, written);
    }
    // Silence the "unused" warning on the Vec import — it is exercised
    // transitively by the alloc paths in web_search_core but the worker code
    // itself never names `Vec`.
    let _ = core::mem::size_of::<Vec<u8>>();
    written
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}
