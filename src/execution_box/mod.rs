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
mod conversation;
mod invocation;

use conversation::*;
pub use conversation::{
    conversation_container_name, conversation_create_invocation, conversation_exec_invocation,
};
use invocation::*;

use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// The operator configuration that names which container backend may be
/// started. Without it a container backend is refused, which is the same
/// default-deny arm `src/agent.rs` applies to commands.
pub const BACKEND_ENV: &str = "FORMAL_AI_EXECUTION_BACKEND";

/// The isolation selected for the installed `start-command` runner.
pub const START_ISOLATION_ENV: &str = "FORMAL_AI_START_ISOLATION";

/// Exact runner prefix used by the Docker Telegram image.
pub const START_RUNNER_ENV: &str = "FORMAL_AI_START_RUNNER";

/// The interpreter a host-sandbox script is handed to.
const HOST_INTERPRETER: &str = "python3";

/// The file a host-sandbox script is written to inside the box's workspace.
const SCRIPT_FILE: &str = "__formal_ai_run.py";

/// Fixed, non-user-authored shell program used to unpack an input archive and
/// execute exact argv passed after `$0`. User data never enters this string.
const CONTAINER_RUNNER: &str =
    "mkdir -p /tmp/formal-ai && tar -xzf - -C /tmp/formal-ai && cd /tmp/formal-ai && exec \"$@\"";

/// Fixed idle process for a reattachable conversation container.
const CONVERSATION_IDLE: &str = "while :; do sleep 3600; done";

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
    /// An explicitly configured `start-command` prefix. The executable and
    /// every argument are parsed once, retained as exact argv and never passed
    /// through a host shell.
    StartRunner {
        /// Runner executable (normally `$`).
        program: String,
        /// Exact prefix before the program being run.
        arguments: Vec<String>,
        /// Isolation mode observed in configuration.
        isolation: String,
    },
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
        /// Image selected by the caller for this conversation.
        image: String,
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
            Self::StartRunner { isolation, .. } => ["start_runner", isolation.as_str()].join(":"),
            Self::Box { image } => ["box", image.as_str()].join(":"),
            Self::SweBenchImage { instance_id } => {
                ["swebench_image", instance_id.as_str()].join(":")
            }
            Self::Conversation {
                conversation_id, ..
            } => ["conversation", conversation_id.as_str()].join(":"),
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

/// One hermetic process invocation. It is public so command construction can
/// be tested without a daemon and audited without executing it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessInvocation {
    /// Executable passed directly to the process API.
    pub program: String,
    /// Exact argv, with no shell interpolation.
    pub arguments: Vec<String>,
}

/// Build the disposable Docker invocation used by [`ExecutionBox`].
///
/// The archive is supplied on stdin. `program` and `arguments` follow the
/// fixed shell program as positional parameters; they are never concatenated
/// into shell source.
#[must_use]
pub fn disposable_container_invocation(
    image: &str,
    network: NetworkPolicy,
    program: &str,
    arguments: &[&str],
) -> ProcessInvocation {
    let mut argv = vec![
        String::from("run"),
        String::from("--rm"),
        String::from("-i"),
    ];
    argv.extend(network.container_flags());
    argv.extend([
        image.to_owned(),
        String::from("sh"),
        String::from("-lc"),
        String::from(CONTAINER_RUNNER),
        String::from("formal-ai"),
        program.to_owned(),
    ]);
    argv.extend(arguments.iter().map(|argument| (*argument).to_owned()));
    ProcessInvocation {
        program: String::from("docker"),
        arguments: argv,
    }
}

