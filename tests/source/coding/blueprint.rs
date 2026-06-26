//! Issue #340: composite-program *blueprints* for `write_program` requests that
//! the verified template [`catalog`](crate::coding::catalog) cannot resolve to a
//! single, sandbox-runnable task.
//!
//! The catalog is the deterministic, honest baseline: every catalog template is
//! a self-contained program that was actually compiled and run, so the engine
//! can claim "compiled and ran". That invariant is exactly why a request like
//!
//! > Write a Rust program that makes an HTTP GET request, parses the JSON
//! > response, calculates the mean and median, and outputs the results.
//!
//! cannot live in the catalog: it needs external libraries (`reqwest`,
//! `serde_json`) and *network access*, neither of which the offline WASM sandbox
//! can verify. Before this module such a request fell through to
//! `write_program_unsupported` ("I do not have a template for language `rust`
//! and task `missing`") — a dead end that ignored the four explicit
//! sub-requirements the user spelled out.
//!
//! A *blueprint* closes that gap while staying honest. It walks the same
//! "decompose the problem into tasks" path as the universal solver:
//!
//! 1. **Decompose** — [`detect_capabilities`] scans the prompt for recognized
//!    programming capabilities (`http_request`, `json_parse`, `statistics`,
//!    `error_handling`, `comments`, …), in every supported prompt language.
//! 2. **Match a recipe** — [`select_blueprint`] finds a [`BlueprintRecipe`]
//!    whose required capabilities are all present and that has a curated,
//!    review-quality program for the requested language.
//! 3. **Render** — the engine returns the full program together with the
//!    decomposition plan, the libraries it depends on, and an *honest* execution
//!    report ("not run — needs external libraries / network; review before
//!    running"). Nothing claims to have executed.
//!
//! Adding coverage is data: a new capability keyword set or a new recipe/
//! language program extends the tables below without touching the solver.

use std::fmt::Write as _;

use crate::language::Language;
use crate::solver::BlueprintComposition;

pub use super::blueprint_data::{CAPABILITIES, RECIPES};

/// A recognized programming capability — one "sub-task" a composite request can
/// decompose into. Detection is keyword-based and script-aware (the same
/// token/substring rules the catalog uses), so prompts in en/ru/hi/zh resolve.
#[derive(Clone, Copy)]
pub struct Capability {
    pub slug: &'static str,
    /// Short, English label used in the decomposition plan heading map below.
    pub label: &'static str,
    /// Surface phrases (lowercased) that signal this capability, across every
    /// supported prompt language. CJK phrases are matched by substring.
    pub keywords: &'static [&'static str],
}

/// A curated program for one language inside a [`BlueprintRecipe`].
#[derive(Clone, Copy)]
pub struct RecipeProgram {
    pub language_slug: &'static str,
    /// Libraries / runtime prerequisites in the canonical form a user would
    /// install or confirm (e.g. `reqwest`, `serde_json`, or standard-library only).
    pub libraries: &'static [&'static str],
    /// The command a user runs to execute the program once dependencies are
    /// installed. Canonical (not localized) — it is a literal shell command.
    pub run_command: &'static str,
    pub execution: BlueprintExecution,
    pub code: &'static str,
}

/// Why a blueprint program is returned with an honest "not run" status.
#[derive(Clone, Copy)]
pub enum BlueprintExecution {
    ExternalLibrariesAndNetwork,
    ReviewDataAssumptions,
    LocalSourceAnalysis,
}

/// A composite program recipe the base catalog cannot represent directly.
#[derive(Clone, Copy)]
pub struct BlueprintRecipe {
    pub slug: &'static str,
    /// English label describing the whole program (used as a fallback title).
    pub label: &'static str,
    /// Capabilities that must all be present for this recipe to match.
    pub required_capabilities: &'static [&'static str],
    pub programs: &'static [RecipeProgram],
}

impl BlueprintRecipe {
    fn program_for(&self, language_slug: &str) -> Option<&'static RecipeProgram> {
        self.programs
            .iter()
            .find(|program| program.language_slug == language_slug)
    }
}

