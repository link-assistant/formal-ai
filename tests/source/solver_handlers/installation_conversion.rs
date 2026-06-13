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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AlgorithmConstructionStage {
    id: &'static str,
    output: &'static str,
    verifier: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CodingSurfaceProjection {
    slug: &'static str,
    projection: &'static str,
}

const ALGORITHM_CONSTRUCTION_STAGES: &[AlgorithmConstructionStage] = &[
    AlgorithmConstructionStage {
        id: "collect_corpus",
        output: "representative problem-class examples",
        verifier: "case-study corpus preserved",
    },
    AlgorithmConstructionStage {
        id: "derive_surfaces",
        output: "source and target surface ontology",
        verifier: "source/target format detection",
    },
    AlgorithmConstructionStage {
        id: "extract_ir",
        output: "shared intermediate representation",
        verifier: "ordered command preservation fixture",
    },
    AlgorithmConstructionStage {
        id: "synthesize_operations",
        output: "recognizers, extractors, renderers, and validators",
        verifier: "round-trip surface invariants",
    },
    AlgorithmConstructionStage {
        id: "project_targets",
        output: "target-specific Markdown, shell, and PowerShell renderers",
        verifier: "per-target rendering fixture",
    },
    AlgorithmConstructionStage {
        id: "mirror_runtimes",
        output: "Rust and browser-worker projections of the same algorithm",
        verifier: "cross-runtime parity checks",
    },
    AlgorithmConstructionStage {
        id: "promote_capability",
        output: "reusable coding-task construction pattern",
        verifier: "catalog, synthesis, blueprint, and rule-synthesis compatibility",
    },
];

const CODING_SURFACE_PROJECTIONS: &[CodingSurfaceProjection] = &[
    CodingSurfaceProjection {
        slug: "coding_catalog",
        projection: "task spec -> parameterized template -> CST/compile check",
    },
    CodingSurfaceProjection {
        slug: "program_synthesis",
        projection: "semantic function tree -> source program -> sandbox tests",
    },
    CodingSurfaceProjection {
        slug: "program_blueprint",
        projection: "capability set -> blueprint recipe -> honest code projection",
    },
    CodingSurfaceProjection {
        slug: "numeric_list",
        projection: "operation/data/language IR -> generated code plus evaluated result",
    },
    CodingSurfaceProjection {
        slug: "rule_synthesis",
        projection: "operation/target binding -> candidate rule -> verification fixture",
    },
    CodingSurfaceProjection {
        slug: "installation_conversion",
        projection: "installation surfaces -> install-step IR -> target renderers",
    },
];

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
    const FUNCTION_WORDS: &[&str] = &[
        "the", "a", "an", "and", "or", "to", "with", "into", "from", "your", "you", "our", "this",
        "that", "these", "those", "then", "will", "should", "must", "please", "manually",
    ];
    tokens.iter().any(|token| {
        let word = token.trim_matches(|c: char| !c.is_alphanumeric());
        FUNCTION_WORDS.contains(&word)
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
        .map_or_else(|| parsed.synthesized_description(), String::from)
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
        let mut index = 0;
        if (program == "python" || program == "python3" || program == "py")
            && rest.first() == Some(&"-m")
        {
            if let Some(module) = rest.get(1) {
                program = module.to_lowercase();
                index = 2;
            }
        }
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

    /// Map the parsed verb/object onto an install-step action category.
    fn action(&self) -> Option<&'static str> {
        if self.is_probe {
            return Some("Verify the installation");
        }
        let mut generic_run = false;
        for argument in &self.arguments {
            match classify_verb(argument) {
                // A generic launcher verb defers to a more concrete object
                // ("npm run build" is a build, not a launch).
                Some("run") => generic_run = true,
                Some(action) => return Some(action),
                None => {}
            }
        }
        match classify_verb(&self.program) {
            Some("run") => generic_run = true,
            Some(action) => return Some(action),
            None => {}
        }
        generic_run.then_some("Start the application")
    }

    /// Fall back to a description synthesized from the program/verb so unseen
    /// but well-formed commands still read meaningfully.
    fn synthesized_description(&self) -> String {
        self.arguments.first().map_or_else(
            || format!("Run {}", self.program),
            |verb| format!("Run the {} {} step", self.program, verb),
        )
    }
}

/// Translate a single verb token into an action category. Keyed on the verb
/// itself (not the surrounding tool), so the same lexicon serves every program.
/// Returns the special marker `"run"` for generic launcher verbs so the caller
/// can prefer a more concrete object.
fn classify_verb(token: &str) -> Option<&'static str> {
    Some(match token {
        "clone" => "Clone the repository",
        "cd" | "chdir" | "pushd" => "Enter the project directory",
        "install" | "add" | "ci" | "restore" | "sync" | "bootstrap" | "vendor" | "i" => {
            "Install dependencies"
        }
        "test" | "check" | "lint" | "doctor" | "verify" | "validate" | "version" | "pytest"
        | "jest" | "mocha" | "vitest" | "tox" => "Run the verification command",
        "build" | "compile" | "configure" | "make" | "package" | "dist" | "bundle" | "cmake"
        | "gradle" | "ninja" | "msbuild" => "Build the project",
        "run" | "serve" | "start" | "up" | "exec" | "dev" | "launch" | "watch" => "run",
        _ => return None,
    })
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
    record_algorithm_construction(log);
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

fn record_algorithm_construction(log: &mut EventLog) {
    log.append(
        "algorithm_construction:meta_algorithm",
        "problem_class_to_shared_ir_to_renderers_to_verification".to_owned(),
    );
    for stage in ALGORITHM_CONSTRUCTION_STAGES {
        log.append(
            "algorithm_construction:stage",
            format!(
                "{} output={} verifier={}",
                stage.id, stage.output, stage.verifier
            ),
        );
    }
    for surface in CODING_SURFACE_PROJECTIONS {
        log.append(
            "algorithm_construction:coding_surface",
            format!("{} projection={}", surface.slug, surface.projection),
        );
    }
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
    output.push('\n');
    render_meta_algorithm(&mut output);

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
    let _ = writeln!(
        lino,
        "  meta_algorithm {}",
        lino_string("problem_class_to_shared_ir_to_renderers_to_verification")
    );
    for stage in ALGORITHM_CONSTRUCTION_STAGES {
        let _ = writeln!(lino, "  construction_stage {}", lino_string(stage.id));
        let _ = writeln!(lino, "  stage_output {}", lino_string(stage.output));
        let _ = writeln!(lino, "  stage_verifier {}", lino_string(stage.verifier));
    }
    for surface in CODING_SURFACE_PROJECTIONS {
        let _ = writeln!(lino, "  coding_surface {}", lino_string(surface.slug));
        let _ = writeln!(
            lino,
            "  surface_projection {}",
            lino_string(surface.projection)
        );
    }
    for step in &conversion.steps {
        let _ = writeln!(lino, "  step {}", lino_string(&step.id));
        let _ = writeln!(lino, "  description {}", lino_string(&step.description));
        let _ = writeln!(lino, "  command {}", lino_string(&step.command));
    }
    lino
}

fn render_meta_algorithm(output: &mut String) {
    output.push_str("Meta algorithm for constructing conversion algorithms:\n");
    for (index, stage) in ALGORITHM_CONSTRUCTION_STAGES.iter().enumerate() {
        let _ = writeln!(
            output,
            "{}. {} -> {}; verification fixture: {}.",
            index + 1,
            stage.id,
            stage.output,
            stage.verifier
        );
    }
    output.push_str("\nExisting coding solutions producible by the same meta algorithm:\n");
    for surface in CODING_SURFACE_PROJECTIONS {
        let _ = writeln!(output, "- {}: {}.", surface.slug, surface.projection);
    }
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

#[path = "../source_tests/solver_handlers/installation_conversion/tests.rs"]
mod tests;