/// Parse execution configuration without reading process-global state.
///
/// A runner is usable only when its isolation is explicitly `docker`.
/// Supplying half of the pair is an error, never permission to fall back to
/// host execution. An explicit `host_sandbox` value is the only configuration
/// that selects the host backend.
pub fn backend_from_configuration(
    start_isolation: Option<&str>,
    start_runner: Option<&str>,
    backend: Option<&str>,
) -> Result<Option<ExecutionBackend>, BoxError> {
    let isolation = start_isolation
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let runner = start_runner
        .map(str::trim)
        .filter(|value| !value.is_empty());
    match (isolation, runner) {
        (Some(isolation), Some(runner)) => {
            if isolation != "docker" {
                return Err(BoxError::InvalidConfiguration {
                    detail: ["unsupported start isolation", isolation].join(": "),
                });
            }
            let words =
                shell_words::split(runner).map_err(|error| BoxError::InvalidConfiguration {
                    detail: error.to_string(),
                })?;
            let Some((program, arguments)) = words.split_first() else {
                return Err(BoxError::InvalidConfiguration {
                    detail: String::from("the start runner is empty"),
                });
            };
            return Ok(Some(ExecutionBackend::StartRunner {
                program: program.clone(),
                arguments: arguments.to_vec(),
                isolation: isolation.to_owned(),
            }));
        }
        (Some(_), None) | (None, Some(_)) => {
            return Err(BoxError::InvalidConfiguration {
                detail: String::from("start isolation and runner must be configured together"),
            });
        }
        (None, None) => {}
    }

    let Some(backend) = backend.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    if backend == "host_sandbox" {
        return Ok(Some(ExecutionBackend::HostSandbox));
    }
    if let Some(language) = backend
        .strip_prefix("box-language:")
        .filter(|language| !language.is_empty())
    {
        let contract = crate::box_language_projects::box_language_contract();
        if let Some(project) = contract
            .projects
            .iter()
            .find(|project| project.language == language)
        {
            return Ok(Some(ExecutionBackend::Box {
                image: format!("{}:{}", project.image, contract.image_tag),
            }));
        }
        if let Some(deferred) = contract
            .deferred
            .iter()
            .find(|project| project.language == language)
        {
            return Err(BoxError::InvalidConfiguration {
                detail: deferred.reason.clone(),
            });
        }
        return Err(BoxError::InvalidConfiguration {
            detail: ["no box-language contract for", language].join(": "),
        });
    }
    if let Some(image) = backend
        .strip_prefix("box:")
        .filter(|image| !image.is_empty())
    {
        validate_image(image)?;
        return Ok(Some(ExecutionBackend::Box {
            image: image.to_owned(),
        }));
    }
    Err(BoxError::InvalidConfiguration {
        detail: ["unknown execution backend", backend].join(": "),
    })
}

/// Read and validate the process execution configuration.
pub fn backend_from_environment() -> Result<Option<ExecutionBackend>, BoxError> {
    let isolation = std::env::var(START_ISOLATION_ENV).ok();
    let runner = std::env::var(START_RUNNER_ENV).ok();
    let backend = std::env::var(BACKEND_ENV).ok();
    backend_from_configuration(isolation.as_deref(), runner.as_deref(), backend.as_deref())
}

/// Whether the box may reach the network at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NetworkPolicy {
    /// `--network none`. The default.
    #[default]
    Denied,
    /// The task's contract declared `network_required`.
    Required,
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

/// Complete observation of a descending-N measurement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LadderReport {
    /// Every bound attempted, in execution order.
    pub rungs: Vec<LadderRung>,
    /// Whether the cumulative hard limit was reached before a runnable bound.
    pub hard_failed: bool,
    /// A durable, human-readable account of every rung.
    pub verbose_log: String,
}

