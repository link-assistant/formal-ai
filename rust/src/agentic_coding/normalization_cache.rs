//! Exact-input memoization with a strict process-wide memory/entry ceiling.
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};
const MAXIMUM_BYTES: usize = 4 * 1024 * 1024;
const MAXIMUM_ENTRIES: usize = 32;
static CACHE: OnceLock<Mutex<VecDeque<(String, String)>>> = OnceLock::new();
fn cache() -> std::sync::MutexGuard<'static, VecDeque<(String, String)>> {
    CACHE
        .get_or_init(|| Mutex::new(VecDeque::new()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
pub(crate) fn normalized(prompt: &str, compute: impl FnOnce(&str) -> String) -> String {
    if let Some(value) = cache()
        .iter()
        .find(|(key, _)| key == prompt)
        .map(|(_, value)| value.clone())
    {
        return value;
    }
    let value = compute(prompt);
    let size = prompt.len().saturating_add(value.len());
    if size <= MAXIMUM_BYTES / 4 {
        {
            let mut cache = cache();
            let mut bytes = cache
                .iter()
                .map(|(key, value)| key.len() + value.len())
                .sum::<usize>();
            while cache.len() >= MAXIMUM_ENTRIES || bytes + size > MAXIMUM_BYTES {
                if let Some((key, value)) = cache.pop_front() {
                    bytes -= key.len() + value.len();
                } else {
                    break;
                }
            }
            cache.push_back((prompt.to_owned(), value.clone()));
        }
    }
    value
}
