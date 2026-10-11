//! Source-owned Read operands and policies; twin: `js/agentic/file_read/ownership.mjs`.
use super::records::same_path;
use super::{clean_file_token, looks_like_local_file_path};
use crate::agentic_coding::planner::{AgenticPlan, Capability, tool_capability};
use crate::seed;
const FUNCTION_ROLES: &[&str] = &["request_function_word", "enumeration_cue"];
const OBJECT_ROLE: &str = "file-read-object-noun";
const READ_ROLE: &str = "file_read_action_cue";
#[derive(Clone)]
struct SourceSpan {
    start: usize,
    end: usize,
}
struct PathSpan {
    path: String,
    span: SourceSpan,
}
struct OwnedSpan {
    span: SourceSpan,
    path_span: SourceSpan,
}
fn role_spans(text: &str, roles: &[&str]) -> Vec<SourceSpan> {
    let quoted = crate::normal_markov::quoted_segment_spans(text);
    let mut result = Vec::new();
    for role in roles {
        for surface in seed::lexicon().words_for_role(role) {
            if surface.is_empty() || surface.contains('…') {
                continue;
            }
            let Ok(expression) = regex::RegexBuilder::new(&regex::escape(&surface))
                .case_insensitive(true)
                .build()
            else {
                continue;
            };
            for matched in expression.find_iter(text) {
                let start = matched.start();
                let end = matched.end();
                let boundary = |character: char| character.is_alphanumeric() || character == '_';
                if !crate::coding::catalog::contains_cjk(&surface)
                    && (text[..start].chars().next_back().is_some_and(boundary)
                        || text[end..].chars().next().is_some_and(boundary))
                {
                    continue;
                }
                if quoted
                    .iter()
                    .any(|span| start >= span.start && end <= span.end)
                {
                    continue;
                }
                result.push(SourceSpan { start, end });
            }
        }
    }
    result
}
fn covered(text: &str, roles: &[&str], explicit: &[String]) -> bool {
    let mut remaining = text.as_bytes().to_vec();
    let mut spans = role_spans(text, roles);
    for surface in explicit {
        let Ok(expression) = regex::RegexBuilder::new(&regex::escape(surface))
            .case_insensitive(true)
            .build()
        else {
            continue;
        };
        spans.extend(expression.find_iter(text).map(|matched| SourceSpan {
            start: matched.start(),
            end: matched.end(),
        }));
    }
    for span in spans {
        remaining[span.start..span.end].fill(b' ');
    }
    String::from_utf8(remaining).is_ok_and(|remaining| {
        remaining.chars().all(|character| {
            character.is_whitespace() || ",;:.!?\"'`。，；！？।".contains(character)
        })
    })
}
fn ownership_local_file_path(path: &str) -> bool {
    let normalized: String = path
        .chars()
        .map(|character| {
            if !character.is_ascii() && character.is_alphanumeric() {
                'a'
            } else {
                character
            }
        })
        .collect();
    looks_like_local_file_path(path)
        || (normalized != path && looks_like_local_file_path(&normalized))
}
fn path_spans(text: &str) -> Vec<PathSpan> {
    super::super::write_request::tokens(text)
        .into_iter()
        .filter_map(|token| {
            let path = clean_file_token(token.text);
            if !ownership_local_file_path(&path) {
                return None;
            }
            let relative = token.text.find(&path)?;
            let start = token.start + relative;
            let end = start + path.len();
            Some(PathSpan {
                path,
                span: SourceSpan { start, end },
            })
        })
        .collect()
}

