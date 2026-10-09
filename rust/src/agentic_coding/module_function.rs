//! Add a synthesized function to an existing source module, a test of it to
//! its test module, and run the stated command (PR #1188 dogfooding, T1):
//!
//! ```text
//! Add a function multiply(a, b) to math.mjs that returns a times b, add a
//! test for it to math.test.mjs, and run node --test to confirm it passes.
//! ```
//!
//! The native twin of `js/agentic/module_function.mjs`, function by function.
//! Nothing here knows the task. The module's language is the one the seeded
//! extension table of `data/seed/page-formalization-rules.lino` names; the
//! test's expected value is computed from the specification itself -- the
//! clause after the seeded return action, its parameters bound to the
//! contract's sample arguments, or the seeded arithmetic relation it names --
//! evaluated by the calculator, never from the synthesized code. The edits and
//! the command run through the execution-recipe reroute, so the module and the
//! test are written whole after they were read, and the request's own command
//! is the verification.
//!
//! One step differs from the browser root: the JavaScript arm synthesizes the
//! function with the shared solver's browser composer, which the native
//! solver does not have. The native arm searches the seeded binary operations
//! instead (`synthesized_source`): the one whose idiom meets the
//! specification at every sample pair of the contract is lowered through the
//! language's IR lowering (`coding/ir_lowering`).

use super::final_result::FinalResult;
use super::planner::{AgenticPlan, Capability};
use crate::coding::fragment_catalog::FragmentCatalog;
use crate::coding::program_ir::{IrNode, IrType, ProgramIr, ReuseMode};
use crate::engine::{ExecutionRecipe, ExecutionRecipeFile, SymbolicAnswer};
use crate::protocol::ChatMessage;
use crate::seed::{self, parser::LinoNode, parser::parse_lino};

const CONTRACTS: &str = include_str!("../../embedded/data/meta/function-test-contracts.lino");
const EXTENSIONS: &str = include_str!("../../embedded/data/seed/page-formalization-rules.lino");

/// The marks that end a command's last word: sentence and clause marks in
/// any script. Mirrors `COMMAND_ENDS` in `js/agentic/module_function.mjs`.
const COMMAND_ENDS: [char; 11] = [
    ',', ';', '.', '!', '?', '\u{ff0c}', '\u{ff1b}', '\u{3002}', '\u{ff01}', '\u{ff1f}', '\u{0964}',
];

/// The ideographic and Devanagari marks that end a clause wherever they sit.
const SCRIPT_CLAUSE_MARKS: [char; 6] = [
    '\u{ff0c}', '\u{ff1b}', '\u{3002}', '\u{ff01}', '\u{ff1f}', '\u{0964}',
];

/// A module-function request: the signature, the module it goes in, the test
/// module (when the request names a test), the module's language, the
/// specification clause and the stated command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleFunctionRequest {
    pub name: String,
    pub parameters: Vec<String>,
    pub at: usize,
    pub module: String,
    pub test: Option<String>,
    pub language: String,
    pub clause: String,
    pub command: Option<String>,
}

/// `name(a, b)` and the byte offset it starts at.
pub(super) struct Signature {
    pub(super) name: String,
    pub(super) parameters: Vec<String>,
    pub(super) at: usize,
}

/// The import-line slot the imported names fill.
const NAMES_SLOT: &str = concat!("{", "names", "}");

/// `template` with each `{slot}` replaced. Mirrors `fill`.
fn fill(template: &str, slots: &[(&str, &str)]) -> String {
    slots
        .iter()
        .fold(template.to_owned(), |text, (slot, value)| {
            text.replace(&format!("{{{slot}}}"), value)
        })
}

/// A word without its edge punctuation, lowercased. Mirrors `bare`.
fn bare(word: &str) -> String {
    word.trim_matches(|character: char| !character.is_alphanumeric())
        .to_lowercase()
}