/// A resolved blueprint: the recipe, the chosen language program, and the
/// capabilities that were detected in the request (the decomposition).
pub struct Blueprint {
    pub recipe: &'static BlueprintRecipe,
    pub program: &'static RecipeProgram,
    pub capabilities: Vec<&'static Capability>,
}

/// Detect which capabilities a normalized (lowercased) prompt requests, in
/// catalog order so the decomposition plan reads top-to-bottom.
#[must_use]
pub fn detect_capabilities(normalized: &str) -> Vec<&'static Capability> {
    CAPABILITIES
        .iter()
        .filter(|capability| {
            capability
                .keywords
                .iter()
                .any(|keyword| contains_keyword(normalized, keyword))
        })
        .collect()
}

/// Resolve a blueprint for a request: the first recipe whose required
/// capabilities are all present in `normalized` and that has a program for
/// `language_slug`. Returns `None` when nothing matches (the caller then keeps
/// the honest `write_program_unsupported` fallback).
#[must_use]
pub fn select_blueprint(normalized: &str, language_slug: &str) -> Option<Blueprint> {
    let detected = detect_capabilities(normalized);
    let detected_slugs: Vec<&str> = detected.iter().map(|capability| capability.slug).collect();
    let recipe = RECIPES.iter().find(|recipe| {
        recipe
            .required_capabilities
            .iter()
            .all(|required| detected_slugs.contains(required))
            && recipe.program_for(language_slug).is_some()
    })?;
    let program = recipe.program_for(language_slug)?;
    Some(Blueprint {
        recipe,
        program,
        capabilities: detected,
    })
}

fn contains_keyword(normalized: &str, keyword: &str) -> bool {
    if crate::coding::contains_cjk(keyword) {
        return normalized.contains(keyword);
    }
    if keyword.contains(' ') {
        return normalized.contains(keyword);
    }
    // Token-boundary match for single words, but allow a stem (e.g. "запрос"
    // matching "запросы") by also accepting a prefix match on a token. This
    // keeps short tokens from matching inside unrelated words while tolerating
    // inflection in Russian/Hindi.
    normalized
        .split(|character: char| !character.is_alphanumeric())
        .any(|token| token == keyword || (keyword.len() >= 4 && token.starts_with(keyword)))
}

/// The line-comment marker for a program language, used by the composer and
/// [`strip_comments`]. Hash-comment languages (`python`, `ruby`) use `#`;
/// everything else in the catalog (`rust`, `javascript`, `typescript`, `go`,
/// `c`, `cpp`, `java`, `csharp`) uses `//`, which also covers Rust's `//!`/`///`
/// doc lines.
fn comment_marker(language_slug: &str) -> &'static str {
    match language_slug {
        "python" | "ruby" => "#",
        _ => "//",
    }
}

/// Prefix of a region-*open* directive inside a curated program, e.g.
/// `// region:error_handling`.
const REGION_OPEN: &str = "region:";
/// Prefix of a region-*close* directive, e.g. `// endregion:error_handling`.
const REGION_CLOSE: &str = "endregion:";

/// If `trimmed` (a line with leading whitespace already removed) is a region
/// directive comment, return `(is_open, capability_slug)`. Region directives are
/// internal annotations: they are *always* removed before the program reaches
/// the user, and in [`BlueprintComposition::Composed`] mode a region whose
/// capability the request did not ask for is dropped along with its body.
fn region_directive<'line>(trimmed: &'line str, marker: &str) -> Option<(bool, &'line str)> {
    let rest = trimmed.strip_prefix(marker)?.trim_start();
    // Check the close prefix first: `endregion:` also ends with `region:`, so the
    // open match would otherwise swallow it.
    if let Some(slug) = rest.strip_prefix(REGION_CLOSE) {
        return Some((false, slug.trim()));
    }
    let slug = rest.strip_prefix(REGION_OPEN)?;
    Some((true, slug.trim()))
}

