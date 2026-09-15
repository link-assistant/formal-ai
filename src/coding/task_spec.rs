//! Structural coding-task recognition and formalization (issue #710).
//!
//! Benchmark prompts and conversational requests use different surface forms,
//! but they all reduce to the same specification: a target language, callable
//! signature, requirements, and executable examples. Prose recognition is
//! driven by seed roles; Python syntax stays structural.

use crate::coding::python_signature::{
    import_preamble, matching_close_paren, split_top_level_commas,
};
use crate::links_format::push_lino_node;
use crate::seed::Slot;

/// The observable artifact requested by a coding task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactShape {
    /// A named callable whose behavior is checked by examples.
    Function,
    /// A runnable script whose process output is the observable result.
    Program,
}

impl ArtifactShape {
    const fn slug(self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Program => "program",
        }
    }
}

/// One parameter in the requested callable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub annotation: Option<String>,
}

/// One call example extracted from a docstring or assertion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Example {
    pub arguments: Vec<String>,
    pub expected: String,
}

/// The language-independent shape consumed by coding discovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodingTaskSpec {
    pub language: String,
    pub artifact_shape: ArtifactShape,
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_annotation: Option<String>,
    pub imports: Vec<String>,
    pub requirement_sentences: Vec<String>,
    pub examples: Vec<Example>,
    /// Exact requested process output when the prompt exposes one through a
    /// seed-declared output slot. `None` means discovery must derive another
    /// executable observation (for example an inclusive counting range).
    pub expected_stdout: Option<String>,
    pub prose_language: String,
}

impl CodingTaskSpec {
    /// A compact signature identity used by cross-language transfer tests.
    #[must_use]
    pub fn signature_identity(&self) -> String {
        let parameters = self
            .parameters
            .iter()
            .map(|parameter| parameter.name.as_str())
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{}({parameters})->{}",
            self.language,
            self.return_annotation.as_deref().unwrap_or("_")
        )
    }

    /// Formalize the spec as a nested Links Notation record.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut out = String::new();
        push_lino_node(&mut out, 0, "coding_task_spec", None);
        push_lino_node(&mut out, 2, "language", Some(&self.language));
        push_lino_node(
            &mut out,
            2,
            "artifact_shape",
            Some(self.artifact_shape.slug()),
        );
        push_lino_node(&mut out, 2, "name", Some(&self.name));
        for parameter in &self.parameters {
            push_lino_node(&mut out, 2, "parameter", Some(&parameter.name));
            if let Some(annotation) = &parameter.annotation {
                push_lino_node(&mut out, 4, "annotation", Some(annotation));
            }
        }
        if let Some(annotation) = &self.return_annotation {
            push_lino_node(&mut out, 2, "return_annotation", Some(annotation));
        }
        for import in &self.imports {
            push_lino_node(&mut out, 2, "import", Some(import));
        }
        for requirement in &self.requirement_sentences {
            push_lino_node(&mut out, 2, "requirement", Some(requirement));
        }
        for example in &self.examples {
            push_lino_node(&mut out, 2, "example", None);
            for argument in &example.arguments {
                push_lino_node(&mut out, 4, "argument", Some(argument));
            }
            push_lino_node(&mut out, 4, "expected", Some(&example.expected));
        }
        if let Some(expected) = &self.expected_stdout {
            push_lino_node(&mut out, 2, "expected_stdout", Some(expected));
        }
        push_lino_node(&mut out, 2, "prose_language", Some(&self.prose_language));
        out.trim_end().to_owned()
    }
}

