#![no_std]
#![allow(clippy::missing_safety_doc)]

extern crate alloc;

use alloc::string::String;
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

#[path = "../../../web_search_core.rs"]
mod web_search_core;

use web_engine_core::{
    detect_language, evaluate_arithmetic_expression, matches_intent_route_payload,
    normalize_prompt, select_unknown_opener, stable_id, Language,
};
use web_search_core::{
    build_request_evidence, default_search_plan_ids, parse_rrf_input, reciprocal_rank_fusion,
    serialize_rrf_output, WEB_SEARCH_CONCURRENCY_PER_CATEGORY, WEB_SEARCH_PROVIDER_LIMIT,
    WEB_SEARCH_PROVIDER_REGISTRY, WEB_SEARCH_RRF_K,
};

const GREETING: u32 = 1;
const WRITE_PROGRAM: u32 = 2;
const IDENTITY: u32 = 8;
const UNKNOWN: u32 = 0;
const INPUT_CAPACITY: usize = 4096;
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
// allocator. We use a single 256 KiB heap with an `AtomicUsize` offset: every
// WASM entry point calls `reset_bump()` first so the heap rolls back between
// calls and no per-allocation deallocation logic is required.
const BUMP_HEAP_SIZE: usize = 262_144;

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

#[no_mangle]
pub extern "C" fn input_ptr() -> *mut u8 {
    core::ptr::addr_of_mut!(INPUT).cast::<u8>()
}

#[no_mangle]
pub extern "C" fn output_ptr() -> *mut u8 {
    core::ptr::addr_of_mut!(OUTPUT).cast::<u8>()
}

#[no_mangle]
pub extern "C" fn input_capacity() -> usize {
    INPUT_CAPACITY
}

#[no_mangle]
pub extern "C" fn output_capacity() -> usize {
    OUTPUT_CAPACITY
}

#[no_mangle]
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

#[no_mangle]
pub extern "C" fn web_search_rrf_k() -> u32 {
    WEB_SEARCH_RRF_K
}

#[no_mangle]
pub extern "C" fn web_search_concurrency_per_category() -> u32 {
    WEB_SEARCH_CONCURRENCY_PER_CATEGORY
}

#[no_mangle]
pub extern "C" fn web_search_provider_limit() -> u32 {
    WEB_SEARCH_PROVIDER_LIMIT
}

#[no_mangle]
pub extern "C" fn web_search_registry_len() -> u32 {
    WEB_SEARCH_PROVIDER_REGISTRY.len() as u32
}

/// Write the canonical default plan ids to `OUTPUT`, one per line.
///
/// Returns the number of bytes written.
#[no_mangle]
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
#[no_mangle]
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
#[no_mangle]
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
#[no_mangle]
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
#[no_mangle]
pub extern "C" fn engine_detect_language(input_length: usize) -> usize {
    reset_bump();
    let bytes = unsafe {
        core::slice::from_raw_parts(
            core::ptr::addr_of!(INPUT).cast::<u8>(),
            min(input_length, INPUT_CAPACITY),
        )
    };
    let text = core::str::from_utf8(bytes).unwrap_or("");
    let slug: &'static str = match detect_language(text) {
        Language::English => "en",
        Language::Russian => "ru",
        Language::Hindi => "hi",
        Language::Chinese => "zh",
        Language::Unknown => "unknown",
    };
    write_output(slug.as_bytes())
}

/// Evaluate an arithmetic expression. `INPUT` holds the raw expression bytes;
/// on success `OUTPUT` carries the formatted decimal result. On failure the
/// payload is `ERR:<reason>` so JS can render the failure in its native UI
/// without duplicating the parser. Returns the number of bytes written.
#[no_mangle]
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

/// Build a stable FNV-1a id. `INPUT` contains `prefix\ntext`; `OUTPUT`
/// receives `prefix_<hash>`.
#[no_mangle]
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
#[no_mangle]
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
#[no_mangle]
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
#[no_mangle]
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
