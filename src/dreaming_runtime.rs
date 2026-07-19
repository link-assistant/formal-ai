//! Cooperative, default-on dreaming for the core server runtime.
//!
//! The worker never runs while a foreground request is active and waits for a
//! real idle window after the latest request. Without persisted issue-494
//! consent it may retain newly learned amendments/patterns but strips every
//! deletion action, so background learning is default-on while freeing remains
//! default-off.

use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Once;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::dreaming::{apply_dreaming_plan, DreamingOutcome};
use crate::memory::MemoryStore;
use crate::storage_policy::{auto_free_space_enabled, plan_for_real_storage};

const DEFAULT_IDLE_SECONDS: u64 = 60;
const DEFAULT_INTERVAL_SECONDS: u64 = 6 * 60 * 60;

static ACTIVE_FOREGROUND: AtomicUsize = AtomicUsize::new(0);
static LAST_FOREGROUND_SECONDS: AtomicU64 = AtomicU64::new(0);
static START: Once = Once::new();

/// Guard one foreground operation. Dropping it wakes the idle clock and lets
/// the low-priority worker yield cooperatively on every supported platform.
pub struct ForegroundActivity;

impl ForegroundActivity {
    #[must_use]
    pub fn begin() -> Self {
        ACTIVE_FOREGROUND.fetch_add(1, Ordering::SeqCst);
        LAST_FOREGROUND_SECONDS.store(now_seconds(), Ordering::SeqCst);
        Self
    }
}

impl Drop for ForegroundActivity {
    fn drop(&mut self) {
        LAST_FOREGROUND_SECONDS.store(now_seconds(), Ordering::SeqCst);
        ACTIVE_FOREGROUND.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Whether the core has no foreground work and has passed the idle threshold.
#[must_use]
pub fn core_is_idle(idle_for: Duration) -> bool {
    if ACTIVE_FOREGROUND.load(Ordering::SeqCst) != 0 {
        return false;
    }
    now_seconds().saturating_sub(LAST_FOREGROUND_SECONDS.load(Ordering::SeqCst))
        >= idle_for.as_secs()
}

/// Start the one-per-process core worker.
///
/// Dreaming is **default-on**: it only stays off when `FORMAL_AI_DREAMING` is
/// explicitly set to `0`/`off`/`false`. The worker uses the same default shared
/// memory path as every foreground surface.
pub fn start_core_dreaming() {
    if dreaming_disabled() {
        return;
    }
    let path = crate::shared_memory::shared_memory_path();
    if let Err(error) = crate::shared_memory::ensure_shared_memory_file(&path) {
        eprintln!(
            "[dreaming] could not initialize {}: {error}",
            path.display()
        );
        return;
    }
    START.call_once(|| {
        LAST_FOREGROUND_SECONDS.store(now_seconds(), Ordering::SeqCst);
        std::thread::Builder::new()
            .name(String::from("formal-ai-dreaming"))
            .spawn(move || {
                lower_current_thread_priority();
                loop {
                    std::thread::sleep(Duration::from_secs(DEFAULT_IDLE_SECONDS));
                    if core_is_idle(Duration::from_secs(DEFAULT_IDLE_SECONDS)) {
                        if let Err(error) = run_core_dreaming_once(&path) {
                            if std::env::var("FORMAL_AI_DREAMING_DEBUG").as_deref() == Ok("1") {
                                eprintln!("[dreaming] background run failed: {error}");
                            }
                        }
                        std::thread::sleep(Duration::from_secs(DEFAULT_INTERVAL_SECONDS));
                    }
                }
            })
            .expect("spawn core dreaming worker");
    });
}

/// Drop the calling thread to the lowest OS scheduling priority so dreaming
/// work never competes with foreground request handling (issue #540 §6). Best
/// effort: an unsupported platform simply keeps the default priority — the
/// cooperative [`core_is_idle`] gate still applies either way.
fn lower_current_thread_priority() {
    let _ = thread_priority::set_current_thread_priority(thread_priority::ThreadPriority::Min);
}

/// Execute one idle run, exposed for deterministic integration tests.
///
/// The run yields cooperatively: if a foreground request arrives between the
/// planning and application steps, the run aborts without writing anything and
/// returns an empty outcome, so dreaming never delays live traffic.
pub fn run_core_dreaming_once(memory_path: &Path) -> io::Result<DreamingOutcome> {
    let mut store = MemoryStore::load_from_file(memory_path)?;
    let mut plan = plan_for_real_storage(&store, memory_path, 0)?;
    if !auto_free_space_enabled(memory_path) {
        plan.actions.clear();
        plan.selected_reclaim_bytes = 0;
    }
    // Mid-run cancellation point: planning is the expensive half, so check the
    // foreground gate again before mutating and persisting the store.
    if ACTIVE_FOREGROUND.load(Ordering::SeqCst) != 0 {
        return Ok(DreamingOutcome::default());
    }
    let outcome = apply_dreaming_plan(&mut store, &plan);
    if outcome.removed_events > 0
        || outcome.learned_amendments > 0
        || outcome.learned_patterns > 0
        || outcome.recorded_failures > 0
        || outcome.recorded_trials > 0
    {
        store.save_to_file(memory_path)?;
        // Keep the composed meta-recipe artifact in step with the amendments
        // that shape solving (issue #540 §1): the recipe next to the memory
        // log always reflects the currently retained amendment set.
        let recipe = crate::dreaming::compose_recipe_with_amendments(store.events());
        let recipe_path = memory_path.with_extension("recipe.lino");
        crate::memory::write_locked_atomic(&recipe_path, &recipe)?;
    }
    Ok(outcome)
}

/// Whether the `FORMAL_AI_DREAMING` opt-out is in force. Dreaming is
/// default-on; only an explicit `0`/`off`/`false` (any case) disables it.
#[must_use]
pub fn dreaming_disabled() -> bool {
    std::env::var("FORMAL_AI_DREAMING")
        .ok()
        .is_some_and(|value| matches!(value.to_ascii_lowercase().as_str(), "0" | "off" | "false"))
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
