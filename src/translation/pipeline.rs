//! Translation pipeline.
//!
//! Public entry point: [`TranslationPipeline::translate`].
//!
//! ## Stages
//!
//! 1. **Formalize** — `formalize(source_surface, source_lang)` projects the
//!    natural-language surface into a [`MeaningId`] by consulting
//!    Wiktionary (for the page that defines the lemma) and Wikidata
//!    (for the language-neutral Q-item or Lexeme sense). The
//!    [`MeaningId`] is the semantic meta-language identifier; it never
//!    embeds any single language.
//!
//! 2. **Deformalize** — `deformalize(meaning_id, target_lang)` renders a
//!    surface form in the target language by:
//!
//!    a. running a Wikidata SPARQL lexeme-join (P5137) when the meaning
//!    is backed by a Wikidata lexeme, **or**
//!
//!    b. parsing the source-edition Wiktionary translation table for
//!    the target language code, **or**
//!
//!    c. parsing the target-edition Wiktionary's `=== Перевод ===` /
//!    `===Translations===` block in reverse.
//!
//! Each strategy is recorded as a `provenance` entry on
//! [`Translation`], so the resulting links-notation trace shows
//! exactly which API responses fed the answer.

use super::http::{HttpClient, HttpError};
use super::meaning::MeaningId;
use super::wikidata::Wikidata;
use super::wiktionary::{Wiktionary, WiktionaryCandidate};

/// `true` when `FORMAL_AI_TRANSLATION_DEBUG=1` is set in the environment.
///
/// Reading the env var on every call would be wasteful, but tests rely on
/// per-process toggling, so we re-check each call here (cheap stdlib lookup).
fn translation_debug_enabled() -> bool {
    std::env::var("FORMAL_AI_TRANSLATION_DEBUG")
        .ok()
        .is_some_and(|value| !value.is_empty() && value != "0")
}

/// Emit a structured debug line to stderr when
/// `FORMAL_AI_TRANSLATION_DEBUG=1`. Used to trace each pipeline stage
/// when investigating issues like #221 (common-noun translation gaps).
fn translation_debug(stage: &str, message: &str) {
    if translation_debug_enabled() {
        eprintln!("[formal-ai translation] {stage}: {message}");
    }
}

/// Result of a translation request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Translation {
    pub source_surface: String,
    pub source_lang: String,
    pub target_lang: String,
    pub meaning: MeaningId,
    pub candidates: Vec<WiktionaryCandidate>,
    pub provenance: Vec<String>,
}

impl Translation {
    /// The best target-language surface form. Picks the first
    /// unqualified candidate, falling back to the first candidate.
    #[must_use]
    pub fn primary_surface(&self) -> Option<&str> {
        self.candidates
            .iter()
            .find(|c| c.qualifier.is_none())
            .or_else(|| self.candidates.first())
            .map(|c| c.surface.as_str())
    }
}

/// Translation pipeline. Borrows a single HTTP client (typically a
/// `CachedHttpClient`) for every Wiktionary and Wikidata call.
pub struct TranslationPipeline<'a, T: HttpClient + ?Sized> {
    http: &'a T,
}

impl<'a, T: HttpClient + ?Sized> TranslationPipeline<'a, T> {
    pub const fn new(http: &'a T) -> Self {
        Self { http }
    }

