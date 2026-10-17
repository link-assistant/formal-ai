//! Shared parsing for boolean environment switches (issue #1181).
//!
//! Boolean switches used to be read ad hoc at every call site, and each site
//! picked its own accepted spellings: the `--silent` clap argument only took
//! `true`/`false` from `FORMAL_AI_SILENT` (so the conventional
//! `FORMAL_AI_SILENT=1` errored out of `formal-ai chat`), debug flags matched
//! the literal string `"1"`, and opt-out switches matched `"0"|"false"|"off"`.
//! This module gives every boolean switch one spelling table —
//! `1`/`0`/`true`/`false`/`yes`/`no`/`on`/`off`, case-insensitive, surrounding
//! whitespace tolerated — and one honest fallback: a value that is none of
//! those is *unrecognised*, never silently coerced to a default.
//!
//! Three reading helpers cover the two idioms the codebase actually uses:
//!
//! - [`flag_enabled`] — an opt-in switch (debug traces, live fetch): the flag
//!   is on only when the variable is set to a true spelling.
//! - [`flag_disabled`] — an opt-out switch on a default-on feature (chat
//!   recording, dreaming): the feature is off only when the variable is set
//!   to a false spelling. Compose at the call site as `!flag_disabled(..)`.
//! - [`bool_env_value_parser`] — the clap value parser that routes
//!   environment values of boolean `#[arg]` fields (such as `--silent`)
//!   through the same table.

pub use crate::solver_config::{flag_disabled, flag_enabled, parse_bool_env};

/// Parse a boolean `#[arg(env = ..)]` value for clap.
///
/// Used as `value_parser` on bool arguments so `FORMAL_AI_SILENT=1` — or
/// `yes`, `on`, `TRUE`, … — is accepted, while a genuinely unrecognised value
/// is reported as a clap error instead of failing the whole command.
pub fn bool_env_value_parser(value: &str) -> Result<bool, String> {
    parse_bool_env(value).ok_or_else(|| {
        // A plain literal, deliberately not a `format!` template: clap already
        // prefixes "invalid value '…' for '--silent'" around this message, so
        // the accepted-spelling list is pure CLI plumbing and stays out of the
        // R379 hardcoded-language ledger.
        String::from("expected one of: 1, 0, true, false, yes, no, on, off (case-insensitive)")
    })
}
