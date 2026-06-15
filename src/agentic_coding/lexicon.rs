//! The closed-class lexicon for grounded extraction, parsed from Links Notation.
//!
//! Open-domain information extraction needs neural inference, which is a
//! documented NON-GOAL for this crate. So the formalizer recognises only a
//! closed lexicon stored as data ([`LEXICON_LINO`]) — never guessing relations it
//! cannot ground. This module owns that lexicon: a minimal Links Notation record
//! parser, the work / lexeme / concept / procedure / context model it parses
//! into, and the deterministic subject–predicate–object extractor. The output
//! term types ([`Term`], [`TermKind`], [`PredicateUse`]) live here too because
//! they are *what the lexicon recognised*; [`super::formalize`] consumes them.

/// The closed-class lexicon (data, not code), loaded at compile time.
pub const LEXICON_LINO: &str = include_str!("../../data/agentic-coding/fisherman-lexicon.lino");

/// The kind of a recognised term.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TermKind {
    Entity,
    Concept,
    Literal,
}

impl TermKind {
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Entity => "entity",
            Self::Concept => "concept",
            Self::Literal => "literal",
        }
    }
}

/// A recognised subject or object term, or a literal fallback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Term {
    pub id: String,
    pub label: String,
    pub kind: TermKind,
}

impl Term {
    pub fn literal(text: &str) -> Self {
        Self {
            id: text.to_owned(),
            label: text.to_owned(),
            kind: TermKind::Literal,
        }
    }
}

/// A reference to a predicate from an assertion (id + surface label).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PredicateUse {
    pub id: String,
    pub label: String,
}