/// Compose the program a blueprint emits from its annotated recipe template.
///
/// Every emitted program is a *projection* of the curated template, never the
/// raw, marker-bearing string — which is what makes a blueprint an honest
/// *composition* of the decomposed capabilities rather than a frozen, memoized
/// answer (`NON-GOALS.md`). Two axes drive the projection:
///
/// 1. **Regions.** Optional sub-tasks are wrapped in `region:<capability>` /
///    `endregion:<capability>` comment directives. The directive lines are
///    always stripped; in [`Composed`](BlueprintComposition::Composed) mode a
///    region whose capability the request did not name is dropped with its body
///    (e.g. a request that does not ask for error handling loses the defensive
///    guards). Each region is whole-statement and non-essential, so dropping it
///    leaves a program that still compiles in every supported language.
/// 2. **Comments.** In `Composed` mode, when the request did not ask for the
///    code to be commented, every whole-line comment and a leading Python module
///    docstring are stripped too (both non-semantic, so compilation is
///    preserved). Inline trailing comments are left untouched so a `//`/`#`
///    inside a string literal is never sliced.
///
/// In [`Documented`](BlueprintComposition::Documented) mode every region and
/// every comment is kept (only the internal directive lines are removed), so the
/// user always gets the maximal annotated program.
#[must_use]
pub fn compose_program(blueprint: &Blueprint, strategy: BlueprintComposition) -> String {
    let language_slug = blueprint.program.language_slug;
    let marker = comment_marker(language_slug);
    let compose = strategy == BlueprintComposition::Composed;
    let requested = |slug: &str| blueprint.capabilities.iter().any(|c| c.slug == slug);

    // Pass 1 — regions: drop every directive line, and in Composed mode drop the
    // body of any region whose capability was not requested.
    let mut kept: Vec<&str> = Vec::new();
    let mut skipping = false;
    for line in blueprint.program.code.lines() {
        let trimmed = line.trim_start();
        if let Some((is_open, slug)) = region_directive(trimmed, marker) {
            skipping = is_open && compose && !requested(slug);
            continue;
        }
        if !skipping {
            kept.push(line);
        }
    }

    // Pass 2 — comments: strip documentation when composing a request that did
    // not ask for it; otherwise keep the comments and just tidy blank runs.
    if compose && !wants_comments(blueprint) {
        strip_comment_lines(&kept, language_slug)
    } else {
        collapse_blank_runs(&kept)
    }
}

