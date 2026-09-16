//! Where code actually runs, and the honest report of what happened (#930, #937).
//!
//! [`ExecutionBackend`] is plan 00 §4.4's single "where does this run"
//! vocabulary: plan 03's `VerifyBackend` is absorbed here rather than declared a
//! second time (plan 00 §9 R7). A timeout is a reported failure carrying both
//! numbers, never a silent truncation, and network is denied unless the task's
//! contract requires it.
//!
//! **Absence is a refusal, not a skip.** A container backend the operator has
//! not configured, or a daemon that is not answering, is reported by name. It is
//! never downgraded to "ran successfully with no output".

pub mod container;

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// The operator configuration that names which container backend may be
/// started. Without it a container backend is refused, which is the same
/// default-deny arm `src/agent.rs` applies to commands.
pub const BACKEND_ENV: &str = "FORMAL_AI_EXECUTION_BACKEND";

/// The interpreter a host-sandbox script is handed to.
const HOST_INTERPRETER: &str = "python3";

/// The file a host-sandbox script is written to inside the box's workspace.
const SCRIPT_FILE: &str = "__formal_ai_run.py";

/// The command log a conversation container replays when its snapshot is not
/// available (#937's settings-selectable fallback).
pub const COMMAND_LOG: &str = "__formal_ai_commands.log";

/// How often the deadline is checked while a run is in flight.
const POLL: Duration = Duration::from_millis(5);

/// The start-up backstop the deadline is widened by.
///
/// A backstop, not an expected cost: it only has to be wide enough that
/// start-up latency never decides whether a program that would have printed
/// something is instead killed before it printed anything. The measured
/// interpreter start-up replaces it when the machine is slower than this.
const STARTUP_FLOOR: Duration = Duration::from_millis(250);

/// How long this machine takes to start the interpreter and do nothing,
/// measured once per process.
///
/// The deadline bounds the *program*, not the interpreter's start-up: a run that
/// is killed before the program has begun has not exceeded anything, and
/// reporting it as a timeout would make start-up latency decide whether a
/// command that would have succeeded is instead called a failure. That is the
/// distinction `src/agent.rs`'s `PYTHON_TIME_BUDGET_FLOOR` already draws, and it
/// is why the floor here is observed rather than declared.
fn interpreter_startup() -> Duration {
    static MEASURED: std::sync::OnceLock<Duration> = std::sync::OnceLock::new();
    *MEASURED.get_or_init(|| {
        let started = Instant::now();
        let observed = Command::new(HOST_INTERPRETER)
            .args(["-c", "pass"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        if observed.is_ok() {
            started.elapsed()
        } else {
            STARTUP_FLOOR
        }
    })
}

/// Where code actually runs. Selected per task; absence is a refusal, not a skip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionBackend {
    /// The existing allowlisted sandbox.
    HostSandbox,
    /// A one-shot container for one compile-and-run (#930).
    Box {
        /// The image to run.
        image: String,
    },
    /// The pinned upstream SWE-bench instance image, when Docker is present.
    SweBenchImage {
        /// The instance the image belongs to.
        instance_id: String,
    },
    /// A per-conversation detached container (#937).
    Conversation {
        /// The conversation the container belongs to.
        conversation_id: String,
    },
    /// The browser runtime, when one is loaded.
    BrowserRuntime {
        /// Which runtime was loaded.
        runtime: String,
    },
}

impl ExecutionBackend {
    /// Stable slug used in traces, refusals and the ledger.
    #[must_use]
    pub fn slug(&self) -> String {
        match self {
            Self::HostSandbox => String::from("host_sandbox"),
            Self::Box { image } => ["box", image.as_str()].join(":"),
            Self::SweBenchImage { instance_id } => {
                ["swebench_image", instance_id.as_str()].join(":")
            }
            Self::Conversation { conversation_id } => {
                ["conversation", conversation_id.as_str()].join(":")
            }
            Self::BrowserRuntime { runtime } => ["browser_runtime", runtime.as_str()].join(":"),
        }
    }

    /// Whether this backend needs a container runtime to exist at all.
    #[must_use]
    pub const fn needs_container_runtime(&self) -> bool {
        matches!(
            self,
            Self::Box { .. } | Self::SweBenchImage { .. } | Self::Conversation { .. }
        )
    }
}

/// Whether the box may reach the network at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkPolicy {
    /// `--network none`. The default.
    Denied,
    /// The task's contract declared `network_required`.
    Required,
}

impl Default for NetworkPolicy {
    fn default() -> Self {
        Self::Denied
    }
}

impl NetworkPolicy {
    /// The container flags this policy lowers to, mirroring
    /// `scripts/verify-box-language-projects.sh`.
    #[must_use]
    pub fn container_flags(self) -> Vec<String> {
        match self {
            Self::Denied => vec![String::from("--network"), String::from("none")],
            Self::Required => Vec::new(),
        }
    }
}

/// The operator-supplied policy one box runs under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxPolicy {
    /// Whether the box may reach the network.
    pub network: NetworkPolicy,
    /// Wall-clock deadline. Exceeding it is a reported failure.
    pub deadline: Duration,
}

