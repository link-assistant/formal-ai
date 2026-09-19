//! The expression pool: atom expressions and lifted literals the enumeration grows from.

use super::*;

#[derive(Debug, Clone)]
pub(super) struct Expression {
    pub(super) node: IrNode,
    pub(super) ty: IrType,
    pub(super) fragments: Vec<String>,
    pub(super) coverage: BTreeSet<String>,
    pub(super) depth: usize,
    /// How many comprehensions inside this expression map a constant: a body
    /// that never reads its binder cannot depend on the iteration, so such an
    /// expression is filler for any requirement about the individual items.
    pub(super) constant_maps: usize,
    /// How many slots anywhere in this tree were filled with a text value
    /// where the fragment demands a sequence of a concrete non-text element.
    /// Iterating text as characters is free (the element is unknown or text);
    /// feeding raw text where integers are expected is a guess about what the
    /// iteration produces, and guesses rank below grounded assemblies.
    pub(super) loose: usize,
    pub(super) grounding: Vec<(String, String)>,
    /// Whether this literal was lifted from an example's expected output. Such
    /// an atom carries the answer's vocabulary, not available data: the slot
    /// filter admits it only as the payload of a conditional fragment, never
    /// as a condition operand — otherwise the search would memorize which
    /// example input produced which label instead of deriving the condition.
    pub(super) observation: bool,
}

pub(super) fn atom_expressions(
    spec: &CodingTaskSpec,
    parameters: &[(String, IrType)],
    fragments: &[&Fragment],
    structure_ids: &[String],
) -> Vec<Expression> {
    let mut atoms = parameters
        .iter()
        .map(|(name, ty)| Expression {
            node: IrNode::Parameter {
                name: name.clone(),
                ty: ty.clone(),
            },
            ty: ty.clone(),
            fragments: Vec::new(),
            coverage: BTreeSet::new(),
            depth: 1,
            constant_maps: 0,
            loose: 0,
            grounding: Vec::new(),
            observation: false,
        })
        .collect::<Vec<_>>();
    for fragment in fragments {
        if fragment.signature.is_empty() {
            atoms.push(Expression {
                node: IrNode::Apply {
                    fragment: fragment.id.clone(),
                    arguments: Vec::new(),
                },
                ty: fragment.result.clone(),
                fragments: vec![fragment.id.clone()],
                coverage: fragment_coverage(fragment, structure_ids),
                depth: 1,
                constant_maps: 0,
                loose: 0,
                grounding: vec![(fragment.grounding.clone(), fragment.license.clone())],
                observation: false,
            });
        }
    }
    // A program's declared stdout is the task's own output slot — stated by
    // the task itself, the same class of input as its examples. An
    // output-display structure has nothing else to ground on: a program has
    // no parameters, so the stated output is the only available atom for the
    // display fragment's payload slot. The executable oracle still has to
    // pass it.
    if spec.artifact_shape == ArtifactShape::Program
        && let Some(expected) = &spec.expected_stdout
        && let Ok(text) = serde_json::to_string(expected)
    {
        atoms.push(literal(&text, IrType::Text));
    }
    atoms.extend(discovered_literals(spec));
    normalize_pool(&mut atoms, 0, 512);
    atoms
}

pub(super) fn discovered_literals(spec: &CodingTaskSpec) -> Vec<Expression> {
    let mut literals = Vec::new();
    for value in 0..=8 {
        let text = value.to_string();
        literals.push(literal(&text, IrType::Integer));
        literals.push(literal(&text, IrType::Float));
    }
    let prose = spec.requirement_sentences.join(" ");
    for token in prose.split(|character: char| !character.is_ascii_digit() && character != '.') {
        if token.is_empty() {
            continue;
        }
        if token.parse::<u64>().is_ok() {
            literals.push(literal(token, IrType::Integer));
        } else if token.parse::<f64>().is_ok() {
            literals.push(literal(token, IrType::Float));
        }
    }
    let quoted = quoted_literals(&prose);
    for value in &quoted {
        let serialized = serde_json::to_string(value).expect("text literal serializes");
        literals.push(literal(&serialized, IrType::Text));
    }
    // An expected output is the answer, not available data — composing with a
    // whole expected value would memorize the example. What an observation may
    // still contribute is the answer's vocabulary: the label a classification
    // returns. Those atoms are flagged `observation`, and the slot filter only
    // admits them as the payload slots of a conditional fragment — the
    // condition itself must still be discovered from the catalog's sources,
    // so 'accepted' names a branch while `text == 'teal_blue'` can never form.
    for example in &spec.examples {
        for value in quoted_literals(&example.expected) {
            let serialized = serde_json::to_string(&value).expect("text literal serializes");
            let mut atom = literal(&serialized, IrType::Text);
            atom.observation = true;
            literals.push(atom);
        }
    }
    if quoted.len() >= 3 {
        let fields = quoted
            .iter()
            .enumerate()
            .map(|(index, value)| {
                format!(
                    "{}: {index}",
                    serde_json::to_string(value).expect("text literal serializes")
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        literals.push(literal(
            &format!("{{{fields}}}"),
            IrType::Mapping(Box::new(IrType::Text), Box::new(IrType::Integer)),
        ));
    }
    let associations = quoted
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            let start = prose.find(value)? + value.len();
            let end = quoted
                .get(index + 1)
                .and_then(|next| prose[start..].find(next).map(|offset| start + offset))
                .unwrap_or(prose.len());
            let number = prose[start..end]
                .split(|character: char| !character.is_ascii_digit())
                .find(|part| !part.is_empty())?;
            Some(format!(
                "{}: {number}",
                serde_json::to_string(value).expect("text literal serializes")
            ))
        })
        .collect::<Vec<_>>();
    if associations.len() >= 2 {
        literals.push(literal(
            &format!("{{{}}}", associations.join(", ")),
            IrType::Mapping(Box::new(IrType::Text), Box::new(IrType::Integer)),
        ));
    }
    literals
}

pub(super) fn literal(text: &str, ty: IrType) -> Expression {
    Expression {
        node: IrNode::Literal {
            text: text.to_owned(),
            ty: ty.clone(),
        },
        ty,
        fragments: Vec::new(),
        coverage: BTreeSet::new(),
        depth: 1,
        constant_maps: 0,
        loose: 0,
        grounding: Vec::new(),
        observation: false,
    }
}

pub(super) fn quoted_literals(text: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut opening: Option<(char, usize)> = None;
    for (index, character) in text.char_indices() {
        if !matches!(character, '\'' | '"') {
            continue;
        }
        if let Some((delimiter, start)) = opening {
            if delimiter == character {
                values.push(text[start + delimiter.len_utf8()..index].to_owned());
                opening = None;
            }
        } else {
            opening = Some((character, index));
        }
    }
    values
}