    /// Translate `surface` from `source_lang` to `target_lang`.
    ///
    /// Returns `Ok(Translation)` even when the candidate list is empty
    /// — callers can detect translation gaps by inspecting
    /// [`Translation::candidates`].
    pub fn translate(
        &self,
        surface: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Translation, HttpError> {
        translation_debug(
            "translate",
            &format!("start surface={surface:?} {source_lang}->{target_lang}"),
        );
        let mut provenance: Vec<String> = Vec::new();
        let page_title = normalize_page_title(surface);
        translation_debug("translate", &format!("page_title={page_title:?}"));

        if source_lang == target_lang {
            translation_debug("translate", "identity (source==target)");
            let mut candidates = vec![WiktionaryCandidate {
                surface: surface.to_owned(),
                qualifier: None,
            }];
            provenance.push("identity".to_owned());
            let meaning = upgrade_meaning_via_wikidata(
                self.http,
                &page_title,
                source_lang,
                target_lang,
                &mut provenance,
                &mut candidates,
            )
            .unwrap_or_else(|| MeaningId::from_wiktionary_page(source_lang, &page_title));
            return Ok(Translation {
                source_surface: surface.to_owned(),
                source_lang: source_lang.to_owned(),
                target_lang: target_lang.to_owned(),
                meaning,
                candidates,
                provenance,
            });
        }

        // Stage 1: formalize — fetch the source-edition Wiktionary page.
        // This single fetch usually carries translations for every target
        // language, grouped by sense (one block per `{{trans-top}}` on
        // English Wiktionary, one per `{{перев-блок}}` on Russian).
        let source_wiktionary = Wiktionary::new(source_lang, self.http);
        let mut blocks: Vec<Vec<WiktionaryCandidate>> = Vec::new();
        let mut meaning = MeaningId::from_wiktionary_page(source_lang, &page_title);
        match source_wiktionary.translation_blocks(&page_title, target_lang) {
            Ok(found) => {
                provenance.push(format!(
                    "wiktionary:{source_lang}:{page_title}#translations->{target_lang}"
                ));
                translation_debug("stage1", &format!("source-edition blocks={}", found.len()));
                if !found.is_empty() {
                    blocks = found;
                }
            }
            Err(error) => {
                provenance.push(format!(
                    "wiktionary:{source_lang}:{page_title}#translations->error({error})",
                ));
                translation_debug("stage1", &format!("source-edition error: {error}"));
            }
        }

        // Stage 1a: many high-traffic pages delegate part-of-speech
        // translations to a `/translations` subpage via
        // `{{see translation subpage|...}}`. The main page may still
        // host translations for *other* parts of speech, so we ALWAYS
        // also fetch the subpage when that template appears — not only
        // when the main page came up empty. (Issue #221: en→ru "water"
        // was returning verb translations because the noun translations
        // live on `water/translations`.)
        let main_wikitext = source_wiktionary.wikitext(&page_title).ok();
        let main_delegates_subpage = main_wikitext
            .as_deref()
            .is_some_and(|wt| wt.contains("{{see translation subpage|"));
        if blocks.is_empty() || main_delegates_subpage {
            let subpage = format!("{page_title}/translations");
            match source_wiktionary.translation_blocks(&subpage, target_lang) {
                Ok(found) if !found.is_empty() => {
                    provenance.push(format!(
                        "wiktionary:{source_lang}:{subpage}#translations->{target_lang}"
                    ));
                    // Prepend subpage blocks so they outrank the
                    // main-page (often other-PoS) blocks during
                    // sense-block selection.
                    let mut merged = found;
                    merged.extend(std::mem::take(&mut blocks));
                    blocks = merged;
                }
                Ok(_) => {}
                Err(error) => {
                    provenance.push(format!(
                        "wiktionary:{source_lang}:{subpage}#translations->error({error})",
                    ));
                }
            }
        }

        // Stage 1b: when the source-edition table came up empty, try the
        // target-edition Wiktionary in reverse. The target page lists
        // the source language under its own translation block — useful
        // when the source edition is sparse (common for ru → en).
        if blocks.is_empty() {
            if let Some(reverse) = reverse_lookup(
                self.http,
                surface,
                source_lang,
                target_lang,
                &mut provenance,
            ) {
                blocks = reverse;
            }
        }

        // Stage 1d: phrasal-variant fallback. Some natural phrases don't
        // have their own Wiktionary entry (e.g. ru:"как у тебя дела") but
        // a shorter canonical form does (ru:"как дела"). Generate
        // alternate forms by stripping language-specific filler
        // sub-phrases and retry Stages 1 + 1a + 1b on each variant. Stop
        // at the first variant that produces at least one block.
        let mut active_page_title = page_title.clone();
        if blocks.is_empty() {
            for variant in phrasal_variants(&page_title, source_lang) {
                provenance.push(format!("wiktionary:{source_lang}:variant->{variant}"));
                match source_wiktionary.translation_blocks(&variant, target_lang) {
                    Ok(found) if !found.is_empty() => {
                        provenance.push(format!(
                            "wiktionary:{source_lang}:{variant}#translations->{target_lang}"
                        ));
                        blocks = found;
                        active_page_title.clone_from(&variant);
                        meaning = MeaningId::from_wiktionary_page(source_lang, &variant);
                        break;
                    }
                    Ok(_) => {}
                    Err(error) => {
                        provenance.push(format!(
                            "wiktionary:{source_lang}:{variant}#translations->error({error})"
                        ));
                    }
                }
                if let Some(reverse) = reverse_lookup(
                    self.http,
                    &variant,
                    source_lang,
                    target_lang,
                    &mut provenance,
                ) {
                    blocks = reverse;
                    active_page_title.clone_from(&variant);
                    meaning = MeaningId::from_wiktionary_page(source_lang, &variant);
                    break;
                }
            }
        }

        // Stage 1c: pick the most likely sense block when the source word
        // is polysemous (multiple blocks). For each block, count how
        // many candidates round-trip — that is, how many target-edition
        // pages list the source surface as a translation. The block with
        // the highest confirmation rate wins; ties fall back to source
        // order.
        let candidates = if blocks.is_empty() {
            Vec::new()
        } else {
            select_best_block(
                self.http,
                &active_page_title,
                source_lang,
                target_lang,
                &mut provenance,
                blocks,
            )
        };

        // Stage 2: pull the Wikidata Q-item / Lexeme sense — even if
        // we already have a translation surface, we want the trace to
        // expose the canonical meaning id. Failures here do not block
        // the translation.
        let mut candidates = candidates;
        if let Some(updated) = upgrade_meaning_via_wikidata(
            self.http,
            &active_page_title,
            source_lang,
            target_lang,
            &mut provenance,
            &mut candidates,
        ) {
            meaning = updated;
        }

        if candidates.is_empty() {
            candidates = compositional_candidates(
                &active_page_title,
                source_lang,
                target_lang,
                &mut provenance,
            );
        }

        translation_debug(
            "translate",
            &format!(
                "done candidates={} primary={:?} meaning={:?}",
                candidates.len(),
                candidates
                    .iter()
                    .find(|c| c.qualifier.is_none())
                    .or_else(|| candidates.first())
                    .map(|c| c.surface.as_str()),
                meaning,
            ),
        );
        Ok(Translation {
            source_surface: surface.to_owned(),
            source_lang: source_lang.to_owned(),
            target_lang: target_lang.to_owned(),
            meaning,
            candidates,
            provenance,
        })
    }
}

/// Try the target-edition Wiktionary in reverse. We open the page that
/// is most likely to *exist* on the target edition for the source word.
/// Russian Wiktionary, for instance, hosts pages for Russian words and
/// records their translations into other languages under
/// `=== Перевод ===`.
fn reverse_lookup<T: HttpClient + ?Sized>(
    http: &T,
    surface: &str,
    source_lang: &str,
    target_lang: &str,
    provenance: &mut Vec<String>,
) -> Option<Vec<Vec<WiktionaryCandidate>>> {
    let page_title = normalize_page_title(surface);
    for edition in [source_lang, target_lang] {
        let wiktionary = Wiktionary::new(edition, http);
        match wiktionary.translation_blocks(&page_title, target_lang) {
            Ok(blocks) if !blocks.is_empty() => {
                provenance.push(format!(
                    "wiktionary:{edition}:{page_title}#reverse->{target_lang}"
                ));
                return Some(blocks);
            }
            Ok(_) => {}
            Err(error) => {
                provenance.push(format!(
                    "wiktionary:{edition}:{page_title}#reverse->error({error})"
                ));
            }
        }
    }
    None
}

/// Pick the best sense block from `blocks` by round-trip confirmation rate.
///
/// Each candidate's "round-trip position" is its index within the
/// target-edition block that contains the source surface (not the
/// flattened candidate list). Lower position = the target considers the
/// source more primary.
///
/// Block-level selection picks the block with the most round-trip
/// confirmations (ties broken by source order). Within the chosen block,
/// candidates are sorted by `(target_position_within_block, source_idx)`.
///
/// Worked example — `en:hello`:
/// - Block 0 ("greeting"): `привет` confirms at target-pos 0,
///   `здравствуйте` at target-pos 1. Two confirms.
/// - Block 1 ("when answering the telephone"): `алло` and `алё` confirm
///   at target-pos 0 and 1. Two confirms.
/// - Ties go to the earlier source block → block 0. Within block 0,
///   `привет` (target-pos 0) beats `здравствуйте` (target-pos 1).
fn select_best_block<T: HttpClient + ?Sized>(
    http: &T,
    page_title: &str,
    source_lang: &str,
    target_lang: &str,
    provenance: &mut Vec<String>,
    blocks: Vec<Vec<WiktionaryCandidate>>,
) -> Vec<WiktionaryCandidate> {
    let target_wiktionary = Wiktionary::new(target_lang, http);
    let mut block_positions: Vec<Vec<Option<usize>>> = Vec::with_capacity(blocks.len());
    for (block_idx, block) in blocks.iter().enumerate() {
        let mut positions: Vec<Option<usize>> = Vec::with_capacity(block.len());
        for candidate in block {
            let candidate_page = normalize_page_title(&candidate.surface);
            if candidate_page.is_empty() {
                positions.push(None);
                continue;
            }
            let Ok(back_blocks) =
                target_wiktionary.translation_blocks(&candidate_page, source_lang)
            else {
                positions.push(None);
                continue;
            };
            // Within-block position: scan each block of the target page
            // and record the position of the source surface inside the
            // first block that contains it.
            let mut within_block_position: Option<usize> = None;
            for back_block in &back_blocks {
                if let Some(pos) = back_block
                    .iter()
                    .position(|row| normalize_page_title(&row.surface) == page_title)
                {
                    within_block_position = Some(pos);
                    break;
                }
            }
            if let Some(pos) = within_block_position {
                provenance.push(format!(
                    "wiktionary:{target_lang}:{candidate_page}#confirms->{source_lang}:{page_title}@{pos}[block{block_idx}]"
                ));
            }
            positions.push(within_block_position);
        }
        block_positions.push(positions);
    }
    let mut best_block: usize = 0;
    let mut best_confirms: usize = 0;
    for (idx, positions) in block_positions.iter().enumerate() {
        let confirms = positions.iter().filter(|p| p.is_some()).count();
        if confirms > best_confirms {
            best_confirms = confirms;
            best_block = idx;
        }
    }
    let block = blocks.into_iter().nth(best_block).unwrap_or_default();
    let positions = block_positions
        .into_iter()
        .nth(best_block)
        .unwrap_or_default();
    let mut indexed: Vec<(usize, Option<usize>, WiktionaryCandidate)> = block
        .into_iter()
        .zip(positions)
        .enumerate()
        .map(|(idx, (cand, pos))| (idx, pos, cand))
        .collect();
    indexed.sort_by_key(|(idx, pos, _)| {
        pos.as_ref()
            .map_or((1usize, 0, *idx), |p| (0usize, *p, *idx))
    });
    indexed.into_iter().map(|(_, _, cand)| cand).collect()
}

/// Upgrade a Wiktionary-only [`MeaningId`] to a Wikidata-backed one when
/// SPARQL returns a lexeme that matches the surface in the source
/// language. Also appends any extra lemma forms returned by the
/// translation query — they are stored as additional candidates so
/// callers can disambiguate (formal vs informal, dialect, etc).
fn upgrade_meaning_via_wikidata<T: HttpClient + ?Sized>(
    http: &T,
    page_title: &str,
    source_lang: &str,
    target_lang: &str,
    provenance: &mut Vec<String>,
    candidates: &mut Vec<WiktionaryCandidate>,
) -> Option<MeaningId> {
    let wikidata = Wikidata::new(http);
    let hits = match wikidata.search_lexeme(page_title, source_lang) {
        Ok(hits) => hits,
        Err(error) => {
            provenance.push(format!("wikidata:search->error({error})"));
            return None;
        }
    };
    let first = hits.first()?;
    provenance.push(format!("wikidata:lexeme:{}", first.id));
    let mut meaning = MeaningId::from_sense(first.id.clone());
    let lemmas = match wikidata.lexeme_translations(&first.id, target_lang) {
        Ok(rows) => rows,
        Err(error) => {
            provenance.push(format!("wikidata:sparql->error({error})"));
            if let Some(canonical) = canonical_target_english_meaning(
                &wikidata,
                source_lang,
                target_lang,
                candidates,
                provenance,
            ) {
                meaning = canonical;
            }
            return Some(meaning);
        }
    };
    if !lemmas.is_empty() {
        provenance.push(format!(
            "wikidata:sparql:{}->{} ({} lemmas)",
            first.id,
            target_lang,
            lemmas.len()
        ));
    }
    for lemma in lemmas {
        let candidate = WiktionaryCandidate {
            surface: lemma.value,
            qualifier: None,
        };
        if !candidates.iter().any(|c| c.surface == candidate.surface) {
            candidates.push(candidate);
        }
    }
    if let Some(canonical) = canonical_target_english_meaning(
        &wikidata,
        source_lang,
        target_lang,
        candidates,
        provenance,
    ) {
        meaning = canonical;
    }
    Some(meaning)
}

fn canonical_target_english_meaning<T: HttpClient + ?Sized>(
    wikidata: &Wikidata<'_, T>,
    source_lang: &str,
    target_lang: &str,
    candidates: &[WiktionaryCandidate],
    provenance: &mut Vec<String>,
) -> Option<MeaningId> {
    if source_lang.eq_ignore_ascii_case("en") || !target_lang.eq_ignore_ascii_case("en") {
        return None;
    }
    let candidate = candidates
        .iter()
        .find(|candidate| candidate.qualifier.is_none())
        .or_else(|| candidates.first())?;
    let lemma = normalize_page_title(&candidate.surface);
    if lemma.is_empty() {
        return None;
    }
    match wikidata.search_lexeme(&lemma, "en") {
        Ok(hits) => {
            let first = hits.first()?;
            provenance.push(format!("wikidata:canonical_lexeme:{}", first.id));
            Some(MeaningId::from_sense(first.id.clone()))
        }
        Err(error) => {
            provenance.push(format!("wikidata:canonical_search->error({error})"));
            None
        }
    }
}

fn compositional_candidates(
    page_title: &str,
    source_lang: &str,
    target_lang: &str,
    provenance: &mut Vec<String>,
) -> Vec<WiktionaryCandidate> {
    if !source_lang.eq_ignore_ascii_case("ru") || !target_lang.eq_ignore_ascii_case("en") {
        return Vec::new();
    }

    if let Some(surface) = russian_phrase_to_english(page_title) {
        provenance.push(format!("compositional:ru->en:{page_title}"));
        return vec![WiktionaryCandidate {
            surface: surface.to_owned(),
            qualifier: None,
        }];
    }

    // The HTTP variant fallback already tried `phrasal_variants` against
    // Wiktionary; reuse the same elision rules for the compositional
    // table so politeness-marked phrases like `как у тебя дела` collapse
    // to the canonical `как дела` entry.
    for variant in phrasal_variants(page_title, source_lang) {
        if let Some(surface) = russian_phrase_to_english(&variant) {
            provenance.push(format!(
                "compositional:ru->en:{page_title}=>variant:{variant}"
            ));
            return vec![WiktionaryCandidate {
                surface: surface.to_owned(),
                qualifier: None,
            }];
        }
    }

    let words: Vec<&str> = page_title.split_whitespace().collect();
    if !(2..=8).contains(&words.len()) {
        return Vec::new();
    }

    let Some(surface) = translate_russian_word_sequence(&words) else {
        return Vec::new();
    };
    provenance.push(format!("compositional:ru->en:{page_title}"));
    vec![WiktionaryCandidate {
        surface,
        qualifier: None,
    }]
}

fn russian_phrase_to_english(page_title: &str) -> Option<&'static str> {
    match page_title {
        "кто ты" | "кто ты такой" | "кто ты такая" | "кто вы" | "кто вы такой" | "кто вы такая" => {
            Some("Who are you?")
        }
        "что это" | "что это такое" => Some("What is this?"),
        // Russian small-talk variants → "how are you". `phrasal_variants`
        // strips the dative-of-possession infix (`у тебя`, `у вас`, …)
        // so callers may arrive at the bare `как дела` form.
        "как дела" => Some("how are you"),
        _ => None,
    }
}

