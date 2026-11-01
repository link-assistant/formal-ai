//! Manual native projection of finite own-key dictionary repair; CI only.
use super::observed_source_usage::view::{
    View, children, concrete, field, form, one, role, trivia,
};
use super::source_model::{
    owned_source_links, source_from_links, source_to_links, verify_source_links,
};
use crate::source_fetch::sha256_hex;
use serde_json::{Value, json};
const MAXIMUM_DICTIONARY_CANDIDATES: usize = 32;
const KNOWLEDGE: &str = include_str!("../../embedded/data/meta/ecmascript-own-key-dictionary.json");
fn is(node: Option<&Value>, key: &str, knowledge: &Value) -> bool {
    node.is_some_and(|node| {
        knowledge["facets"][key]
            .as_array()
            .is_some_and(|forms| forms.iter().any(|kind| kind.as_str() == Some(form(node))))
    })
}
fn parent<'a>(view: &View<'a>, node: &Value) -> Option<&'a Value> {
    view.record(node)?.ancestors.last().copied()
}
fn contains(view: &View<'_>, ancestor: &Value, node: &Value) -> bool {
    std::ptr::eq(ancestor, node)
        || view.record(node).is_some_and(|record| {
            record
                .ancestors
                .iter()
                .any(|item| std::ptr::eq(*item, ancestor))
        })
}
fn simple(node: Option<&Value>, knowledge: &Value) -> Option<String> {
    let node = node?;
    (is(Some(node), "identifier", knowledge) && field(node, "text").is_some())
        .then(|| concrete(node))
}
fn computed(node: Option<&Value>, knowledge: &Value) -> bool {
    is(node, "member", knowledge)
        && node.is_some_and(|node| children(node).iter().any(|child| concrete(child) == "["))
}
struct Selected {
    binding: String,
    start: usize,
    end: usize,
    references: Vec<Value>,
    writes: usize,
    reads: usize,
    spreads: usize,
}
fn select(
    source: &str,
    document: &Value,
    knowledge: &Value,
    repaired: bool,
) -> Result<Selected, String> {
    let view = View::new(source, document)?;
    if view.records.iter().any(|record| {
        form(record.node) == "with_statement"
            || simple(Some(record.node), knowledge).is_some_and(|name| {
                knowledge["dynamicLookup"].as_array().is_some_and(|names| {
                    names
                        .iter()
                        .any(|value| value.as_str() == Some(name.as_str()))
                })
            })
    }) {
        return Err("DictionaryDynamicScopeUnsupported".to_owned());
    }
    let intrinsic = knowledge["intrinsic"]
        .as_str()
        .ok_or("DictionaryKnowledgeUnavailable")?;
    for record in view
        .records
        .iter()
        .filter(|record| simple(Some(record.node), knowledge).as_deref() == Some(intrinsic))
    {
        let member = parent(&view, record.node);
        if !is(member, "member", knowledge)
            || member
                .and_then(|node| one(node, &["object"]))
                .is_none_or(|node| !std::ptr::eq(node, record.node))
            || computed(member, knowledge)
        {
            return Err("DictionaryIntrinsicShadowedOrEscaped".to_owned());
        }
        let member = member.ok_or("DictionaryIntrinsicShadowedOrEscaped")?;
        let upper = parent(&view, member);
        if (is(upper, "assignment", knowledge)
            && upper
                .and_then(|node| one(node, &["left"]))
                .is_some_and(|node| std::ptr::eq(node, member)))
            || upper
                .is_some_and(|node| matches!(form(node), "update_expression" | "unary_expression"))
        {
            return Err("DictionaryIntrinsicMutationUnsupported".to_owned());
        }
    }
    let mut candidates = Vec::new();
    let mut eligible_bindings = 0;
    for record in view
        .records
        .iter()
        .filter(|record| is(Some(record.node), "declarator", knowledge))
    {
        let declaration = parent(&view, record.node);
        let binding = one(record.node, &["name"]);
        let initializer = one(record.node, &["init", "value"]);
        if !is(declaration, "declaration", knowledge)
            || !declaration.is_some_and(|node| {
                children(node)
                    .iter()
                    .any(|child| concrete(child) == "const")
            })
        {
            continue;
        }
        let Some(initializer) = initializer else {
            continue;
        };
        let Some(name) = simple(binding, knowledge) else {
            continue;
        };
        if name == intrinsic {
            continue;
        }
        let text = concrete(initializer);
        let recipe = knowledge["initializer"]
            .as_str()
            .ok_or("DictionaryKnowledgeUnavailable")?;
        if if repaired {
            text.chars()
                .filter(|character| !crate::meta_reasoner::text::is_js_whitespace(*character))
                .collect::<String>()
                != recipe
        } else {
            !is(Some(initializer), "object", knowledge)
                || text
                    .strip_prefix('{')
                    .and_then(|value| value.strip_suffix('}'))
                    .is_none_or(|body| {
                        !body
                            .chars()
                            .all(crate::meta_reasoner::text::is_js_whitespace)
                    })
        } {
            continue;
        }
        let block = record
            .ancestors
            .iter()
            .rev()
            .find(|node| is(Some(node), "block", knowledge))
            .copied();
        let function = record
            .ancestors
            .iter()
            .rev()
            .find(|node| is(Some(node), "function", knowledge))
            .copied();
        let (Some(block), Some(function)) = (block, function) else {
            continue;
        };
        eligible_bindings += 1;
        if eligible_bindings > MAXIMUM_DICTIONARY_CANDIDATES {
            return Err("DictionarySelectionBudgetExceeded".to_owned());
        }
        let mut writes = 0;
        let mut reads = 0;
        let mut spreads = 0;
        let mut supported = true;
        let mut references = Vec::new();
        for occurrence in view
            .records
            .iter()
            .filter(|other| simple(Some(other.node), knowledge).as_deref() == Some(name.as_str()))
        {
            let node = occurrence.node;
            if binding.is_some_and(|binding| std::ptr::eq(binding, node)) {
                continue;
            }
            if !occurrence
                .ancestors
                .iter()
                .any(|ancestor| std::ptr::eq(*ancestor, block))
            {
                supported = false;
                break;
            }
            let upper = parent(&view, node);
            if is(upper, "spread", knowledge)
                && upper
                    .and_then(|upper| {
                        one(upper, &["argument"]).or_else(|| {
                            children(upper)
                                .into_iter()
                                .find(|child| !trivia(child) && concrete(child) != "...")
                        })
                    })
                    .is_some_and(|child| std::ptr::eq(child, node))
                && upper
                    .and_then(|upper| parent(&view, upper))
                    .is_some_and(|node| is(Some(node), "object", knowledge))
            {
                spreads += 1;
                references
                    .push(json!({"span":[occurrence.start,occurrence.end],"kind":"own-spread"}));
                continue;
            }
            if !computed(upper, knowledge)
                || upper
                    .and_then(|node| one(node, &["object"]))
                    .is_none_or(|child| !std::ptr::eq(child, node))
                || upper
                    .and_then(|node| one(node, &["property", "index"]))
                    .is_none()
            {
                supported = false;
                break;
            }
            let upper = upper.ok_or("DictionaryReferenceUnsupported")?;
            let container = parent(&view, upper);
            if is(container, "assignment", knowledge)
                && container
                    .and_then(|node| one(node, &["left"]))
                    .is_some_and(|node| std::ptr::eq(node, upper))
            {
                let operators: Vec<_> =
                    children(container.ok_or("DictionaryReferenceUnsupported")?)
                        .into_iter()
                        .filter(|child| {
                            !role(child, "left")
                                && !role(child, "right")
                                && matches!(
                                    concrete(child).as_str(),
                                    "=" | "+=" | "-=" | "*=" | "/=" | "??=" | "||=" | "&&="
                                )
                        })
                        .collect();
                if operators.len() != 1 || concrete(operators[0]) != "=" {
                    supported = false;
                    break;
                }
                writes += 1;
                references.push(
                    json!({"span":[occurrence.start,occurrence.end],"kind":"computed-write"}),
                );
                continue;
            }
            if container
                .is_some_and(|node| matches!(form(node), "update_expression" | "unary_expression"))
            {
                supported = false;
                break;
            }
            if occurrence.ancestors.iter().any(|ancestor| {
                is(Some(ancestor), "call", knowledge)
                    && ["callee", "function", "constructor", "tag"]
                        .iter()
                        .any(|role| {
                            one(ancestor, &[*role])
                                .is_some_and(|called| contains(&view, called, upper))
                        })
            }) {
                supported = false;
                break;
            }
            reads += 1;
            references
                .push(json!({"span":[occurrence.start,occurrence.end],"kind":"computed-read"}));
        }
        if !supported || writes == 0 || reads + spreads == 0 {
            continue;
        }
        if view.records.iter().any(|other| {
            contains(&view, function, other.node)
                && (field(other.node, "meaning").is_some()
                    || field(other.node, "semantic-gap").is_some())
        }) {
            return Err("DictionaryUnresolvedSourceMeaning".to_owned());
        }
        let selected = view
            .record(initializer)
            .ok_or("DictionaryInitializerSpanRequired")?;
        candidates.push(Selected {
            binding: name,
            start: selected.start,
            end: selected.end,
            references,
            writes,
            reads,
            spreads,
        });
    }
    if candidates.len() != 1 {
        return Err(if candidates.is_empty() {
            "DictionaryBindingUnsupported"
        } else {
            "DictionaryBindingAmbiguous"
        }
        .to_owned());
    }
    candidates
        .pop()
        .ok_or_else(|| "DictionaryBindingUnsupported".to_owned())
}
pub(super) fn derive_own_key_dictionary_repair(owner: &str, source: &str) -> Result<Value, String> {
    let knowledge: Value =
        serde_json::from_str(KNOWLEDGE).map_err(|_| "DictionaryKnowledgeUnavailable")?;
    let intrinsic = knowledge["intrinsic"]
        .as_str()
        .ok_or("DictionaryKnowledgeUnavailable")?;
    let method = knowledge["method"]
        .as_str()
        .ok_or("DictionaryKnowledgeUnavailable")?;
    let recipe = knowledge["initializer"]
        .as_str()
        .ok_or("DictionaryKnowledgeUnavailable")?;
    if knowledge["schema"] != "ecmascript-own-key-dictionary-knowledge/v1"
        || knowledge["language"] != "JavaScript"
        || knowledge["operation"] != "own-key-storage"
        || !knowledge["prototype"].is_null()
        || recipe != format!("{intrinsic}.{method}(null)")
        || knowledge["sources"]
            .as_array()
            .is_none_or(|rows| rows.len() < 2)
    {
        return Err("DictionaryKnowledgeUnavailable".to_owned());
    }
    static OWNER: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    if !OWNER
        .get_or_init(|| {
            regex::Regex::new(r"^[A-Za-z0-9_][A-Za-z0-9_./-]*\.(?:mjs|js)$")
                .expect("StaticDictionaryOwner")
        })
        .is_match(owner)
        || owner.split('/').any(|part| matches!(part, "" | "." | ".."))
        || source.len() > 262_144
    {
        return Err("DictionaryObservedSourceRequired".to_owned());
    }
    let document = owned_source_links(owner, &source_to_links(source, "JavaScript")?)?;
    verify_source_links(&document)?;
    if source_from_links(&document)? != source {
        return Err("DictionarySourceReconstructionMismatch".to_owned());
    }
    let selected = select(source, &document, &knowledge, false)?;
    let prefix = &source[..selected.start];
    let before = &source[selected.start..selected.end];
    let suffix = &source[selected.end..];
    let replacement = format!("{intrinsic}.{method}({}null)", &before[1..before.len() - 1]);
    let trivia = |text: &str| {
        text.chars()
            .filter(|character| crate::meta_reasoner::text::is_js_whitespace(*character))
            .collect::<String>()
    };
    if trivia(before) != trivia(&replacement) {
        return Err("DictionaryInitializerTriviaMismatch".to_owned());
    }
    let candidate = format!("{prefix}{replacement}{suffix}");
    if candidate.len() > 262_144 {
        return Err("DictionaryCandidateBudgetExceeded".to_owned());
    }
    let changed = owned_source_links(owner, &source_to_links(&candidate, "JavaScript")?)?;
    verify_source_links(&changed)?;
    if source_from_links(&changed)? != candidate {
        return Err("DictionaryCandidateReconstructionMismatch".to_owned());
    }
    let after = select(&candidate, &changed, &knowledge, true)?;
    if after.binding != selected.binding
        || after.writes != selected.writes
        || after.reads != selected.reads
        || after.spreads != selected.spreads
    {
        return Err("DictionaryBindingProofMismatch".to_owned());
    }
    let restored = format!(
        "{}{before}{}",
        &candidate[..prefix.len()],
        &candidate[prefix.len() + replacement.len()..]
    );
    if restored != source
        || owned_source_links(owner, &source_to_links(&restored, "JavaScript")?)? != document
    {
        return Err("DictionaryExactInverseMismatch".to_owned());
    }
    Ok(
        json!({"schema":"observed-own-key-dictionary-repair/v1","owner":owner,"candidate":{"path":owner,
        "source":candidate,"beforeSha256":sha256_hex(source.as_bytes()),"afterSha256":sha256_hex(candidate.as_bytes()),
        "changed":true},"binding":{"name":selected.binding,"initializerSpan":[selected.start,selected.end],"references":selected.references},
        "knowledge":{"schema":knowledge["schema"],"sources":knowledge["sources"],"assumptions":knowledge["assumptions"]},
        "verification":{"schema":"own-key-dictionary-source-proof/v1","completeConcreteResidual":true,"initializerTriviaPreserved":true,
        "independentGrammarReparse":true,"exactInverse":true,"writeCount":selected.writes,"readCount":selected.reads,
        "spreadCount":selected.spreads,"scope":knowledge["claim"],"arbitrarySemanticEquivalence":false,"sourceReflectionEquivalence":false,
        "compilerValidity":"not-checked","authorshipCredit":0},"authored":false,"verified":false,"independentAcceptance":false}),
    )
}

#[cfg(test)]
#[path = "../../tests/unit/agentic-coding/source_dictionary_binding.rs"]
mod tests;
