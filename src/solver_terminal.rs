//! Terminal-command intent detection (issue #513, visible fix for #511).
//!
//! When a prompt asks to run a shell/terminal command, the symbolic solver
//! used to fall through to the `unknown` fallback. This module recognizes the
//! shape of a terminal request (fenced/backtick command, a "run ... in
//! terminal" phrasing, or an explicit leading shell token) and returns an
//! `agent_suggestion` intent that (a) names the detected command, (b) explains
//! agent mode, and (c) offers to switch agent mode on and grant the `shell`
//! capability.
//!
//! The detection rules are intentionally mirrored in the JavaScript worker
//! (`src/web/formal_ai_worker.js`, `tryTerminalCommand`) so both engines stay
//! at parity.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::Language;
use crate::solver_handlers::finalize_simple;

/// Phrases that explicitly mention a terminal/shell/console context. These are
/// matched as substrings, so they work for both space-delimited (en/ru/hi) and
/// non-space-delimited (zh) scripts.
const TERMINAL_PHRASES: &[&str] = &[
    "в терминале",
    "в консоли",
    "в командной строке",
    "в шелле",
    "in terminal",
    "in the terminal",
    "in a terminal",
    "in console",
    "in the console",
    "in shell",
    "in the shell",
    "in a shell",
    "terminal command",
    "shell command",
    "command line",
    "command-line",
    // Hindi
    "टर्मिनल में",
    "टर्मिनल पर",
    "कमांड लाइन",
    "शेल में",
    // Chinese
    "在终端",
    "终端中",
    "终端里",
    "命令行",
    "在命令行",
    "在 shell",
    "在shell",
];

/// Verbs that request execution of a command, matched against word tokens.
/// Space-delimited scripts (en/ru/hi) tokenize cleanly; Chinese verbs are
/// matched separately as substrings via [`CJK_RUN_VERBS`].
const RUN_VERBS: &[&str] = &[
    "выполни",
    "выполнить",
    "запусти",
    "запустить",
    "run",
    "execute",
    // Hindi
    "चलाओ",
    "चलाएं",
    "चलाएँ",
    "चलाइए",
    "निष्पादित",
];

/// Chinese run/execute verbs. Chinese text has no word boundaries, so these are
/// matched as substrings of the lowercased prompt.
const CJK_RUN_VERBS: &[&str] = &["运行", "执行", "跑一下", "跑下"];

/// Common leading shell tokens that strongly imply a terminal command.
const SHELL_TOKENS: &[&str] = &[
    "ls", "pwd", "cd", "cat", "echo", "mkdir", "rmdir", "rm", "cp", "mv", "touch", "grep", "git",
    "npm", "npx", "node", "cargo", "python", "python3", "pip", "curl", "wget", "chmod", "chown",
    "df", "du", "ps", "kill", "head", "tail", "whoami", "uname", "export", "which", "sudo", "ssh",
    "tar", "find", "sed", "awk", "make", "docker", "kubectl",
];

/// Extract the first backtick-delimited span, if any (single or fenced).
fn extract_backtick_command(prompt: &str) -> Option<String> {
    let bytes: Vec<char> = prompt.chars().collect();
    let first = bytes.iter().position(|&c| c == '`')?;
    // Skip any run of backticks (handles ``` fenced blocks).
    let mut start = first;
    while start < bytes.len() && bytes[start] == '`' {
        start += 1;
    }
    let mut end = start;
    while end < bytes.len() && bytes[end] != '`' {
        end += 1;
    }
    if end <= start {
        return None;
    }
    let command: String = bytes[start..end].iter().collect();
    let command = command.trim();
    if command.is_empty() {
        None
    } else {
        Some(command.to_owned())
    }
}