/// A recognised subject–predicate–object triple.
pub struct ExtractedTriple {
    pub subject: Term,
    pub predicate: PredicateLexeme,
    pub object: Term,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PredicateLexeme {
    pub id: String,
    pub label: String,
    pub modal: Option<String>,
    pub time: Option<String>,
}

impl PredicateLexeme {
    pub fn as_ref(&self) -> PredicateUse {
        PredicateUse {
            id: self.id.clone(),
            label: self.label.clone(),
        }
    }
}

struct Lexeme {
    surface: Vec<String>,
    kind: LexemeKind,
    id: String,
    label: String,
    modal: Option<String>,
    time: Option<String>,
}

impl Lexeme {
    fn as_term(&self) -> Option<Term> {
        match self.kind {
            LexemeKind::Entity => Some(Term {
                id: self.id.clone(),
                label: self.label.clone(),
                kind: TermKind::Entity,
            }),
            LexemeKind::Concept => Some(Term {
                id: self.id.clone(),
                label: self.label.clone(),
                kind: TermKind::Concept,
            }),
            LexemeKind::Predicate => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexemeKind {
    Entity,
    Concept,
    Predicate,
}

pub struct WorkConcept {
    pub id: String,
    pub label: String,
    pub kind: String,
}

pub struct Procedure {
    pub id: String,
    pub signature: String,
    pub description: String,
    pub trigger: String,
}

pub struct WorkContext {
    pub id: String,
    pub label: String,
    pub description: String,
}

pub struct Work {
    pub id: String,
    pub doc_id: String,
    pub title: String,
    signature: Vec<String>,
    lexemes: Vec<Lexeme>,
    pub concepts: Vec<WorkConcept>,
    pub procedures: Vec<Procedure>,
    pub contexts: Vec<WorkContext>,
}

impl Work {
    pub fn primary_context(&self) -> Option<String> {
        self.contexts.first().map(|context| context.id.clone())
    }

    pub fn final_context(&self) -> Option<String> {
        self.contexts
            .iter()
            .find(|context| context.id.ends_with(":final"))
            .or_else(|| self.contexts.last())
            .map(|context| context.id.clone())
    }

    /// Extract a subject–predicate–object triple from `sentence`, grounded in
    /// this work's closed lexicon. Returns `None` when no predicate lexeme is
    /// present; an unrecognised object becomes an honest literal rather than an
    /// invented entity.
    pub fn extract(&self, sentence: &str) -> Option<ExtractedTriple> {
        let tokens = tokenize(sentence);
        // Locate the first predicate.
        let mut predicate: Option<(usize, usize, PredicateLexeme)> = None;
        let mut index = 0;
        while index < tokens.len() && predicate.is_none() {
            if let Some((width, lexeme)) = self.match_at(&tokens, index) {
                if matches!(lexeme.kind, LexemeKind::Predicate) {
                    predicate = Some((
                        index,
                        index + width,
                        PredicateLexeme {
                            id: lexeme.id.clone(),
                            label: lexeme.label.clone(),
                            modal: lexeme.modal.clone(),
                            time: lexeme.time.clone(),
                        },
                    ));
                }
            }
            index += 1;
        }
        let (pred_start, pred_end, predicate) = predicate?;

        // Subject: the last entity/concept match before the predicate.
        let mut subject: Option<Term> = None;
        let mut position = 0;
        while position < pred_start {
            if let Some((width, lexeme)) = self.match_at(&tokens, position) {
                if let Some(term) = lexeme.as_term() {
                    subject = Some(term);
                }
                position += width;
            } else {
                position += 1;
            }
        }

        // Object: the first entity/concept match after the predicate; else the
        // remaining phrase as a literal.
        let mut object: Option<Term> = None;
        let mut position = pred_end;
        while position < tokens.len() {
            if let Some((width, lexeme)) = self.match_at(&tokens, position) {
                if let Some(term) = lexeme.as_term() {
                    object = Some(term);
                    break;
                }
                position += width;
            } else {
                position += 1;
            }
        }
        let object = object.unwrap_or_else(|| Term::literal(&tokens[pred_end..].join(" ")));

        Some(ExtractedTriple {
            subject: subject.unwrap_or_else(|| Term::literal("—")),
            predicate,
            object,
        })
    }

    /// Match the longest lexeme surface starting at `tokens[index]` (width 2,
    /// then width 1).
    fn match_at<'lexicon>(
        &'lexicon self,
        tokens: &[String],
        index: usize,
    ) -> Option<(usize, &'lexicon Lexeme)> {
        for width in [2usize, 1] {
            if index + width > tokens.len() {
                continue;
            }
            let window = &tokens[index..index + width];
            if let Some(lexeme) = self
                .lexemes
                .iter()
                .find(|lexeme| lexeme.surface.len() == width && lexeme.surface == window)
            {
                return Some((width, lexeme));
            }
        }
        None
    }
}

pub struct Lexicon {
    works: Vec<Work>,
}

impl Lexicon {
    /// Load the standard lexicon bundled with the crate.
    pub fn standard() -> Self {
        Self::load(LEXICON_LINO)
    }

    pub fn load(source: &str) -> Self {
        let records = parse_records(source);
        let mut works: Vec<Work> = Vec::new();
        // First pass: works.
        for record in &records {
            if record.kind == "work" {
                works.push(Work {
                    id: record.head.clone(),
                    doc_id: record.field("doc_id").unwrap_or(&record.head).to_owned(),
                    title: record.field("title").unwrap_or(&record.head).to_owned(),
                    signature: record
                        .field("signature")
                        .map(|value| value.split_whitespace().map(str::to_lowercase).collect())
                        .unwrap_or_default(),
                    lexemes: Vec::new(),
                    concepts: Vec::new(),
                    procedures: Vec::new(),
                    contexts: Vec::new(),
                });
            }
        }
        // Second pass: attach members to their work.
        for record in &records {
            let Some(work_id) = record.field("work") else {
                continue;
            };
            let Some(index) = works.iter().position(|work| work.id == work_id) else {
                continue;
            };
            match record.kind.as_str() {
                "lexeme" => {
                    let kind = match record.field("kind") {
                        Some("entity") => LexemeKind::Entity,
                        Some("concept") => LexemeKind::Concept,
                        Some("predicate") => LexemeKind::Predicate,
                        _ => continue,
                    };
                    works[index].lexemes.push(Lexeme {
                        surface: tokenize(&record.head),
                        kind,
                        id: record.field("id").unwrap_or(&record.head).to_owned(),
                        label: record.field("label").unwrap_or(&record.head).to_owned(),
                        modal: record.field("modal").map(ToOwned::to_owned),
                        time: record.field("time").map(ToOwned::to_owned),
                    });
                }
                "concept" => works[index].concepts.push(WorkConcept {
                    id: record.head.clone(),
                    label: record.field("label").unwrap_or(&record.head).to_owned(),
                    kind: record.field("type").unwrap_or("abstract").to_owned(),
                }),
                "procedure" => works[index].procedures.push(Procedure {
                    id: record.head.clone(),
                    signature: record.field("signature").unwrap_or_default().to_owned(),
                    description: record.field("description").unwrap_or_default().to_owned(),
                    trigger: record.field("trigger").unwrap_or_default().to_owned(),
                }),
                "context" => works[index].contexts.push(WorkContext {
                    id: record.head.clone(),
                    label: record.field("label").unwrap_or(&record.head).to_owned(),
                    description: record.field("description").unwrap_or_default().to_owned(),
                }),
                _ => {}
            }
        }
        Self { works }
    }

    pub fn best_work_for(&self, text: &str) -> Option<&Work> {
        let tokens: Vec<String> = tokenize(text);
        let mut best: Option<(&Work, usize)> = None;
        for work in &self.works {
            if work.signature.is_empty() {
                continue;
            }
            let hits = work
                .signature
                .iter()
                .filter(|needle| tokens.iter().any(|token| token == *needle))
                .count();
            // Integer ceiling of `len / 2` (`div_ceil` is stable since Rust
            // 1.73, well within the crate's 1.77 MSRV).
            let threshold = work.signature.len().div_ceil(2);
            if hits >= threshold && best.is_none_or(|(_, top)| hits > top) {
                best = Some((work, hits));
            }
        }
        best.map(|(work, _)| work)
    }
}

pub fn tokenize(text: &str) -> Vec<String> {
    text.split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_lowercase)
        .collect()
}

// ---------------------------------------------------------------------------
// Minimal Links Notation record parser (2-space indent, quoted values).
// ---------------------------------------------------------------------------

struct Record {
    kind: String,
    head: String,
    fields: Vec<(String, String)>,
}

impl Record {
    fn field(&self, key: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.as_str())
    }
}

fn parse_records(source: &str) -> Vec<Record> {
    let mut records: Vec<Record> = Vec::new();
    for line in source.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if line.starts_with(' ') {
            let Some(record) = records.last_mut() else {
                continue;
            };
            if let Some((key, value)) = parse_pair(line.trim()) {
                record.fields.push((key, value));
            }
        } else if let Some((kind, head)) = parse_pair(line.trim()) {
            records.push(Record {
                kind,
                head,
                fields: Vec::new(),
            });
        }
    }
    records
}

fn parse_pair(line: &str) -> Option<(String, String)> {
    let (key, rest) = line.split_once(char::is_whitespace)?;
    let value = rest.trim();
    let value = value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value);
    Some((key.to_owned(), value.to_owned()))
}