/// Consume paired path punctuation while retaining byte coordinates and quoted instruction authority.
fn balanced_path_text(text: &str) -> String {
    let mut bytes = text.as_bytes().to_vec();
    let quotations = crate::normal_markov::quoted_segment_spans(text);
    for operand in path_spans(text) {
        if quotations.iter().any(|span| {
            operand.span.start >= span.start
                && operand.span.end <= span.end
                && span.text != operand.path
        }) {
            continue;
        }
        let mut start = operand.span.start;
        let mut end = operand.span.end;
        while start > 0
            && end < text.len()
            && matches!(text.as_bytes()[start - 1], b'"' | b'\'' | 96)
            && text.as_bytes()[start - 1] == text.as_bytes()[end]
        {
            start -= 1;
            end += 1;
        }
        let mut consumed = Vec::new();
        while start > 0 && end < text.len() {
            let closing = match text.as_bytes()[start - 1] {
                b'(' => b')',
                b'[' => b']',
                b'{' => b'}',
                _ => break,
            };
            if text.as_bytes()[end] != closing {
                break;
            }
            consumed.push(start - 1);
            consumed.push(end);
            start -= 1;
            end += 1;
        }
        let wrappers = b"()[]{}";
        if start
            .checked_sub(1)
            .is_some_and(|index| wrappers.contains(&text.as_bytes()[index]))
            || text
                .as_bytes()
                .get(end)
                .is_some_and(|byte| wrappers.contains(byte))
        {
            continue;
        }
        for index in consumed {
            bytes[index] = b' ';
        }
    }
    String::from_utf8(bytes).expect("Only ASCII path punctuation was replaced")
}