/// The function-test contract of `language`. Mirrors `contract`.
pub(super) fn contract(language: &str) -> Option<LinoNode> {
    let root = parse_lino(CONTRACTS);
    root.children
        .first()?
        .children
        .iter()
        .find(|node| node.name == "language" && node.id == language)
        .cloned()
}

/// The language the seeded extension table names for `path`. Mirrors
/// `extensionLanguage`.
pub(super) fn extension_language(path: &str) -> Option<String> {
    let root = parse_lino(EXTENSIONS);
    root.children
        .first()?
        .children
        .iter()
        .filter(|node| node.name == "extension")
        .find(|node| {
            let suffix = node.find_child_value("suffix");
            !suffix.is_empty() && path.ends_with(suffix)
        })
        .map(|node| node.find_child_value("language").to_owned())
}

/// The request without its quoted segments. Mirrors `outsideQuotes`.
fn outside_quotes(request: &str) -> String {
    let mut outside = String::new();
    let mut cursor = 0;
    for segment in crate::normal_markov::quoted_segment_spans(request) {
        if segment.start < cursor {
            continue;
        }
        outside.push_str(&request[cursor..segment.start]);
        outside.push(' ');
        cursor = segment.end;
    }
    outside.push_str(&request[cursor..]);
    outside
}

/// The request's clauses: split at `,` `;` and sentence ends (in any script)
/// outside parentheses. Mirrors `clauses`.
fn clauses(request: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth: usize = 0;
    let mut start = 0;
    let characters: Vec<(usize, char)> = request.char_indices().collect();
    for (position, (index, character)) in characters.iter().copied().enumerate() {
        if character == '(' {
            depth += 1;
        } else if character == ')' {
            depth = depth.saturating_sub(1);
        } else if depth == 0
            && (SCRIPT_CLAUSE_MARKS.contains(&character)
                || (matches!(character, ',' | ';' | '.' | '!' | '?')
                    && characters
                        .get(position + 1)
                        .is_none_or(|(_, next)| next.is_whitespace())))
        {
            out.push(request[start..index].to_owned());
            start = index + character.len_utf8();
        }
    }
    out.push(request[start..].to_owned());
    out.into_iter()
        .map(|clause| clause.trim().to_owned())
        .filter(|clause| !clause.is_empty())
        .collect()
}

/// Whether `text` is an identifier of the signature grammar (`[A-Za-z_$][\w$]*`).
fn is_identifier(text: &str) -> bool {
    let mut characters = text.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_' || first == '$')
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '$'
        })
}

/// `name(a, b)`: the first call-shaped signature whose parameters are
/// identifiers. Mirrors `signature`.
pub(super) fn signature(text: &str) -> Option<Signature> {
    for (open, _) in text.match_indices('(') {
        let Some(close) = text[open + 1..]
            .find([')', '('])
            .map(|offset| open + 1 + offset)
        else {
            continue;
        };
        if !text[close..].starts_with(')') {
            continue;
        }
        let head = text[..open].trim_end();
        let name_start = head
            .char_indices()
            .rev()
            .take_while(|(_, character)| {
                character.is_ascii_alphanumeric() || *character == '_' || *character == '$'
            })
            .last()
            .map(|(index, _)| index);
        // The name starts at its first letter, `_` or `$`, as the regular
        // expression's leftmost match does ("2both(" names `both`).
        let Some(name_start) = name_start.and_then(|start| {
            head[start..]
                .char_indices()
                .find(|(_, character)| {
                    character.is_ascii_alphabetic() || *character == '_' || *character == '$'
                })
                .map(|(offset, _)| start + offset)
        }) else {
            continue;
        };
        let name = &head[name_start..];
        if !is_identifier(name) {
            continue;
        }
        let parameters: Vec<String> = text[open + 1..close]
            .split(',')
            .map(|part| part.trim().to_owned())
            .collect();
        if parameters.iter().all(|parameter| is_identifier(parameter)) {
            return Some(Signature {
                name: name.to_owned(),
                parameters,
                at: name_start,
            });
        }
    }
    None
}

