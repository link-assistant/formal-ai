//! Shared finite child-command supervision for caller-owned host operations.
//!
//! Captured bytes are exact, status-only calls retain configured stdio, and a
//! deadline never certifies incomplete pipe or process cleanup as success.
use std::fmt;
use std::io;
use std::process::{Child, Command, ExitStatus, Output};
use std::time::{Duration, Instant};

#[cfg(not(target_arch = "wasm32"))]
const CLEANUP_GRACE: Duration = Duration::from_millis(250);
#[cfg(not(target_arch = "wasm32"))]
const POLL: Duration = Duration::from_millis(5);

/// A failed finite command observation, including bounded partial output and cleanup custody.
#[derive(Debug)]
pub struct CommandFailure {
    source: io::Error,
    timed_out: bool,
    observed_status: Option<ExitStatus>,
    partial_stdout: Vec<u8>,
    partial_stderr: Vec<u8>,
    elapsed: Duration,
    cleanup_complete: bool,
}
impl CommandFailure {
    /// Underlying I/O error category.
    #[must_use]
    pub fn kind(&self) -> io::ErrorKind {
        self.source.kind()
    }
    /// Whether the complete command observation exceeded its deadline.
    #[must_use]
    pub const fn timed_out(&self) -> bool {
        self.timed_out
    }
    /// Last observed child exit status; this alone does not certify complete output.
    #[must_use = "inspect the observed status together with cleanup_complete"]
    pub const fn observed_status(&self) -> Option<ExitStatus> {
        self.observed_status
    }
    /// Exact bounded stdout captured before failure.
    #[must_use]
    pub const fn partial_stdout(&self) -> &[u8] {
        self.partial_stdout.as_slice()
    }
    /// Exact bounded stderr captured before failure.
    #[must_use]
    pub const fn partial_stderr(&self) -> &[u8] {
        self.partial_stderr.as_slice()
    }
    /// Time spent supervising and cleaning up this invocation.
    #[must_use]
    pub const fn elapsed(&self) -> Duration {
        self.elapsed
    }
    /// Whether child exit and captured stream completion were confirmed.
    #[must_use]
    pub const fn cleanup_complete(&self) -> bool {
        self.cleanup_complete
    }
}
impl fmt::Display for CommandFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.source, formatter)
    }
}
impl std::error::Error for CommandFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

pub(crate) struct TimeBudget {
    started: Instant,
    limit: Duration,
}
impl TimeBudget {
    #[must_use]
    pub(crate) fn new(limit: Duration) -> Self {
        Self {
            started: Instant::now(),
            limit,
        }
    }
    #[must_use]
    pub(crate) fn remaining(&self) -> Duration {
        self.limit.saturating_sub(self.started.elapsed())
    }
}

/// Capture exact binary stdout and stderr; a zero stream limit discards that stream.
///
/// # Errors
/// Returns typed spawn, I/O, limit, deadline, cleanup or unsupported-platform failures.
pub fn command_output(
    command: Command,
    timeout: Duration,
    stdout_limit: usize,
    stderr_limit: usize,
) -> Result<Output, CommandFailure> {
    supervise(
        command,
        None,
        timeout,
        CaptureMode::Split {
            stdout_limit,
            stderr_limit,
        },
    )
}

/// Capture bounded binary output while writing caller-owned input within the same deadline.
///
/// # Errors
/// Returns typed spawn, I/O, limit, deadline, cleanup or unsupported-platform failures.
pub fn command_output_with_input(
    command: Command,
    input: Vec<u8>,
    timeout: Duration,
    stdout_limit: usize,
    stderr_limit: usize,
) -> Result<Output, CommandFailure> {
    supervise(
        command,
        Some(input),
        timeout,
        CaptureMode::Split {
            stdout_limit,
            stderr_limit,
        },
    )
}

/// Observe exit status within a finite deadline while preserving configured stdio.
///
/// # Errors
/// Returns typed spawn, I/O, limit, deadline, cleanup or unsupported-platform failures.
pub fn command_status(command: Command, timeout: Duration) -> Result<ExitStatus, CommandFailure> {
    supervise(command, None, timeout, CaptureMode::Discard).map(|observed| observed.status)
}

