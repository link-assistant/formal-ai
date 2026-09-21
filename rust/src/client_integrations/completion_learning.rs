//! Cross-run learning for the software-authoring completion contract (issue #879).
//!
//! The completion gate in [`super::completion`] can already notice that a client
//! exited zero without touching the workspace, and it can retry under a
//! *different* recovery strategy. On its own that is self-healing but not
//! self-learning: every run would rediscover, from scratch, which strategy
//! actually moves a given client from "planned something" to "wrote a file".
//!
//! This module closes that loop. Each attempt is appended as one durable
//! `completion_recovery` Links Notation record carrying the client slug, the
//! observable postcondition, the strategy that was spent, and whether that
//! attempt produced a workspace effect. The next run reads those records back
//! and reorders the seed-declared strategy list so the strategies that have
//! actually worked for *this* client and *this* postcondition are tried first,
//! and the ones that have only ever failed are tried last.
//!
//! Two properties matter and are tested:
//!
//! - The ledger lives outside the caller's working tree (defect 4 of the issue).
//!   It is written under `$FORMAL_AI_STATE_DIR`, else `$XDG_STATE_HOME`, else
//!   `~/.local/state`, never under the workspace the client was invoked in.
//! - Learning only ever *reorders* a data-declared list. It cannot invent a
//!   strategy, drop one, or change the bounded attempt budget, so a corrupted or
//!   absent ledger degrades exactly to the seeded order.

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::io::Write as _;
use std::path::PathBuf;

use crate::links_format::format_lino_record;
use crate::seed::parser::parse_lino;

/// File name of the durable ledger inside the state directory.
const LEDGER_FILE: &str = "completion-recovery.lino";
/// Directory the ledger lives in, under the resolved state root.
const STATE_NAMESPACE: &str = "formal-ai";
/// Environment variable that overrides the state root outright.
const STATE_DIR_ENV: &str = "FORMAL_AI_STATE_DIR";

/// Which run an outcome belongs to: one client driving one postcondition.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct RecoveryKey {
    pub(super) client: String,
    pub(super) postcondition: String,
}

/// One recorded attempt: a strategy was spent and either did or did not leave an
/// observable effect behind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RecoveryOutcome {
    pub(super) key: RecoveryKey,
    pub(super) strategy: String,
    pub(super) effect: bool,
}

/// Running tally for one strategy under one key.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Tally {
    effective: usize,
    attempts: usize,
}

/// Resolve the durable state directory, creating it on demand.
///
/// Returns `Ok(None)` when no state root can be determined at all, so a run in a
/// stripped environment stays fully functional and merely stops learning.
fn state_directory() -> Result<Option<PathBuf>, Box<dyn Error>> {
    let root = std::env::var_os(STATE_DIR_ENV)
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("XDG_STATE_HOME").map(PathBuf::from))
        .or_else(|| {
            super::session_files::user_home_dir()
                .ok()
                .map(|home| home.join(".local").join("state"))
        });
    let Some(root) = root else {
        return Ok(None);
    };
    if root.as_os_str().is_empty() {
        return Ok(None);
    }
    let directory = root.join(STATE_NAMESPACE);
    fs::create_dir_all(&directory)?;
    Ok(Some(directory))
}

/// Durable, append-only record of which recovery strategies actually worked.
#[derive(Debug, Default)]
pub(super) struct RecoveryLedger {
    path: Option<PathBuf>,
    tallies: BTreeMap<(RecoveryKey, String), Tally>,
}

impl RecoveryLedger {
    /// Load the ledger, tolerating every failure by degrading to "no history".
    pub(super) fn load() -> Self {
        let path = state_directory()
            .ok()
            .flatten()
            .map(|directory| directory.join(LEDGER_FILE));
        let mut ledger = Self {
            path,
            tallies: BTreeMap::new(),
        };
        let Some(path) = ledger.path.clone() else {
            return ledger;
        };
        let Ok(text) = fs::read_to_string(path) else {
            return ledger;
        };
        for outcome in parse_outcomes(&text) {
            ledger.tally(&outcome);
        }
        ledger
    }

    fn tally(&mut self, outcome: &RecoveryOutcome) {
        let entry = self
            .tallies
            .entry((outcome.key.clone(), outcome.strategy.clone()))
            .or_default();
        entry.attempts += 1;
        if outcome.effect {
            entry.effective += 1;
        }
    }

    /// Order `strategies` best-first for `key`, keeping the seeded order as the
    /// tie-break so the result is deterministic and never loses a strategy.
    pub(super) fn rank(&self, key: &RecoveryKey, strategies: &[String]) -> Vec<String> {
        let mut ranked = strategies.to_vec();
        ranked.sort_by(|left, right| {
            let left_score = self.score(key, left);
            let right_score = self.score(key, right);
            right_score
                .partial_cmp(&left_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    let index = |slug: &String| strategies.iter().position(|item| item == slug);
                    index(left).cmp(&index(right))
                })
        });
        ranked
    }

    /// Observed effectiveness of one strategy: unseen strategies score neutrally
    /// so an untried option outranks one that has only ever failed but does not
    /// displace one that has worked.
    fn score(&self, key: &RecoveryKey, strategy: &str) -> f64 {
        let Some(tally) = self
            .tallies
            .get(&(key.clone(), strategy.to_owned()))
            .filter(|tally| tally.attempts > 0)
        else {
            return 0.5;
        };
        #[expect(
            clippy::cast_precision_loss,
            reason = "attempt counts stay far below the f64 integer range"
        )]
        let ratio = tally.effective as f64 / tally.attempts as f64;
        ratio
    }

    /// Append one attempt's outcome and fold it into the in-memory tallies.
    pub(super) fn record(&mut self, outcome: &RecoveryOutcome) -> Result<(), Box<dyn Error>> {
        self.tally(outcome);
        let Some(path) = self.path.clone() else {
            return Ok(());
        };
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        writeln!(file, "{}", render(outcome))?;
        Ok(())
    }

    /// Where the ledger is stored, for the machine-readable completion record.
    pub(super) fn location(&self) -> Option<String> {
        self.path.as_ref().map(|path| path.display().to_string())
    }
}

/// Render one outcome as a canonical Links Notation record.
fn render(outcome: &RecoveryOutcome) -> String {
    format_lino_record(
        "completion_recovery",
        &[
            ("record_type", "completion_recovery".to_owned()),
            ("client", outcome.key.client.clone()),
            ("postcondition", outcome.key.postcondition.clone()),
            ("strategy", outcome.strategy.clone()),
            ("effect", outcome.effect.to_string()),
        ],
    )
}

/// Read every well-formed `completion_recovery` record out of a ledger file.
fn parse_outcomes(text: &str) -> Vec<RecoveryOutcome> {
    parse_lino(text)
        .children
        .iter()
        .filter(|node| node.name == "completion_recovery")
        .filter_map(|node| {
            let client = node.find_child_value("client");
            let postcondition = node.find_child_value("postcondition");
            let strategy = node.find_child_value("strategy");
            let effect = node.find_child_value("effect");
            if client.is_empty() || postcondition.is_empty() || strategy.is_empty() {
                return None;
            }
            Some(RecoveryOutcome {
                key: RecoveryKey {
                    client: client.to_owned(),
                    postcondition: postcondition.to_owned(),
                },
                strategy: strategy.to_owned(),
                effect: effect == "true",
            })
        })
        .collect()
}
