//! Hard output budgets around curl, independent of curl version and compression behavior.
use super::FetchError;
use std::io::{self, Read};
use std::process::{Child, Command, Output, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// Every read must make progress; an oversized stream never becomes a full buffered result.
pub(crate) fn read_limited(mut reader: impl Read, maximum: usize) -> io::Result<Vec<u8>> {
    let capacity = maximum
        .checked_add(1)
        .ok_or_else(|| io::Error::other("source_transport_invalid_output_limit"))?;
    let mut bytes = Vec::with_capacity(capacity);
    let mut chunk = [0_u8; 64 * 1024];
    for _ in 0..=maximum {
        let remaining = maximum.saturating_sub(bytes.len());
        let length = remaining.saturating_add(1).min(chunk.len());
        let count = reader.read(&mut chunk[..length])?;
        if count == 0 {
            return Ok(bytes);
        }
        if count > remaining {
            return Err(io::Error::other("source_transport_output_budget_exceeded"));
        }
        bytes.extend_from_slice(&chunk[..count]);
    }
    Err(io::Error::other("source_transport_read_progress_exceeded"))
}

fn stop(child: &mut Child) -> bool {
    let _ = child.kill();
    for _ in 0..50 {
        match child.try_wait() {
            Ok(Some(_)) => return true,
            Err(_) => return false,
            Ok(None) => std::thread::sleep(Duration::from_millis(5)),
        }
    }
    false
}

/// Reader completion is observed through a bounded channel deadline, never an unbounded join.
pub(super) fn bounded_output(
    command: &mut Command,
    body_limit: usize,
    diagnostic_limit: usize,
) -> Result<Output, FetchError> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            FetchError::Transport(if error.kind() == io::ErrorKind::NotFound {
                "curl is required for live retrieval".to_owned()
            } else {
                error.to_string()
            })
        })?;
    let Some(stdout) = child.stdout.take() else {
        stop(&mut child);
        return Err(FetchError::Transport(
            "source_transport_stdout_missing".to_owned(),
        ));
    };
    let Some(stderr) = child.stderr.take() else {
        stop(&mut child);
        return Err(FetchError::Transport(
            "source_transport_stderr_missing".to_owned(),
        ));
    };
    let (sender, receiver) = mpsc::sync_channel(2);
    let output_sender = sender.clone();
    if let Err(error) = std::thread::Builder::new().spawn(move || {
        let _ = output_sender.send((true, read_limited(stdout, body_limit)));
    }) {
        stop(&mut child);
        return Err(FetchError::Transport(error.to_string()));
    }
    if let Err(error) = std::thread::Builder::new().spawn(move || {
        let _ = sender.send((false, read_limited(stderr, diagnostic_limit)));
    }) {
        stop(&mut child);
        return Err(FetchError::Transport(error.to_string()));
    }
    let started = Instant::now();
    let mut status = None;
    let mut output = None;
    let mut diagnostic = None;
    for _ in 0..7001 {
        match receiver.recv_timeout(Duration::from_millis(5)) {
            Ok((is_output, Ok(bytes))) => {
                if is_output {
                    output = Some(bytes);
                } else {
                    diagnostic = Some(bytes);
                }
            }
            Ok((_, Err(error))) => {
                let cleaned = stop(&mut child);
                return Err(FetchError::Transport(format!(
                    "{error}; cleanup_complete={cleaned}"
                )));
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                if output.is_none() || diagnostic.is_none() {
                    stop(&mut child);
                    return Err(FetchError::Transport(
                        "source_transport_reader_disconnected".to_owned(),
                    ));
                }
            }
        }
        if status.is_none() {
            match child.try_wait() {
                Ok(observed) => status = observed,
                Err(error) => {
                    stop(&mut child);
                    return Err(FetchError::Transport(error.to_string()));
                }
            }
        }
        if status.is_some() && output.is_some() && diagnostic.is_some() {
            return Ok(Output {
                status: status.ok_or_else(|| {
                    FetchError::Transport("source_transport_status_missing".to_owned())
                })?,
                stdout: output.ok_or_else(|| {
                    FetchError::Transport("source_transport_stdout_missing".to_owned())
                })?,
                stderr: diagnostic.ok_or_else(|| {
                    FetchError::Transport("source_transport_stderr_missing".to_owned())
                })?,
            });
        }
        if started.elapsed() >= Duration::from_secs(35) {
            break;
        }
        // Disconnected readers may finish before the child exits; retain a finite polling cadence.
        if output.is_some() && diagnostic.is_some() {
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    let cleaned = stop(&mut child);
    Err(FetchError::Transport(format!(
        "source_transport_timeout; cleanup_complete={cleaned}"
    )))
}
