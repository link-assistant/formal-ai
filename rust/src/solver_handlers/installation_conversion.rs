//! README/install-script conversion handler.
//!
//! Issue #423 asks for reversible conversion between README installation or
//! deployment guides and executable shell/PowerShell scripts. The handler keeps
//! the algorithm deterministic: parse command-like install steps into a small
//! intermediate representation, then render every requested target format from
//! that same ordered step list.
//!
//! Issue #918: the request cues, the source and target markers, the prose
//! function words, the verb-to-step map and the project marker are the
//! `installation_*` tables and policy of `data/seed/handler-rules.lino`, and
//! every sentence is a seeded `installation_*` response. Only the script
//! syntax itself (fence infos, strict-mode lines, wrappers, probe flags) stays
//! here, as the browser twin in `js/worker/formal_ai_worker_software_project_plans.js` and
//! `js/worker/formal_ai_worker_installation_and_software_followups.js` keeps it too.

use std::fmt::Write as _;

use crate::engine::{SymbolicAnswer, stable_id};
use crate::event_log::EventLog;
use crate::meta_algorithm_builder::{CodingSurface, MetaAlgorithmBuilder};
use crate::rule_interpreter::{handler_policy, handler_table_rows, handler_table_value};
use crate::seed;
use crate::solver_handlers::finalize_simple;

/// The step action a generic launcher verb maps to in
/// `table installation_verb_action`: it defers to a more concrete verb.
const RUN_ACTION: &str = "run";

/// The target marker of `table installation_target_marker` that asks for both
/// scripts at once.
const BOTH_SCRIPTS: &str = "shell_and_powershell";

/// A seeded `installation_*` response with each slot filled once. The
/// conversion is a structured English record, so it renders with `en`.
fn install_text(intent: &str, values: &[(&str, &str)]) -> String {
    seed::fill_template_once(
        &seed::localized_response(intent, "en").unwrap_or_default(),
        values,
    )
}

/// The seeded `installation_step_<action>` description.
fn step_text(action: &str, values: &[(&str, &str)]) -> String {
    install_text(&format!("installation_step_{action}"), values)
}

/// Whether `value` contains a key of the seeded `table <name>`.
fn names_any(value: &str, table: &str) -> bool {
    handler_table_rows(table)
        .iter()
        .any(|(cue, _)| !cue.is_empty() && value.contains(cue.as_str()))
}

/// The values of every row of the seeded `table <name>` whose key `value`
/// contains, in seed order.
fn named_values(value: &str, table: &str) -> Vec<&'static str> {
    handler_table_rows(table)
        .iter()
        .filter(|(cue, _)| !cue.is_empty() && value.contains(cue.as_str()))
        .map(|(_, named)| named.as_str())
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InstallFormat {
    Markdown,
    ShellScript,
    PowerShellScript,
}