/// The command the request's last seeded run verb names, headed by a seeded
/// shell token: the words after the verb, or, in a verb-final clause, the
/// words from the shell token up to the verb. Mirrors `statedCommand`.
fn stated_command(request: &str) -> Option<String> {
    let vocabulary = seed::terminal_command_vocabulary();
    let heads: Vec<&String> = vocabulary
        .shell_tokens
        .iter()
        .chain(vocabulary.bare_shell_tokens.iter())
        .collect();
    let lexicon = seed::lexicon();
    let mut stops = lexicon.words_for_role(seed::ROLE_STATEMENT_FUNCTION_WORD);
    stops.extend(lexicon.words_for_role(seed::ROLE_SKILL_PROCEDURE_CLAUSE_SEPARATOR));
    let words: Vec<&str> = request.split_whitespace().collect();
    let runs = |word: &str| {
        let plain = bare(word);
        vocabulary
            .run_verbs
            .contains(&super::shell_command_policy::normalize_command_word(word))
            || vocabulary.run_verbs.contains(&plain)
            || vocabulary
                .cjk_run_verbs
                .iter()
                .any(|verb| plain.ends_with(verb.as_str()))
    };
    let verb = words.iter().rposition(|word| runs(word))?;
    let command_from = |span: &[&str]| -> Option<String> {
        let mut command: Vec<&str> = Vec::new();
        for word in span {
            if stops.contains(&bare(word)) {
                break;
            }
            let ends = word.ends_with(COMMAND_ENDS);
            command.push(word.trim_end_matches(COMMAND_ENDS));
            if ends {
                break;
            }
        }
        command
            .first()
            .is_some_and(|head| heads.iter().any(|known| known.as_str() == *head))
            .then(|| command.join(" "))
    };
    if let Some(after) = command_from(&words[verb + 1..]) {
        return Some(after);
    }
    let head = words[..verb]
        .iter()
        .rposition(|word| heads.iter().any(|known| known.as_str() == *word))?;
    command_from(&words[head..verb])
}

/// The paths `text` names, each once, in order.
pub(super) fn paths_in(text: &str) -> Vec<String> {
    let mut paths: Vec<String> = Vec::new();
    for token in super::write_request::tokens(text) {
        let path = super::write_request::clean_path_token(token.text);
        if super::write_request::looks_like_file_path(path)
            && super::write_request::safe_relative_path(path)
            && !paths.iter().any(|seen| seen == path)
        {
            paths.push(path.to_owned());
        }
    }
    paths
}

/// Whether a clause states the seeded return action.
fn returns_in(part: &str) -> bool {
    let lexicon = seed::lexicon();
    part.split_whitespace()
        .any(|word| lexicon.mentions_role("coding_return_action", &bare(word)))
}

