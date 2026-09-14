//! Issue #947 (E95): bounded autonomy -- a stuck-recovery limit, full trust as an
//! opt-in, and per-command permission as the default.
//!
//! Issue #873 asks that "it should be impossible for the system to get stuck and
//! fail", and bounds that ambition in the same breath: after a configurable limit
//! -- one hour by default -- the run stops, presents its current plan, and asks
//! permission to continue. The two halves are one requirement. An unattended loop
//! with no limit does not become reliable by trying harder; it becomes a process
//! nobody can account for.
//!
//! Three decisions live here, and each defaults to handing control back:
//!
//! 1. **The limit.** [`RecoveryLoop::step`] answers [`LoopStep::Continue`] until
//!    the injected clock says the limit is spent, then answers
//!    [`LoopStep::AskPermission`] carrying the plan as it stands. It keeps
//!    answering that until the operator grants more time, so a caller that
//!    ignores the answer loops on the question rather than on the work.
//! 2. **The mode.** [`AutonomyMode::PerCommandPermission`] is the default, so a
//!    command is gated unless the operator asked for a full-autonomous run.
//! 3. **Full trust.** With several viable resolutions, the loop asks -- unless
//!    full trust is opted into explicitly, in which case
//!    [`weigh`] picks by weighted advantages and disadvantages and records why.
//!
//! The clock is injected rather than read from [`std::time::Instant`] because a
//! one-hour limit is otherwise a one-hour test. [`ManualClock`] advances by hand,
//! so the pathological "stuck" scenario issue #947 asks for runs in microseconds
//! and still exercises the same arithmetic the default hour does.
//!
//! The module renders no prose: a caller that needs to explain a halt looks the
//! wording up by [`PermissionRequest::slug`] (R379).

use std::cell::Cell;
use std::env;
use std::time::{Duration, Instant};

/// The default stuck-recovery limit issue #873 names: one hour.
pub const DEFAULT_STUCK_RECOVERY_LIMIT: Duration = Duration::from_hours(1);

/// Environment variable selecting the autonomy mode.
pub const AUTONOMY_MODE_VARIABLE: &str = "FORMAL_AI_AUTONOMY";

/// Value of [`AUTONOMY_MODE_VARIABLE`] that selects a full-autonomous run.
pub const FULL_AUTONOMOUS_VALUE: &str = "full";

/// Environment variable carrying the full-trust opt-in.
pub const FULL_TRUST_VARIABLE: &str = "FORMAL_AI_FULL_TRUST";

/// Value [`FULL_TRUST_VARIABLE`] must hold for the opt-in to count.
pub const FULL_TRUST_VALUE: &str = "1";

/// Environment variable overriding the stuck-recovery limit, in seconds.
pub const STUCK_RECOVERY_LIMIT_VARIABLE: &str = "FORMAL_AI_STUCK_RECOVERY_SECONDS";

/// A monotonic source of elapsed time, injected so a one-hour limit is not a
/// one-hour test.
pub trait Clock {
    /// Time elapsed since the loop this clock was handed to began.
    fn elapsed(&self) -> Duration;
}

/// The real clock: elapsed time since construction.
#[derive(Debug, Clone)]
pub struct SystemClock {
    start: Instant,
}

impl SystemClock {
    /// Start the clock now.
    #[must_use]
    pub fn started() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::started()
    }
}

impl Clock for SystemClock {
    fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

/// A clock that only moves when a test moves it.
///
/// It is deliberately not [`Clone`]: a copied clock would advance on its own,
/// and a test that advanced the copy while the loop read the original would sit
/// at zero forever while appearing to tick. Hand the loop `&clock` instead --
/// [`Clock`] is implemented for references, so one clock serves both.
#[derive(Debug, Default)]
pub struct ManualClock {
    elapsed: Cell<Duration>,
}

impl ManualClock {
    /// A clock reading zero.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Move the clock forward by `step`.
    pub fn advance(&self, step: Duration) {
        self.elapsed.set(self.elapsed.get().saturating_add(step));
    }
}

impl Clock for ManualClock {
    fn elapsed(&self) -> Duration {
        self.elapsed.get()
    }
}

impl<C: Clock + ?Sized> Clock for &C {
    fn elapsed(&self) -> Duration {
        (**self).elapsed()
    }
}

/// How much of a run the operator delegated.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AutonomyMode {
    /// Every command is gated on the operator's permission. The default,
    /// because delegating a whole run is a decision the operator makes rather
    /// than one they fall into.
    #[default]
    PerCommandPermission,
    /// Commands run without a per-command gate, still under the stuck-recovery
    /// limit.
    FullAutonomous,
}

impl AutonomyMode {
    /// Stable identifier a caller logs or looks a wording up by.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::PerCommandPermission => "autonomy_per_command_permission",
            Self::FullAutonomous => "autonomy_full_autonomous",
        }
    }
}

