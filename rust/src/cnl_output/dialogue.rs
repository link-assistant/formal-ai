use super::{OutputConfig, OutputDensity, OutputPolicy, OutputStandard, output_configuration};

fn directive(configuration: OutputConfig, rest: String, paraphrase: bool) -> OutputDirective {
    let density_only = configuration.density != OutputDensity::Standard;
    let reset = !density_only && configuration.standard == OutputStandard::CnlDefault;
    OutputDirective {
        config: configuration,
        rest,
        paraphrase,
        density_only,
        reset,
    }
}

fn apply(selected: OutputConfig, directive: &OutputDirective) -> OutputConfig {
    if directive.reset {
        return directive.config;
    }
    if directive.density_only {
        OutputConfig {
            density: directive.config.density,
            ..selected
        }
    } else {
        OutputConfig {
            standard: directive.config.standard,
            ..selected
        }
    }
}
use crate::{ConversationRole, ConversationTurn};

#[derive(Debug, Clone)]
pub struct OutputDirective {
    pub config: OutputConfig,
    pub rest: String,
    pub paraphrase: bool,
    pub density_only: bool,
    pub reset: bool,
}

pub fn output_directive(text: &str, policy: &OutputPolicy) -> Option<OutputDirective> {
    let trimmed = text.trim();
    if trimmed.starts_with(['`', '"', '\'', '«', '>']) {
        return None;
    }
    for language in &policy.languages {
        let words = trimmed.trim_end_matches(['.', '!', '。', '！']);
        if let Some((prefix, suffix)) = language.frequency.split_once("{count}") {
            if !suffix.is_empty()
                && words.to_lowercase().starts_with(&prefix.to_lowercase())
                && words.to_lowercase().ends_with(&suffix.to_lowercase())
            {
                let pattern = format!(
                    "(?i)^{}([0-9]+){}$",
                    regex::escape(prefix),
                    regex::escape(suffix)
                );
                let Some(captures) = regex::Regex::new(&pattern).ok()?.captures(words) else {
                    continue;
                };
                let density = format!("top-{}", &captures[1]);
                if let Ok(configuration) = output_configuration(None, Some(&density), policy) {
                    return Some(directive(configuration, String::new(), true));
                }
            }
        }
        for (cue, mode) in &language.switches {
            if trimmed
                .trim_end_matches(['.', '!', '。', '！'])
                .to_lowercase()
                == cue.to_lowercase()
            {
                return Some(directive(
                    output_configuration(Some(mode), None, policy).ok()?,
                    String::new(),
                    true,
                ));
            }
        }
        for cue in &language.directives {
            let prefix = format!("{cue} ");
            if !trimmed.to_lowercase().starts_with(&prefix.to_lowercase()) {
                continue;
            }
            let pattern = format!("(?i)^{}", regex::escape(&prefix));
            let Some(matched) = regex::Regex::new(&pattern).ok()?.find(trimmed) else {
                continue;
            };
            let tail = &trimmed[matched.end()..];
            let (name, rest) =
                tail.find(['.', '!', ';', '。', '！', '；', '\n'])
                    .map_or((tail, ""), |index| {
                        let separator = tail[index..].chars().next().expect("found character");
                        (&tail[..index], tail[index + separator.len_utf8()..].trim())
                    });
            if let Ok(configuration) = output_configuration(Some(name.trim()), None, policy) {
                return Some(directive(configuration, rest.to_owned(), false));
            }
        }
    }
    None
}

#[derive(Debug, Clone)]
pub struct OutputTurn {
    pub config: OutputConfig,
    pub prompt: String,
    pub rewrite: Option<String>,
}

pub fn output_turn(
    prompt: &str,
    history: &[ConversationTurn],
    configuration: OutputConfig,
    policy: &OutputPolicy,
) -> OutputTurn {
    let mut selected = configuration;
    for turn in history {
        if turn.role == ConversationRole::User {
            if let Some(directive) = output_directive(&turn.content, policy) {
                selected = apply(selected, &directive);
            }
        }
    }
    let directive = output_directive(prompt, policy);
    if let Some(directive) = &directive {
        selected = apply(selected, directive);
    }
    let previous = history
        .iter()
        .rev()
        .find(|turn| turn.role == ConversationRole::Assistant);
    let rewrite = directive
        .as_ref()
        .filter(|item| item.rest.is_empty())
        .and_then(|_| previous.map(|turn| turn.content.clone()));
    let prompt = directive
        .filter(|item| !item.rest.is_empty())
        .map_or_else(|| prompt.to_owned(), |item| item.rest);
    OutputTurn {
        config: selected,
        prompt,
        rewrite,
    }
}

pub fn output_source(text: &str, language: &str, policy: &OutputPolicy) -> String {
    let Some(locale) = policy.languages.iter().find(|item| item.id == language) else {
        return text.to_owned();
    };
    if locale.notice.is_empty() {
        return text.to_owned();
    }
    text.strip_prefix(&format!("{}\n\n", locale.notice))
        .unwrap_or(text)
        .to_owned()
}