fn policy_starts(text: &str) -> bool {
    let lower = text.trim_start().to_lowercase();
    let vocabulary = seed::caller_context_vocabulary();
    vocabulary.policy_lead_clause(&lower).is_some()
        || vocabulary
            .policy_leads
            .iter()
            .any(|lead| crate::coding::catalog::contains_cjk(lead) && lower.starts_with(lead))
}
pub(in crate::agentic_coding) fn pending_read_condition(prompt: &str, role: &str) -> bool {
    if balanced_path_text(prompt) != prompt && mode_paths_for_clause(prompt).is_none() {
        return true;
    }
    let sentences = instruction_sentence_texts(prompt);
    let unresolved = sentences.iter().any(|sentence| {
        path_spans(sentence).iter().any(|path| {
            let operand_end = crate::normal_markov::quoted_segment_spans(sentence)
                .iter()
                .filter(|span| {
                    span.text.as_str() == path.path.as_str()
                        && path.span.start >= span.start
                        && path.span.end <= span.end
                })
                .fold(path.span.end, |end, span| end.max(span.end));
            let tail = &sentence[operand_end..];
            policy_starts(tail)
                && bound_read_paths(&sentence[..operand_end], role).contains(&path.path)
                && !read_precedes_owned_authoring(prompt, tail)
        })
    });
    if unresolved {
        return true;
    }
    if bound_read_paths(prompt, role).is_empty() {
        return false;
    }
    sentences.iter().any(|sentence| {
        if !bound_read_paths(sentence, role).is_empty() {
            return false;
        }
        role_spans(sentence, &["enumeration_cue", "file_edit_joiner_cue"])
            .iter()
            .any(|span| {
                covered(&sentence[..span.start], FUNCTION_ROLES, &[])
                    && !covered(&sentence[span.end..], FUNCTION_ROLES, &[])
            })
    })
}
fn negative_read_objects(prompt: &str) -> Vec<Option<String>> {
    let mut result = Vec::new();
    for sentence in instruction_sentence_texts(prompt) {
        for action in role_spans(&sentence, &[READ_ROLE]) {
            let prefix = sentence[..action.start].to_lowercase();
            let Some(rest) = seed::caller_context_vocabulary().policy_lead_clause(&prefix) else {
                continue;
            };
            if !seed::lexicon().mentions_role("statement_negation_cue", &prefix)
                || !covered(rest, FUNCTION_ROLES, &[])
            {
                continue;
            }
            let mut objects = path_spans(&sentence)
                .into_iter()
                .map(|object| (object.span, Some(object.path)))
                .collect::<Vec<_>>();
            objects.extend(
                role_spans(&sentence, &[OBJECT_ROLE])
                    .into_iter()
                    .map(|span| (span, None)),
            );
            for (object, scope) in objects {
                if object.start < action.end
                    || !covered(&sentence[action.end..object.start], FUNCTION_ROLES, &[])
                {
                    continue;
                }
                result.push(scope);
            }
        }
    }
    for sentence in instruction_sentence_texts(prompt) {
        for action in role_spans(&sentence, &["file-read-negative-operation"]) {
            let mut objects = path_spans(&sentence)
                .into_iter()
                .map(|object| (object.span, Some(object.path)))
                .collect::<Vec<_>>();
            objects.extend(
                role_spans(&sentence, &[OBJECT_ROLE])
                    .into_iter()
                    .map(|span| (span, None)),
            );
            for (object, scope) in objects {
                let prefix = &sentence[..action.start.min(object.start)];
                let gap = if object.start >= action.end {
                    Some(&sentence[action.end..object.start])
                } else if object.end <= action.start {
                    Some(&sentence[object.end..action.start])
                } else {
                    None
                };
                let mut prefix_roles = FUNCTION_ROLES.to_vec();
                prefix_roles.push("statement_negation_cue");
                if (covered(prefix, &prefix_roles, &[]) || policy_starts(prefix))
                    && gap.is_some_and(|gap| covered(gap, FUNCTION_ROLES, &[]))
                {
                    result.push(scope);
                }
            }
        }
    }
    result
}
fn unbound_effect_operation(prompt: &str) -> bool {
    instruction_sentence_texts(prompt)
        .into_iter()
        .any(|sentence| {
            let paths = path_spans(&sentence);
            let bound = bound_read_paths(&sentence, READ_ROLE);
            role_spans(&sentence, &["file-read-independent-effect-operation"])
                .into_iter()
                .any(|action| {
                    if paths
                        .iter()
                        .any(|path| action.start >= path.span.start && action.end <= path.span.end)
                    {
                        return false;
                    }
                    if !bound.is_empty()
                        && role_spans(&sentence, &[READ_ROLE])
                            .iter()
                            .any(|span| action.start >= span.start && action.end <= span.end)
                    {
                        return false;
                    }
                    let mut prefix = sentence.as_bytes()[..action.start].to_vec();
                    if !bound.is_empty() {
                        let mut spans = paths
                            .iter()
                            .filter(|path| bound.contains(&path.path))
                            .map(|path| path.span.clone())
                            .collect::<Vec<_>>();
                        spans.extend(role_spans(&sentence, &[READ_ROLE]));
                        for span in spans {
                            if span.end <= action.start {
                                prefix[span.start..span.end].fill(b' ');
                            }
                        }
                    }
                    String::from_utf8(prefix)
                        .is_ok_and(|prefix| covered(&prefix, FUNCTION_ROLES, &[]))
                })
        })
}
fn conflicts(scopes: &[Option<String>], paths: &[String]) -> bool {
    scopes.iter().any(|scope| {
        scope.as_ref().is_none_or(|scope| {
            paths
                .iter()
                .any(|path| same_path(path, scope) || same_path(scope, path))
        })
    })
}
/// Structural operands only; callers still preflight every immutable request constraint.
pub(in crate::agentic_coding) fn bound_read_paths(prompt: &str, role: &str) -> Vec<String> {
    if role == READ_ROLE
        && let Some(paths) = mode_paths_for_clause(prompt)
    {
        return if conflicts(&negative_read_objects(prompt), &paths) {
            Vec::new()
        } else {
            paths
        };
    }
    let mut result = Vec::new();
    for sentence in instruction_sentence_texts(prompt) {
        let mut owned: Vec<OwnedSpan> = Vec::new();
        for object in path_spans(&sentence) {
            for action in role_spans(&sentence, &[role]) {
                let prefix_end = action.start.min(object.span.start);
                let mut before = sentence.as_bytes()[..prefix_end].to_vec();
                for prior in &owned {
                    if prior.span.end <= before.len() {
                        before[prior.span.start..prior.span.end].fill(b' ');
                    }
                }
                let question_words = if role == "module_export_question" {
                    seed::caller_context_vocabulary().question_words.clone()
                } else {
                    Vec::new()
                };
                let mut prefix_roles = FUNCTION_ROLES.to_vec();
                if role == "module_export_question" {
                    prefix_roles.extend([
                        READ_ROLE,
                        "coding_declaration_noun",
                        "translation_stop_word",
                    ]);
                }
                let Ok(before) = String::from_utf8(before) else {
                    continue;
                };
                let mut leading_roles = prefix_roles.clone();
                leading_roles.push("social_greeting");
                if !covered(&before, &leading_roles, &question_words) {
                    continue;
                }
                if object.span.start >= action.end {
                    let mut gap = sentence.as_bytes()[action.end..object.span.start].to_vec();
                    for prior in &owned {
                        if prior.path_span.start >= action.end
                            && prior.path_span.end <= object.span.start
                        {
                            gap[prior.path_span.start - action.end
                                ..prior.path_span.end - action.end]
                                .fill(b' ');
                        }
                    }
                    let mut gap_roles = FUNCTION_ROLES.to_vec();
                    gap_roles.push(OBJECT_ROLE);
                    if role == "module_export_question" {
                        gap_roles.extend(["coding_declaration_noun", "translation_stop_word"]);
                    }
                    let Ok(gap) = String::from_utf8(gap) else {
                        continue;
                    };
                    if !covered(&gap, &gap_roles, &[]) {
                        continue;
                    }
                } else if object.span.end <= action.start {
                    if role == "module_export_question"
                        && !question_words
                            .iter()
                            .any(|word| sentence.to_lowercase().starts_with(word))
                    {
                        continue;
                    }
                    if !covered(&sentence[object.span.end..action.start], &prefix_roles, &[]) {
                        continue;
                    }
                    let mut trailing_roles = prefix_roles.clone();
                    trailing_roles.push(role);
                    if !covered(&sentence[action.end..], &trailing_roles, &[]) {
                        continue;
                    }
                } else {
                    continue;
                }
                owned.push(OwnedSpan {
                    span: SourceSpan {
                        start: action.start.min(object.span.start),
                        end: action.end.max(object.span.end),
                    },
                    path_span: object.span.clone(),
                });
                if !result.contains(&object.path) {
                    result.push(object.path.clone());
                }
                break;
            }
        }
        if !owned.is_empty() && owned.len() != path_spans(&sentence).len() {
            return Vec::new();
        }
    }
    if conflicts(&negative_read_objects(prompt), &result) {
        Vec::new()
    } else {
        result
    }
}
pub(in crate::agentic_coding) fn owned_read_paths(prompt: &str, role: &str) -> Vec<String> {
    let prompt = read_request_envelope(prompt).map_or(prompt, |(text, _, _)| text);
    let paths = bound_read_paths(prompt, role);
    if unbound_effect_operation(prompt) || pending_read_condition(prompt, role) {
        Vec::new()
    } else {
        paths
    }
}
pub(in crate::agentic_coding) fn read_policy_blocks_plan(prompt: &str, plan: &AgenticPlan) -> bool {
    let prompt = read_request_envelope(prompt).map_or(prompt, |(text, _, _)| text);
    let scopes = negative_read_objects(prompt);
    let unbound_operation =
        unbound_effect_operation(prompt) || pending_read_condition(prompt, READ_ROLE);
    if scopes.is_empty() && !unbound_operation {
        return false;
    }
    let AgenticPlan::ToolCalls(calls) = plan else {
        return false;
    };
    for call in calls {
        let capability = tool_capability(&call.tool);
        if !scopes.is_empty()
            && matches!(
                capability,
                Some(Capability::Run | Capability::Grep | Capability::ReadMany)
            )
        {
            return true;
        }
        if capability != Some(Capability::Read) {
            continue;
        }
        if unbound_operation {
            return true;
        }
        let Ok(arguments_value) = serde_json::from_str::<serde_json::Value>(&call.arguments) else {
            return true;
        };
        let fields = ["path", "filePath", "file_path"]
            .into_iter()
            .filter_map(|key| arguments_value.get(key))
            .collect::<Vec<_>>();
        let Some(path) = fields
            .first()
            .and_then(|field| field.as_str())
            .filter(|path| !path.trim().is_empty())
        else {
            return true;
        };
        if fields.iter().any(|field| field.as_str() != Some(path))
            || conflicts(&scopes, &[path.to_owned()])
        {
            return true;
        }
    }
    false
}