/// The module-function request `task` states, or `None`.
///
/// The request names a seeded code construct (`coding_request_object`) to
/// write or add, a signature, the module it goes in, and -- when it names a
/// test (`coding_test_artifact_kind`) -- the one other path, the test module.
/// Mirrors `moduleFunctionRequest`.
#[must_use]
pub fn module_function_request(task: &str) -> Option<ModuleFunctionRequest> {
    let lexicon = seed::lexicon();
    let outside = outside_quotes(task);
    let normalized = crate::engine::normalize_prompt(&outside).to_lowercase();
    if !lexicon.mentions_role("coding_request_object", &normalized)
        || !(lexicon.mentions_role("coding_request_verb", &normalized)
            || lexicon.mentions_role("coding_member_add_action", &normalized))
    {
        return None;
    }
    let stated = signature(&outside)?;
    let parts = clauses(&outside);
    let at = parts
        .iter()
        .position(|part| signature(part).is_some_and(|found| found.name == stated.name));
    // The module is the path the signature's own clause names, before or
    // after it; otherwise the first path after the signature.
    let own = at.map_or_else(Vec::new, |index| paths_in(&parts[index]));
    let paths: Vec<String> = if own.len() == 1 {
        std::iter::once(own[0].clone())
            .chain(
                paths_in(&outside)
                    .into_iter()
                    .filter(|path| path != &own[0]),
            )
            .collect()
    } else {
        paths_in(&outside[stated.at..])
    };
    let module = paths.first()?.clone();
    let language = extension_language(&module)?;
    contract(&language)?;
    crate::coding::program_language_by_slug(&language)?;
    let others = &paths[1..];
    let wants_test = lexicon.mentions_role("coding_test_artifact_kind", &normalized);
    if others.len() > 1 || (wants_test && others.len() != 1) || (!wants_test && !others.is_empty())
    {
        return None;
    }
    // The specification is the signature's clause, joined by the relative
    // clause right after it when the return is stated there.
    let clause = match at {
        None => outside.clone(),
        Some(index)
            if !returns_in(&parts[index])
                && parts.get(index + 1).is_some_and(|next| returns_in(next)) =>
        {
            format!("{} {}", parts[index], parts[index + 1])
        }
        Some(index) => parts[index].clone(),
    };
    Some(ModuleFunctionRequest {
        name: stated.name,
        parameters: stated.parameters,
        at: stated.at,
        module,
        test: others.first().cloned(),
        language,
        clause,
        command: stated_command(&outside),
    })
}

/// One seeded binary operation: its fragment id, its `{left} <operator>
/// {right}` idiom, and the relations it `supports`.
struct RelationOperation {
    id: String,
    idiom: String,
    supports: Vec<String>,
}

/// Whether `idiom` is `{left} <operator> {right}`.
fn is_binary_idiom(idiom: &str) -> bool {
    idiom
        .strip_prefix("{left} ")
        .and_then(|rest| rest.strip_suffix(" {right}"))
        .is_some_and(|operator| !operator.is_empty() && !operator.contains(char::is_whitespace))
}

/// The binary operations the seed relates to a named arithmetic relation:
/// each `coding_fragment` of data/seed/coding-composition-fragments.lino whose
/// idiom is `{left} <operator> {right}`, with the reductions it `supports`.
/// Mirrors `relationOperations`.
fn relation_operations() -> Vec<RelationOperation> {
    let Some(text) =
        crate::coding::fragment_catalog::bootstrap_seed_text("coding-composition-fragments.lino")
    else {
        return Vec::new();
    };
    let root = parse_lino(&text);
    let Some(record) = root.children.first() else {
        return Vec::new();
    };
    record
        .children
        .iter()
        .map(|node| RelationOperation {
            // A record's line is its fragment id (`integer_add`), as the
            // fragment catalog reads the same file; a `meaning <id>` record
            // carries it as the id.
            id: if node.name == "meaning" {
                node.id.clone()
            } else {
                node.name.clone()
            },
            idiom: node.find_child_value("idiom").to_owned(),
            supports: node
                .children
                .iter()
                .filter(|child| child.name == "supports")
                .map(|child| child.id.clone())
                .collect(),
        })
        .filter(|operation| is_binary_idiom(&operation.idiom))
        .collect()
}

/// `left <operator> right` for the one arithmetic relation `text` names in
/// any seeded language, or `None` when it names none or several. Mirrors
/// `relationExpression`.
fn relation_expression(text: &str, samples: &[String]) -> Option<String> {
    let [left, right] = samples else {
        return None;
    };
    let lexicon = seed::lexicon();
    let normalized = crate::engine::normalize_prompt(text).to_lowercase();
    let named: Vec<RelationOperation> = relation_operations()
        .into_iter()
        .filter(|operation| {
            operation.supports.iter().any(|relation| {
                lexicon
                    .meaning(relation)
                    .is_some_and(|meaning| meaning.evidenced_in(&normalized))
            })
        })
        .collect();
    match named.as_slice() {
        [operation] => Some(fill(
            &operation.idiom,
            &[("left", left.as_str()), ("right", right.as_str())],
        )),
        _ => None,
    }
}

