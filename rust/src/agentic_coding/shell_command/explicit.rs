//! Explicit command boundary shared by the shell cascade and capability router.
use super::{
    asks_for_directory_listing, governs_commands_rather_than_requesting_one, intent_shell_command,
    is_prose_word, normalize_command_word, prefixed_shell_command, seed,
    strip_balanced_outer_quotes,
};

pub(super) fn command(prompt: &str) -> Option<String> {
    let prompt = strip_balanced_outer_quotes(prompt.trim());
    if governs_commands_rather_than_requesting_one(prompt) {
        return None;
    }
    let vocabulary = seed::terminal_command_vocabulary();
    let command = prefixed_shell_command(prompt, &vocabulary)?;
    let first = normalize_command_word(command.split_whitespace().next()?);
    let semantic = if asks_for_directory_listing(prompt) {
        Some(String::from("ls"))
    } else {
        intent_shell_command(prompt, &seed::shell_intent_vocabulary())
    };
    if (semantic.is_some() || crate::solver_handlers::web_search_query_for(prompt).is_some())
        && !vocabulary.shell_tokens.contains(&first)
    {
        return None;
    }
    if let Some(semantic) = semantic {
        let same_command = semantic
            .split_whitespace()
            .next()
            .is_some_and(|word| normalize_command_word(word) == first);
        let opaque = command.chars().any(|character| {
            [
                '"', '\'', '`', '|', '&', ';', '<', '>', '$', '(', ')', '{', '}',
            ]
            .contains(&character)
        });
        if same_command && !opaque && command.split_whitespace().skip(1).any(is_prose_word) {
            return Some(semantic);
        }
    }
    super::super::shell_command_policy::shell_quotes_paired(&command).then_some(command)
}
