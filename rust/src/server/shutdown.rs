//! A native signal reactor supplies bounded cancellation without an unsafe signal handler.
#[cfg(all(not(target_arch = "wasm32"), any(unix, windows)))]
mod native {
    use std::io;
    use std::task::Poll;
    use std::time::Duration;
    pub struct Shutdown {
        runtime: tokio::runtime::Runtime,
        #[cfg(unix)]
        terminate: tokio::signal::unix::Signal,
        #[cfg(unix)]
        interrupt: tokio::signal::unix::Signal,
        #[cfg(windows)]
        terminate: tokio::signal::windows::CtrlBreak,
        #[cfg(windows)]
        interrupt: tokio::signal::windows::CtrlC,
    }
    impl Shutdown {
        pub fn new() -> io::Result<Self> {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            let enter = runtime.enter();
            #[cfg(unix)]
            let terminate =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
            #[cfg(unix)]
            let interrupt =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;
            #[cfg(windows)]
            let terminate = tokio::signal::windows::ctrl_break()?;
            #[cfg(windows)]
            let interrupt = tokio::signal::windows::ctrl_c()?;
            drop(enter);
            Ok(Self {
                runtime,
                terminate,
                interrupt,
            })
        }
        pub fn wait(&mut self, duration: Duration) -> bool {
            let terminate = &mut self.terminate;
            let interrupt = &mut self.interrupt;
            self.runtime.block_on(async {
                tokio::time::timeout(
                    duration,
                    std::future::poll_fn(|context| {
                        if terminate.poll_recv(context).is_ready()
                            || interrupt.poll_recv(context).is_ready()
                        {
                            Poll::Ready(())
                        } else {
                            Poll::Pending
                        }
                    }),
                )
                .await
                .is_ok()
            })
        }
    }
}
#[cfg(all(not(target_arch = "wasm32"), any(unix, windows)))]
pub(super) use native::Shutdown;
#[cfg(any(target_arch = "wasm32", not(any(unix, windows))))]
pub(super) struct Shutdown;
#[cfg(any(target_arch = "wasm32", not(any(unix, windows))))]
impl Shutdown {
    pub(super) fn new() -> std::io::Result<Self> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "server_signals_unavailable",
        ))
    }
    pub(super) fn wait(&mut self, _duration: std::time::Duration) -> bool {
        false
    }
}