/// The configured bounds of one unattended run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutonomyPolicy {
    /// Whether commands are gated one by one.
    pub mode: AutonomyMode,
    /// Whether the run may choose between viable options on its own.
    pub full_trust: bool,
    /// How long the run may work before it must present its plan and ask.
    pub stuck_recovery_limit: Duration,
}

impl Default for AutonomyPolicy {
    /// The safe default: every command gated, no auto-selection, one hour.
    fn default() -> Self {
        Self {
            mode: AutonomyMode::PerCommandPermission,
            full_trust: false,
            stuck_recovery_limit: DEFAULT_STUCK_RECOVERY_LIMIT,
        }
    }
}

impl AutonomyPolicy {
    /// Read the policy from the environment.
    ///
    /// Every field falls back to its default, so an unset environment is the
    /// gated one. A limit that fails to parse, or that reads zero, is ignored
    /// rather than obeyed: a zero limit would halt the run on its first step,
    /// which is a typo's worth of damage this need not accept.
    #[must_use]
    pub fn from_env() -> Self {
        let mut policy = Self::default();
        if env::var(AUTONOMY_MODE_VARIABLE).as_deref() == Ok(FULL_AUTONOMOUS_VALUE) {
            policy.mode = AutonomyMode::FullAutonomous;
        }
        policy.full_trust = env::var(FULL_TRUST_VARIABLE).as_deref() == Ok(FULL_TRUST_VALUE);
        if let Some(seconds) = env::var(STUCK_RECOVERY_LIMIT_VARIABLE)
            .ok()
            .and_then(|value| value.trim().parse::<u64>().ok())
            .filter(|seconds| *seconds > 0)
        {
            policy.stuck_recovery_limit = Duration::from_secs(seconds);
        }
        policy
    }

    /// Whether `command` may run without asking first.
    #[must_use]
    pub const fn permits_unattended(&self) -> bool {
        matches!(self.mode, AutonomyMode::FullAutonomous)
    }
}

/// Why the run stopped to ask.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionRequest {
    /// The stuck-recovery limit is spent; the plan is presented for a decision.
    StuckRecoveryLimitReached,
    /// The run is in per-command mode and the next command needs approval.
    CommandNotYetApproved,
    /// Several resolutions are viable and full trust was not opted into.
    ChoiceNotDelegated,
}

impl PermissionRequest {
    /// Stable identifier a caller logs or looks a wording up by.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::StuckRecoveryLimitReached => "autonomy_stuck_recovery_limit_reached",
            Self::CommandNotYetApproved => "autonomy_command_not_yet_approved",
            Self::ChoiceNotDelegated => "autonomy_choice_not_delegated",
        }
    }
}

/// What the loop should do next.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoopStep {
    /// Keep working: the next step may run.
    Continue,
    /// Stop and ask, presenting the plan as it stands.
    AskPermission {
        /// Why the run stopped.
        request: PermissionRequest,
        /// The plan presented with the question, newest step last.
        plan: Vec<String>,
        /// Time spent when the run stopped.
        elapsed: Duration,
        /// The limit that was in force.
        limit: Duration,
    },
}

impl LoopStep {
    /// Whether this step lets the run proceed.
    #[must_use]
    pub const fn proceeds(&self) -> bool {
        matches!(self, Self::Continue)
    }
}

/// The bounded recovery loop: it works until the limit is spent, then asks.
///
/// The loop never reports failure and never runs forever, which is exactly the
/// pair issue #873 asks for. Being stuck is not an error state here; it is a
/// question addressed to the operator, carrying the plan that got this far.
#[derive(Debug, Clone)]
pub struct RecoveryLoop<C: Clock> {
    policy: AutonomyPolicy,
    clock: C,
    plan: Vec<String>,
    granted: Duration,
    approvals: usize,
    questions: usize,
}

impl<C: Clock> RecoveryLoop<C> {
    /// Start a loop under `policy`, timed by `clock`.
    #[must_use]
    pub const fn new(policy: AutonomyPolicy, clock: C) -> Self {
        Self {
            policy,
            clock,
            plan: Vec::new(),
            granted: Duration::ZERO,
            approvals: 0,
            questions: 0,
        }
    }

    /// The policy this loop runs under.
    #[must_use]
    pub const fn policy(&self) -> &AutonomyPolicy {
        &self.policy
    }

    /// The plan as it stands, newest step last.
    #[must_use]
    pub fn plan(&self) -> &[String] {
        &self.plan
    }

    /// How many times the loop has stopped to ask.
    #[must_use]
    pub const fn questions(&self) -> usize {
        self.questions
    }

    /// Record a planned or attempted step.
    pub fn record(&mut self, step: impl Into<String>) {
        self.plan.push(step.into());
    }

    /// The deadline currently in force: the configured limit plus every
    /// extension the operator has granted.
    #[must_use]
    pub const fn deadline(&self) -> Duration {
        self.policy
            .stuck_recovery_limit
            .saturating_add(self.granted)
    }