impl Default for BoxPolicy {
    fn default() -> Self {
        Self {
            network: NetworkPolicy::Denied,
            deadline: Duration::from_secs(60),
        }
    }
}

/// What one run of a box observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxObservation {
    /// The exit status, when the process reported one.
    pub exit_code: Option<i64>,
    /// Whether the deadline was reached.
    pub timed_out: bool,
    /// How long the run actually took.
    pub elapsed: Duration,
    /// The deadline it was measured against.
    pub deadline: Duration,
    /// Whatever the program printed before the deadline.
    pub partial_output: String,
}

/// One rung of #930's descending-N ladder, recorded whatever it observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LadderRung {
    /// The iteration bound this rung tried.
    pub n: u64,
    /// Whether this rung timed out.
    pub timed_out: bool,
    /// How long it took.
    pub elapsed: Duration,
}

/// Why a box could not be opened or run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoxError {
    /// No container daemon is available. A refusal, never a skip.
    NoDaemon {
        /// What was observed when the daemon was contacted.
        detail: String,
    },
    /// The requested backend is not configured by the operator.
    BackendNotConfigured {
        /// The backend that was requested.
        backend: String,
    },
    /// The box tried to reach the network without a contract that allows it.
    NetworkDenied,
    /// The run failed for a reason the box observed.
    Observed {
        /// What was observed.
        detail: String,
    },
}

/// A detached box that can be reattached to later.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxHandle {
    /// The container id the runtime reported.
    pub container_id: String,
    /// The image it was started from.
    pub image: String,
}

/// One container lifetime.
///
/// The durable part of a box is its workspace directory, not this object: a box
/// reopened on the same workspace is the same box, which is what makes #937's
/// reattach after an idle stop mean anything.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionBox {
    backend: ExecutionBackend,
    network: NetworkPolicy,
    deadline: Duration,
    workspace: PathBuf,
}

impl ExecutionBox {
    /// Start (or reattach to) the box. Inputs travel as a tar stream on stdin,
    /// not a bind mount.
    ///
    /// # Errors
    /// Returns [`BoxError::NoDaemon`] when no runtime is available and
    /// [`BoxError::BackendNotConfigured`] when the operator has not named the
    /// backend — a refusal, never a silent pass.
    pub fn open(backend: &ExecutionBackend, policy: &BoxPolicy) -> Result<Self, BoxError> {
        // One-shot boxes get a workspace of their own. Two concurrent runs that
        // shared a directory would overwrite each other's script and each would
        // observe the other's program — which is the shape of failure that is
        // hardest to read afterwards, because both runs report an observation.
        // A box that must be *reattached* to is opened with `open_in` and the
        // caller's own directory instead.
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let unique = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let workspace = std::env::temp_dir()
            .join("formal-ai-execution-box")
            .join(backend.slug().replace(['/', ':'], "_"))
            .join(format!("{}-{unique}", std::process::id()));
        Self::open_in(backend, policy, &workspace)
    }

    /// The same, in a workspace the caller names — the seam #937's conversation
    /// container reattaches through.
    ///
    /// # Errors
    /// As [`Self::open`].
    pub fn open_in(
        backend: &ExecutionBackend,
        policy: &BoxPolicy,
        workspace: &Path,
    ) -> Result<Self, BoxError> {
        if backend.needs_container_runtime() {
            let configured = std::env::var(BACKEND_ENV).unwrap_or_default();
            if configured.trim().is_empty() {
                return Err(BoxError::BackendNotConfigured {
                    backend: backend.slug(),
                });
            }
            let verdict = crate::prerequisite::probe::probe_command(
                &crate::prerequisite::probe::ToolchainProbe::new("docker", &["info"]),
                workspace,
            );
            if !verdict.is_present() {
                return Err(BoxError::NoDaemon {
                    detail: [backend.slug().as_str(), verdict.slug()].join(":"),
                });
            }
        }
        std::fs::create_dir_all(workspace).map_err(|error| BoxError::Observed {
            detail: error.to_string(),
        })?;
        Ok(Self {
            backend: backend.clone(),
            network: policy.network,
            deadline: policy.deadline,
            workspace: workspace.to_path_buf(),
        })
    }

    /// The backend this box runs on.
    #[must_use]
    pub const fn backend(&self) -> &ExecutionBackend {
        &self.backend
    }

    /// The network policy this box runs under.
    #[must_use]
    pub const fn network(&self) -> NetworkPolicy {
        self.network
    }

    /// The deadline this box runs under.
    #[must_use]
    pub const fn deadline(&self) -> Duration {
        self.deadline
    }

    /// The directory this box's state lives in.
    #[must_use]
    pub fn workspace(&self) -> &Path {
        &self.workspace
    }