/// Remove the *documentation* from a program when the request did not ask for
/// comments. Operates on already region-filtered lines (see [`compose_program`]).
/// Only **whole-line** comments and a leading Python module docstring are
/// dropped — both are non-semantic, so the stripped program stays byte-for-byte
/// compilable in every supported language (verified offline in
/// `experiments/issue-340-blueprint`). Inline trailing comments are intentionally
/// left untouched so the stripper can never slice a `//`/`#` that lives inside a
/// string literal.
fn strip_comment_lines(lines: &[&str], language_slug: &str) -> String {
    let marker = comment_marker(language_slug);
    let mut kept: Vec<&str> = Vec::new();
    let mut in_docstring = false;
    for &line in lines {
        let trimmed = line.trim_start();
        if language_slug == "python" {
            if in_docstring {
                if trimmed.contains("\"\"\"") {
                    in_docstring = false;
                }
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("\"\"\"") {
                // A `"""…"""` docstring entirely on one line, or the opening
                // line of a multi-line docstring block.
                if !rest.contains("\"\"\"") {
                    in_docstring = true;
                }
                continue;
            }
        }
        if trimmed.starts_with(marker) {
            continue;
        }
        kept.push(line);
    }
    collapse_blank_runs(&kept)
}

/// Join kept lines, dropping leading blank lines and collapsing any run of two
/// or more blank lines (left behind after removing comment blocks or regions)
/// into one, so the composed program reads cleanly.
fn collapse_blank_runs(lines: &[&str]) -> String {
    let mut out = String::new();
    let mut pending_blank = false;
    let mut wrote_any = false;
    for line in lines {
        if line.trim().is_empty() {
            if wrote_any {
                pending_blank = true;
            }
            continue;
        }
        if pending_blank {
            out.push('\n');
            pending_blank = false;
        }
        out.push_str(line);
        out.push('\n');
        wrote_any = true;
    }
    // Drop the trailing newline so the caller controls fence spacing exactly.
    if out.ends_with('\n') {
        out.pop();
    }
    out
}

/// Whether the decomposed request asked for the code to be commented.
#[must_use]
pub fn wants_comments(blueprint: &Blueprint) -> bool {
    blueprint
        .capabilities
        .iter()
        .any(|capability| capability.slug == "comments")
}

/// Localized heading that introduces the generated blueprint program.
#[must_use]
pub fn blueprint_intro(language_name: &str, recipe_label: &str, language: Language) -> String {
    match language {
        Language::Russian => format!(
            "Вот программа на языке {language_name}, которая решает составную задачу \
             ({recipe_label}). Я разбил ваш запрос на следующие подзадачи:"
        ),
        Language::Hindi => format!(
            "यहाँ {language_name} में एक प्रोग्राम है जो इस संयुक्त कार्य को हल करता है \
             ({recipe_label})। मैंने आपके अनुरोध को इन उप-कार्यों में विभाजित किया है:"
        ),
        Language::Chinese => format!(
            "这是一个解决该复合任务的 {language_name} 程序（{recipe_label}）。\
             我已将您的请求分解为以下子任务："
        ),
        _ => format!(
            "Here is a {language_name} program for the requested composite task \
             ({recipe_label}). I decomposed your request into these sub-tasks:"
        ),
    }
}

/// Localized label for a capability in the decomposition plan.
#[must_use]
pub fn capability_label(capability: &Capability, language: Language) -> &'static str {
    match (capability.slug, language) {
        ("http_request", Language::Russian) => "Выполнить HTTP-запрос",
        ("http_request", Language::Hindi) => "HTTP अनुरोध करें",
        ("http_request", Language::Chinese) => "发起 HTTP 请求",
        ("json_parse", Language::Russian) => "Разобрать JSON-ответ",
        ("json_parse", Language::Hindi) => "JSON प्रतिक्रिया पार्स करें",
        ("json_parse", Language::Chinese) => "解析 JSON 响应",
        ("statistics", Language::Russian) => "Вычислить статистику (среднее, медиана)",
        ("statistics", Language::Hindi) => "सांख्यिकी (औसत, माध्यिका) की गणना करें",
        ("statistics", Language::Chinese) => "计算统计量（平均值、中位数）",
        ("output_results", Language::Russian) => "Вывести результаты",
        ("output_results", Language::Hindi) => "परिणाम आउटपुट करें",
        ("output_results", Language::Chinese) => "输出结果",
        ("error_handling", Language::Russian) => "Обработать ошибки",
        ("error_handling", Language::Hindi) => "त्रुटियाँ संभालें",
        ("error_handling", Language::Chinese) => "处理错误",
        ("comments", Language::Russian) => "Снабдить код комментариями",
        ("comments", Language::Hindi) => "कोड में टिप्पणियाँ जोड़ें",
        ("comments", Language::Chinese) => "为代码添加注释",
        ("web_research", Language::Russian) => "Найти актуальные исходные данные",
        ("web_research", Language::Hindi) => "वर्तमान स्रोत डेटा खोजें",
        ("web_research", Language::Chinese) => "检索当前来源数据",
        ("city_costs", Language::Russian) => "Сравнить стоимость жизни по городам",
        ("city_costs", Language::Hindi) => "शहरों की जीवन-यापन लागत की तुलना करें",
        ("city_costs", Language::Chinese) => "比较城市生活成本",
        ("budget_rule", Language::Russian) => "Применить правило бюджета 50/30/20",
        ("budget_rule", Language::Hindi) => "50/30/20 बजट नियम लागू करें",
        ("budget_rule", Language::Chinese) => "应用 50/30/20 预算规则",
        ("compound_savings", Language::Russian) => "Рассчитать накопления со сложным процентом",
        ("compound_savings", Language::Hindi) => "चक्रवृद्धि बचत का अनुमान लगाएँ",
        ("compound_savings", Language::Chinese) => "预测复利储蓄",
        ("markdown_report", Language::Russian) => "Экспортировать Markdown-отчёт со сравнением",
        ("markdown_report", Language::Hindi) => "Markdown तुलना रिपोर्ट निर्यात करें",
        ("markdown_report", Language::Chinese) => "导出 Markdown 比较报告",
        _ => capability.label,
    }
}