/// Why a box could not be opened or run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoxError {
    /// Operator configuration was present but incomplete or unsupported.
    InvalidConfiguration {
        /// Exact validation failure.
        detail: String,
    },
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
        std::fs::create_dir_all(workspace).map_err(|error| BoxError::Observed {
            detail: error.to_string(),
        })?;
        match backend {
            ExecutionBackend::Box { image } | ExecutionBackend::Conversation { image, .. } => {
                validate_image(image)?;
            }
            _ => {}
        }
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
            if let ExecutionBackend::Conversation {
                conversation_id,
                image,
            } = backend
            {
                ensure_conversation_container(conversation_id, image, policy.network)?;
            }
        }
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

    /// Exact primary invocation used to execute the private `SCRIPT_FILE`
    /// script (`__formal_ai_run.py`) written into the workspace. Container
    /// archive transfer is a preceding observation; this is the process whose
    /// exit/output decides the program result.
    pub fn script_invocation(&self) -> Result<ProcessInvocation, BoxError> {
        let script_argument = self.workspace.join(SCRIPT_FILE).display().to_string();
        match &self.backend {
            ExecutionBackend::HostSandbox => Ok(ProcessInvocation {
                program: String::from(HOST_INTERPRETER),
                arguments: vec![script_argument],
            }),
            ExecutionBackend::StartRunner {
                program, arguments, ..
            } => Ok(prefixed_invocation(
                program,
                arguments,
                HOST_INTERPRETER,
                &["-"],
            )),
            ExecutionBackend::Box { image } => Ok(disposable_container_invocation(
                image,
                self.network,
                HOST_INTERPRETER,
                &[SCRIPT_FILE],
            )),
            ExecutionBackend::SweBenchImage { instance_id } => Ok(disposable_container_invocation(
                &swebench_image(instance_id),
                self.network,
                HOST_INTERPRETER,
                &[SCRIPT_FILE],
            )),
            ExecutionBackend::Conversation {
                conversation_id, ..
            } => Ok(conversation_exec_invocation(
                conversation_id,
                HOST_INTERPRETER,
                &[SCRIPT_FILE],
            )),
            ExecutionBackend::BrowserRuntime { runtime } => Err(BoxError::Observed {
                detail: [
                    "browser runtime is unavailable to the native executor",
                    runtime,
                ]
                .join(": "),
            }),
        }
    }

    /// Run `script` with `inputs` and report exactly what was observed.
    ///
    /// # Errors
    /// Propagates the box's own refusals.
    pub fn run(
        &self,
        script: &str,
        inputs: &[(String, Vec<u8>)],
    ) -> Result<BoxObservation, BoxError> {
        for (name, bytes) in inputs {
            let relative = Path::new(name);
            if relative.is_absolute()
                || relative.components().any(|component| {
                    matches!(
                        component,
                        std::path::Component::ParentDir
                            | std::path::Component::RootDir
                            | std::path::Component::Prefix(_)
                    )
                })
            {
                return Err(BoxError::Observed {
                    detail: ["input path escapes the execution workspace", name].join(": "),
                });
            }
            let path = checked_workspace_path(&self.workspace, relative)?;
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
        match &self.backend {
            ExecutionBackend::HostSandbox => {
                self.observe_process(HOST_INTERPRETER, &[script_argument.as_str()], None)
            }
            ExecutionBackend::StartRunner {
                program, arguments, ..
            } => {
                if !inputs.is_empty() {
                    return Err(BoxError::Observed {
                        detail: String::from(
                            "the configured start runner has no declared input-file transport",
                        ),
                    });
                }
                let invocation = prefixed_invocation(program, arguments, HOST_INTERPRETER, &["-"]);
                self.observe_invocation(&invocation, Some(script.as_bytes()))
            }
            ExecutionBackend::Box { image } => {
                self.observe_disposable_container(image, HOST_INTERPRETER, &[SCRIPT_FILE])
            }
            ExecutionBackend::SweBenchImage { instance_id } => {
                let image = swebench_image(instance_id);
                self.observe_disposable_container(&image, HOST_INTERPRETER, &[SCRIPT_FILE])
            }
            ExecutionBackend::Conversation {
                conversation_id, ..
            } => self.observe_conversation(conversation_id, HOST_INTERPRETER, &[SCRIPT_FILE]),
            ExecutionBackend::BrowserRuntime { runtime } => Err(BoxError::Observed {
                detail: [
                    "browser runtime is unavailable to the native executor",
                    runtime,
                ]
                .join(": "),
            }),
        }
    }

    /// Run one program with its arguments in this box's workspace, under the
    /// same deadline and the same honest reporting as a script.
    ///
    /// # Errors
    /// Propagates the box's own refusals.
    pub fn run_command(&self, program: &str, argv: &[&str]) -> Result<BoxObservation, BoxError> {
        match &self.backend {
            ExecutionBackend::HostSandbox => self.observe_process(program, argv, None),
            ExecutionBackend::StartRunner {
                program: runner,
                arguments,
                ..
            } => {
                let invocation = prefixed_invocation(runner, arguments, program, argv);
                self.observe_invocation(&invocation, None)
            }
            ExecutionBackend::Box { image } => {
                self.observe_disposable_container(image, program, argv)
            }
            ExecutionBackend::SweBenchImage { instance_id } => {
                let image = swebench_image(instance_id);
                self.observe_disposable_container(&image, program, argv)
            }
            ExecutionBackend::Conversation {
                conversation_id, ..
            } => self.observe_conversation(conversation_id, program, argv),
            ExecutionBackend::BrowserRuntime { runtime } => Err(BoxError::Observed {
                detail: [
                    "browser runtime is unavailable to the native executor",
                    runtime,
                ]
                .join(": "),
            }),
        }
    }

    fn observe_disposable_container(
        &self,
        image: &str,
        program: &str,
        argv: &[&str],
    ) -> Result<BoxObservation, BoxError> {
        let archive = archive_workspace(&self.workspace)?;
        let invocation = disposable_container_invocation(image, self.network, program, argv);
        self.observe_invocation(&invocation, Some(&archive))
    }

    fn observe_conversation(
        &self,
        conversation_id: &str,
        program: &str,
        argv: &[&str],
    ) -> Result<BoxObservation, BoxError> {
        let name = conversation_container_name(conversation_id);
        let archive = archive_workspace(&self.workspace)?;
        let unpack = ProcessInvocation {
            program: String::from("docker"),
            arguments: vec![
                String::from("exec"),
                String::from("-i"),
                name.clone(),
                String::from("sh"),
                String::from("-lc"),
                String::from("mkdir -p /tmp/formal-ai && tar -xzf - -C /tmp/formal-ai"),
            ],
        };
        let transferred = self.observe_invocation(&unpack, Some(&archive))?;
        if transferred.timed_out || transferred.exit_code != Some(0) {
            return Ok(transferred);
        }
        self.observe_invocation(
            &conversation_exec_invocation(conversation_id, program, argv),
            None,
        )
    }

    /// Spawn one exact process, stream what it prints, and stop it at the deadline.
    fn observe_process(
        &self,
        program: &str,
        argv: &[&str],
        stdin: Option<&[u8]>,
    ) -> Result<BoxObservation, BoxError> {
        self.observe_invocation(
            &ProcessInvocation {
                program: program.to_owned(),
                arguments: argv.iter().map(|argument| (*argument).to_owned()).collect(),
            },
            stdin,
        )
    }

    fn observe_invocation(
        &self,
        invocation: &ProcessInvocation,
        stdin: Option<&[u8]>,
    ) -> Result<BoxObservation, BoxError> {
        let mut command = Command::new(&invocation.program);
        command
            .args(&invocation.arguments)
            .current_dir(&self.workspace)
            .stdin(if stdin.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if self.network == NetworkPolicy::Denied
            && matches!(self.backend, ExecutionBackend::HostSandbox)
        {
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
        if let Some(bytes) = stdin
            && let Some(mut pipe) = child.stdin.take()
            && let Err(error) = pipe.write_all(bytes)
        {
            let _ = child.kill();
            let _ = child.wait();
            return Err(BoxError::Observed {
                detail: error.to_string(),
            });
        }

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

        let partial_output = observed.lock().map(|text| text.clone()).unwrap_or_default();
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
        self.halving_ladder_with_hard_limit(script, start_n, Duration::from_mins(10))
            .map(|report| report.rungs)
    }

    /// Run the descending-N ladder while enforcing #930's cumulative ten-minute
    /// ceiling. `script` uses the explicit `{N}` placeholder; ordinary letters
    /// `N` in identifiers or strings are never rewritten.
    ///
    /// The separate `hard_limit` parameter makes the ten-minute policy
    /// instantaneously testable without weakening the production value.
    ///
    /// # Errors
    /// Propagates the box's own refusals.
    pub fn halving_ladder_with_hard_limit(
        &self,
        script: &str,
        start_n: u64,
        hard_limit: Duration,
    ) -> Result<LadderReport, BoxError> {
        let mut rungs = Vec::new();
        let mut verbose_log = String::new();
        let started = Instant::now();
        let mut n = start_n;
        let mut hard_failed = false;
        loop {
            if started.elapsed() >= hard_limit {
                hard_failed = true;
                break;
            }
            let observation = self.run(&script.replace("{N}", &n.to_string()), &[])?;
            rungs.push(LadderRung {
                n,
                timed_out: observation.timed_out,
                elapsed: observation.elapsed,
            });
            let _ = writeln!(
                verbose_log,
                "N={n} timed_out={} elapsed_ms={} deadline_ms={} exit={:?} output={:?}",
                observation.timed_out,
                observation.elapsed.as_millis(),
                observation.deadline.as_millis(),
                observation.exit_code,
                observation.partial_output,
            );
            if !observation.timed_out || n == 0 || rungs.len() >= 64 {
                break;
            }
            n /= 2;
        }
        Ok(LadderReport {
            rungs,
            hard_failed,
            verbose_log,
        })
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
                ExecutionBackend::Conversation { image, .. } => image.clone(),
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
