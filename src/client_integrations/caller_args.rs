//! Structured view of the arguments a caller passed after the tool name.
//!
//! `formal-ai with <tool> …` used to concatenate the caller's argv onto the
//! seed-declared invocation, which leaked one client's vocabulary into another
//! (`codex` receiving `-p`) and made the completion ladder rebuild a retry from
//! the prompt alone. Parsing the argv once into `prompt` plus `options` lets
//! every client re-render the same request in its own vocabulary and lets a
//! retry substitute only the prompt.

use std::collections::BTreeSet;

use crate::seed::ClientIntegration;

#[derive(Debug, Clone, Default)]
pub(super) struct CallerArgs {
    /// Caller flags (and their values) forwarded to the client verbatim. Each
    /// entry records whether it is unattributed request text rather than part
    /// of an option, so a retry can replace the text and keep the options.
    options: Vec<CallerOption>,
    /// The prompt text the caller supplied, in whichever vocabulary they used.
    prompt: Option<String>,
}

#[derive(Debug, Clone)]
struct CallerOption {
    value: String,
    text: bool,
}

impl CallerArgs {
    /// Split `user_args` into a prompt and the options to forward.
    ///
    /// `integration` supplies the target client's own mode vocabulary (its mode
    /// flags are always re-rendered, so a caller-supplied copy is dropped rather
    /// than duplicated), and `integrations` supplies the union of every declared
    /// prompt flag, so a prompt written for one client is recognised when it is
    /// handed to another.
    pub(super) fn parse(
        user_args: &[String],
        integration: &ClientIntegration,
        integrations: &[ClientIntegration],
    ) -> Self {
        let prompt_flags = prompt_flag_vocabulary(integrations);
        let value_flags = value_flag_vocabulary(integrations);
        let mode_tokens = mode_token_vocabulary(integration);
        let mut parsed = Self::default();
        // Set while the walk is inside the values of a flag no client declares:
        // `--mcp-config <path>` and `--disallowedTools Bash Edit` have to reach
        // the client — and survive a retry — as options rather than as request
        // text, even though nothing here knows their arity.
        let mut in_unknown_flag_values = false;
        let mut index = 0;
        while index < user_args.len() {
            let argument = &user_args[index];
            if let Some((flag, value)) = argument.split_once('=') {
                if prompt_flags.contains(flag) {
                    parsed.set_prompt(value);
                    in_unknown_flag_values = false;
                    index += 1;
                    continue;
                }
            }
            if prompt_flags.contains(argument.as_str()) {
                in_unknown_flag_values = false;
                match user_args.get(index + 1) {
                    Some(value) => {
                        parsed.set_prompt(value);
                        index += 2;
                    }
                    None => index += 1,
                }
                continue;
            }
            if mode_tokens.contains(argument.as_str()) {
                in_unknown_flag_values = false;
                index += 1;
                continue;
            }
            if is_flag(argument) {
                parsed.push_option(argument);
                index += 1;
                in_unknown_flag_values = !argument.contains('=');
                if in_unknown_flag_values && value_flags.contains(argument.as_str()) {
                    if let Some(value) = user_args.get(index) {
                        parsed.push_option(value);
                        index += 1;
                    }
                    in_unknown_flag_values = false;
                }
                continue;
            }
            let is_last = index + 1 == user_args.len();
            if in_unknown_flag_values {
                parsed.push_option(argument);
            } else if parsed.prompt.is_none() && is_last {
                parsed.prompt = Some(argument.clone());
            } else {
                parsed.push_text(argument);
            }
            index += 1;
        }
        parsed
    }

    fn push_option(&mut self, value: &str) {
        self.options.push(CallerOption {
            value: value.to_owned(),
            text: false,
        });
    }

    fn push_text(&mut self, value: &str) {
        self.options.push(CallerOption {
            value: value.to_owned(),
            text: true,
        });
    }

    fn set_prompt(&mut self, value: &str) {
        if self.prompt.is_none() {
            self.prompt = Some(value.to_owned());
        }
    }

    pub(super) fn options(&self) -> Vec<String> {
        self.options
            .iter()
            .map(|option| option.value.clone())
            .collect()
    }

    pub(super) fn prompt(&self) -> Option<&str> {
        self.prompt.as_deref()
    }

    /// Whether the caller supplied request text at all. A bare option value the
    /// parser could not attribute still counts, so forwarding an unrecognised
    /// `--flag text` pair keeps selecting the client's headless mode exactly as
    /// it did before the argv was parsed at all.
    pub(super) fn has_text(&self) -> bool {
        self.prompt.is_some() || self.options.iter().any(|option| option.text)
    }

    /// The request text used to decide whether this run has an observable
    /// software-authoring postcondition.
    pub(super) fn task(&self) -> String {
        self.prompt.clone().unwrap_or_else(|| {
            self.options
                .iter()
                .filter(|option| option.text)
                .map(|option| option.value.clone())
                .collect::<Vec<_>>()
                .join(" ")
        })
    }

    /// The same option set with a different prompt, used by the completion
    /// ladder so a retry keeps every caller flag the first attempt had.
    pub(super) fn with_prompt(&self, prompt: &str) -> Self {
        Self {
            options: self
                .options
                .iter()
                .filter(|option| !option.text)
                .cloned()
                .collect(),
            prompt: Some(prompt.to_owned()),
        }
    }

    pub(super) fn contains_flag(&self, flag: &str) -> bool {
        self.options.iter().any(|option| {
            option.value == flag
                || option
                    .value
                    .split_once('=')
                    .is_some_and(|(name, _)| name == flag)
        })
    }
}

fn is_flag(argument: &str) -> bool {
    argument.len() > 1 && argument.starts_with('-')
}

/// Every flag any declared client uses to carry prompt text.
fn prompt_flag_vocabulary(integrations: &[ClientIntegration]) -> BTreeSet<&str> {
    let mut flags = BTreeSet::new();
    for integration in integrations {
        let invocation = &integration.invocation;
        flags.extend(invocation.prompt_args.iter().map(String::as_str));
        if invocation.interactive_args_require_prompt {
            flags.extend(invocation.interactive_args.iter().map(String::as_str));
        }
    }
    flags
}

/// Flags declared by some client as taking a value, recognised so the value is
/// forwarded with its flag instead of being mistaken for prompt text.
fn value_flag_vocabulary(integrations: &[ClientIntegration]) -> BTreeSet<&str> {
    let mut flags = BTreeSet::new();
    for integration in integrations {
        let invocation = &integration.invocation;
        for list in [
            &invocation.args,
            &invocation.prepend_args,
            &invocation.orchestration_args,
            &invocation.vendor_orchestration_args,
            &invocation.no_summarize_args,
        ] {
            for pair in list.windows(2) {
                if is_flag(&pair[0]) && !is_flag(&pair[1]) {
                    flags.insert(pair[0].as_str());
                }
            }
        }
        for model_arg in [&invocation.model_arg, &invocation.vendor_model_arg] {
            if is_flag(model_arg) {
                flags.insert(model_arg.as_str());
            }
        }
    }
    flags
}

/// The target client's own mode tokens, which are always re-rendered from seed
/// data and therefore dropped from the caller's argv.
fn mode_token_vocabulary(integration: &ClientIntegration) -> BTreeSet<&str> {
    integration
        .invocation
        .interactive_args
        .iter()
        .chain(integration.invocation.non_interactive_args.iter())
        .map(String::as_str)
        .collect()
}
