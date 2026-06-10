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
use crate::seed::{
    ROLE_COMPOSITIONAL_GENITIVE_HEAD, ROLE_COMPOSITIONAL_LEMMA, ROLE_COMPOSITIONAL_PHRASE,
};

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

        if let Some((slug, target_surface)) =
            seed_compositional_translation(&page_title, source_lang, target_lang)
        {
            translation_debug("translate", "seed compositional hit");
            provenance.push(format!(
                "compositional:{source_lang}->{target_lang}:{page_title}"
            ));
            return Ok(Translation {
                source_surface: surface.to_owned(),
                source_lang: source_lang.to_owned(),
                target_lang: target_lang.to_owned(),
                meaning: seed_meaning_id(slug),
                candidates: vec![WiktionaryCandidate {
                    surface: target_surface.to_owned(),
                    qualifier: None,
                }],
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
                &mut meaning,
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
    meaning: &mut MeaningId,
    provenance: &mut Vec<String>,
) -> Vec<WiktionaryCandidate> {
    if let Some((slug, surface)) =
        seed_compositional_translation(page_title, source_lang, target_lang)
    {
        provenance.push(format!(
            "compositional:{source_lang}->{target_lang}:{page_title}"
        ));
        *meaning = seed_meaning_id(slug);
        return vec![WiktionaryCandidate {
            surface: surface.to_owned(),
            qualifier: None,
        }];
    }

    if source_lang.eq_ignore_ascii_case("ru") && target_lang.eq_ignore_ascii_case("en") {
        if let Some(surface) = russian_phrase_to_english(page_title) {
            provenance.push(format!("compositional:ru->en:{page_title}"));
            return vec![WiktionaryCandidate {
                surface: surface.to_owned(),
                qualifier: None,
            }];
        }
    }

    // The HTTP variant fallback already tried `phrasal_variants` against
    // Wiktionary; reuse the same elision rules for the compositional
    // table so politeness-marked phrases like `как у тебя дела` collapse
    // to the canonical `как дела` entry.
    for variant in phrasal_variants(page_title, source_lang) {
        if let Some((slug, surface)) =
            seed_compositional_translation(&variant, source_lang, target_lang)
        {
            provenance.push(format!(
                "compositional:{source_lang}->{target_lang}:{page_title}=>variant:{variant}"
            ));
            *meaning = seed_meaning_id(slug);
            return vec![WiktionaryCandidate {
                surface: surface.to_owned(),
                qualifier: None,
            }];
        }
    }

    if !source_lang.eq_ignore_ascii_case("ru") || !target_lang.eq_ignore_ascii_case("en") {
        return Vec::new();
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

fn seed_meaning_id(slug: &str) -> MeaningId {
    MeaningId::from_wiktionary_page("seed", slug)
}

pub(crate) fn seed_meaning_for_surface(surface: &str, language: &str) -> Option<MeaningId> {
    let page_title = normalize_page_title(surface);
    seed_compositional_translation(&page_title, language, language)
        .map(|(slug, _)| seed_meaning_id(slug))
}

fn seed_compositional_translation(
    page_title: &str,
    source_lang: &str,
    target_lang: &str,
) -> Option<(&'static str, &'static str)> {
    seed_role_translation(
        ROLE_COMPOSITIONAL_PHRASE,
        source_lang,
        target_lang,
        page_title,
    )
    .or_else(|| {
        seed_role_translation(
            ROLE_COMPOSITIONAL_LEMMA,
            source_lang,
            target_lang,
            page_title,
        )
    })
}

fn seed_role_translation(
    role: &str,
    source_lang: &str,
    target_lang: &str,
    surface: &str,
) -> Option<(&'static str, &'static str)> {
    crate::seed::lexicon()
        .meanings
        .iter()
        .filter(|meaning| meaning.has_role(role))
        .find(|meaning| {
            meaning.lexemes.iter().any(|lexeme| {
                lexeme.language.eq_ignore_ascii_case(source_lang)
                    && lexeme
                        .words
                        .iter()
                        .any(|word| same_surface(&word.text, surface))
            })
        })
        .and_then(|meaning| {
            meaning
                .word_in(target_lang)
                .map(|target| (meaning.slug.as_str(), target))
        })
}

fn same_surface(left: &str, right: &str) -> bool {
    left == right || left.eq_ignore_ascii_case(right)
}

/// The normalized Russian title rendered as a whole, when it is a fixed phrase.
///
/// Looks the title up among the meanings carrying [`ROLE_COMPOSITIONAL_PHRASE`]
/// and returns the meaning's English form verbatim — capitalization and terminal
/// punctuation included. The phrases live in
/// `data/seed/meanings-translation.lino`; this names only the semantic role and
/// the language pair (issue #386).
fn russian_phrase_to_english(page_title: &str) -> Option<&'static str> {
    crate::seed::lexicon().role_surface_translation(
        ROLE_COMPOSITIONAL_PHRASE,
        "ru",
        "en",
        page_title,
    )
}

/// The English form of a single Russian compositional lemma surface.
///
/// Resolves `word` through the meaning carrying [`ROLE_COMPOSITIONAL_LEMMA`] that
/// lexicalises it, returning that meaning's first English form. Every inflection
/// lives in `data/seed/meanings-translation.lino`; this names only the role and
/// the language pair (issue #386).
fn russian_word_to_english(word: &str) -> Option<&'static str> {
    crate::seed::lexicon().role_surface_translation(ROLE_COMPOSITIONAL_LEMMA, "ru", "en", word)
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

/// Whether `word` is a relation noun that governs a Russian genitive complement.
///
/// True when some meaning carrying [`ROLE_COMPOSITIONAL_GENITIVE_HEAD`]
/// lexicalises `word` in Russian. The heads live in
/// `data/seed/meanings-translation.lino`; only the construction rule is code
/// (issue #386).
fn russian_genitive_relation_head(word: &str) -> bool {
    crate::seed::lexicon().role_lists_surface(ROLE_COMPOSITIONAL_GENITIVE_HEAD, "ru", word)
}

/// The English lemma of a Russian genitive-inflected complement, when tagged.
///
/// Resolves `word` only if a compositional lemma form lists it in Russian under
/// `action "genitive"`, returning that meaning's English form. The single tagged
/// inflection lives in `data/seed/meanings-translation.lino` (issue #386).
fn russian_genitive_noun(word: &str) -> Option<&'static str> {
    crate::seed::lexicon().role_action_surface_translation(
        ROLE_COMPOSITIONAL_LEMMA,
        "genitive",
        "ru",
        "en",
        word,
    )
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

#[path = "../source_tests/translation/pipeline/tests.rs"]
mod tests;