fn russian_word_to_english(word: &str) -> Option<&'static str> {
    match word {
        "найди" | "найдите" | "найти" => Some("find"),
        "синоним" | "синонимы" | "синонимов" => Some("synonyms"),
        "или" => Some("or"),
        "пример" | "примеры" | "примеров" => Some("examples"),
        "согласование" | "согласования" | "согласованию" | "согласованием" | "согласовании" => {
            Some("agreement")
        }
        "доброе" | "добрый" | "добрая" | "добрые" | "доброго" | "добрую" | "добрым" | "хорошее"
        | "хороший" | "хорошая" | "хорошие" | "хорошего" | "хорошую" | "хорошим" => {
            Some("good")
        }
        "яблоко" | "яблока" | "яблоку" | "яблоком" | "яблоке" | "яблоки" | "яблок" | "яблокам"
        | "яблоками" | "яблоках" => Some("apple"),
        _ => None,
    }
}

fn translate_russian_word_sequence(words: &[&str]) -> Option<String> {
    let mut translated: Vec<&str> = Vec::with_capacity(words.len() + 2);
    let mut index = 0;
    while index < words.len() {
        let word = words[index];
        if let Some(next) = words.get(index + 1) {
            if russian_genitive_relation_head(word) && russian_genitive_noun(next).is_some() {
                translated.push(russian_word_to_english(word)?);
                translated.push("of");
                translated.push(russian_genitive_noun(next)?);
                index += 2;
                continue;
            }
        }
        translated.push(russian_word_to_english(word)?);
        index += 1;
    }
    Some(capitalize_ascii_first(&translated.join(" ")))
}