/// The calculator's value of `expression`. Mirrors `evaluated`.
fn evaluated(expression: &str) -> Option<String> {
    crate::calculation::evaluate_calculation(expression)
        .ok()
        .map(|evaluation| evaluation.formatted)
}

/// The specification's value at `samples`, computed by the calculator: the
/// clause after the seeded return action with the parameters bound to the
/// samples, or, when that names no operands, the arithmetic relation the
/// clause names around its return action applied to the samples in parameter
/// order. Mirrors `specifiedValue`.
fn specified_value(request: &ModuleFunctionRequest, samples: &[String]) -> Option<String> {
    let lexicon = seed::lexicon();
    let words: Vec<&str> = request.clause.split_whitespace().collect();
    let returns = words
        .iter()
        .position(|word| lexicon.mentions_role("coding_return_action", &bare(word)))?;
    let after_signature = request
        .clause
        .find(')')
        .map_or(request.clause.as_str(), |close| {
            &request.clause[close + 1..]
        });
    stated_value(
        &words[returns + 1..],
        after_signature,
        &request.parameters,
        samples,
    )
}

/// The value a stated return computes at `samples`.
///
/// The longest expression the words open with, its parameters bound to the
/// samples (`a - b to m.mjs` reads `2 - 3`; the words after it belong to the
/// request, PR #1188 T92), or else the arithmetic relation `relation_text`
/// names applied to the samples. Mirrors `statedValue`.
pub(super) fn stated_value(
    words: &[&str],
    relation_text: &str,
    parameters: &[String],
    samples: &[String],
) -> Option<String> {
    let bound = words
        .iter()
        .map(|word| {
            parameters
                .iter()
                .position(|parameter| *parameter == bare(word))
                .and_then(|index| samples.get(index))
                .map_or_else(|| (*word).to_owned(), Clone::clone)
        })
        .collect::<Vec<_>>();
    for end in (1..=bound.len()).rev() {
        if let Some(candidate) =
            crate::calculation::calculation_expression_candidates(&bound[..end].join(" "))
                .into_iter()
                .next()
        {
            return evaluated(&candidate.expression);
        }
    }
    evaluated(&relation_expression(relation_text, samples)?)
}

/// The function's source in `request.language`: the one seeded binary
/// operation whose idiom meets the specification at every sample pair of the
/// contract, applied to the parameters in order and lowered through the
/// language's IR lowering. The native counterpart of the shared solver's
/// synthesis in `moduleFunctionRecipe`.
fn synthesized_source(request: &ModuleFunctionRequest, samples: &[String]) -> Option<String> {
    let arity = request.parameters.len();
    if arity != 2 || samples.len() < arity {
        return None;
    }
    let pairs: Vec<&[String]> = samples.chunks_exact(arity).collect();
    let specified: Vec<String> = pairs
        .iter()
        .map(|pair| specified_value(request, pair))
        .collect::<Option<Vec<_>>>()?;
    let parameter = |name: &String| IrNode::Parameter {
        name: name.clone(),
        ty: IrType::Integer,
    };
    let program = |operation: &RelationOperation| ProgramIr {
        name: request.name.clone(),
        parameters: request
            .parameters
            .iter()
            .map(|name| (name.clone(), IrType::Integer))
            .collect(),
        result: IrType::Integer,
        body: IrNode::Apply {
            fragment: operation.id.clone(),
            arguments: request.parameters.iter().map(parameter).collect(),
        },
        fragments: vec![operation.id.clone()],
        source_urls: Vec::new(),
        source_licenses: Vec::new(),
        reuse: ReuseMode::Verbatim,
    };
    let catalog = FragmentCatalog::bootstrap();
    // The integer samples type the operands: an idiom shared by an integer
    // and a float operation (`{left} * {right}`) is the one whose typed
    // signature takes the integer parameters.
    let meeting: Vec<ProgramIr> = relation_operations()
        .iter()
        .filter(|operation| {
            pairs.iter().zip(&specified).all(|(pair, expected)| {
                evaluated(&fill(
                    &operation.idiom,
                    &[("left", pair[0].as_str()), ("right", pair[1].as_str())],
                ))
                .as_ref()
                    == Some(expected)
            })
        })
        .map(program)
        .filter(|ir| ir.type_check(&catalog).is_ok())
        .collect();
    let [ir] = meeting.as_slice() else {
        return None;
    };
    crate::coding::ir_lowering::lowering_for(&request.language)?
        .lower(ir, &catalog)
        .ok()
}