/// Recognize `HumanEval`, `MBPP`, and conversational Python task shapes.
#[must_use]
pub fn recognise(prompt: &str) -> Option<CodingTaskSpec> {
    let detected_prose_language = crate::language::detect(prompt).slug().to_owned();
    let imports = import_preamble(prompt)
        .lines()
        .map(str::to_owned)
        .collect::<Vec<_>>();

    if let Some(signature) = python_definition_signature(prompt) {
        let (name, parameters, return_annotation) = parse_signature(signature)?;
        let (requirement_sentences, examples) = docstring_contract(prompt, &name);
        return Some(CodingTaskSpec {
            language: "python".to_owned(),
            artifact_shape: ArtifactShape::Function,
            name,
            parameters,
            return_annotation,
            imports,
            requirement_sentences,
            examples,
            expected_stdout: None,
            prose_language: detected_prose_language,
        });
    }

    let assertion_examples = assertion_contract(prompt);
    if let Some((name, examples)) = assertion_examples {
        let arity = examples
            .iter()
            .map(|example| example.arguments.len())
            .max()
            .unwrap_or(0);
        let parameters = (1..=arity)
            .map(|position| Parameter {
                name: format!("arg{position}"),
                annotation: None,
            })
            .collect();
        let requirement_sentences = prompt
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty() && line.strip_prefix("assert ").is_none())
            .map(split_sentences)
            .unwrap_or_default();
        return Some(CodingTaskSpec {
            language: "python".to_owned(),
            artifact_shape: ArtifactShape::Function,
            name,
            parameters,
            return_annotation: None,
            imports,
            requirement_sentences,
            examples,
            expected_stdout: None,
            prose_language: detected_prose_language,
        });
    }

    // A conversational request is governed by the outer instruction, not by
    // words that happen to occur inside a fenced document it asks another
    // handler to transform. Definition/assertion benchmark shapes were already
    // parsed above from the complete payload.
    let conversational_prompt = outside_markdown_fences(prompt);
    let normalized = crate::engine::normalize_prompt(&conversational_prompt);
    let lexicon = crate::seed::lexicon();
    let is_conversational_request = lexicon
        .mentions_role(crate::seed::ROLE_CODING_REQUEST_VERB, &normalized)
        && lexicon.mentions_role(crate::seed::ROLE_CODING_REQUEST_OBJECT, &normalized)
        && normalized.split_whitespace().any(|word| word == "python");
    if !is_conversational_request {
        return None;
    }
    let signature = backtick_signature(prompt).or_else(|| inline_function_signature(prompt));
    let (artifact_shape, name, parameters, return_annotation) = if let Some(signature) = signature {
        let (name, parameters, return_annotation) = parse_signature(signature)?;
        (ArtifactShape::Function, name, parameters, return_annotation)
    } else if lexicon
        .meaning("coding_request_program")
        .is_some_and(|meaning| meaning.evidenced_in(&normalized))
    {
        (ArtifactShape::Program, "main".to_owned(), Vec::new(), None)
    } else if lexicon.mentions_role(crate::seed::ROLE_PROGRAM_SYNTHESIS_SUBJECT, &normalized) {
        // The source catalog owns the callable identity and arity when prose
        // names a concept rather than spelling a signature. This provisional
        // name is never rendered by a source-derived candidate.
        (
            ArtifactShape::Function,
            "discovered_function".to_owned(),
            Vec::new(),
            None,
        )
    } else {
        return None;
    };
    let language_priority = crate::language::registered_languages()
        .into_iter()
        .map(crate::language::Language::slug)
        .collect::<Vec<_>>();
    let prose_language = lexicon
        .first_role_language(
            crate::seed::ROLE_CODING_REQUEST_VERB,
            &normalized,
            &language_priority,
        )
        .map_or(detected_prose_language, str::to_owned);
    Some(CodingTaskSpec {
        language: "python".to_owned(),
        artifact_shape,
        name,
        parameters,
        return_annotation,
        imports,
        requirement_sentences: split_sentences(prompt.trim()),
        examples: Vec::new(),
        expected_stdout: (artifact_shape == ArtifactShape::Program)
            .then(|| extract_expected_stdout(prompt))
            .flatten(),
        prose_language,
    })
}

