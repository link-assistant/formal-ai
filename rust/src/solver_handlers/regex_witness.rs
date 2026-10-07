//! Witness check for the composed regular expression (issue #1177 R1177-2).
//!
//! The regex synthesizer composes a pattern from seeded counts, classes and
//! separators. This module derives positive and negative example strings
//! from those same constraints (the browser twin is `regexSynthesisExamples`
//! in `js/worker/formal_ai_worker_code_synthesis.js`) and matches each one
//! against the emitted pattern text with a small backtracking matcher over
//! the pattern subset the composer can emit: anchors, `\d`/`\w`/`\s` and
//! escaped literals, bracket classes with ranges, `.`, groups, and the `?`,
//! `*`, `+`, `{n}`, `{n,}` and `{n,m}` repetitions. The regex crate is a
//! dev-only dependency, so the native runtime reads the pattern itself
//! instead; the browser compiles it with `RegExp`. No user input is run.

/// One parsed pattern atom.
enum Atom {
    Literal(char),
    Any,
    Class(Vec<(char, char)>, bool),
    Escape(char),
    Group(Vec<Node>),
}

/// An atom with its repetition bounds (`max` of `None` is unbounded).
struct Node {
    atom: Atom,
    min: usize,
    max: Option<usize>,
}

/// A parsed pattern: its nodes and whether it is anchored at either end.
struct Pattern {
    nodes: Vec<Node>,
    start: bool,
    end: bool,
}

/// Read the digits at `chars[*at..]` as a number.
fn number(chars: &[char], at: &mut usize) -> Option<usize> {
    let begin = *at;
    while *at < chars.len() && chars[*at].is_ascii_digit() {
        *at += 1;
    }
    chars[begin..*at].iter().collect::<String>().parse().ok()
}

/// Parse a bracket class body starting just after `[`.
fn class(chars: &[char], at: &mut usize) -> Option<Atom> {
    let negated = chars.get(*at) == Some(&'^');
    if negated {
        *at += 1;
    }
    let mut ranges = Vec::new();
    while *at < chars.len() && chars[*at] != ']' {
        let mut low = chars[*at];
        if low == '\\' {
            *at += 1;
            low = *chars.get(*at)?;
        }
        *at += 1;
        if chars.get(*at) == Some(&'-') && chars.get(*at + 1).is_some_and(|ch| *ch != ']') {
            ranges.push((low, chars[*at + 1]));
            *at += 2;
        } else {
            ranges.push((low, low));
        }
    }
    (*at < chars.len()).then(|| {
        *at += 1;
        Atom::Class(ranges, negated)
    })
}

/// Parse a sequence of nodes until `)` or the end.
fn sequence(chars: &[char], at: &mut usize) -> Option<Vec<Node>> {
    let mut nodes = Vec::new();
    while *at < chars.len() && chars[*at] != ')' {
        let atom = match chars[*at] {
            '\\' => {
                *at += 2;
                Atom::Escape(*chars.get(*at - 1)?)
            }
            '[' => {
                *at += 1;
                class(chars, at)?
            }
            '(' => {
                *at += 1;
                let inner = sequence(chars, at)?;
                if chars.get(*at) != Some(&')') {
                    return None;
                }
                *at += 1;
                Atom::Group(inner)
            }
            '.' => {
                *at += 1;
                Atom::Any
            }
            literal => {
                *at += 1;
                Atom::Literal(literal)
            }
        };
        let (min, max) = match chars.get(*at) {
            Some('?') => (0, Some(1)),
            Some('*') => (0, None),
            Some('+') => (1, None),
            Some('{') => {
                *at += 1;
                let low = number(chars, at)?;
                let high = if chars.get(*at) == Some(&',') {
                    *at += 1;
                    number(chars, at)
                } else {
                    Some(low)
                };
                if chars.get(*at) != Some(&'}') {
                    return None;
                }
                (low, high)
            }
            _ => {
                nodes.push(Node {
                    atom,
                    min: 1,
                    max: Some(1),
                });
                continue;
            }
        };
        *at += 1;
        nodes.push(Node { atom, min, max });
    }
    Some(nodes)
}