/// Tokenize a lowercase prompt into alphanumeric/underscore word tokens.
fn word_tokens(lower: &str) -> Vec<String> {
    lower
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .filter(|t| !t.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

/// Return the leading shell command (the prompt itself) when it starts with a
/// recognized shell token, e.g. `ls ~` or `git status`.
fn leading_shell_command(prompt: &str) -> Option<String> {
    let trimmed = prompt.trim().trim_matches('`').trim();
    let first = trimmed.split_whitespace().next()?;
    let normalized: String = first
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>()
        .to_lowercase();
    if SHELL_TOKENS.contains(&normalized.as_str()) {
        Some(trimmed.to_owned())
    } else {
        None
    }
}

/// Detect a terminal-command request and, if found, return the command text.
fn detect_terminal_command(prompt: &str) -> Option<String> {
    let lower = prompt.to_lowercase();
    let has_phrase = TERMINAL_PHRASES.iter().any(|p| lower.contains(p));
    let tokens = word_tokens(&lower);
    let has_verb = RUN_VERBS.iter().any(|v| tokens.iter().any(|t| t == v))
        || CJK_RUN_VERBS.iter().any(|v| lower.contains(v));
    let backtick = extract_backtick_command(prompt);
    let leading = leading_shell_command(prompt);

    // Classify as a terminal request when:
    //  - a backtick command is paired with a run verb or a terminal phrase, or
    //  - a run verb is paired with an explicit terminal phrase, or
    //  - the prompt itself starts with a known shell token.
    if backtick.is_some() && (has_verb || has_phrase) {
        return backtick;
    }
    if has_phrase && has_verb {
        return backtick.or(leading);
    }
    if let Some(cmd) = leading {
        return Some(cmd);
    }
    None
}

/// Build the localized response body for a detected terminal command.
fn terminal_body(command: &str, language: Language) -> String {
    match language {
        Language::Russian => format!(
            "Похоже, вы просите выполнить команду в терминале: `{command}`.\n\n\
             Чтобы выполнять команды оболочки, нужен режим агента (Agent). \
             В режиме чата (Chat) я только рассуждаю и не запускаю команды.\n\n\
             Включить режим агента и выдать доступ к оболочке (shell), чтобы я мог \
             выполнить `{command}`? Переключите режим на «Agent» на панели инструментов \
             (или «Full Auto» для автоматического выполнения)."
        ),
        Language::Hindi => format!(
            "ऐसा लगता है कि आप टर्मिनल में एक कमांड चलाना चाहते हैं: `{command}`।\n\n\
             शेल कमांड चलाने के लिए एजेंट (Agent) मोड चाहिए। चैट (Chat) मोड में \
             मैं केवल आपके अनुरोध पर विचार करता हूँ और कमांड नहीं चलाता।\n\n\
             एजेंट मोड चालू करें और `shell` क्षमता प्रदान करें ताकि मैं `{command}` \
             चला सकूँ? टूलबार में मोड रेडियो से \"Agent\" चुनें (या स्वचालित निष्पादन \
             के लिए \"Full Auto\")।"
        ),
        Language::Chinese => format!(
            "看起来您想在终端中运行一个命令：`{command}`。\n\n\
             运行 shell 命令需要智能体（Agent）模式。在聊天（Chat）模式下，\
             我只会分析您的请求，而不会执行命令。\n\n\
             切换到智能体模式并授予 `shell` 权限，以便我运行 `{command}`？\
             请在工具栏的模式单选框中选择 \"Agent\"（或选择 \"Full Auto\" 自动运行命令）。"
        ),
        _ => format!(
            "It looks like you want to run a terminal command: `{command}`.\n\n\
             Running shell commands requires Agent mode. In Chat mode I only reason \
             about your request and do not execute commands.\n\n\
             Switch to Agent mode and grant the `shell` capability so I can run \
             `{command}`? Use the mode radio in the toolbar to pick \"Agent\" \
             (or \"Full Auto\" to run commands automatically)."
        ),
    }
}

/// Try to recognize a terminal-command request. Returns `Some` with an
/// `agent_suggestion` answer when the prompt looks like a shell command.
pub fn try_terminal_command(
    prompt: &str,
    language: Language,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let command = detect_terminal_command(prompt)?;
    log.append("terminal:command", command.clone());
    log.append("terminal:agent_suggestion", "shell".to_owned());
    let body = terminal_body(&command, language);
    Some(finalize_simple(
        prompt,
        log,
        "agent_suggestion",
        "response:agent_suggestion",
        &body,
        0.6,
    ))
}