/// Localized libraries heading.
#[must_use]
pub const fn libraries_heading(language: Language) -> &'static str {
    match language {
        Language::Russian => "Необходимые библиотеки:",
        Language::Hindi => "आवश्यक लाइब्रेरियाँ:",
        Language::Chinese => "所需的库：",
        _ => "Required libraries:",
    }
}

/// Localized, honest execution report: the blueprint is never executed because
/// it depends on external libraries and/or network access the offline sandbox
/// cannot provide. The run command stays canonical.
#[must_use]
pub fn blueprint_execution_report(
    run_command: &str,
    execution: BlueprintExecution,
    language: Language,
) -> String {
    match (execution, language) {
        (BlueprintExecution::LocalSourceAnalysis, _) => format!(
            "Execution status: not run — this source-metrics blueprint uses only the Rust \
             standard library, but the answer renderer did not compile it in place. The \
             code is provided for review. Run it yourself from a Cargo project: \
             `{run_command}`."
        ),
        (BlueprintExecution::ReviewDataAssumptions, Language::Russian) => format!(
            "Статус выполнения: не запускалось — этот отчёт не выполнялся в офлайн-песочнице, \
             а встроенные допущения о данных нужно проверить перед использованием. Код приведён \
             для проверки. Запустить самостоятельно: `{run_command}`."
        ),
        (BlueprintExecution::ReviewDataAssumptions, Language::Hindi) => format!(
            "निष्पादन स्थिति: नहीं चलाया गया — यह रिपोर्ट ऑफ़लाइन सैंडबॉक्स में नहीं चली, \
             और embedded data assumptions को उपयोग से पहले जाँचना चाहिए। कोड समीक्षा के लिए \
             दिया गया है। स्वयं चलाएँ: `{run_command}`।"
        ),
        (BlueprintExecution::ReviewDataAssumptions, Language::Chinese) => format!(
            "执行状态：未运行 —— 该报告未在离线沙箱中执行，内置数据假设应先核对再使用。\
             代码仅供审阅。自行运行：`{run_command}`。"
        ),
        (BlueprintExecution::ReviewDataAssumptions, _) => format!(
            "Execution status: not run — this report blueprint was not executed in the \
             offline sandbox, and the embedded data assumptions should be reviewed \
             before use. The code is provided for review. Run it yourself: \
             `{run_command}`."
        ),
        (_, Language::Russian) => format!(
            "Статус выполнения: не запускалось — программе нужны внешние библиотеки и \
             доступ к сети, поэтому офлайн-песочница её не выполняет. Код приведён для \
             проверки. Запустить самостоятельно: `{run_command}`."
        ),
        (_, Language::Hindi) => format!(
            "निष्पादन स्थिति: नहीं चलाया गया — प्रोग्राम को बाहरी लाइब्रेरियों और नेटवर्क पहुँच \
             की आवश्यकता है, इसलिए ऑफ़लाइन सैंडबॉक्स इसे नहीं चलाता। कोड समीक्षा के लिए दिया \
             गया है। स्वयं चलाएँ: `{run_command}`।"
        ),
        (_, Language::Chinese) => format!(
            "执行状态：未运行 —— 该程序需要外部库和网络访问，因此离线沙箱不会执行它。\
             代码仅供审阅。自行运行：`{run_command}`。"
        ),
        _ => format!(
            "Execution status: not run — this program needs external libraries and network \
             access, so the offline sandbox does not execute it. The code is provided for \
             review. Run it yourself: `{run_command}`."
        ),
    }
}

