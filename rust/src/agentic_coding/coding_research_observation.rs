//! Lossless lexical/catalog normalization; generated kernels make research decisions.
use crate::es_tokenizer::{TokenKind, Tree};
use serde_json::{Value, json};

struct LexicalToken {
    start: usize,
    end: usize,
    text: String,
    kind: &'static str,
}
impl<'a> crate::callback_values::CallbackTextReceiver for &'a LexicalToken {
    type Output = &'a String;
    fn text(self) -> Self::Output {
        &self.text
    }
}
fn lexical_tokens(trees: &[Tree<'_>], output: &mut Vec<LexicalToken>) {
    for tree in trees {
        let span = tree.span();
        match tree {
            Tree::Leaf(token) => {
                let kind = match token.kind {
                    TokenKind::Identifier => "identifier",
                    TokenKind::Numeric => "numeric",
                    TokenKind::String => "string",
                    TokenKind::TemplateChunk => "chunk",
                    TokenKind::RegExp => "regexp",
                    TokenKind::Punctuator => "punctuator",
                };
                output.push(LexicalToken {
                    start: span.start,
                    end: span.end,
                    text: token.text.to_owned(),
                    kind,
                });
            }
            Tree::Group { delim, trees, .. } => {
                output.push(LexicalToken {
                    start: span.start,
                    end: span.start + 1,
                    text: char::from(delim.opens()).to_string(),
                    kind: "punctuator",
                });
                lexical_tokens(trees, output);
                output.push(LexicalToken {
                    start: span.end - 1,
                    end: span.end,
                    text: char::from(delim.closes()).to_string(),
                    kind: "punctuator",
                });
            }
            Tree::Template { .. } => output.push(LexicalToken {
                start: span.start,
                end: span.end,
                text: String::new(),
                kind: "unsupported",
            }),
        }
    }
}
/// Normalize balanced token trees without parsing an assertion contract.
#[must_use]
pub fn observe_coding_research_acceptance_tokens(source: &str) -> Value {
    let Ok(trees) = crate::es_tokenizer::tokenize(source) else {
        return json!({"texts":[], "kinds":[], "complete":false});
    };
    let mut tokens = Vec::new();
    lexical_tokens(&trees, &mut tokens);
    json!({"texts":tokens.iter().map(crate::callback_values::callback_text).collect::<Vec<_>>(),
        "kinds":tokens.iter().map(|token| token.kind).collect::<Vec<_>>(), "complete":true})
}
/// Observe parser/catalog/spans without selecting, accepting or executing a procedure.
#[must_use]
pub fn observe_coding_research_rebinding_candidate(
    source: &str,
    language: &str,
    module_path: &str,
) -> Value {
    let catalog =
        super::module_function::callable_catalog::observe_source_callables(source, module_path);
    let declaration = &catalog["declarations"][0];
    let text = declaration["source"].as_str().unwrap_or_default();
    let mut tokens = Vec::new();
    let mut lexical = crate::es_tokenizer::tokenize(text).is_ok_and(|trees| {
        lexical_tokens(&trees, &mut tokens);
        true
    });
    let mut pieces = Vec::new();
    let mut bindings = Vec::new();
    let mut cursor = 0;
    for token in tokens {
        if token.start < cursor
            || token.end < token.start
            || token.end > text.len()
            || !text.is_char_boundary(token.start)
            || !text.is_char_boundary(token.end)
        {
            lexical = false;
            break;
        }
        if token.start > cursor {
            pieces.push(text[cursor..token.start].to_owned());
            bindings.push(String::new());
        }
        pieces.push(text[token.start..token.end].to_owned());
        bindings.push(if token.kind == "identifier" {
            token.text
        } else {
            String::new()
        });
        cursor = token.end;
    }
    if cursor < text.len() {
        pieces.push(text[cursor..].to_owned());
        bindings.push(String::new());
    }
    let exported = catalog["exports"].as_array().is_some_and(|items| {
        items
            .iter()
            .any(|item| item["local"] == declaration["name"])
    });
    json!({"$":"candidate", "language":language,
        "syntaxValid":lexical && crate::coding::cst::validated_program_cst(language, source).is_some(),
        "moduleEffectsNone":catalog["moduleEffects"] == "none",
        "moduleSyntaxSupported":catalog["moduleSyntax"] == "supported",
        "importsCount":catalog["imports"].as_array().map_or(0, Vec::len),
        "declarationsCount":catalog["declarations"].as_array().map_or(0, Vec::len),
        "contractSupported":declaration["contract"]["status"] == "supported",
        "callEffectsNone":declaration["contract"]["callEffects"] == "none",
        "name":declaration["name"].as_str().unwrap_or_default(),
        "parameters":declaration["parameters"].as_array().cloned().unwrap_or_default(),
        "exported":exported, "pieces":pieces, "pieceBindings":bindings})
}