/// Parse the pattern subset the composer emits; `None` outside it.
fn parse(pattern: &str) -> Option<Pattern> {
    let mut chars: Vec<char> = pattern.chars().collect();
    let start = chars.first() == Some(&'^');
    if start {
        chars.remove(0);
    }
    let end = chars.last() == Some(&'$') && chars.len() >= 2 && chars[chars.len() - 2] != '\\';
    if end {
        chars.pop();
    }
    let mut at = 0usize;
    let nodes = sequence(&chars, &mut at)?;
    (at == chars.len()).then_some(Pattern { nodes, start, end })
}

/// Whether one character satisfies a single-character atom.
fn accepts(atom: &Atom, ch: char) -> bool {
    match atom {
        Atom::Literal(literal) => *literal == ch,
        Atom::Any => ch != '\n',
        Atom::Escape('d') => ch.is_ascii_digit(),
        Atom::Escape('w') => ch.is_ascii_alphanumeric() || ch == '_',
        Atom::Escape('s') => ch.is_whitespace(),
        Atom::Escape(literal) => *literal == ch,
        Atom::Class(ranges, negated) => {
            ranges.iter().any(|(low, high)| *low <= ch && ch <= *high) != *negated
        }
        Atom::Group(_) => false,
    }
}

/// Every position one occurrence of `atom` can end at, from `from`.
fn atom_ends(atom: &Atom, text: &[char], from: usize) -> Vec<usize> {
    match atom {
        Atom::Group(nodes) => sequence_ends(nodes, text, from),
        single => text
            .get(from)
            .filter(|ch| accepts(single, **ch))
            .map_or_else(Vec::new, |_| vec![from + 1]),
    }
}

/// Every position a repeated node can end at, from `from`.
fn node_ends(node: &Node, text: &[char], from: usize) -> Vec<usize> {
    let mut result = if node.min == 0 {
        vec![from]
    } else {
        Vec::new()
    };
    let mut current = vec![from];
    let limit = node.max.unwrap_or(text.len() + 1);
    for count in 1..=limit {
        let mut next: Vec<usize> = current
            .iter()
            .flat_map(|position| atom_ends(&node.atom, text, *position))
            .collect();
        next.sort_unstable();
        next.dedup();
        if next.is_empty() || next == current {
            break;
        }
        if count >= node.min {
            result.extend(next.iter().copied());
        }
        current = next;
    }
    result.sort_unstable();
    result.dedup();
    result
}

/// Every position a node sequence can end at, from `from`.
fn sequence_ends(nodes: &[Node], text: &[char], from: usize) -> Vec<usize> {
    let mut positions = vec![from];
    for node in nodes {
        let mut next: Vec<usize> = positions
            .iter()
            .flat_map(|position| node_ends(node, text, *position))
            .collect();
        next.sort_unstable();
        next.dedup();
        positions = next;
    }
    positions
}

/// Whether the pattern finds a match in `text` (search semantics, as
/// `RegExp.test`); `None` when the pattern is outside the subset.
fn finds(pattern: &str, text: &str) -> Option<bool> {
    let parsed = parse(pattern)?;
    let chars: Vec<char> = text.chars().collect();
    let starts = if parsed.start { 0..=0 } else { 0..=chars.len() };
    Some(starts.into_iter().any(|start| {
        sequence_ends(&parsed.nodes, &chars, start)
            .iter()
            .any(|end| !parsed.end || *end == chars.len())
    }))
}

/// One class mention as the witness derivation reads it.
pub(super) struct WitnessPart {
    pub class: String,
    pub count: u32,
    pub at_least: bool,
    pub optional: bool,
    /// The separator pattern written before this part (empty for none).
    pub separator: String,
}