fn outside_markdown_fences(prompt: &str) -> String {
    let mut outside = true;
    prompt
        .lines()
        .filter(|line| {
            if line.trim_start().starts_with("```") {
                outside = !outside;
                return false;
            }
            outside
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Capture a program's requested stdout through the same multilingual slot
/// records used by the software-project surface. Prefix and circumfix forms
/// cover natural word order without putting any output literal in code.
fn extract_expected_stdout(prompt: &str) -> Option<String> {
    let lower = prompt.to_lowercase();
    let mut forms = crate::seed::lexicon()
        .role_word_forms(crate::seed::ROLE_OUTPUT_DISPLAY_REQUEST)
        .into_iter()
        .filter(|form| form.slot() != Slot::Bare)
        .collect::<Vec<_>>();
    forms.sort_by_key(|form| std::cmp::Reverse(form.text.chars().count()));
    for form in forms {
        let before = form.before_slot();
        let after = form.after_slot();
        let start = if before.is_empty() {
            0
        } else if let Some(found) = lower.find(before) {
            found + before.len()
        } else {
            continue;
        };
        let Some(tail) = prompt.get(start..) else {
            continue;
        };
        let end = if after.is_empty() {
            tail.len()
        } else if let Some(found) = tail.to_lowercase().find(after) {
            found
        } else {
            continue;
        };
        let value = tail[..end]
            .trim()
            .trim_end_matches(['.', '?', '。', '？'])
            .trim();
        if !value.is_empty() {
            return Some(value.to_owned());
        }
    }
    None
}

fn python_definition_signature(prompt: &str) -> Option<&str> {
    prompt.lines().rev().find_map(|line| {
        line.trim()
            .strip_prefix("def ")
            .map(|signature| signature.trim_end_matches(':').trim())
    })
}

fn backtick_signature(prompt: &str) -> Option<&str> {
    prompt
        .split('`')
        .enumerate()
        .filter(|(index, _)| index % 2 == 1)
        .map(|(_, candidate)| candidate.trim())
        .find(|candidate| {
            candidate.find('(').is_some_and(|open| {
                matching_close_paren(candidate, open).is_some_and(|close| close == candidate.len())
            })
        })
}

fn inline_function_signature(prompt: &str) -> Option<&str> {
    for (open, _) in prompt.match_indices('(') {
        let bytes = prompt.as_bytes();
        let mut start = open;
        while start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_') {
            start -= 1;
        }
        if start == open || !is_identifier(&prompt[start..open]) {
            continue;
        }
        let close = matching_close_paren(prompt, open)?;
        return Some(prompt[start..close].trim());
    }
    None
}

fn parse_signature(signature: &str) -> Option<(String, Vec<Parameter>, Option<String>)> {
    let open = signature.find('(')?;
    let close = matching_close_paren(signature, open)?;
    let name = signature[..open].split_whitespace().last()?.trim();
    if !is_identifier(name) {
        return None;
    }
    let parameters = split_top_level_commas(&signature[open + 1..close - 1])
        .into_iter()
        .filter_map(parse_parameter)
        .collect::<Vec<_>>();
    let tail = signature[close..].trim();
    let return_annotation = tail
        .strip_prefix("->")
        .map(|annotation| annotation.trim().trim_end_matches(':').trim().to_owned())
        .filter(|annotation| !annotation.is_empty());
    Some((name.to_owned(), parameters, return_annotation))
}

fn parse_parameter(raw: &str) -> Option<Parameter> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let declaration = raw.split_once('=').map_or(raw, |(left, _)| left).trim();
    let (name, annotation) = declaration.split_once(':').map_or_else(
        || (declaration, None),
        |(name, annotation)| (name, Some(annotation.trim().to_owned())),
    );
    let name = name.trim().trim_start_matches('*');
    is_identifier(name).then(|| Parameter {
        name: name.to_owned(),
        annotation,
    })
}

fn is_identifier(candidate: &str) -> bool {
    let mut characters = candidate.chars();
    characters
        .next()
        .is_some_and(|first| first == '_' || first.is_ascii_alphabetic())
        && characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn docstring_contract(prompt: &str, function_name: &str) -> (Vec<String>, Vec<Example>) {
    let Some(content) = docstring_content(prompt, function_name) else {
        return (Vec::new(), Vec::new());
    };
    let lines = content.lines().map(str::trim).collect::<Vec<_>>();
    let mut prose = Vec::new();
    let mut examples = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        if let Some(call) = line.strip_prefix(">>>") {
            if let Some((name, arguments)) = parse_call(call.trim())
                && name == function_name
            {
                let next = lines.get(index + 1).copied().unwrap_or_default();
                let expected = if next.is_empty() || next.starts_with(">>>") {
                    "None"
                } else {
                    next
                };
                examples.push(Example {
                    arguments,
                    expected: expected.to_owned(),
                });
                index += usize::from(!next.is_empty()) + 1;
                continue;
            }
        } else if !line.is_empty() {
            prose.push(line);
        }
        index += 1;
    }
    (split_sentences(&prose.join(" ")), examples)
}

fn docstring_content<'a>(prompt: &'a str, function_name: &str) -> Option<&'a str> {
    let marker = format!("def {function_name}(");
    let function_start = prompt.rfind(&marker)?;
    let function = &prompt[function_start..];
    let double = function.find("\"\"\"").map(|start| (start, "\"\"\""));
    let single = function.find("'''").map(|start| (start, "'''"));
    let (start, delimiter) = match (double, single) {
        (Some(left), Some(right)) => left.min(right),
        (Some(found), None) | (None, Some(found)) => found,
        (None, None) => return None,
    };
    let content_start = start + delimiter.len();
    let content_end = function[content_start..].find(delimiter)? + content_start;
    Some(&function[content_start..content_end])
}

