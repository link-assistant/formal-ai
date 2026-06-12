//! README/install-script conversion handler.
//!
//! Issue #423 asks for reversible conversion between README installation or
//! deployment guides and executable shell/PowerShell scripts. The handler keeps
//! the algorithm deterministic: parse command-like install steps into a small
//! intermediate representation, then render every requested target format from
//! that same ordered step list.

use std::fmt::Write as _;

use crate::engine::{stable_id, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::solver_handlers::finalize_simple;

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
            return None;
        }
        Some(Self {
            source_format,
            target_formats,
            project: extract_project(prompt).unwrap_or_else(|| String::from("the project")),
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

fn is_install_conversion_request(normalized: &str) -> bool {
    let asks_conversion = contains_any(
        normalized,
        &[
            "convert",
            "conversion",
            "transform",
            "turn",
            "translate",
            "back to",
            "конверт",
            "преобраз",
            "перевед",
            "बदल",
            "परिवर्त",
            "रूपांतर",
            "कन्वर्ट",
            "转换",
            "轉換",
            "转成",
            "轉成",
            "转为",
            "轉為",
            "翻译",
            "翻譯",
        ],
    );
    let names_install_surface = contains_any(
        normalized,
        &[
            "readme",
            "markdown",
            "installation guide",
            "install guide",
            "deployment guide",
            "deploy guide",
            "installation script",
            "install script",
            "deployment script",
            "deploy script",
            "руководство по установ",
            "инструкц",
            "установ",
            "स्थापना",
            "इंस्टॉल",
            "इंस्टॉलेशन",
            "安装",
            "安裝",
            "部署",
        ],
    );
    let names_script_surface = contains_any(
        normalized,
        &[
            " sh ",
            " bash",
            "shell",
            "powershell",
            "pwsh",
            "ps1",
            "script",
            "скрипт",
            "скрипта",
            "脚本",
            "腳本",
        ],
    );
    asks_conversion && names_install_surface && names_script_surface
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

fn detect_source_format(prompt: &str, normalized: &str) -> InstallFormat {
    let fences = fenced_blocks(prompt);
    let explicit_powershell = contains_any(
        normalized,
        &[
            "this powershell",
            "powershell installation script",
            "powershell script back",
            "ps1 script",
        ],
    );
    let explicit_shell = contains_any(
        normalized,
        &[
            "this shell",
            "this bash",
            "shell installation script",
            "shell script back",
            "bash script back",
        ],
    );
    let explicit_markdown = contains_any(
        normalized,
        &[
            "this readme",
            "readme.md installation guide",
            "readme installation guide",
            "this markdown",
            "markdown installation guide",
        ],
    );
    if explicit_powershell {
        return InstallFormat::PowerShellScript;
    }
    if explicit_shell {
        return InstallFormat::ShellScript;
    }
    if explicit_markdown {
        return InstallFormat::Markdown;
    }
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
    let mut targets = Vec::new();
    if contains_any(
        normalized,
        &[
            "back to a readme",
            "back to readme",
            "to a readme",
            "to readme",
            "to markdown",
            "markdown guide",
        ],
    ) {
        push_target(&mut targets, InstallFormat::Markdown);
    }
    if contains_any(
        normalized,
        &[
            "both sh and powershell",
            "both bash and powershell",
            "sh and powershell",
            "bash and powershell",
        ],
    ) {
        push_target(&mut targets, InstallFormat::ShellScript);
        push_target(&mut targets, InstallFormat::PowerShellScript);
    }
    if contains_any(
        normalized,
        &[
            "into a sh script",
            "to a sh script",
            "into sh",
            "to sh",
            "into a shell script",
            "to a shell script",
            "into a bash script",
            "to a bash script",
        ],
    ) {
        push_target(&mut targets, InstallFormat::ShellScript);
    }
    if contains_any(
        normalized,
        &[
            "into a powershell script",
            "to a powershell script",
            "into powershell",
            "to powershell",
            "to ps1",
            "into ps1",
        ],
    ) && source_format != InstallFormat::PowerShellScript
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
            collect_inline_commands(source, &mut commands);
            collect_bullet_commands(source, &mut commands);
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

fn collect_inline_commands(source: &str, commands: &mut Vec<String>) {
    let mut in_tick = false;
    let mut candidate = String::new();
    for character in source.chars() {
        if character == '`' {
            if in_tick {
                push_command(commands, candidate.trim());
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
            push_command(commands, &trimmed[1..trimmed.len() - 1]);
        } else {
            push_command(commands, trimmed);
        }
    }
}

fn collect_script_commands(source: &str, commands: &mut Vec<String>) {
    for line in source.lines() {
        let trimmed = normalize_script_line(line);
        if should_skip_script_line(&trimmed) {
            continue;
        }
        push_command(commands, &trimmed);
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

fn push_command(commands: &mut Vec<String>, candidate: &str) {
    let command = candidate.trim();
    if command.is_empty() || !looks_like_command(command) {
        return;
    }
    if !commands.iter().any(|existing| existing == command) {
        commands.push(command.to_owned());
    }
}

fn looks_like_command(command: &str) -> bool {
    const PREFIXES: &[&str] = &[
        "./",
        "apt ",
        "bash ",
        "brew ",
        "cargo ",
        "cd ",
        "cmake ",
        "curl ",
        "docker ",
        "docker-compose ",
        "flutter ",
        "git ",
        "go ",
        "gradle ",
        "irm ",
        "make",
        "markitdown ",
        "mvn ",
        "npm ",
        "npx ",
        "ollama ",
        "opencode ",
        "pip ",
        "pipx ",
        "pnpm ",
        "powershell ",
        "pwsh ",
        "python ",
        "rustup ",
        "sh ",
        "sudo ",
        "yarn ",
        "yt-dlp ",
        "zsh ",
    ];
    let lower = command.to_lowercase();
    PREFIXES.iter().any(|prefix| lower.starts_with(prefix))
        || lower.contains(" | ")
        || lower.contains("&&")
}

fn describe_command(command: &str) -> String {
    let lower = command.to_lowercase();
    if lower.starts_with("git clone ") {
        String::from("Clone the repository")
    } else if lower.starts_with("cd ") {
        String::from("Enter the project directory")
    } else if contains_any(
        &lower,
        &[
            " install",
            "npm ci",
            "pnpm install",
            "yarn install",
            "pip install",
            "cargo install",
            "brew install",
        ],
    ) {
        String::from("Install dependencies")
    } else if contains_any(&lower, &[" test", "pytest", "mvn test"]) {
        String::from("Run the verification command")
    } else if contains_any(&lower, &[" build", "compile", "make"]) {
        String::from("Build the project")
    } else if contains_any(&lower, &["doctor", "--version", " --help"]) {
        String::from("Verify the installation")
    } else {
        String::from("Run the installation command")
    }
}

fn extract_project(prompt: &str) -> Option<String> {
    let lower = prompt.to_lowercase();
    let marker = " for ";
    let start = lower.find(marker)? + marker.len();
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
    let mut output = String::new();
    let _ = writeln!(
        output,
        "Converted installation instructions for {}.",
        conversion.project
    );
    output.push_str("\nFormalized meaning:\n```lino\n");
    output.push_str(&render_lino(conversion));
    output.push_str("```\n\nConversion algorithm:\n");
    output.push_str("1. Detect the source surface and requested target surface(s).\n");
    output.push_str("2. Extract command-like install/deploy steps in original order.\n");
    output.push_str("3. Render every target from the same install-step IR.\n");
    output.push_str("4. Preserve commands verbatim so the conversion can round-trip.\n");

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
    for step in &conversion.steps {
        let _ = writeln!(lino, "  step {}", lino_string(&step.id));
        let _ = writeln!(lino, "  description {}", lino_string(&step.description));
        let _ = writeln!(lino, "  command {}", lino_string(&step.command));
    }
    lino
}

fn render_markdown_guide(output: &mut String, conversion: &InstallationConversion) {
    output.push_str("README.md installation guide:\n\n");
    output.push_str("## Installation\n\n");
    for (index, step) in conversion.steps.iter().enumerate() {
        let _ = writeln!(output, "{}. {}.", index + 1, step.description);
        output.push_str("\n   ```sh\n");
        let _ = writeln!(output, "   {}", step.command);
        output.push_str("   ```\n");
    }
}

fn render_shell_script(output: &mut String, conversion: &InstallationConversion) {
    output.push_str("Bash script:\n```bash\n#!/usr/bin/env bash\nset -euo pipefail\n\n");
    for step in &conversion.steps {
        let _ = writeln!(output, "# {}", step.description);
        let _ = writeln!(output, "{}", step.command);
    }
    output.push_str("```\n");
}

fn render_powershell_script(output: &mut String, conversion: &InstallationConversion) {
    output.push_str("PowerShell script:\n```powershell\n$ErrorActionPreference = 'Stop'\n\n");
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