/// Stop only this caller-owned child; this does not assert ownership of a process group.
///
/// # Errors
/// Returns the original child observation or termination error, or Unsupported on WebAssembly.
pub fn stop_child(child: &mut Child, grace: Duration) -> io::Result<bool> {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (child, grace);
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        if Instant::now().checked_add(grace).is_none() {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }
        if child.try_wait()?.is_some() {
            return Ok(true);
        }
        if let Err(error) = child.kill() {
            if child.try_wait()?.is_some() {
                return Ok(true);
            }
            return Err(error);
        }
        let started = Instant::now();
        loop {
            if child.try_wait()?.is_some() {
                return Ok(true);
            }
            let remaining = grace.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                return Ok(false);
            }
            std::thread::sleep(POLL.min(remaining));
        }
    }
}

#[derive(Clone, Copy)]
enum CaptureMode {
    Split {
        stdout_limit: usize,
        stderr_limit: usize,
    },
    Discard,
}
#[derive(Default)]
struct Captured {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    failure: Option<io::Error>,
}
fn failure(
    source: io::Error,
    captured: Captured,
    status: Option<ExitStatus>,
    timed_out: bool,
    elapsed: Duration,
    cleanup_complete: bool,
) -> CommandFailure {
    let _ = captured.failure;
    CommandFailure {
        source,
        timed_out,
        observed_status: status,
        partial_stdout: captured.stdout,
        partial_stderr: captured.stderr,
        elapsed,
        cleanup_complete,
    }
}

#[cfg(target_arch = "wasm32")]
fn supervise(
    command: Command,
    input: Option<Vec<u8>>,
    timeout: Duration,
    mode: CaptureMode,
) -> Result<Output, CommandFailure> {
    let _ = (command, input, timeout);
    match mode {
        CaptureMode::Split {
            stdout_limit,
            stderr_limit,
        } => {
            let _ = (stdout_limit, stderr_limit);
        }
        CaptureMode::Discard => {}
    }
    Err(failure(
        io::Error::from(io::ErrorKind::Unsupported),
        Captured::default(),
        None,
        false,
        Duration::ZERO,
        true,
    ))
}

/// The identifier must belong to a process group created by this supervisor.
pub(crate) fn kill_owned_group(identifier: u32) {
    #[cfg(all(unix, not(target_arch = "wasm32")))]
    if identifier > 1
        && let Some(group) = i32::try_from(identifier)
            .ok()
            .and_then(rustix::process::Pid::from_raw)
    {
        let _ = rustix::process::kill_process_group(group, rustix::process::Signal::KILL);
    }
    #[cfg(not(all(unix, not(target_arch = "wasm32"))))]
    let _ = identifier;
}

#[cfg(not(target_arch = "wasm32"))]
fn supervise(
    command: Command,
    input: Option<Vec<u8>>,
    timeout: Duration,
    mode: CaptureMode,
) -> Result<Output, CommandFailure> {
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicU32, Ordering},
        mpsc,
    };
    let started = Instant::now();
    let valid_limits = match mode {
        CaptureMode::Split {
            stdout_limit,
            stderr_limit,
        } => stdout_limit.checked_add(stderr_limit).is_some(),
        CaptureMode::Discard => true,
    };
    if !valid_limits
        || timeout.is_zero()
        || timeout
            .checked_add(CLEANUP_GRACE * 2)
            .and_then(|duration| started.checked_add(duration))
            .is_none()
    {
        return Err(failure(
            io::Error::from(io::ErrorKind::InvalidInput),
            Captured::default(),
            None,
            false,
            started.elapsed(),
            true,
        ));
    }
    let observed = Arc::new(Mutex::new(Captured::default()));
    let child_identifier = Arc::new(AtomicU32::new(0));
    let child_worker = Arc::clone(&child_identifier);
    let output_worker = Arc::clone(&observed);
    let (sender, receiver) = mpsc::sync_channel(1);
    let worker = std::thread::Builder::new()
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            let result = match runtime {
                Ok(runtime) => {
                    let result = runtime.block_on(observe(
                        command,
                        input,
                        timeout,
                        mode,
                        started,
                        &child_worker,
                        &output_worker,
                    ));
                    runtime.shutdown_timeout(Duration::ZERO);
                    result
                }
                Err(error) => Err(failure(
                    error,
                    Captured::default(),
                    None,
                    false,
                    started.elapsed(),
                    true,
                )),
            };
            let _ = sender.send(result);
        })
        .map_err(|error| {
            failure(
                error,
                Captured::default(),
                None,
                false,
                started.elapsed(),
                true,
            )
        })?;
    let remaining = timeout
        .saturating_sub(started.elapsed())
        .saturating_add(CLEANUP_GRACE * 2);
    let result = receiver.recv_timeout(remaining).unwrap_or_else(|_| {
        kill_owned_group(child_identifier.load(Ordering::Acquire));
        let captured = observed
            .try_lock()
            .ok()
            .map(|mut data| std::mem::take(&mut *data))
            .unwrap_or_default();
        Err(failure(
            io::Error::other("command_cleanup_incomplete"),
            captured,
            None,
            true,
            started.elapsed(),
            false,
        ))
    });
    if worker.is_finished() {
        let _ = worker.join();
    }
    result
}