fn mode_paths_for_clause(prompt: &str) -> Option<Vec<String>> {
    let normalized = balanced_path_text(prompt);
    let prompt = normalized.as_str();
    let parsed = crate::seed::parser::parse_lino(include_str!(
        "../../../embedded/data/seed/meanings-file-write.lino"
    ));
    let root = parsed.children.first()?;
    let contract = root
        .children
        .iter()
        .find(|node| node.name == "file-read-mode-contract")?;
    let placeholders = regex::Regex::new(r"\{([a-z]+(?:-[a-z]+)*)\}").ok()?;
    for form in contract.children.iter().filter(|node| node.name == "form") {
        let Some(template) = form
            .children
            .iter()
            .find(|node| node.name == "pattern")
            .map(|node| node.id.as_str())
        else {
            continue;
        };
        let mut missing = false;
        let pattern = placeholders
            .replace_all(template, |captures: &regex::Captures<'_>| {
                let name = &captures[1];
                if ["source", "sources"].contains(&name) {
                    return format!("(?P<{name}>.+?)");
                }
                let exact = seed::lexicon().words_for_role(name);
                let legacy = seed::lexicon().words_for_role(&name.replace('-', "_"));
                if !exact.is_empty() && !legacy.is_empty() && exact != legacy {
                    missing = true;
                    return String::new();
                }
                let surfaces = if exact.is_empty() { legacy } else { exact };
                if surfaces.is_empty() {
                    missing = true;
                }
                format!(
                    "(?:{})",
                    surfaces
                        .iter()
                        .map(|surface| regex::escape(surface))
                        .collect::<Vec<_>>()
                        .join("|")
                )
            })
            .replace("(?<", "(?P<");
        if missing {
            continue;
        }
        let Ok(expression) = regex::RegexBuilder::new(&pattern)
            .case_insensitive(true)
            .build()
        else {
            continue;
        };
        let Some(captures) = expression.captures(prompt) else {
            continue;
        };
        if captures.get(0)?.as_str().len() != prompt.len() {
            continue;
        }
        if !form
            .children
            .iter()
            .filter(|node| node.name == "capture-domain")
            .all(|domain| {
                let Some(value) = captures.name(&domain.id).map(|matched| matched.as_str()) else {
                    return false;
                };
                let Some(domain_pattern) = domain
                    .children
                    .iter()
                    .find(|node| node.name == "pattern")
                    .map(|node| node.id.as_str())
                else {
                    return false;
                };
                regex::Regex::new(domain_pattern).is_ok_and(|expression| {
                    expression
                        .find(value)
                        .is_some_and(|matched| matched.as_str().len() == value.len())
                })
            })
        {
            continue;
        }
        let Some(source) = captures
            .name("source")
            .or_else(|| captures.name("sources"))
            .map(|matched| matched.as_str())
        else {
            continue;
        };
        if crate::normal_markov::quote_fault(source).is_some() {
            continue;
        }
        let paths = path_spans(source);
        if paths.is_empty() || captures.name("source").is_some() && paths.len() != 1 {
            continue;
        }
        let mut remaining = source.as_bytes().to_vec();
        for path in &paths {
            remaining[path.span.start..path.span.end].fill(b' ');
        }
        let Ok(remaining) = String::from_utf8(remaining) else {
            continue;
        };
        let mut roles = FUNCTION_ROLES.to_vec();
        roles.push(OBJECT_ROLE);
        roles.push("file-read-mode-joiner");
        if !covered(&remaining, &roles, &[]) {
            continue;
        }
        let mut result = Vec::new();
        for path in paths {
            if !result.contains(&path.path) {
                result.push(path.path);
            }
        }
        return Some(result);
    }
    None
}

