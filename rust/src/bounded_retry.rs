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

#[cfg(test)]
mod bounded_connection_tests {
    use super::{MAXIMUM_FAILED_ATTEMPTS, dispatch_incoming};
    fn serve_connections<Connection>(
        connections: impl IntoIterator<Item = std::io::Result<Connection>>,
        dispatch: impl FnMut(Connection) -> std::io::Result<()>,
    ) -> std::io::Result<()> {
        dispatch_incoming(connections, dispatch, |_| {}, |_| {})
    }
    use std::cell::Cell;
    use std::io::{Error, ErrorKind};

    #[test]
    fn repeated_accept_failures_stop_without_reading_another_attempt() {
        let attempts = Cell::new(0usize);
        let connections = std::iter::from_fn(|| {
            attempts.set(attempts.get() + 1);
            assert!(attempts.get() <= MAXIMUM_FAILED_ATTEMPTS);
            Some(Err::<(), _>(Error::new(
                ErrorKind::PermissionDenied,
                format!("accept-{}", attempts.get()),
            )))
        });
        let error =
            serve_connections(connections, |()| panic!("no accepted connection")).unwrap_err();
        assert_eq!(attempts.get(), MAXIMUM_FAILED_ATTEMPTS);
        assert_eq!(error.kind(), ErrorKind::PermissionDenied);
        assert_eq!(
            error.to_string(),
            format!("accept-{}", MAXIMUM_FAILED_ATTEMPTS)
        );
    }

    #[test]
    fn repeated_dispatch_failures_keep_the_original_error_and_stop() {
        let attempts = Cell::new(0usize);
        let error = serve_connections((0..5).map(Ok), |_| {
            attempts.set(attempts.get() + 1);
            Err(Error::new(
                ErrorKind::WouldBlock,
                format!("dispatch-{}", attempts.get()),
            ))
        })
        .unwrap_err();
        assert_eq!(attempts.get(), MAXIMUM_FAILED_ATTEMPTS);
        assert_eq!(error.kind(), ErrorKind::WouldBlock);
        assert_eq!(
            error.to_string(),
            format!("dispatch-{}", MAXIMUM_FAILED_ATTEMPTS)
        );
    }

    #[test]
    fn accepted_connections_reset_only_the_accept_failure_budget() {
        let accepted = Cell::new(0usize);
        let connections = [
            Err(Error::other("first")),
            Err(Error::other("second")),
            Ok(()),
            Err(Error::other("third")),
            Err(Error::other("fourth")),
            Ok(()),
        ];
        serve_connections(connections, |()| {
            accepted.set(accepted.get() + 1);
            Ok(())
        })
        .unwrap();
        assert_eq!(accepted.get(), 2);
        let dispatches = Cell::new(0usize);
        let connections = [Ok(()), Err(Error::other("accept")), Ok(()), Ok(())];
        assert!(
            serve_connections(connections, |()| {
                dispatches.set(dispatches.get() + 1);
                Err(Error::other("dispatch"))
            })
            .is_err()
        );
        assert_eq!(dispatches.get(), MAXIMUM_FAILED_ATTEMPTS);
    }

    #[test]
    fn successfully_dispatched_connections_reset_the_dispatch_failure_budget() {
        let attempts = Cell::new(0usize);
        let connections = [true, true, false, true, true, false, true, true]
            .into_iter()
            .map(Ok);
        serve_connections(connections, |failed| {
            attempts.set(attempts.get() + 1);
            if failed {
                Err(Error::other("dispatch"))
            } else {
                Ok(())
            }
        })
        .unwrap();
        assert_eq!(attempts.get(), 8);
    }
}