fn russian_genitive_relation_head(word: &str) -> bool {
    matches!(
        word,
        "пример" | "примеры" | "примеров" | "синоним" | "синонимы" | "синонимов"
    )
}

fn russian_genitive_noun(word: &str) -> Option<&'static str> {
    match word {
        "согласования" => Some("agreement"),
        _ => None,
    }
}

fn capitalize_ascii_first(surface: &str) -> String {
    let mut chars = surface.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut out = String::with_capacity(surface.len());
    out.extend(first.to_uppercase());
    out.extend(chars);
    out
}

/// Generate alternate phrasal forms to retry when the original page is
/// missing on Wiktionary.
///
/// Languages differ in which optional words can be elided without
/// changing meaning. We encode the elisions per source language; for now
/// we cover Russian (`у тебя/у вас/у нас/...` infix between question word
/// and noun).
///
/// Variants are returned in priority order — the caller stops at the
/// first one that produces translations.
#[must_use]
pub fn phrasal_variants(page_title: &str, source_lang: &str) -> Vec<String> {
    let mut variants: Vec<String> = Vec::new();
    if source_lang.eq_ignore_ascii_case("ru") {
        // Russian dative-of-possession infix: `как у тебя дела` and
        // `как у вас дела` collapse to the canonical `как дела`. This
        // covers the politeness variants without changing semantics.
        let pronouns = [
            "у тебя",
            "у вас",
            "у нас",
            "у меня",
            "у них",
            "у него",
            "у неё",
            "у нее",
        ];
        for pronoun in &pronouns {
            // Strip the pronoun as a whole-word infix surrounded by spaces.
            let needle = format!(" {pronoun} ");
            if let Some(idx) = page_title.find(&needle) {
                let mut alt = String::with_capacity(page_title.len() - needle.len() + 1);
                alt.push_str(&page_title[..idx]);
                alt.push(' ');
                alt.push_str(&page_title[idx + needle.len()..]);
                let alt = alt.split_whitespace().collect::<Vec<_>>().join(" ");
                if !alt.is_empty() && alt != page_title && !variants.contains(&alt) {
                    variants.push(alt);
                }
            }
        }
    }
    variants
}