/// The first probe character a class pattern accepts (`want`) or rejects.
fn probe(class: &str, want: bool) -> Option<char> {
    let anchored = ["^", class, "$"].concat();
    ['7', 'a', 'Q', '_', '-', ' ', '\n']
        .into_iter()
        .find(|ch| finds(&anchored, &ch.to_string()) == Some(want))
}

/// Positive and negative examples derived from the constraints, the same
/// sets the browser twin derives; `None` when a class has no probe.
pub(super) fn examples(
    parts: &[WitnessPart],
    anchored: bool,
) -> Option<(Vec<String>, Vec<String>)> {
    let mut probes = Vec::new();
    for part in parts {
        probes.push((probe(&part.class, true)?, probe(&part.class, false)?));
    }
    let prefix = |part: &WitnessPart| {
        part.separator
            .strip_prefix('\\')
            .unwrap_or(&part.separator)
            .to_owned()
    };
    let render = |overrides: &[(usize, Option<String>)]| -> String {
        let mut text = String::new();
        for (index, part) in parts.iter().enumerate() {
            match overrides.iter().find(|(at, _)| *at == index) {
                Some((_, None)) => {}
                Some((_, Some(value))) => {
                    text.push_str(&prefix(part));
                    text.push_str(value);
                }
                None => {
                    text.push_str(&prefix(part));
                    text.push_str(&probes[index].0.to_string().repeat(part.count as usize));
                }
            }
        }
        text
    };
    let mut positives = vec![render(&[])];
    let omitted: Vec<(usize, Option<String>)> = parts
        .iter()
        .enumerate()
        .skip(1)
        .filter(|(_, part)| part.optional)
        .map(|(index, _)| (index, None))
        .collect();
    if !omitted.is_empty() {
        positives.push(render(&omitted));
    }
    for (index, part) in parts.iter().enumerate().filter(|(_, part)| part.at_least) {
        let longer = probes[index].0.to_string().repeat(part.count as usize + 2);
        positives.push(render(&[(index, Some(longer))]));
    }
    let mut negatives = Vec::new();
    let limit = if anchored { parts.len() } else { 1 };
    for (index, part) in parts.iter().enumerate().take(limit) {
        if part.count == 0 {
            continue;
        }
        let good = probes[index].0.to_string();
        let shorter = good.repeat(part.count as usize - 1);
        negatives.push(render(&[(index, Some(shorter.clone()))]));
        negatives.push(render(&[(
            index,
            Some([probes[index].1.to_string(), shorter].concat()),
        )]));
    }
    if anchored && parts.last().is_some_and(|last| !last.at_least) {
        let tail = probes[parts.len() - 1].0;
        negatives.push([render(&[]), tail.to_string()].concat());
    }
    let mut unique_positives: Vec<String> = Vec::new();
    for sample in positives {
        if !unique_positives.contains(&sample) {
            unique_positives.push(sample);
        }
    }
    let mut unique_negatives: Vec<String> = Vec::new();
    for sample in negatives {
        if !unique_negatives.contains(&sample) && !unique_positives.contains(&sample) {
            unique_negatives.push(sample);
        }
    }
    Some((unique_positives, unique_negatives))
}

/// Match the pattern against the examples: the failures, one per example the
/// pattern judges against its expectation; `None` outside the subset.
pub(super) fn check(
    pattern: &str,
    positives: &[String],
    negatives: &[String],
) -> Option<Vec<String>> {
    let mut failures = Vec::new();
    for sample in positives {
        if !finds(pattern, sample)? {
            failures.push(["+", &quote_list(std::slice::from_ref(sample))].concat());
        }
    }
    for sample in negatives {
        if finds(pattern, sample)? {
            failures.push(["-", &quote_list(std::slice::from_ref(sample))].concat());
        }
    }
    Some(failures)
}

/// Quote a list of examples as the browser twin does (`JSON.stringify`).
pub(super) fn quote_list(samples: &[String]) -> String {
    samples
        .iter()
        .map(|sample| serde_json::to_string(sample).unwrap_or_default())
        .collect::<Vec<_>>()
        .join(", ")
}