/// `source` with `name` imported from `specifier` through the contract's
/// import line. Mirrors `withImport`.
fn with_import(source: &str, template: &str, name: &str, specifier: &str) -> String {
    let filled = fill(template, &[("specifier", specifier)]);
    let (prefix, suffix) = filled
        .split_once(NAMES_SLOT)
        .unwrap_or((filled.as_str(), ""));
    let mut lines: Vec<String> = source.split('\n').map(str::to_owned).collect();
    if let Some(at) = lines
        .iter()
        .position(|line| line.starts_with(prefix) && line.ends_with(suffix))
    {
        let line = &lines[at];
        let mut names: Vec<String> = line[prefix.len()..line.len() - suffix.len()]
            .split(',')
            .map(|part| part.trim().to_owned())
            .collect();
        if !names.iter().any(|existing| existing == name) {
            names.push(name.to_owned());
            lines[at] = format!("{prefix}{}{suffix}", names.join(", "));
        }
        return lines.join("\n");
    }
    let head = prefix.split_whitespace().next().unwrap_or(prefix);
    let last = lines.iter().rposition(|line| line.starts_with(head));
    let insert_at = last.map_or(0, |index| index + 1);
    lines.insert(insert_at, format!("{prefix}{name}{suffix}"));
    lines.join("\n")
}

/// The import specifier of `module` from a file at `from`, both
/// workspace-relative. Mirrors `specifierFrom`.
fn specifier_from(from: &str, module: &str) -> String {
    let from_parts: Vec<&str> = from.split('/').collect();
    let base = &from_parts[..from_parts.len() - 1];
    let target: Vec<&str> = module.split('/').collect();
    let mut shared = 0;
    while shared < base.len() && shared + 1 < target.len() && base[shared] == target[shared] {
        shared += 1;
    }
    let up: Vec<&str> = base[shared..].iter().map(|_| "..").collect();
    let path = up
        .iter()
        .copied()
        .chain(target[shared..].iter().copied())
        .collect::<Vec<_>>()
        .join("/");
    if up.is_empty() {
        format!("./{path}")
    } else {
        path
    }
}

/// `text` ending in a newline unless it is empty. Mirrors `withFinalNewline`.
fn with_final_newline(text: &str) -> String {
    if text.is_empty() || text.ends_with('\n') {
        text.to_owned()
    } else {
        format!("{text}\n")
    }
}