#[cfg(not(target_arch = "wasm32"))]
fn stream<R: tokio::io::AsyncRead + Unpin + Send + 'static>(
    mut reader: R,
    stdout: bool,
    mode: CaptureMode,
    into: std::sync::Arc<std::sync::Mutex<Captured>>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        use tokio::io::AsyncReadExt;
        let mut buffer = [0_u8; 1024];
        loop {
            let count = match reader.read(&mut buffer).await {
                Ok(count) => count,
                Err(error) => {
                    if let Ok(mut output) = into.lock() {
                        output.failure = Some(error);
                    }
                    break;
                }
            };
            if count == 0 {
                break;
            }
            if let Ok(mut output) = into.lock() {
                if output.failure.is_some() {
                    break;
                }
                match mode {
                    CaptureMode::Discard => {}
                    CaptureMode::Split {
                        stdout_limit,
                        stderr_limit,
                    } => {
                        let limit = if stdout { stdout_limit } else { stderr_limit };
                        let target = if stdout {
                            &mut output.stdout
                        } else {
                            &mut output.stderr
                        };
                        let accepted = limit.saturating_sub(target.len()).min(count);
                        target.extend_from_slice(&buffer[..accepted]);
                        if accepted != count {
                            output.failure = Some(io::Error::new(
                                io::ErrorKind::InvalidData,
                                if stdout {
                                    "command_stdout_limit"
                                } else {
                                    "command_stderr_limit"
                                },
                            ));
                            break;
                        }
                    }
                }
            }
        }
    })
}

