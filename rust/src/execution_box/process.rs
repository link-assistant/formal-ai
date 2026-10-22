//! Deadline-bound native invocation I/O; owned by `ExecutionBox`.
use super::{BoxError, BoxObservation, POLL};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[cfg(not(target_arch = "wasm32"))]
const CAPTURE_LIMIT: usize = 1024 * 1024;
#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
struct CapturedOutput {
    text: String,
    exceeded: bool,
    read_error: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn kill_observed_group(identifier: u32) {
    #[cfg(unix)]
    if identifier > 1
        && let Some(group) = i32::try_from(identifier)
            .ok()
            .and_then(rustix::process::Pid::from_raw)
    {
        let _ = rustix::process::kill_process_group(group, rustix::process::Signal::KILL);
    }
    #[cfg(not(unix))]
    let _ = identifier;
}

/// Async reads are aborted and dropped at the same invocation deadline; no blocking reader joins.
#[cfg(not(target_arch = "wasm32"))]
fn stream<R: tokio::io::AsyncRead + Unpin + Send + 'static>(
    mut pipe: R,
    into: Arc<Mutex<CapturedOutput>>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        use tokio::io::AsyncReadExt;
        let mut buffer = [0_u8; 1024];
        loop {
            let read = match pipe.read(&mut buffer).await {
                Ok(read) => read,
                Err(error) => {
                    if let Ok(mut output) = into.lock() {
                        output.read_error = Some(error.to_string());
                    }
                    break;
                }
            };
            if read == 0 {
                break;
            }
            if let Ok(mut output) = into.lock() {
                let decoded = String::from_utf8_lossy(&buffer[..read]);
                let available = CAPTURE_LIMIT.saturating_sub(output.text.len());
                let mut accepted = available.min(decoded.len());
                while !decoded.is_char_boundary(accepted) {
                    accepted -= 1;
                }
                output.text.push_str(&decoded[..accepted]);
                if accepted != decoded.len() {
                    output.exceeded = true;
                    break;
                }
            }
        }
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) async fn observe_async(
    command: Command,
    input: Option<Vec<u8>>,
    started: Instant,
    bound: Duration,
    deadline: Duration,
    process: &std::sync::atomic::AtomicU32,
) -> Result<BoxObservation, BoxError> {
    use tokio::io::AsyncWriteExt;
    let mut configured = tokio::process::Command::from(command);
    configured.kill_on_drop(true);
    let mut child = configured.spawn().map_err(|error| BoxError::Observed {
        detail: error.to_string(),
    })?;
    let identifier = child.id().unwrap_or(0);
    process.store(identifier, std::sync::atomic::Ordering::Release);
    let observed = Arc::new(Mutex::new(CapturedOutput::default()));
    let mut readers = Vec::new();
    if let Some(stdout) = child.stdout.take() {
        readers.push(stream(stdout, Arc::clone(&observed)));
    }
    if let Some(stderr) = child.stderr.take() {
        readers.push(stream(stderr, Arc::clone(&observed)));
    }
    let mut writer = input.zip(child.stdin.take()).map(|(bytes, mut pipe)| {
        tokio::spawn(async move {
            pipe.write_all(&bytes).await?;
            pipe.shutdown().await
        })
    });
    let remaining = bound.saturating_sub(started.elapsed());
    let completed = tokio::time::timeout(remaining, async {
        let mut status = None;
        loop {
            if let Some(error) = observed
                .lock()
                .ok()
                .and_then(|output| output.read_error.clone())
            {
                return Err(error);
            }
            if observed.lock().is_ok_and(|output| output.exceeded) {
                return Err("execution_output_limit_exceeded".to_owned());
            }
            if writer
                .as_ref()
                .is_some_and(tokio::task::JoinHandle::is_finished)
                && let Some(finished) = writer.take()
            {
                finished
                    .await
                    .map_err(|error| error.to_string())?
                    .map_err(|error| error.to_string())?;
            }
            if status.is_none() {
                status = child.try_wait().map_err(|error| error.to_string())?;
            }
            if status.is_some()
                && writer.is_none()
                && readers.iter().all(tokio::task::JoinHandle::is_finished)
            {
                for reader in &mut readers {
                    reader.await.map_err(|error| error.to_string())?;
                }
                if let Some(error) = observed
                    .lock()
                    .ok()
                    .and_then(|output| output.read_error.clone())
                {
                    return Err(error);
                }
                return Ok(status.and_then(|status| status.code()).map(i64::from));
            }
            tokio::time::sleep(POLL).await;
        }
    })
    .await;
    if let Some(writer) = writer {
        writer.abort();
    }
    kill_observed_group(identifier);
    let _ = child.start_kill();
    let cleanup = Instant::now();
    let mut reaped = false;
    loop {
        if let Ok(Some(_)) = child.try_wait() {
            reaped = true;
        }
        if reaped && readers.iter().all(tokio::task::JoinHandle::is_finished) {
            break;
        }
        if cleanup.elapsed() >= Duration::from_millis(250) {
            for reader in readers {
                reader.abort();
            }
            return Err(BoxError::Observed {
                detail: [
                    "execution_cleanup_incomplete".to_owned(),
                    observed
                        .lock()
                        .map(|output| output.text.clone())
                        .unwrap_or_default(),
                ]
                .join("\n"),
            });
        }
        tokio::time::sleep(POLL).await;
    }
    let partial_output = observed
        .lock()
        .map(|output| output.text.clone())
        .unwrap_or_default();
    let (exit_code, timed_out) = match completed {
        Ok(Ok(code)) => (code, false),
        Ok(Err(detail)) => {
            return Err(BoxError::Observed {
                detail: if partial_output.is_empty() {
                    detail
                } else {
                    [detail, partial_output].join("\n")
                },
            });
        }
        Err(_) => (None, true),
    };
    Ok(BoxObservation {
        exit_code,
        timed_out,
        elapsed: started.elapsed(),
        deadline,
        partial_output,
    })
}