impl InstallFormat {
    const fn label(self) -> &'static str {
        match self {
            Self::Markdown => "markdown",
            Self::ShellScript => "shell_script",
            Self::PowerShellScript => "powershell_script",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InstallStep {
    id: String,
    description: String,
    command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InstallationConversion {
    source_format: InstallFormat,
    target_formats: Vec<InstallFormat>,
    project: String,
    steps: Vec<InstallStep>,
}

pub fn try_installation_conversion(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let conversion = InstallationConversion::from_prompt(prompt, normalized)?;
    record_conversion(log, &conversion);
    let body = render_conversion(&conversion);
    Some(finalize_simple(
        prompt,
        log,
        "installation_conversion",
        "response:installation_conversion",
        &body,
        0.84,
    ))
}

impl InstallationConversion {
    fn from_prompt(prompt: &str, normalized: &str) -> Option<Self> {
        if !is_install_conversion_request(normalized) {
            return None;
        }
        let source_format = detect_source_format(prompt, normalized);
        let target_formats = detect_target_formats(normalized, source_format);
        let source_text = extract_source_text(prompt, source_format);
        let mut steps = extract_install_steps(&source_text, source_format);
        if steps.is_empty() && source_format == InstallFormat::Markdown && source_text != prompt {
            steps = extract_install_steps(prompt, source_format);
        }
        if steps.is_empty() {
            steps = colon_payload_steps(prompt);
        }
        if steps.is_empty() {
            return None;
        }
        Some(Self {
            source_format,
            target_formats,
            project: extract_project(prompt)
                .unwrap_or_else(|| install_text("installation_default_project", &[])),
            steps,
        })
    }

    fn canonical_key(&self) -> String {
        let mut key = format!(
            "source={};project={}",
            self.source_format.label(),
            self.project
        );
        for target in &self.target_formats {
            let _ = write!(key, ";target={}", target.label());
        }
        for step in &self.steps {
            let _ = write!(key, ";command={}", step.command);
        }
        key
    }

    fn meaning_id(&self) -> String {
        stable_id("installation_conversion_request", &self.canonical_key())
    }
}

/// Whether a prompt asks for an installation or deployment surface conversion.
/// Shared with `intent_formalization` so routing and the handler use the same
/// recogniser when promoting this reading ahead of other guides (issue #932).
pub fn is_install_conversion_request(normalized: &str) -> bool {
    [
        "installation_conversion_action",
        "installation_conversion_surface",
        "installation_conversion_script",
    ]
    .iter()
    .all(|table| names_any(normalized, table))
}

fn detect_source_format(prompt: &str, normalized: &str) -> InstallFormat {
    let named = named_values(normalized, "installation_source_marker");
    // An explicit PowerShell source outranks a shell one, which outranks a
    // Markdown one, whatever order the markers occur in.
    if let Some(format) = [
        InstallFormat::PowerShellScript,
        InstallFormat::ShellScript,
        InstallFormat::Markdown,
    ]
    .into_iter()
    .find(|format| named.contains(&format.label()))
    {
        return format;
    }
    let fences = fenced_blocks(prompt);
    if fences
        .iter()
        .any(|block| is_powershell_fence(block.info.as_str()))
    {
        return InstallFormat::PowerShellScript;
    }
    if fences
        .iter()
        .any(|block| is_shell_fence(block.info.as_str()))
    {
        return InstallFormat::ShellScript;
    }
    if fences
        .iter()
        .any(|block| block.info == "markdown" || block.info == "md")
    {
        return InstallFormat::Markdown;
    }
    InstallFormat::Markdown
}

fn detect_target_formats(normalized: &str, source_format: InstallFormat) -> Vec<InstallFormat> {
    let target_markers = named_values(normalized, "installation_target_marker");
    let names = |label: &str| target_markers.contains(&label);
    let mut targets = Vec::new();
    if names(InstallFormat::Markdown.label()) {
        push_target(&mut targets, InstallFormat::Markdown);
    }
    if names(BOTH_SCRIPTS) {
        push_target(&mut targets, InstallFormat::ShellScript);
        push_target(&mut targets, InstallFormat::PowerShellScript);
    }
    if names(InstallFormat::ShellScript.label()) {
        push_target(&mut targets, InstallFormat::ShellScript);
    }
    if names(InstallFormat::PowerShellScript.label())
        && source_format != InstallFormat::PowerShellScript
    {
        push_target(&mut targets, InstallFormat::PowerShellScript);
    }
    if targets.is_empty() {
        match source_format {
            InstallFormat::Markdown => push_target(&mut targets, InstallFormat::ShellScript),
            InstallFormat::ShellScript | InstallFormat::PowerShellScript => {
                push_target(&mut targets, InstallFormat::Markdown);
            }
        }
    }
    targets
}

fn push_target(targets: &mut Vec<InstallFormat>, target: InstallFormat) {
    if !targets.contains(&target) {
        targets.push(target);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FencedBlock {
    info: String,
    body: String,
}

fn fenced_blocks(text: &str) -> Vec<FencedBlock> {
    let mut blocks = Vec::new();
    let mut current_info: Option<String> = None;
    let mut current_body = String::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("```") {
            if let Some(info) = current_info.take() {
                blocks.push(FencedBlock {
                    info,
                    body: current_body.trim_end().to_owned(),
                });
                current_body.clear();
            } else {
                current_info = Some(
                    rest.split_whitespace()
                        .next()
                        .unwrap_or_default()
                        .to_lowercase(),
                );
            }
            continue;
        }
        if current_info.is_some() {
            current_body.push_str(line);
            current_body.push('\n');
        }
    }
    blocks
}

/// The document with its fenced blocks removed, the same line walk
/// [`fenced_blocks`] performs. Fence bodies are consumed as code by
/// `collect_script_commands`; the inline and bullet collectors must see only
/// the prose around them, because the fence markers themselves toggle the
/// inline-code state machine and the block re-enters the step list whole,
/// info string embedded (issue #1138).
fn text_without_fences(text: &str) -> String {
    let mut prose = String::new();
    let mut in_fence = false;
    for line in text.lines() {
        if line.trim_start().strip_prefix("```").is_some() {
            in_fence = !in_fence;
        } else if !in_fence {
            prose.push_str(line);
            prose.push('\n');
        }
    }
    prose
}

fn extract_source_text(prompt: &str, source_format: InstallFormat) -> String {
    let fences = fenced_blocks(prompt);
    let matching = fences.iter().find(|block| match source_format {
        InstallFormat::Markdown => block.info == "markdown" || block.info == "md",
        InstallFormat::ShellScript => is_shell_fence(&block.info),
        InstallFormat::PowerShellScript => is_powershell_fence(&block.info),
    });
    if let Some(block) = matching {
        return block.body.clone();
    }
    if source_format == InstallFormat::Markdown {
        return prompt.to_owned();
    }
    if let Some(block) = fences.first() {
        return block.body.clone();
    }
    prompt.to_owned()
}

fn is_shell_fence(info: &str) -> bool {
    matches!(info, "bash" | "sh" | "shell" | "zsh")
}

fn is_powershell_fence(info: &str) -> bool {
    matches!(info, "powershell" | "pwsh" | "ps1")
}

fn extract_install_steps(source: &str, source_format: InstallFormat) -> Vec<InstallStep> {
    let mut commands = Vec::new();
    match source_format {
        InstallFormat::Markdown => {
            for block in fenced_blocks(source) {
                if is_shell_fence(&block.info) || is_powershell_fence(&block.info) {
                    collect_script_commands(&block.body, &mut commands);
                }
            }
            let prose = text_without_fences(source);
            collect_inline_commands(&prose, &mut commands);
            collect_bullet_commands(&prose, &mut commands);
        }
        InstallFormat::ShellScript | InstallFormat::PowerShellScript => {
            collect_script_commands(source, &mut commands);
        }
    }
    commands
        .into_iter()
        .enumerate()
        .map(|(index, command)| InstallStep {
            id: format!("S{}", index + 1),
            description: describe_command(&command),
            command,
        })
        .collect()
}

/// Whether the request carries installation commands to convert.
///
/// The `install_steps` claim evidence of issue #1175 R3.
#[must_use]
pub fn carries_install_steps(prompt: &str, normalized: &str) -> bool {
    let source_format = detect_source_format(prompt, normalized);
    let source_text = extract_source_text(prompt, source_format);
    !extract_install_steps(&source_text, source_format).is_empty()
        || (source_format == InstallFormat::Markdown
            && source_text != prompt
            && !extract_install_steps(prompt, source_format).is_empty())
        || !colon_payload_steps(prompt).is_empty()
}

/// The commands a one-line request states after its colon ("Turn this
/// installation guide into a bash script: git clone … && make install"): the
/// request line itself is prose, so its payload is read as the guide's text.
fn colon_payload_steps(prompt: &str) -> Vec<InstallStep> {
    prompt
        .split_once(": ")
        .map(|(_, payload)| extract_install_steps(payload, InstallFormat::Markdown))
        .unwrap_or_default()
}

fn collect_inline_commands(source: &str, commands: &mut Vec<String>) {
    let mut in_tick = false;
    let mut candidate = String::new();
    for character in source.chars() {
        if character == '`' {
            if in_tick {
                // Inline code spans are author-marked code: trust the shape.
                push_command(commands, candidate.trim(), Provenance::CodeSpan);
                candidate.clear();
                in_tick = false;
            } else {
                in_tick = true;
            }
            continue;
        }
        if in_tick {
            candidate.push(character);
        }
    }
}

fn collect_bullet_commands(source: &str, commands: &mut Vec<String>) {
    for line in source.lines() {
        let trimmed = line
            .trim()
            .trim_start_matches(|character: char| {
                character == '-' || character == '*' || character == '+' || character.is_numeric()
            })
            .trim_start_matches(['.', ')', ' ']);
        if trimmed.starts_with('`') && trimmed.ends_with('`') && trimmed.len() > 2 {
            // The whole bullet is a single code span: code provenance.
            push_command(
                commands,
                &trimmed[1..trimmed.len() - 1],
                Provenance::CodeSpan,
            );
        } else {
            // Raw document line with no code markup: prove it structurally.
            push_command(commands, trimmed, Provenance::BareLine);
        }
    }
}

fn collect_script_commands(source: &str, commands: &mut Vec<String>) {
    for line in source.lines() {
        let trimmed = normalize_script_line(line);
        if should_skip_script_line(&trimmed) {
            continue;
        }
        // Lines inside a shell/PowerShell fence are code by construction.
        push_command(commands, &trimmed, Provenance::CodeSpan);
    }
}

fn normalize_script_line(line: &str) -> String {
    line.trim()
        .trim_start_matches("$ ")
        .trim_start_matches("PS> ")
        .trim()
        .to_owned()
}

fn should_skip_script_line(line: &str) -> bool {
    line.is_empty()
        || line.starts_with("#!")
        || line.starts_with('#')
        || matches!(
            line,
            "set -e" | "set -eu" | "set -euo pipefail" | "$ErrorActionPreference = 'Stop'"
        )
}

/// Where a candidate line came from in the source document. Provenance is the
/// first structural signal the recognizer reasons about: an author who already
/// fenced or back-ticked a line told us it is code, so a weaker shape check is
/// enough; a raw prose line has to prove itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Provenance {
    /// Verbatim contents of a Markdown code span or a shell/PowerShell fence.
    CodeSpan,
    /// A raw document line (bullet text, prose) with no code markup.
    BareLine,
}

fn push_command(commands: &mut Vec<String>, candidate: &str, provenance: Provenance) {
    let command = candidate.trim();
    if command.is_empty() || !looks_like_command(command, provenance) {
        return;
    }
    if !commands.iter().any(|existing| existing == command) {
        commands.push(command.to_owned());
    }
}

/// Decide whether `command` is an install/deploy command by reasoning about its
/// structure and provenance instead of matching a fixed tool whitelist. Any
/// well-formed command line is accepted regardless of which tool it invokes,
/// while prose lines are rejected even when they mention a tool.
fn looks_like_command(command: &str, provenance: Provenance) -> bool {
    let command = command.trim();
    if command.is_empty() {
        return false;
    }

    // A raw prose line that *embeds* a code span ("Run `npm install`.") is
    // prose, not a command: the inline/fence collectors already lifted the real
    // command out of the back-ticks, so the surrounding sentence is noise.
    if provenance == Provenance::BareLine && command.contains('`') {
        return false;
    }

    let tokens: Vec<&str> = command.split_whitespace().collect();
    let head = tokens[0];

    // The leading token has to read as an executable name or path rather than an
    // English word: lowercase identifier characters, optionally a `./` or `/`
    // path. This is what separates `npm`/`yt-dlp`/`./webui.sh` from `Clone`,
    // `Установи`, or `运行`.
    if !is_executable_head(head) {
        return false;
    }

    // Shell composition (`|`, `&&`, `||`, `;`) is unambiguous command shape and
    // settles the decision regardless of provenance — `curl … | sh` is a
    // command even though `sh` alone would be skipped on a bare line.
    if has_shell_operator(command) {
        return true;
    }

    // An executable-looking head can still front a wrapped prose note
    // ("make sure you have node"); English function words betray it.
    if reads_as_prose(&tokens) {
        return false;
    }

    match provenance {
        // Already marked as code: an executable head is sufficient.
        Provenance::CodeSpan => true,
        // A raw line needs more than a lone bare word so a stray "make" or
        // "test" in prose is not promoted to a command: require an argument,
        // flag, or path.
        Provenance::BareLine => tokens.len() >= 2 || head.contains('/'),
    }
}

/// True when `token` is shaped like an executable name or a path to one, rather
/// than a natural-language word. Commands are lowercase by convention, so an
/// uppercase or non-ASCII lead immediately reads as prose.
fn is_executable_head(token: &str) -> bool {
    let mut chars = token.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_lowercase() || first.is_ascii_digit() || first == '.' || first == '/') {
        return false;
    }
    token.chars().all(|character| {
        character.is_ascii_lowercase()
            || character.is_ascii_digit()
            || matches!(character, '.' | '/' | '_' | '-' | '+')
    })
}

/// True when the line joins sub-commands with a shell composition operator.
fn has_shell_operator(command: &str) -> bool {
    command.contains(" | ")
        || command.contains("&&")
        || command.contains("||")
        || command.contains(" ; ")
}

/// True when the tokens lean on English function words that real commands omit,
/// catching lowercase prose that slipped past the executable-head check
/// ("clone the repository manually").
fn reads_as_prose(tokens: &[&str]) -> bool {
    let function_words = handler_table_rows("installation_prose_word");
    tokens.iter().any(|token| {
        let word = token.trim_matches(|c: char| !c.is_alphanumeric());
        function_words
            .iter()
            .any(|(function_word, _)| function_word == word)
    })
}

/// Derive a human-readable step description from the parsed verb/object of the
/// command rather than matching the whole string against a substring table. The
/// action is inferred from the sub-command verb (or the program itself), so an
/// unseen tool with a recognizable verb (`pdm install`, `just build`) still gets
/// an accurate description without extending a table.
fn describe_command(command: &str) -> String {
    let parsed = ParsedCommand::parse(command);
    parsed
        .action()
        .unwrap_or_else(|| parsed.synthesized_description())
}

/// Structural view of a command: the program (last path segment of the
/// executable), the ordered non-flag argument tokens (its verb then objects),
/// and whether a version/help probe flag is present.
struct ParsedCommand {
    program: String,
    arguments: Vec<String>,
    is_probe: bool,
}

impl ParsedCommand {
    fn parse(command: &str) -> Self {
        let mut tokens = command.split_whitespace().peekable();
        // Drop leading privilege/escape wrappers so the real program surfaces.
        while matches!(tokens.peek().copied(), Some("sudo" | "env" | "command")) {
            tokens.next();
        }
        let raw_program = tokens.next().unwrap_or_default();
        let mut program = raw_program
            .rsplit('/')
            .next()
            .unwrap_or(raw_program)
            .to_lowercase();

        let mut arguments = Vec::new();
        let mut is_probe = false;
        // `python -m pip install …`: the module after `-m` behaves as the
        // effective program, so fold it in.
        let rest: Vec<&str> = tokens.collect();
        let index = if (program == "python" || program == "python3" || program == "py")
            && rest.first() == Some(&"-m")
            && let Some(module) = rest.get(1)
        {
            program = module.to_lowercase();
            2
        } else {
            0
        };
        for token in &rest[index..] {
            let bare = token.trim_matches(|c: char| c == '"' || c == '\'');
            if bare == "--version"
                || bare == "-v"
                || bare == "-V"
                || bare == "--help"
                || bare == "-h"
            {
                is_probe = true;
                continue;
            }
            if bare.starts_with('-') {
                continue;
            }
            arguments.push(bare.to_lowercase());
        }
        Self {
            program,
            arguments,
            is_probe,
        }
    }

    /// Map the parsed verb/object onto an install-step description.
    fn action(&self) -> Option<String> {
        if self.is_probe {
            return Some(step_text("probe", &[]));
        }
        let mut generic_run = false;
        // The verbs first, then the program itself.
        for verb in self.arguments.iter().chain(std::iter::once(&self.program)) {
            match classify_verb(verb) {
                // A generic launcher verb defers to a more concrete object
                // ("npm run build" is a build, not a launch).
                Some(RUN_ACTION) => generic_run = true,
                Some(action) => return Some(step_text(action, &[])),
                None => {}
            }
        }
        generic_run.then(|| step_text("start", &[]))
    }

    /// Fall back to a description synthesized from the program/verb so unseen
    /// but well-formed commands still read meaningfully.
    fn synthesized_description(&self) -> String {
        self.arguments.first().map_or_else(
            || step_text("run_program", &[("program", self.program.as_str())]),
            |verb| {
                step_text(
                    "run_verb",
                    &[("program", self.program.as_str()), ("verb", verb.as_str())],
                )
            },
        )
    }
}

/// Translate a single verb token into its step action through the seeded
/// `table installation_verb_action`. Keyed on the verb itself (not the
/// surrounding tool), so the same table serves every program. Returns
/// [`RUN_ACTION`] for generic launcher verbs so the caller can prefer a more
/// concrete object.
fn classify_verb(token: &str) -> Option<&'static str> {
    handler_table_value("installation_verb_action", token)
}

fn extract_project(prompt: &str) -> Option<String> {
    let lower = prompt.to_lowercase();
    let marker = handler_policy("installation_conversion", "project-marker")?;
    let start = lower.find(marker.as_str())? + marker.len();
    let tail = &prompt[start..];
    let stop = tail
        .find(|character: char| {
            character.is_whitespace() || matches!(character, ',' | ':' | ';' | '\n')
        })
        .unwrap_or(tail.len());
    let project = tail[..stop].trim();
    if project.contains('/') || project.contains('-') {
        Some(project.to_owned())
    } else {
        None
    }
}

fn record_conversion(log: &mut EventLog, conversion: &InstallationConversion) {
    log.append("formalization", "install_steps_ir".to_owned());
    log.append("meaning", conversion.meaning_id());
    MetaAlgorithmBuilder::for_surface(CodingSurface::InstallationConversion).record(log);
    log.append(
        "installation_conversion:source_format",
        conversion.source_format.label().to_owned(),
    );
    log.append(
        "installation_conversion:project",
        conversion.project.clone(),
    );
    for target in &conversion.target_formats {
        log.append(
            "installation_conversion:target_format",
            target.label().to_owned(),
        );
    }
    for step in &conversion.steps {
        log.append(
            "installation_conversion:step",
            format!("{}:{}", step.id, step.command),
        );
    }
    log.append(
        "installation_conversion:validation",
        "ordered_commands_preserved".to_owned(),
    );
}

fn render_conversion(conversion: &InstallationConversion) -> String {
    let mut output = install_text(
        "installation_conversion_heading",
        &[("project", conversion.project.as_str())],
    );
    output.push_str("\n\n");
    output.push_str(&install_text(
        "installation_formalized_meaning_heading",
        &[],
    ));
    output.push_str("\n```lino\n");
    output.push_str(&render_lino(conversion));
    output.push_str("```\n\n");
    output.push_str(&install_text("installation_conversion_algorithm", &[]));
    output.push_str("\n\n");
    MetaAlgorithmBuilder::for_surface(CodingSurface::InstallationConversion)
        .write_explanation(&mut output);

    for target in &conversion.target_formats {
        output.push('\n');
        match target {
            InstallFormat::Markdown => render_markdown_guide(&mut output, conversion),
            InstallFormat::ShellScript => render_shell_script(&mut output, conversion),
            InstallFormat::PowerShellScript => render_powershell_script(&mut output, conversion),
        }
    }
    output.trim_end().to_owned()
}

fn render_lino(conversion: &InstallationConversion) -> String {
    let mut lino = String::from("installation_conversion_request\n");
    let _ = writeln!(lino, "  source_format {}", conversion.source_format.label());
    for target in &conversion.target_formats {
        let _ = writeln!(lino, "  target_format {}", target.label());
    }
    let _ = writeln!(lino, "  project {}", lino_string(&conversion.project));
    let _ = writeln!(
        lino,
        "  validation {}",
        lino_string("ordered_commands_preserved")
    );
    let _ = writeln!(
        lino,
        "  validation {}",
        lino_string("single_ir_renders_markdown_shell_powershell")
    );
    MetaAlgorithmBuilder::for_surface(CodingSurface::InstallationConversion).write_lino(&mut lino);
    for step in &conversion.steps {
        let _ = writeln!(lino, "  step {}", lino_string(&step.id));
        let _ = writeln!(lino, "  description {}", lino_string(&step.description));
        let _ = writeln!(lino, "  command {}", lino_string(&step.command));
    }
    lino
}

fn render_markdown_guide(output: &mut String, conversion: &InstallationConversion) {
    output.push_str(&install_text("installation_markdown_heading", &[]));
    output.push_str("\n\n");
    for (index, step) in conversion.steps.iter().enumerate() {
        let _ = writeln!(output, "{}. {}.", index + 1, step.description);
        output.push_str("\n   ```sh\n");
        let _ = writeln!(output, "   {}", step.command);
        output.push_str("   ```\n");
    }
}

fn render_shell_script(output: &mut String, conversion: &InstallationConversion) {
    output.push_str(&install_text("installation_shell_heading", &[]));
    output.push_str("\n```bash\n#!/usr/bin/env bash\nset -euo pipefail\n\n");
    for step in &conversion.steps {
        let _ = writeln!(output, "# {}", step.description);
        let _ = writeln!(output, "{}", step.command);
    }
    output.push_str("```\n");
}

fn render_powershell_script(output: &mut String, conversion: &InstallationConversion) {
    output.push_str(&install_text("installation_powershell_heading", &[]));
    output.push_str("\n```powershell\n$ErrorActionPreference = 'Stop'\n\n");
    for step in &conversion.steps {
        let _ = writeln!(output, "# {}", step.description);
        let _ = writeln!(output, "{}", step.command);
    }
    output.push_str("```\n");
}

fn lino_string(value: &str) -> String {
    let mut escaped = String::from("\"");
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            _ => escaped.push(character),
        }
    }
    escaped.push('"');
    escaped
}