#[cfg(not(target_arch = "wasm32"))]
async fn observe(
    command: Command,
    input: Option<Vec<u8>>,
    timeout: Duration,
    mode: CaptureMode,
    started: Instant,
    child_identifier: &std::sync::atomic::AtomicU32,
    observed: &std::sync::Arc<std::sync::Mutex<Captured>>,
) -> Result<Output, CommandFailure> {
    use std::process::Stdio;
    use std::sync::{Arc, atomic::Ordering};
    use tokio::io::AsyncWriteExt;
    if started.elapsed() >= timeout {
        return Err(failure(
            io::Error::from(io::ErrorKind::TimedOut),
            Captured::default(),
            None,
            true,
            started.elapsed(),
            true,
        ));
    }
    let mut command = tokio::process::Command::from(command);
    command.kill_on_drop(true);
    match mode {
        CaptureMode::Split {
            stdout_limit,
            stderr_limit,
        } => {
            command.stdout(if stdout_limit == 0 {
                Stdio::null()
            } else {
                Stdio::piped()
            });
            command.stderr(if stderr_limit == 0 {
                Stdio::null()
            } else {
                Stdio::piped()
            });
        }
        CaptureMode::Discard => {}
    }
    if input.is_some() {
        command.stdin(Stdio::piped());
    } else if !matches!(mode, CaptureMode::Discard) {
        command.stdin(Stdio::null());
    }
    #[cfg(unix)]
    command.process_group(0);
    let mut child = command.spawn().map_err(|error| {
        failure(
            error,
            Captured::default(),
            None,
            false,
            started.elapsed(),
            true,
        )
    })?;
    let identifier = child.id().unwrap_or(0);
    child_identifier.store(identifier, Ordering::Release);
    let mut readers = Vec::new();
    if let Some(stdout) = child.stdout.take() {
        readers.push(stream(stdout, true, mode, Arc::clone(observed)));
    }
    if let Some(stderr) = child.stderr.take() {
        readers.push(stream(stderr, false, mode, Arc::clone(observed)));
    }
    let mut writer = input.zip(child.stdin.take()).map(|(bytes, mut pipe)| {
        tokio::spawn(async move {
            pipe.write_all(&bytes).await?;
            pipe.shutdown().await
        })
    });
    let mut status = None;
    let remaining = timeout.saturating_sub(started.elapsed());
    let completed = tokio::time::timeout(remaining, async {
        loop {
            if observed.lock().is_ok_and(|output| output.failure.is_some()) {
                return Err(None);
            }
            if writer
                .as_ref()
                .is_some_and(tokio::task::JoinHandle::is_finished)
                && let Some(finished) = writer.take()
            {
                match finished.await {
                    Ok(Ok(())) => {}
                    Ok(Err(error)) => return Err(Some(error)),
                    Err(error) => return Err(Some(io::Error::other(error))),
                }
            }
            if status.is_none() {
                status = match child.try_wait() {
                    Ok(status) => status,
                    Err(error) => return Err(Some(error)),
                };
            }
            if status.is_some()
                && writer.is_none()
                && readers.iter().all(tokio::task::JoinHandle::is_finished)
            {
                for reader in readers.drain(..) {
                    if let Err(error) = reader.await {
                        return Err(Some(io::Error::other(error)));
                    }
                }
                return if observed.lock().is_ok_and(|output| output.failure.is_some()) {
                    Err(None)
                } else {
                    Ok(())
                };
            }
            tokio::time::sleep(POLL).await;
        }
    })
    .await;
    // Recheck real completion before deadline cleanup; a completed child/EOF wins the timer race.
    if status.is_none() {
        status = child.try_wait().ok().flatten();
    }
    let mut reader_error = None;
    if completed.is_err()
        && writer
            .as_ref()
            .is_some_and(tokio::task::JoinHandle::is_finished)
        && let Some(finished) = writer.take()
    {
        match finished.await {
            Ok(Ok(())) => {}
            Ok(Err(error)) => reader_error = Some(error),
            Err(error) => reader_error = Some(io::Error::other(error)),
        }
    }
    if completed.is_err() && readers.iter().all(tokio::task::JoinHandle::is_finished) {
        for reader in readers.drain(..) {
            if let Err(error) = reader.await {
                reader_error = Some(io::Error::other(error));
            }
        }
    }
    let complete_at_deadline = reader_error.is_none()
        && status.is_some()
        && writer.is_none()
        && readers.iter().all(tokio::task::JoinHandle::is_finished)
        && observed.lock().is_ok_and(|output| output.failure.is_none());
    let timed_out = completed.is_err() && !complete_at_deadline;
    let mut error = match completed {
        Ok(Ok(())) => None,
        Ok(Err(error)) => error,
        Err(_) if complete_at_deadline => None,
        Err(_) => Some(io::Error::new(
            io::ErrorKind::TimedOut,
            "command_deadline_exceeded",
        )),
    };
    if reader_error.is_some() {
        error = reader_error;
    }
    if let Some(writer) = &writer {
        writer.abort();
    }
    kill_owned_group(identifier);
    let _ = child.start_kill();
    let cleanup = Instant::now();
    let mut reaped = status.is_some();
    let cleanup_complete = loop {
        if !reaped && let Ok(Some(observed_status)) = child.try_wait() {
            status = Some(observed_status);
            reaped = true;
        }
        if reaped
            && readers.iter().all(tokio::task::JoinHandle::is_finished)
            && writer
                .as_ref()
                .is_none_or(tokio::task::JoinHandle::is_finished)
        {
            break true;
        }
        if cleanup.elapsed() >= CLEANUP_GRACE {
            for reader in readers {
                reader.abort();
            }
            break false;
        }
        tokio::time::sleep(POLL).await;
    };
    child_identifier.store(0, Ordering::Release);
    let mut captured = observed
        .lock()
        .map(|mut data| std::mem::take(&mut *data))
        .unwrap_or_default();
    if error.is_none() {
        error = captured.failure.take();
    }
    if !cleanup_complete && error.is_none() {
        error = Some(io::Error::other("command_cleanup_incomplete"));
    }
    if let Some(error) = error {
        return Err(failure(
            error,
            captured,
            status,
            timed_out,
            started.elapsed(),
            cleanup_complete,
        ));
    }
    let Some(status) = status else {
        return Err(failure(
            io::Error::other("command_status_missing"),
            captured,
            None,
            timed_out,
            started.elapsed(),
            cleanup_complete,
        ));
    };
    Ok(Output {
        status,
        stdout: captured.stdout,
        stderr: captured.stderr,
    })
}
