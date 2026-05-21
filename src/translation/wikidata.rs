//! Wikidata SPARQL + Lexeme client.
//!
//! Wikidata is the language-neutral identity layer. We use it to:
//!
//! 1. Find the Q-item that backs a Wiktionary page (so two Wiktionary
//!    pages in different languages can collapse onto the same id).
//! 2. Pull translation pairs through the Lexeme graph using P5972
//!    ("translation") and P5137 ("item for this sense").
//!
//! The SPARQL endpoint is `https://query.wikidata.org/sparql`. We pass
//! the query in the `query=` parameter and request JSON via the
//! `format=json` parameter (also accepted as `Accept: application/json`,
//! but a URL parameter keeps the curl invocation simple).

use super::http::{HttpClient, HttpError};
use super::wiktionary::query_encode;

/// Wikidata client.
pub struct Wikidata<'a, T: HttpClient + ?Sized> {
    http: &'a T,
}

impl<'a, T: HttpClient + ?Sized> Wikidata<'a, T> {
    pub const fn new(http: &'a T) -> Self {
        Self { http }
    }

    /// Run a SPARQL query against the Wikidata Query Service. Returns
    /// the raw JSON body. SPARQL `SELECT` results follow the structure
    /// `{ "results": { "bindings": [ {"var": {"value": "..."}}, ... ] } }`.
    pub fn sparql(&self, query: &str) -> Result<String, HttpError> {
        // SPARQL queries contain whitespace-separated keywords; we must
        // use percent-encoding for spaces (RFC 3986) rather than the
        // Wikimedia page-title convention (`_`). The endpoint returns
        // HTTP 400 if underscores replace SPARQL whitespace.
        let url = format!(
            "https://query.wikidata.org/sparql?format=json&query={query}",
            query = query_encode(query),
        );
        self.http.get(&url)
    }

    /// Find translations of a Lexeme into `target_lang` by joining on
    /// the shared "item for this sense" (P5137).
    ///
    /// This is the canonical Wikidata translation pivot: two lexemes
    /// translate each other when they share at least one sense whose
    /// `P5137` value is the same Q-item. The query is robust because it
    /// does not depend on P5972 ("translation") which is sparsely
    /// populated.
    pub fn lexeme_translations(
        &self,
        source_lexeme_id: &str,
        target_lang_iso: &str,
    ) -> Result<Vec<SparqlLemma>, HttpError> {
        let query = format!(
            "SELECT DISTINCT ?lemma WHERE {{ \
               wd:{source_lexeme_id} ontolex:sense ?source_sense . \
               ?source_sense wdt:P5137 ?meaning . \
               ?lexeme ontolex:sense ?sense . \
               ?sense wdt:P5137 ?meaning . \
               ?lexeme dct:language ?language . \
               ?language wdt:P218 \"{target_lang_iso}\" . \
               ?lexeme wikibase:lemma ?lemma . \
             }}"
        );
        let body = self.sparql(&query)?;
        Ok(parse_sparql_lemmas(&body))
    }

    /// Search for a Lexeme by lemma on a specific language.
    pub fn search_lexeme(
        &self,
        lemma: &str,
        language_iso: &str,
    ) -> Result<Vec<LexemeSearchHit>, HttpError> {
        let url = format!(
            "https://www.wikidata.org/w/api.php?action=wbsearchentities\
             &search={lemma}&language={lang}&type=lexeme&format=json\
             &uselang={lang}&limit=5",
            lemma = query_encode(lemma),
            lang = query_encode(language_iso),
        );
        let body = self.http.get(&url)?;
        Ok(parse_wbsearch_hits(&body))
    }
}

/// One row of a SPARQL SELECT result for `?lemma`. The `xml:lang` tag is
/// preserved so callers can distinguish multiple forms returned for the
/// same lexeme.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SparqlLemma {
    pub value: String,
    pub language: Option<String>,
}

/// One hit from `action=wbsearchentities&type=lexeme`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexemeSearchHit {
    pub id: String,
    pub label: String,
}