/// Paired literal quotations retain global scope before clause splitting.
fn instruction_sentence_texts(prompt: &str) -> Vec<String> {
    let normalized = balanced_path_text(prompt);
    let prompt = normalized.as_str();
    let mut bytes = prompt.as_bytes().to_vec();
    for span in crate::normal_markov::quoted_segment_spans(prompt) {
        let path = clean_file_token(&span.text);
        if path == span.text && ownership_local_file_path(&path) {
            continue;
        }
        bytes[span.start..span.end].fill(b' ');
        bytes[span.end - 1] = b'.';
    }
    let Ok(text) = String::from_utf8(bytes) else {
        return Vec::new();
    };
    let mut protected_bytes = text.as_bytes().to_vec();
    for span in path_spans(&text) {
        for byte in &mut protected_bytes[span.span.start..span.span.end] {
            if *byte == b'.' {
                *byte = b'_';
            }
        }
    }
    let Ok(protected) = String::from_utf8(protected_bytes) else {
        return Vec::new();
    };
    super::super::shell_command_policy::sentences(&protected)
        .into_iter()
        .map(|sentence| text[sentence.span].trim().to_owned())
        .collect()
}

/// A declared source goal may require Read before its deferred authoring.
fn read_precedes_owned_authoring(prompt: &str, tail: &str) -> bool {
    if super::super::module_function::observed_callable_request(prompt).is_none() {
        return false;
    }
    let parsed = crate::seed::parser::parse_lino(include_str!(
        "../../../embedded/data/seed/meanings-file-write.lino"
    ));
    let Some(root) = parsed.children.first() else {
        return false;
    };
    let Some(contract) = root
        .children
        .iter()
        .find(|node| node.name == "file-read-order-contract")
    else {
        return false;
    };
    if !contract
        .children
        .iter()
        .any(|node| node.name == "requires-goal" && node.id == "observed-callable-authoring")
    {
        return false;
    }
    contract
        .children
        .iter()
        .filter(|node| node.name == "form")
        .any(|form| {
            let Some(pattern) = form
                .children
                .iter()
                .find(|node| node.name == "pattern")
                .map(|node| node.id.as_str())
            else {
                return false;
            };
            regex::RegexBuilder::new(pattern)
                .case_insensitive(true)
                .build()
                .is_ok_and(|expression| {
                    expression
                        .find(tail)
                        .is_some_and(|matched| matched.as_str().len() == tail.len())
                })
        })
}

/// Source-bound UTF8 counterpart of Read request envelope ownership.
pub(in crate::agentic_coding) fn read_request_envelope(
    request: &str,
) -> Option<(&str, usize, usize)> {
    if !request.starts_with('"') || crate::normal_markov::quote_fault(request).is_some() {
        return None;
    }
    let spans = crate::normal_markov::quoted_segment_spans(request);
    if spans.len() != 1 || spans[0].start != 0 || spans[0].end != request.len() {
        return None;
    }
    let start = 1;
    let end = request.len().checked_sub(1)?;
    let text = request.get(start..end)?;
    mode_paths_for_clause(text)?;
    if !matches!(
        super::file_read_task_for(text)?,
        super::FileReadTask::Direct {
            mode: super::FileReadMode::Full,
            ..
        }
    ) {
        return None;
    }
    Some((text, start, end))
}