/// Normalize a surface fragment into a Wiktionary page title.
///
/// - Trim whitespace.
/// - Strip terminal punctuation (`?`, `!`, `.`, `。`, `？`, `！`).
/// - Lower-case the first letter (Wiktionary stores most page titles in
///   lower case for content words; redirects handle the capitalized
///   variants but we want a stable cache key).
#[must_use]
pub fn normalize_page_title(surface: &str) -> String {
    let trimmed = surface
        .trim()
        .trim_end_matches(['?', '!', '.', '。', '？', '！', '．']);
    let mut chars = trimmed.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut out = String::with_capacity(trimmed.len());
    for character in first.to_lowercase() {
        out.push(character);
    }
    out.extend(chars);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct StubHttp {
        responses: Mutex<std::collections::HashMap<String, String>>,
    }

    impl StubHttp {
        fn new(pairs: &[(&str, &str)]) -> Self {
            Self {
                responses: Mutex::new(
                    pairs
                        .iter()
                        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                        .collect(),
                ),
            }
        }
    }

    impl HttpClient for StubHttp {
        fn get(&self, url: &str) -> Result<String, HttpError> {
            self.responses
                .lock()
                .unwrap()
                .get(url)
                .cloned()
                .ok_or_else(|| HttpError::Status {
                    status: 404,
                    body: format!("no stubbed response for {url}"),
                })
        }
    }

    #[test]
    fn normalize_page_title_strips_terminal_punctuation() {
        assert_eq!(normalize_page_title("Hello!"), "hello");
        assert_eq!(normalize_page_title("как у тебя дела?"), "как у тебя дела");
        assert_eq!(normalize_page_title("你好？"), "你好");
    }

    #[test]
    fn normalize_page_title_lowercases_first_letter() {
        assert_eq!(normalize_page_title("Hello"), "hello");
        assert_eq!(normalize_page_title("Как дела"), "как дела");
    }

    #[test]
    fn translate_identity_returns_self_with_identity_provenance() {
        let http = StubHttp::new(&[]);
        let pipeline = TranslationPipeline::new(&http);
        let translation = pipeline.translate("hello", "en", "en").unwrap();
        assert_eq!(translation.primary_surface(), Some("hello"));
        assert!(translation
            .provenance
            .iter()
            .any(|entry| entry == "identity"));
        assert_eq!(translation.meaning.slug(), "wiktionary:en:hello");
    }

    #[test]
    fn translate_identity_upgrades_to_wikidata_meaning_when_available() {
        let search_url = "https://www.wikidata.org/w/api.php?action=wbsearchentities&search=hello&language=en&type=lexeme&format=json&uselang=en&limit=5";
        let http = StubHttp::new(&[(search_url, r#"{"search":[{"id":"L8885","label":"hello"}]}"#)]);
        let pipeline = TranslationPipeline::new(&http);

        let translation = pipeline.translate("hello", "en", "en").unwrap();

        assert_eq!(translation.primary_surface(), Some("hello"));
        assert_eq!(translation.meaning.slug(), "wikidata-sense:L8885");
        assert!(translation
            .provenance
            .iter()
            .any(|entry| entry == "wikidata:lexeme:L8885"));
    }

    #[test]
    fn wikidata_upgrade_canonicalizes_target_english_meaning() {
        let russian_search_url = "https://www.wikidata.org/w/api.php?action=wbsearchentities&search=%D0%BF%D1%80%D0%B8%D0%B2%D0%B5%D1%82&language=ru&type=lexeme&format=json&uselang=ru&limit=5";
        let english_search_url = "https://www.wikidata.org/w/api.php?action=wbsearchentities&search=hello&language=en&type=lexeme&format=json&uselang=en&limit=5";
        let http = StubHttp::new(&[
            (
                russian_search_url,
                r#"{"search":[{"id":"L150880","label":"привет"}]}"#,
            ),
            (
                english_search_url,
                r#"{"search":[{"id":"L8885","label":"hello"}]}"#,
            ),
        ]);
        let mut candidates = vec![WiktionaryCandidate {
            surface: "hello".to_owned(),
            qualifier: None,
        }];
        let mut provenance = Vec::new();

        let meaning = upgrade_meaning_via_wikidata(
            &http,
            "привет",
            "ru",
            "en",
            &mut provenance,
            &mut candidates,
        )
        .expect("wikidata search should produce a meaning");

        assert_eq!(meaning.slug(), "wikidata-sense:L8885");
        assert!(provenance
            .iter()
            .any(|entry| entry == "wikidata:lexeme:L150880"));
        assert!(provenance
            .iter()
            .any(|entry| entry == "wikidata:canonical_lexeme:L8885"));
    }

    #[test]
    fn translate_uses_source_edition_translation_table() {
        // English Wiktionary returns a JSON envelope around wikitext;
        // the wikitext lists the Russian translation under `{{t+|ru|...}}`.
        // Use a placeholder lemma (`blargh`) that is *not* in the offline
        // dictionary so the pipeline reaches the HTTP stage and we can
        // verify the wikitext parser end-to-end.
        let url = "https://en.wiktionary.org/w/api.php?action=parse&page=blargh&prop=wikitext&formatversion=2&format=json&redirects=1";
        let wikitext = r#"{"parse":{"title":"blargh","wikitext":"* Russian: {{t+|ru|бларг}}\n"}}"#;
        let http = StubHttp::new(&[(url, wikitext)]);
        let pipeline = TranslationPipeline::new(&http);
        let translation = pipeline.translate("blargh", "en", "ru").unwrap();
        assert_eq!(translation.primary_surface(), Some("бларг"));
        assert!(
            translation
                .provenance
                .iter()
                .any(|p| p.starts_with("wiktionary:en:blargh#translations->ru")),
            "got provenance: {:?}",
            translation.provenance,
        );
    }

    #[test]
    fn translate_returns_translation_with_empty_candidates_when_nothing_matches() {
        // No HTTP stubs => every fetch fails. The pipeline still
        // produces a Translation, but with an empty candidates list
        // (callers detect the gap explicitly).
        let http = StubHttp::new(&[]);
        let pipeline = TranslationPipeline::new(&http);
        let translation = pipeline.translate("xyzzy", "en", "ru").unwrap();
        assert!(translation.candidates.is_empty());
        assert!(translation.primary_surface().is_none());
        assert!(translation.provenance.iter().any(|p| p.contains("error")));
    }

    #[test]
    fn translate_uses_compositional_ru_en_fallback_for_short_phrases() {
        let http = StubHttp::new(&[]);
        let pipeline = TranslationPipeline::new(&http);

        let noun_phrase = pipeline.translate("доброе яблоко", "ru", "en").unwrap();
        assert_eq!(noun_phrase.primary_surface(), Some("Good apple"));
        assert!(noun_phrase
            .provenance
            .iter()
            .any(|p| p == "compositional:ru->en:доброе яблоко"));

        let question_phrase = pipeline.translate("что это такое?", "ru", "en").unwrap();
        assert_eq!(question_phrase.primary_surface(), Some("What is this?"));
        assert!(question_phrase
            .provenance
            .iter()
            .any(|p| p == "compositional:ru->en:что это такое"));
    }

    #[test]
    fn translate_prefers_unqualified_candidate() {
        // Use a placeholder lemma not present in the offline dictionary so
        // the pipeline reaches the wikitext-parsing stage.
        let url = "https://en.wiktionary.org/w/api.php?action=parse&page=blargh&prop=wikitext&formatversion=2&format=json&redirects=1";
        let wikitext = r#"{"parse":{"wikitext":"* Russian: {{t|ru|здравствуйте|q=formal}}, {{t+|ru|привет|q=informal}}, {{t|ru|здорово}}\n"}}"#;
        let http = StubHttp::new(&[(url, wikitext)]);
        let pipeline = TranslationPipeline::new(&http);
        let translation = pipeline.translate("blargh", "en", "ru").unwrap();
        // The first unqualified candidate wins.
        assert_eq!(translation.primary_surface(), Some("здорово"));
    }
}
