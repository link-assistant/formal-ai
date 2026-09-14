//! Measure cumulative allocations made by one browser statement-fusion call.
//!
//! The WASM worker uses a resettable bump allocator, so cumulative allocated
//! bytes—not live bytes—determine whether a request fits. Compile from the
//! repository root with:
//!
//! `rustc --edition=2024 experiments/issue709_wasm_heap.rs -o /tmp/issue709-wasm-heap`

extern crate alloc;

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[path = "../src/web_search_fusion_core.rs"]
mod web_search_fusion_core;

struct CountingAllocator;

static COUNTING: AtomicBool = AtomicBool::new(false);
static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static PEAK_ALIGNMENT: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if COUNTING.load(Ordering::Relaxed) {
            ALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
            PEAK_ALIGNMENT.fetch_max(layout.align(), Ordering::Relaxed);
        }
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        System.dealloc(pointer, layout);
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn measure(label: &str, payload: &str) {
    ALLOCATED.store(0, Ordering::SeqCst);
    PEAK_ALIGNMENT.store(0, Ordering::SeqCst);
    COUNTING.store(true, Ordering::SeqCst);
    let output = web_search_fusion_core::fuse_statement_search_payload(payload);
    COUNTING.store(false, Ordering::SeqCst);

    println!("case={label}");
    println!("payload_bytes={}", payload.len());
    println!("output_bytes={}", output.len());
    println!("cumulative_allocated={}", ALLOCATED.load(Ordering::SeqCst));
    println!("peak_alignment={}", PEAK_ALIGNMENT.load(Ordering::SeqCst));
}

fn main() {
    let apple = concat!(
        "Q\tApple\ten\tRead more\tvia\tOther sources\n",
        "S\thttps://duckduckgo.com/Apple\tApple\tApple is a fruit produced by an apple tree.\t\ten\tduckduckgo#1\t2\tprimary\n",
        "S\thttps://en.wikipedia.org/wiki/Apple\tApple\tApple is the edible fruit of an apple tree.\t\ten\twikipedia#1\t1\tprimary\n",
        "S\thttps://www.wikidata.org/wiki/Q89\tApple\tfruit of the apple tree\t\ten\twikidata#1\t1\talternate",
    );
    measure("apple", apple);

    let excerpt = "Apple benchmark source text presents one bounded captured passage. ".repeat(8);
    let mut provider_limit = String::from("Q\tApple\ten\tRead more\tvia\tOther sources");
    for index in 0..24 {
        provider_limit.push_str(&format!(
            "\nS\thttps://example{index}.invalid/apple\tApple source {index}\t{excerpt}\t\ten\tprovider#{index}\t{}\tprimary",
            index + 1,
        ));
    }
    measure("source_limit", &provider_limit);
}
