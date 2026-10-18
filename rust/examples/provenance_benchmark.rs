//! Hosted measurement of actual native hash costs; no extrapolated GPU claims.
use formal_ai::provenance_native::{CpuHashMode, digest};
use serde_json::json;
use std::time::Instant;
fn measure(bytes: &[u8], mode: CpuHashMode, rounds: usize) -> (u128, u128) {
    let expected = digest(bytes, CpuHashMode::Serial);
    let mut timings = Vec::with_capacity(rounds);
    for _ in 0..rounds {
        let started = Instant::now();
        let result = digest(std::hint::black_box(bytes), mode);
        timings.push(started.elapsed().as_nanos());
        assert_eq!(result, expected);
        std::hint::black_box(result);
    }
    timings.sort_unstable();
    (
        timings[rounds / 2],
        timings[(rounds * 95).div_ceil(100).min(rounds) - 1],
    )
}
fn main() {
    assert_eq!(
        std::env::var("GITHUB_ACTIONS").as_deref(),
        Ok("true"),
        "benchmark is hosted-CI-only"
    );
    let small = vec![42; 1024];
    let source = vec![73; 100 * 1024 * 1024];
    let mut observations = Vec::new();
    for mode in [
        CpuHashMode::Serial,
        CpuHashMode::Parallel,
        CpuHashMode::Auto,
    ] {
        let (message_median, message_p95) = measure(&small, mode, 1001);
        let (source_median, source_p95) = measure(&source, mode, 21);
        observations.push(json!({"backend":format!("{mode:?}"),"message_bytes":small.len(),"message_rounds":1001,"message_median_ns":message_median,"message_p95_ns":message_p95,"source_bytes":source.len(),"source_rounds":21,"source_median_ns":source_median,"source_p95_ns":source_p95,"message_target_met":message_p95<1_000_000,"source_target_met":source_p95<1_000_000_000}));
    }
    println!(
        "{}",
        json!({"schema":"formal-ai-native-provenance-benchmark-v1","system":std::env::consts::OS,"architecture":std::env::consts::ARCH,"logical_cpus":std::thread::available_parallelism().map(|count|count.get()).unwrap_or(1),"runner_name":std::env::var("RUNNER_NAME").ok(),"runner_arch":std::env::var("RUNNER_ARCH").ok(),"source_sha":std::env::var("GITHUB_SHA").ok(),"measurements":observations,"gpu":"not-measured","scope":"BLAKE3 digest primitives; complete-session and source-inventory costs require their separate measurements"})
    );
}
