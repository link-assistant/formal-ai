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

/// Parse one boolean environment value.
///
/// Recognises `1`/`0`/`true`/`false`/`yes`/`no`/`on`/`off`, case-insensitive,
/// with surrounding whitespace tolerated. Anything else is `None` (the caller
/// decides what an unrecognised value means — usually "keep the default").
#[must_use]
pub fn parse_bool_env(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

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

/// Whether the boolean switch `name` is explicitly enabled.
///
/// `false` when the variable is unset or holds an unrecognised value, which
/// preserves the pre-#1181 behaviour of every opt-in switch: garbage never
/// turns a debug flag on.
#[must_use]
pub fn flag_enabled(name: &str) -> bool {
    std::env::var(name).as_deref().ok().and_then(parse_bool_env) == Some(true)
}

/// Whether the boolean switch `name` is explicitly disabled.
///
/// The mirror of [`flag_enabled`] for default-on features: `true` only when
/// the variable is set to a false spelling. Unset or unrecognised values
/// leave the feature on.
#[must_use]
pub fn flag_disabled(name: &str) -> bool {
    std::env::var(name).as_deref().ok().and_then(parse_bool_env) == Some(false)
}