/// The recipe that adds the function and its test and runs the check.
/// Mirrors `moduleFunctionRecipe`.
fn module_function_recipe(
    request: &ModuleFunctionRequest,
    module_source: &str,
    test_source: &str,
) -> Option<ExecutionRecipe> {
    let terms = contract(&request.language)?;
    let catalog = crate::coding::program_language_by_slug(&request.language)?;
    let samples: Vec<String> = terms
        .find_child_value("samples")
        .split_whitespace()
        .map(str::to_owned)
        .collect();
    let program = synthesized_source(request, &samples)?;
    let definition = fill(
        terms.find_child_value("definition"),
        &[("name", request.name.as_str())],
    );
    let defined = module_source
        .split('\n')
        .any(|line| line.starts_with(&definition));
    let source = if defined {
        module_source.to_owned()
    } else {
        let separator = if module_source.trim().is_empty() {
            ""
        } else {
            "\n"
        };
        format!("{}{separator}{program}", with_final_newline(module_source))
    };
    let mut commands: Vec<String> = catalog
        .execution
        .check_command
        .as_deref()
        .into_iter()
        .map(|command| command.replace(catalog.save_as.as_ref(), &request.module))
        .collect();
    let mut supporting_files = Vec::new();
    if let Some(test) = &request.test {
        let used: Vec<String> = samples
            .iter()
            .take(request.parameters.len())
            .cloned()
            .collect();
        if used.len() != request.parameters.len() {
            return None;
        }
        let expected = specified_value(request, &used)?;
        let specifier = specifier_from(test, &request.module);
        let base = if test_source.trim().is_empty() {
            terms.find_child_value("header").to_owned()
        } else {
            with_final_newline(test_source)
        };
        let imported = with_import(
            &base,
            terms.find_child_value("import"),
            &request.name,
            &specifier,
        );
        let test_case = fill(
            terms.find_child_value("case"),
            &[
                ("name", request.name.as_str()),
                ("arguments", used.join(", ").as_str()),
                ("expected", expected.as_str()),
            ],
        );
        let source = if imported.contains(&test_case) {
            imported
        } else {
            format!("{imported}{test_case}")
        };
        supporting_files.push(ExecutionRecipeFile {
            path: test.clone(),
            source,
        });
    }
    let run = request.command.clone().or_else(|| {
        request
            .test
            .as_ref()
            .map(|test| fill(terms.find_child_value("run"), &[("test", test.as_str())]))
    });
    commands.extend(run);
    Some(ExecutionRecipe {
        language: request.language.clone(),
        source,
        path: request.module.clone(),
        supporting_files,
        commands,
    })
}

/// The file `path` as the transcript's read returned it; empty for a missing
/// file, `None` when unread. Mirrors `readSource`.
pub(super) fn read_source(current_turn: &[ChatMessage], path: &str) -> Option<String> {
    let read =
        super::workspace_change::result_for_path(current_turn, Capability::Read, path, None)?;
    let missing = super::code_artifact::source_from_agent_read_result(&read).is_none()
        && super::tool_result::failure_message(&read, false, true).is_some();
    Some(if missing {
        String::new()
    } else {
        super::code_artifact::source_from_read_result(&read)
    })
}

/// Plan the next step of a module-function request: read the module and the
/// test module, then hand the computed recipe to the execution-recipe
/// reroute. Mirrors `planModuleFunctionStep`.
pub(super) fn plan_module_function_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    _result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let request = module_function_request(task)?;
    let current_turn = &messages[super::planner::evidence_window_start(messages)..];
    let read_tool = super::capability_router::tool_for(tool_names, Capability::Read);
    let mut sources = Vec::new();
    for path in std::iter::once(&request.module).chain(request.test.iter()) {
        let Some(source) = read_source(current_turn, path) else {
            return read_tool.map(|tool| {
                super::planner::plan_one(tool, super::workspace_change::read_arguments(path))
            });
        };
        sources.push(source);
    }
    let module_source = sources.first().map_or("", String::as_str);
    let test_source = sources.get(1).map_or("", String::as_str);
    let recipe = module_function_recipe(&request, module_source, test_source)?;
    super::command_reroute::plan_symbolic_command_reroute(
        messages,
        tool_names,
        &SymbolicAnswer {
            intent: "module_function".to_owned(),
            answer: String::new(),
            confidence: 1.0,
            evidence_links: Vec::new(),
            thinking_steps: Vec::new(),
            links_notation: String::new(),
            execution_recipe: Some(Box::new(recipe)),
        },
    )
}