fn assertion_contract(prompt: &str) -> Option<(String, Vec<Example>)> {
    let mut declared_name = None;
    let mut examples = Vec::new();
    for line in prompt.lines().map(str::trim) {
        let Some(expression) = line.strip_prefix("assert ") else {
            continue;
        };
        let Some((call, expected)) = split_top_level_equality(expression) else {
            continue;
        };
        let Some((name, arguments)) = parse_call(call.trim()) else {
            continue;
        };
        if declared_name.as_deref().is_some_and(|known| known != name) {
            continue;
        }
        declared_name.get_or_insert_with(|| name.to_owned());
        examples.push(Example {
            arguments,
            expected: expected.trim().to_owned(),
        });
    }
    Some((declared_name?, examples))
}

fn split_top_level_equality(expression: &str) -> Option<(&str, &str)> {
    let mut depth = 0usize;
    let bytes = expression.as_bytes();
    let mut index = 0;
    while index + 1 < bytes.len() {
        match bytes[index] {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth = depth.saturating_sub(1),
            b'=' if depth == 0 && bytes[index + 1] == b'=' => {
                return Some((&expression[..index], &expression[index + 2..]));
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn parse_call(expression: &str) -> Option<(&str, Vec<String>)> {
    let open = expression.find('(')?;
    let close = matching_close_paren(expression, open)?;
    let name = expression[..open].trim();
    if !is_identifier(name) || !expression[close..].trim().is_empty() {
        return None;
    }
    let arguments = split_top_level_commas(&expression[open + 1..close - 1])
        .into_iter()
        .map(str::trim)
        .filter(|argument| !argument.is_empty())
        .map(str::to_owned)
        .collect();
    Some((name, arguments))
}

fn split_sentences(text: &str) -> Vec<String> {
    let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut sentences = Vec::new();
    let mut start = 0;
    let mut quote = None;
    for (index, character) in compact.char_indices() {
        if quote.is_some_and(|delimiter| delimiter == character) {
            quote = None;
            continue;
        }
        if quote.is_none() && matches!(character, '\'' | '"') {
            quote = Some(character);
            continue;
        }
        if quote.is_none() && matches!(character, '.' | '?' | '!' | '。' | '？' | '！') {
            let end = index + character.len_utf8();
            let sentence = compact[start..end].trim();
            if !sentence.is_empty() {
                sentences.push(sentence.to_owned());
            }
            start = end;
        }
    }
    let tail = compact[start..].trim();
    if !tail.is_empty() {
        sentences.push(tail.to_owned());
    }
    sentences
}
