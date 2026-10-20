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
    pub const fn failed(&mut self) -> bool {
        self.failures = self.failures.saturating_add(1);
        self.failures >= MAXIMUM_FAILED_ATTEMPTS
    }
    pub const fn succeeded(&mut self) {
        self.failures = 0;
    }
}

/// Dispatch new connections while bounding consecutive acceptance and dispatch failures separately.
pub fn dispatch_incoming<Connection>(
    connections: impl IntoIterator<Item = std::io::Result<Connection>>,
    mut dispatch: impl FnMut(Connection) -> std::io::Result<()>,
    mut accept_error: impl FnMut(&std::io::Error),
    mut dispatch_error: impl FnMut(&std::io::Error),
) -> std::io::Result<()> {
    let mut accept_failures = FailureBudget::default();
    let mut dispatch_failures = FailureBudget::default();
    for connection in connections {
        let connection = match connection {
            Ok(connection) => {
                accept_failures.succeeded();
                connection
            }
            Err(error) => {
                accept_error(&error);
                if accept_failures.failed() {
                    return Err(error);
                }
                continue;
            }
        };
        match dispatch(connection) {
            Ok(()) => dispatch_failures.succeeded(),
            Err(error) => {
                dispatch_error(&error);
                if dispatch_failures.failed() {
                    return Err(error);
                }
            }
        }
    }
    Ok(())
}
