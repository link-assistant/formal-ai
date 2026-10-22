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
#[cfg(not(target_arch = "wasm32"))]
mod process;
#[cfg(not(target_arch = "wasm32"))]
use process::{kill_observed_group, observe_async};

use conversation::ensure_conversation_container;
pub use conversation::{
    conversation_container_name, conversation_create_invocation, conversation_exec_invocation,
};
use invocation::{
    archive_workspace, checked_workspace_path, prefixed_invocation, swebench_image, validate_image,
};

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
#[cfg(not(target_arch = "wasm32"))]
use std::process::{Command, Stdio};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
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
#[cfg(not(target_arch = "wasm32"))]
const POLL: Duration = Duration::from_millis(5);

/// The start-up backstop the deadline is widened by.
///
/// A backstop, not an expected cost: it only has to be wide enough that
/// start-up latency never decides whether a program that would have printed
/// something is instead killed before it printed anything. The measured
/// interpreter start-up replaces it when the machine is slower than this.
#[cfg(not(target_arch = "wasm32"))]
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
#[cfg(not(target_arch = "wasm32"))]
fn interpreter_startup() -> Duration {
    static MEASURED: std::sync::OnceLock<Duration> = std::sync::OnceLock::new();
    *MEASURED.get_or_init(|| {
        let started = Instant::now();
        let mut command = Command::new(HOST_INTERPRETER);
        command
            .args(["-c", "pass"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        let Ok(mut child) = command.spawn() else {
            return STARTUP_FLOOR;
        };
        loop {
            match child.try_wait() {
                Ok(Some(_)) => return started.elapsed(),
                Ok(None) if started.elapsed() < Duration::from_secs(1) => std::thread::sleep(POLL),
                _ => break,
            }
        }
        kill_observed_group(child.id());
        let _ = child.kill();
        let cleanup = Instant::now();
        while cleanup.elapsed() < Duration::from_millis(250) {
            if child.try_wait().ok().flatten().is_some() {
                break;
            }
            std::thread::sleep(POLL);
        }
        STARTUP_FLOOR
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
        let preparation =
            crate::bounded_process::TimeBudget::new(policy.deadline.min(Duration::from_secs(10)));
        if backend.needs_container_runtime() {
            let configured = std::env::var(BACKEND_ENV).unwrap_or_default();
            if configured.trim().is_empty() {
                return Err(BoxError::BackendNotConfigured {
                    backend: backend.slug(),
                });
            }
            let verdict = crate::prerequisite::probe::probe_command_with_timeout(
                &crate::prerequisite::probe::ToolchainProbe::new("docker", &["info"]),
                workspace,
                preparation.remaining(),
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
                ensure_conversation_container(
                    conversation_id,
                    image,
                    policy.network,
                    &preparation,
                )?;
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
        let archive =
            archive_workspace(&self.workspace, self.deadline.min(Duration::from_secs(10)))?;
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
        let archive =
            archive_workspace(&self.workspace, self.deadline.min(Duration::from_secs(10)))?;
        let unpack = ProcessInvocation {
            program: String::from("docker"),
            arguments: vec![
                String::from("exec"),
                String::from("-i"),
                name,
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
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (invocation, stdin);
            Err(BoxError::Observed {
                detail: "execution_process_runtime_unavailable".to_owned(),
            })
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
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
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                command.process_group(0);
            }
            if self.network == NetworkPolicy::Denied
                && matches!(self.backend, ExecutionBackend::HostSandbox)
            {
                command.env("FORMAL_AI_NETWORK", "denied");
            }
            let started = Instant::now();
            let bound = self
                .deadline
                .saturating_add(interpreter_startup().max(STARTUP_FLOOR));
            let deadline = self.deadline;
            let input = stdin.map(<[u8]>::to_vec);
            let process = Arc::new(std::sync::atomic::AtomicU32::new(0));
            let process_worker = Arc::clone(&process);
            let (sender, receiver) = std::sync::mpsc::sync_channel(1);
            let worker = std::thread::spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build();
                let result = match runtime {
                    Ok(runtime) => {
                        let result = runtime.block_on(observe_async(
                            command,
                            input,
                            started,
                            bound,
                            deadline,
                            &process_worker,
                        ));
                        runtime.shutdown_timeout(Duration::ZERO);
                        result
                    }
                    Err(error) => Err(BoxError::Observed {
                        detail: error.to_string(),
                    }),
                };
                let _ = sender.send(result);
            });
            let remaining = bound
                .saturating_sub(started.elapsed())
                .saturating_add(Duration::from_millis(500));
            let result = receiver.recv_timeout(remaining).unwrap_or_else(|_| {
                kill_observed_group(process.load(std::sync::atomic::Ordering::Acquire));
                Err(BoxError::Observed {
                    detail: "execution_cleanup_incomplete".to_owned(),
                })
            });
            // A worker never holds this API past its deadline through an unconditional join.
            if worker.is_finished() {
                let _ = worker.join();
            }
            result
        }
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
                ExecutionBackend::Box { image } | ExecutionBackend::Conversation { image, .. } => {
                    image.clone()
                }
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
