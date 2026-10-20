//! Shared finite ceiling for consecutive failed attempts at an external action.
/// Includes the first failed attempt. Success opens a new budget.
pub const MAXIMUM_FAILED_ATTEMPTS: usize = 3;
pub const MAXIMUM_ACTIONS_PER_OBJECTIVE: usize = 128;
#[derive(Default)]
pub struct FailureBudget {
    failures: usize,
}
impl FailureBudget {
    /// True means exhausted: report the original error and stop, without another attempt.
    pub fn failed(&mut self) -> bool {
        self.failures = self.failures.saturating_add(1);
        self.failures >= MAXIMUM_FAILED_ATTEMPTS
    }
    pub fn succeeded(&mut self) {
        self.failures = 0;
    }
}