    /// Decide whether the next step may run.
    ///
    /// Once the deadline passes this keeps answering [`LoopStep::AskPermission`]
    /// -- a caller that ignores the answer loops on the question, not on the
    /// work.
    pub fn step(&mut self) -> LoopStep {
        let elapsed = self.clock.elapsed();
        let limit = self.deadline();
        if elapsed >= limit {
            self.questions += 1;
            return LoopStep::AskPermission {
                request: PermissionRequest::StuckRecoveryLimitReached,
                plan: self.plan.clone(),
                elapsed,
                limit,
            };
        }
        LoopStep::Continue
    }

    /// Decide whether `command` may run, honouring both the mode and the limit.
    ///
    /// The limit is checked first: a run that has spent its hour stops for the
    /// hour, not for a per-command approval it would then have to ask for again.
    pub fn step_command(&mut self, command: impl Into<String>) -> LoopStep {
        let command = command.into();
        let bounded = self.step();
        if !bounded.proceeds() {
            return bounded;
        }
        if self.policy.permits_unattended() {
            self.record(command);
            return LoopStep::Continue;
        }
        self.questions += 1;
        let mut plan = self.plan.clone();
        plan.push(command);
        LoopStep::AskPermission {
            request: PermissionRequest::CommandNotYetApproved,
            plan,
            elapsed: self.clock.elapsed(),
            limit: self.deadline(),
        }
    }

    /// The operator approved the pending command; record it and proceed.
    pub fn approve(&mut self, command: impl Into<String>) {
        self.approvals += 1;
        self.record(command);
    }

    /// How many commands the operator has approved one by one.
    #[must_use]
    pub const fn approvals(&self) -> usize {
        self.approvals
    }

    /// The operator granted `extension` more time; the loop resumes.
    pub const fn grant(&mut self, extension: Duration) {
        self.granted = self.granted.saturating_add(extension);
    }

    /// Choose between viable resolutions.
    ///
    /// Under full trust the heaviest net option is taken and its score recorded;
    /// otherwise the choice goes back to the operator, who is in the driving
    /// seat by default.
    pub fn resolve<'options>(
        &mut self,
        options: &'options [ResolutionOption],
    ) -> Resolution<'options> {
        if !self.policy.full_trust {
            self.questions += 1;
            return Resolution::Ask(PermissionRequest::ChoiceNotDelegated);
        }
        weigh(options).map_or(
            Resolution::Ask(PermissionRequest::ChoiceNotDelegated),
            Resolution::Chose,
        )
    }
}

/// One viable way out of a stuck state, with the weights that argue for and
/// against it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionOption {
    /// Stable identifier of the option; never prose.
    pub id: String,
    /// Weighted advantages, each a slug and a weight.
    pub advantages: Vec<(String, u32)>,
    /// Weighted disadvantages, each a slug and a weight.
    pub disadvantages: Vec<(String, u32)>,
}

impl ResolutionOption {
    /// Construct an option from its slug and weighted arguments.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        advantages: Vec<(String, u32)>,
        disadvantages: Vec<(String, u32)>,
    ) -> Self {
        Self {
            id: id.into(),
            advantages,
            disadvantages,
        }
    }

    /// Advantages minus disadvantages. Signed, because an option can argue
    /// against itself on balance.
    #[must_use]
    pub fn net_weight(&self) -> i64 {
        let sum = |arguments: &[(String, u32)]| -> i64 {
            arguments.iter().map(|(_, weight)| i64::from(*weight)).sum()
        };
        sum(&self.advantages) - sum(&self.disadvantages)
    }
}

/// The option full trust selected, with the arithmetic that selected it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeighedChoice<'options> {
    /// The chosen option.
    pub option: &'options ResolutionOption,
    /// Its net weight.
    pub net_weight: i64,
    /// The runner-up's net weight, when there was one.
    pub runner_up: Option<i64>,
}

/// What the loop decided about a set of viable options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution<'options> {
    /// Full trust selected an option.
    Chose(WeighedChoice<'options>),
    /// The choice goes back to the operator.
    Ask(PermissionRequest),
}

/// Pick the option with the heaviest net weight.
///
/// Ties and empty sets both yield `None`: an auto-selection that cannot say why
/// it preferred one option is the operator's decision, not the run's. That is the
/// same doctrine [`crate::promotion::PromotionProposal::passes_all_gates`]
/// applies to a proposal with no gates -- positive evidence, or no.
#[must_use]
pub fn weigh(options: &[ResolutionOption]) -> Option<WeighedChoice<'_>> {
    let mut ranked: Vec<&ResolutionOption> = options.iter().collect();
    ranked.sort_by(|left, right| {
        right
            .net_weight()
            .cmp(&left.net_weight())
            .then_with(|| left.id.cmp(&right.id))
    });
    let best = ranked.first()?;
    let runner_up = ranked.get(1).map(|option| option.net_weight());
    if runner_up == Some(best.net_weight()) {
        return None;
    }
    Some(WeighedChoice {
        option: best,
        net_weight: best.net_weight(),
        runner_up,
    })
}