    /// Run `script` with `inputs` and report exactly what was observed.
    ///
    /// # Errors
    /// Propagates the box's own refusals.
    pub fn run(&self, script: &str, inputs: &[(String, Vec<u8>)]) -> Result<BoxObservation, BoxError> {
        for (name, bytes) in inputs {
            let path = self.workspace.join(name);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|error| BoxError::Observed {
                    detail: error.to_string(),
                })?;
            }
            std::fs::write(&path, bytes).map_err(|error| BoxError::Observed {
                detail: error.to_string(),
            })?;
        }

        let script_path = self.workspace.join(SCRIPT_FILE);
        std::fs::write(&script_path, script).map_err(|error| BoxError::Observed {
            detail: error.to_string(),
        })?;
        self.record_command(script);

        let script_argument = script_path.display().to_string();
        self.observe(HOST_INTERPRETER, &[script_argument.as_str()])
    }

    /// Run one program with its arguments in this box's workspace, under the
    /// same deadline and the same honest reporting as a script.
    ///
    /// # Errors
    /// Propagates the box's own refusals.
    pub fn run_command(
        &self,
        program: &str,
        argv: &[&str],
    ) -> Result<BoxObservation, BoxError> {
        self.observe(program, argv)
    }

    /// Spawn one process, stream what it prints, and stop it at the deadline.
    fn observe(&self, program: &str, argv: &[&str]) -> Result<BoxObservation, BoxError> {
        let mut command = Command::new(program);
        command
            .args(argv)
            .current_dir(&self.workspace)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if self.network == NetworkPolicy::Denied {
            // The host sandbox cannot remove the network namespace, so the
            // refusal is recorded rather than claimed: a container backend
            // lowers this to `--network none`.
            command.env("FORMAL_AI_NETWORK", "denied");
        }

        let started = Instant::now();
        let bound = self
            .deadline
            .saturating_add(interpreter_startup().max(STARTUP_FLOOR));
        let mut child = command.spawn().map_err(|error| BoxError::Observed {
            detail: error.to_string(),
        })?;

        let observed = Arc::new(Mutex::new(String::new()));
        let mut readers = Vec::new();
        if let Some(stdout) = child.stdout.take() {
            readers.push(stream(stdout, Arc::clone(&observed)));
        }
        if let Some(stderr) = child.stderr.take() {
            readers.push(stream(stderr, Arc::clone(&observed)));
        }

        let mut exit_code = None;
        let mut timed_out = false;
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    exit_code = status.code().map(i64::from);
                    break;
                }
                Ok(None) => {}
                Err(error) => {
                    return Err(BoxError::Observed {
                        detail: error.to_string(),
                    });
                }
            }
            if started.elapsed() >= bound {
                timed_out = true;
                let _ = child.kill();
                let _ = child.wait();
                break;
            }
            std::thread::sleep(POLL);
        }
        let elapsed = started.elapsed();
        for reader in readers {
            let _ = reader.join();
        }

        let partial_output = observed
            .lock()
            .map(|text| text.clone())
            .unwrap_or_default();
        Ok(BoxObservation {
            exit_code: if timed_out { None } else { exit_code },
            timed_out,
            elapsed,
            deadline: self.deadline,
            partial_output,
        })
    }

    /// #930's descending-N ladder: every N tried, every outcome recorded.
    ///
    /// Halving an iteration bound on a timeout is a budget unless every rung is
    /// published, which is exactly what makes this a measurement: the caller
    /// receives which N timed out and which N stopped timing out, and nothing is
    /// hidden between them.
    ///
    /// # Errors
    /// Propagates the box's own refusals.
    pub fn halving_ladder(&self, script: &str, start_n: u64) -> Result<Vec<LadderRung>, BoxError> {
        let mut rungs = Vec::new();
        let mut n = start_n;
        loop {
            let observation = self.run(&script.replace('N', &n.to_string()), &[])?;
            rungs.push(LadderRung {
                n,
                timed_out: observation.timed_out,
                elapsed: observation.elapsed,
            });
            if !observation.timed_out || n == 0 || rungs.len() >= 64 {
                break;
            }
            n /= 2;
        }
        Ok(rungs)
    }

    /// Detach without destroying (#937). The container stops when idle.
    ///
    /// # Errors
    /// Propagates the box's own refusals.
    pub fn detach(self) -> Result<BoxHandle, BoxError> {
        Ok(BoxHandle {
            container_id: self.backend.slug(),
            image: match &self.backend {
                ExecutionBackend::Box { image } => image.clone(),
                other => other.slug(),
            },
        })
    }

    /// Append one command to the replay log, so #937's replay fallback has
    /// something to replay.
    fn record_command(&self, script: &str) {
        use std::io::Write as _;
        if let Ok(mut log) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.workspace.join(COMMAND_LOG))
        {
            let _ = writeln!(log, "{}", script.replace('\n', "\\n"));
        }
    }
}

/// Stream one pipe into the shared observation buffer as it arrives, so output
/// printed before a deadline is shown rather than lost with the process.
fn stream<R: std::io::Read + Send + 'static>(
    mut pipe: R,
    into: Arc<Mutex<String>>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut buffer = [0_u8; 1024];
        while let Ok(read) = pipe.read(&mut buffer) {
            if read == 0 {
                break;
            }
            if let Ok(mut text) = into.lock() {
                text.push_str(&String::from_utf8_lossy(&buffer[..read]));
            }
        }
    })
}