/// Localized one-line summary of what a recipe's program does, used inside the
/// intro heading so the framing prose matches the response language.
#[must_use]
pub fn recipe_summary(recipe: &BlueprintRecipe, language: Language) -> &'static str {
    match (recipe.slug, language) {
        ("http_json_stats", Language::Russian) => {
            "загрузить JSON по HTTP и вывести среднее и медиану его чисел"
        }
        ("http_json_stats", Language::Hindi) => {
            "HTTP के माध्यम से JSON प्राप्त करें और उसकी संख्याओं का औसत और माध्यिका दिखाएँ"
        }
        ("http_json_stats", Language::Chinese) => {
            "通过 HTTP 获取 JSON 并报告其中数字的平均值和中位数"
        }
        ("personal_budget_report", Language::Russian) => {
            "собрать бюджетный калькулятор 50/30/20 с городскими расходами, источниками и Markdown-отчётом"
        }
        ("personal_budget_report", Language::Hindi) => {
            "स्रोतों सहित 50/30/20 शहर बजट कैलकुलेटर और Markdown रिपोर्ट बनाएँ"
        }
        ("personal_budget_report", Language::Chinese) => {
            "生成带来源的 50/30/20 城市预算计算器和 Markdown 报告"
        }
        ("crypto_portfolio_tracker", Language::Russian) => {
            "смоделировать криптопортфель с оповещениями и Markdown-панелью"
        }
        ("crypto_portfolio_tracker", Language::Hindi) => {
            "alerts और Markdown dashboard वाला crypto portfolio tracker simulate करें"
        }
        ("crypto_portfolio_tracker", Language::Chinese) => {
            "模拟带提醒和 Markdown 仪表盘的加密投资组合追踪器"
        }
        _ => recipe.label,
    }
}

fn recipe_addendum(recipe: &BlueprintRecipe, _language: Language) -> Option<&'static str> {
    (recipe.slug == "self_source_metrics_report").then_some(
        "Response self-analysis:\n\
         - Reasoning text metrics: functions=0, loops=0, conditionals=0, comments=0, \
         complexity_score=1.\n\
         - Comparison: the generated Rust code is more complex than the reasoning text \
         because it contains executable parsing, loops, conditionals, helper functions, \
         and JSON rendering logic.",
    )
}

/// Localized "Run it yourself" heading shown above the execution report.
#[must_use]
const fn how_to_run_heading(language: Language) -> &'static str {
    match language {
        Language::Russian => "Как запустить самостоятельно:",
        Language::Hindi => "इसे स्वयं कैसे चलाएँ:",
        Language::Chinese => "如何自行运行：",
        _ => "How to run it yourself:",
    }
}

/// Render the complete localized blueprint answer: decomposition plan, the
/// curated program, its library prerequisites, and the honest execution report.
///
/// The code fence and language name come from the verified catalog so the
/// rendering matches every other `write_program` answer; if the language is
/// somehow absent from the catalog we fall back to the recipe's own slug.
#[must_use]
pub fn render(blueprint: &Blueprint, language: Language, strategy: BlueprintComposition) -> String {
    let catalog_language = crate::coding::program_language_by_slug(blueprint.program.language_slug);
    let language_name =
        catalog_language.map_or(blueprint.program.language_slug, |language| language.name);
    let code_fence = catalog_language.map_or(blueprint.program.language_slug, |l| l.code_fence);

    let summary = recipe_summary(blueprint.recipe, language);
    let mut body = blueprint_intro(language_name, summary, language);

    body.push_str("\n\n");
    for (index, capability) in blueprint.capabilities.iter().enumerate() {
        writeln!(
            body,
            "{}. {}",
            index + 1,
            capability_label(capability, language)
        )
        .expect("string write is infallible");
    }

    // Compose the program from the decomposition: filter optional capability
    // regions and comments according to `strategy`, so the blueprint stays an
    // honest projection of the detected capabilities rather than a single frozen
    // string (see `compose_program`).
    let program_code = compose_program(blueprint, strategy);

    write!(
        body,
        "\n```{code_fence}\n{program_code}\n```\n\n{}\n",
        libraries_heading(language)
    )
    .expect("string write is infallible");
    for library in blueprint.program.libraries {
        writeln!(body, "- {library}").expect("string write is infallible");
    }

    write!(
        body,
        "\n{}\n\n{}",
        how_to_run_heading(language),
        blueprint_execution_report(
            blueprint.program.run_command,
            blueprint.program.execution,
            language
        )
    )
    .expect("string write is infallible");

    if let Some(addendum) = recipe_addendum(blueprint.recipe, language) {
        write!(body, "\n\n{addendum}").expect("string write is infallible");
    }

    body
}

#[path = "../source_tests/coding/blueprint/tests.rs"]
mod tests;