/// Parse the bindings array of a SPARQL `SELECT ?lemma` result.
///
/// We do a targeted string scan rather than full JSON parsing — the
/// structure is fixed and pulling in `serde_json::Value` for one shape
/// would be heavyweight. The scan handles unicode escapes via the
/// wiktionary JSON-string decoder helper.
#[must_use]
pub fn parse_sparql_lemmas(body: &str) -> Vec<SparqlLemma> {
    let mut out: Vec<SparqlLemma> = Vec::new();
    let mut cursor = 0usize;
    while let Some(start) = body[cursor..].find("\"lemma\"") {
        let absolute = cursor + start;
        // The lemma binding is encoded as a JSON object such as
        // `{"xml:lang": "ru", "type": "literal", "value": "привет"}`. The
        // order of keys is not guaranteed, so we scan the whole object body
        // up to the matching `}` and pull out `value` and `xml:lang`.
        let Some(open_brace_offset) = body[absolute..].find('{') else {
            break;
        };
        let object_start = absolute + open_brace_offset;
        let Some(object_end) = find_matching_brace(&body[object_start..]) else {
            break;
        };
        let object_body = &body[object_start..object_start + object_end];
        let value = read_json_field(object_body, "value").unwrap_or_default();
        let language = read_json_field(object_body, "xml:lang");
        out.push(SparqlLemma { value, language });
        cursor = object_start + object_end;
    }
    out
}

/// Find the matching `}` for the opening `{` at `input[0]`. Returns the
/// byte offset of the closing brace (inclusive of itself).
fn find_matching_brace(input: &str) -> Option<usize> {
    let bytes = input.as_bytes();
    if bytes.is_empty() || bytes[0] != b'{' {
        return None;
    }
    let mut depth: u32 = 0;
    let mut in_string = false;
    let mut escape = false;
    for (offset, byte) in bytes.iter().enumerate() {
        if escape {
            escape = false;
            continue;
        }
        match (*byte, in_string) {
            (b'\\', true) => escape = true,
            (b'"', _) => in_string = !in_string,
            (b'{', false) => depth += 1,
            (b'}', false) => {
                depth -= 1;
                if depth == 0 {
                    return Some(offset + 1);
                }
            }
            _ => {}
        }
    }
    None
}

/// Look up a JSON string field by name within a single-object scope.
/// Returns the decoded value when found.
fn read_json_field(object: &str, name: &str) -> Option<String> {
    let needle = format!("\"{name}\"");
    let mut search_start = 0usize;
    while let Some(field_offset) = object[search_start..].find(&needle) {
        let absolute = search_start + field_offset;
        let after_key = absolute + needle.len();
        let colon_offset = object[after_key..].find(':')?;
        let after_colon = after_key + colon_offset + 1;
        let Some(open_quote) = object[after_colon..].find('"') else {
            search_start = after_colon;
            continue;
        };
        let value_start = after_colon + open_quote + 1;
        return Some(read_json_string(&object[value_start..]));
    }
    None
}

/// Parse `wbsearchentities` JSON response into search hits.
#[must_use]
pub fn parse_wbsearch_hits(body: &str) -> Vec<LexemeSearchHit> {
    let mut out: Vec<LexemeSearchHit> = Vec::new();
    let Some(search_idx) = body.find("\"search\"") else {
        return out;
    };
    let mut cursor = search_idx;
    while let Some(id_offset) = body[cursor..].find("\"id\"") {
        let absolute = cursor + id_offset + "\"id\"".len();
        let Some(qstart) = body[absolute..].find('"') else {
            break;
        };
        let id_start = absolute + qstart + 1;
        let id = read_json_string(&body[id_start..]);
        let after_id = id_start + escaped_string_advance(&body[id_start..]);
        // Look ahead for "label" within the same object.
        let mut label = String::new();
        if let Some(label_offset) = body[after_id..].find("\"label\"") {
            let close_idx = body[after_id..].find('}').unwrap_or(usize::MAX);
            if label_offset < close_idx {
                let after_label = after_id + label_offset + "\"label\"".len();
                if let Some(lq) = body[after_label..].find('"') {
                    let label_start = after_label + lq + 1;
                    label = read_json_string(&body[label_start..]);
                }
            }
        }
        out.push(LexemeSearchHit { id, label });
        cursor = after_id;
    }
    out
}

/// Read a JSON string starting at `input[0]` (after the opening quote)
/// up to the closing quote, decoding `\uXXXX`, `\\`, `\"`, `\n`, etc.
fn read_json_string(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut iter = input.chars();
    while let Some(character) = iter.next() {
        if character == '"' {
            break;
        }
        if character == '\\' {
            let Some(next) = iter.next() else { break };
            match next {
                'n' => out.push('\n'),
                't' => out.push('\t'),
                'r' => out.push('\r'),
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                '/' => out.push('/'),
                'u' => {
                    let mut hex = String::with_capacity(4);
                    for _ in 0..4 {
                        let Some(c) = iter.next() else { break };
                        hex.push(c);
                    }
                    let Ok(codepoint) = u32::from_str_radix(&hex, 16) else {
                        break;
                    };
                    if (0xD800..=0xDBFF).contains(&codepoint) {
                        if iter.next() != Some('\\') || iter.next() != Some('u') {
                            break;
                        }
                        let mut low_hex = String::with_capacity(4);
                        for _ in 0..4 {
                            let Some(c) = iter.next() else { break };
                            low_hex.push(c);
                        }
                        let Ok(low) = u32::from_str_radix(&low_hex, 16) else {
                            break;
                        };
                        let combined = 0x1_0000 + ((codepoint - 0xD800) << 10) + (low - 0xDC00);
                        if let Some(c) = char::from_u32(combined) {
                            out.push(c);
                        }
                    } else if let Some(c) = char::from_u32(codepoint) {
                        out.push(c);
                    }
                }
                other => out.push(other),
            }
        } else {
            out.push(character);
        }
    }
    out
}

/// Return the number of bytes consumed by the JSON string starting at
/// `input[0]` (after the opening quote), including the closing quote.
fn escaped_string_advance(input: &str) -> usize {
    let mut consumed = 0usize;
    let mut bytes = input.bytes();
    while let Some(byte) = bytes.next() {
        consumed += 1;
        if byte == b'"' {
            return consumed;
        }
        if byte == b'\\' {
            if let Some(next) = bytes.next() {
                consumed += 1;
                if next == b'u' {
                    for _ in 0..4 {
                        if bytes.next().is_some() {
                            consumed += 1;
                        }
                    }
                }
            }
        }
    }
    consumed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sparql_lemmas_extracts_values_and_languages() {
        let body = r#"{
            "results": {
                "bindings": [
                    {"lemma": {"xml:lang": "ru", "type": "literal", "value": "привет"}},
                    {"lemma": {"xml:lang": "fr", "type": "literal", "value": "bonjour"}}
                ]
            }
        }"#;
        let rows = parse_sparql_lemmas(body);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].value, "привет");
        assert_eq!(rows[0].language.as_deref(), Some("ru"));
        assert_eq!(rows[1].value, "bonjour");
        assert_eq!(rows[1].language.as_deref(), Some("fr"));
    }

    #[test]
    fn parse_sparql_lemmas_handles_empty_bindings() {
        let body = r#"{"results": {"bindings": []}}"#;
        let rows = parse_sparql_lemmas(body);
        assert!(rows.is_empty());
    }

    #[test]
    fn parse_wbsearch_hits_extracts_lexeme_ids_and_labels() {
        let body = r#"{"searchinfo": {"search": "hello"}, "search": [
            {"id": "L8485", "label": "hello", "description": "..."},
            {"id": "L52", "label": "hello", "description": "..."}
        ]}"#;
        let hits = parse_wbsearch_hits(body);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].id, "L8485");
        assert_eq!(hits[0].label, "hello");
        assert_eq!(hits[1].id, "L52");
    }

    #[test]
    fn read_json_string_decodes_unicode_escapes() {
        let input = "\\u0041BC\"rest";
        assert_eq!(read_json_string(input), "ABC");
    }

    #[test]
    fn escaped_string_advance_skips_to_closing_quote() {
        let input = "hello\"world";
        assert_eq!(escaped_string_advance(input), 6);
    }

    #[test]
    fn escaped_string_advance_handles_unicode_escapes() {
        let input = "\\u0041\"after";
        assert_eq!(escaped_string_advance(input), 7);
    }
}
