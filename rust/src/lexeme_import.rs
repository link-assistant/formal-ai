//! Bulk lexeme importer (issue #660, R378).
//!
//! Generalises the one-off `scripts/ground-meanings.rs` into a reusable,
//! deterministic pipeline that turns a list of Wikidata concepts into grounded
//! [`Meaning`](crate::seed::Meaning) blocks for the seed. Each concept is a
//! `<slug> <Qid>` pair; the importer reads every full-support language registered
//! in `data/seed/languages.lino` from the committed Wikidata entity cache and
//! emits a `meanings` block whose surfaces denote the meaning and carry their
//! `part_of_speech`/`grammatical_number` facets. A partial-support language is
//! read the same way, but as an optional surface: it is emitted when the record
//! holds a clean surface for it and left out, not refused, when it does not.
//!
//! Two invariants make this safe to ship:
//!
//!   * **Offline determinism.** With `--offline` (or without
//!     `FORMAL_AI_LIVE_API`) the importer only ever reads the committed
//!     `data/cache/wikidata/entity/<Qid>.json` records, so it reproduces the
//!     committed batch byte-for-byte and the tests never touch the network.
//!   * **Validate-then-write.** Every generated block is parsed back through the
//!     real seed loader ([`parse_lexicon_text`]); a concept whose surfaces fail
//!     to parse, fail to denote their meaning, or lack a facet is refused and
//!     recorded as an `import_rejected` event rather than written to the seed.
//!
//! Live population (fetch + trim + cache-write) is gated behind
//! `FORMAL_AI_LIVE_API` and honours the bounded-cache policy
//! ([`cache_capacity`]); it is never exercised in CI because the committed
//! snapshots make the pipeline offline.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::event_log::EventLog;
use crate::json_lino::json_cache_file;
use crate::knowledge::cache_capacity;
use crate::seed::{LANGUAGES_LINO, localized_response, parse_lexicon_text};
use crate::translation::http::HttpClient;

mod cache_trim;

use cache_trim::{serialize_trimmed, trim_entity};

/// Full-support project languages, derived from the registry in ledger order.
/// Every imported concept must carry a surface in each of them.
#[must_use]
pub fn import_languages() -> Vec<&'static str> {
    languages_with_status("full")
}

/// Partial-support project languages, in the registry's ledger order.
///
/// A concept carries a surface in each of them when its cache record
/// holds a clean one; a missing one leaves the language out of the block
/// instead of refusing the concept.
#[must_use]
pub fn optional_import_languages() -> Vec<&'static str> {
    languages_with_status("partial")
}

/// Every language the importer reads and caches: the full-support languages,
/// then the partial-support ones.
fn cached_languages() -> Vec<&'static str> {
    let mut languages = import_languages();
    languages.extend(optional_import_languages());
    languages
}

/// The registry languages whose `status` is `status`, in ledger order.
fn languages_with_status(status: &str) -> Vec<&'static str> {
    let status_line = format!("    status {status}");
    let mut current = None;
    let mut languages = Vec::new();
    for line in LANGUAGES_LINO.lines() {
        if let Some(language) = line.strip_prefix("  language ") {
            current = Some(language);
        } else if line == status_line
            && let Some(language) = current.take()
        {
            languages.push(language);
        }
    }
    languages
}

/// The languages `lexeme` is emitted in, in block order: every full-support
/// language, then each partial-support language it has a surface for.
fn emitted_languages(lexeme: &GroundedLexeme) -> Vec<&'static str> {
    let mut languages = import_languages();
    languages.extend(
        optional_import_languages()
            .into_iter()
            .filter(|language| lexeme.labels.contains_key(*language)),
    );
    languages
}

/// The part of speech every imported common noun is tagged with.
pub const PART_OF_SPEECH: &str = "noun";

/// The grammatical number every imported label lexicalises (the citation form).
pub const GRAMMATICAL_NUMBER: &str = "singular";

/// The genus every imported meaning is defined by.
pub const DEFINED_BY: &str = "entity";

/// Maximum lines written per shard file, header included.
///
/// A block's length grows with the languages it carries, so shards are filled by lines rather
/// than by a concept count; the budget is the data-file warning threshold of
/// `scripts/check-file-size.rs`, under the 1500-line ceiling enforced by
/// `tests/unit/data_files.rs`.
pub const SHARD_LINE_BUDGET: usize = 1_400;

/// Render an importer diagnostic from the grounded seed response registry.
///
/// Callers provide semantic placeholder names rather than embedding prose in
/// Rust. Missing records deliberately degrade to the stable intent key, making
/// a damaged seed observable without introducing a second wording authority.
#[must_use]
pub fn diagnostic(intent: &str, values: &[(&str, &str)]) -> String {
    diagnostic_for_language(intent, "en", values)
}

/// Render a localized importer diagnostic, falling back to English when the
/// requested locale has no grounded record.
#[must_use]
pub fn diagnostic_for_language(intent: &str, language: &str, values: &[(&str, &str)]) -> String {
    let mut rendered = localized_response(intent, language).unwrap_or_else(|| intent.to_owned());
    for (name, value) in values {
        rendered = rendered.replace(&format!("{{{name}}}"), value);
    }
    rendered
}

/// The comment header prepended to every generated shard.
const SHARD_HEADER: &str = "\
# `Bulk Wikidata lexeme import for issue 660, R378.`
# `Generated by formal-ai import lexemes; regenerate instead of hand-editing.`
# `Source records live under data/cache/wikidata/entity/<Qid>.json.`
meanings
";
const MEANINGS_HEAD: &str = "meanings";

/// A concept to import: a meaning slug bound to a Wikidata entity id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Concept {
    pub slug: String,
    pub qid: String,
}

/// A validated, ready-to-emit grounded lexeme.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroundedLexeme {
    pub slug: String,
    pub qid: String,
    /// Language code → single-token surface: one entry per [`import_languages`],
    /// plus one per [`optional_import_languages`] the record has a surface for.
    pub labels: BTreeMap<String, String>,
    /// Language code → the exact cache record field that supplied the surface.
    /// Item labels are not misrepresented as Wikidata Lexeme (`L…`) records.
    pub sources: BTreeMap<String, SurfaceSource>,
}

/// Auditable provenance for one imported surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceSource {
    pub record_id: String,
    pub field: String,
}

/// Chosen language surfaces paired with their exact source fields.
pub type SurfaceMaps = (BTreeMap<String, String>, BTreeMap<String, SurfaceSource>);

/// A refused concept and the reason it failed validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rejection {
    pub slug: String,
    pub qid: String,
    pub reason: String,
}

/// One emitted seed shard file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shard {
    pub file_name: String,
    pub content: String,
}

/// The outcome of an import run.
#[derive(Debug, Clone, Default)]
pub struct ImportReport {
    pub accepted: Vec<GroundedLexeme>,
    pub rejected: Vec<Rejection>,
    pub shards: Vec<Shard>,
    pub coverage: ImportCoverage,
}

/// Explicit coverage denominator for an import run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportCoverage {
    pub requested_concepts: usize,
    pub accepted_concepts: usize,
    pub expected_surfaces: usize,
    pub emitted_surfaces: usize,
}

impl ImportCoverage {
    /// Surface coverage in parts per thousand; an empty request is complete.
    #[must_use]
    pub fn permille(&self) -> u32 {
        if self.expected_surfaces == 0 {
            return 1_000;
        }
        u32::try_from(self.emitted_surfaces.saturating_mul(1_000) / self.expected_surfaces)
            .unwrap_or(u32::MAX)
    }
}

/// How to run the importer.
pub struct ImportConfig {
    /// The concepts to import, in the order they should be emitted.
    pub concepts: Vec<Concept>,
    /// Directory holding `<Qid>.json` (and `<Qid>.lino`) entity records.
    pub cache_dir: PathBuf,
    /// When `true`, missing cache records may be fetched live (subject to the
    /// bounded-cache budget). Resolve this from `--offline` and
    /// [`live_api_enabled`] before constructing the config.
    pub online: bool,
}

/// Whether live Wikidata population is enabled via the `FORMAL_AI_LIVE_API`
/// environment variable (`1`, `true`, `yes`, `on`, case-insensitive).
#[must_use]
pub fn live_api_enabled() -> bool {
    crate::cli_env::flag_enabled("FORMAL_AI_LIVE_API")
}

/// Parse a concepts document: a `concepts` node whose children are `<slug>
/// <Qid>` pairs. Lines that are not `<slug> Q<number>` pairs are ignored.
#[must_use]
pub fn parse_concepts(text: &str) -> Vec<Concept> {
    let root = crate::seed::parser::parse_lino(text);
    let mut concepts = Vec::new();
    let containers: Vec<_> = root
        .children
        .iter()
        .filter(|child| child.name == "concepts")
        .collect();
    let sources = if containers.is_empty() {
        vec![&root]
    } else {
        containers
    };
    for container in sources {
        for child in &container.children {
            let slug = child.name.trim();
            let qid = child.id.trim();
            if slug.is_empty() || !is_entity_id(qid) {
                continue;
            }
            concepts.push(Concept {
                slug: slug.to_string(),
                qid: qid.to_string(),
            });
        }
    }
    concepts
}

/// Whether `id` is a Wikidata item id (`Q` followed by digits).
#[must_use]
pub fn is_entity_id(id: &str) -> bool {
    id.strip_prefix('Q')
        .is_some_and(|rest| !rest.is_empty() && rest.bytes().all(|byte| byte.is_ascii_digit()))
}

/// Path of the committed JSON entity record for `qid`.
#[must_use]
pub fn entity_json_path(cache_dir: &Path, qid: &str) -> PathBuf {
    cache_dir.join(format!("{qid}.json"))
}

/// Path of the canonical `LiNo` entity record for `qid`.
#[must_use]
pub fn entity_lino_path(cache_dir: &Path, qid: &str) -> PathBuf {
    cache_dir.join(format!("{qid}.lino"))
}

/// Run the importer.
///
/// Returns the accepted lexemes, the refused concepts, and the emitted shard
/// files. Every rejection is also appended to `events` as an `import_rejected`
/// event so the refusal is auditable.
pub fn run(
    config: &ImportConfig,
    http: Option<&dyn HttpClient>,
    events: &mut EventLog,
) -> ImportReport {
    let budget = cache_capacity(config.concepts.len());
    let mut cached = config
        .concepts
        .iter()
        .filter(|concept| entity_json_path(&config.cache_dir, &concept.qid).is_file())
        .count();

    let mut report = ImportReport::default();
    let mut seen_slugs: BTreeMap<String, String> = BTreeMap::new();
    let mut seen_qids: BTreeMap<String, String> = BTreeMap::new();

    for concept in &config.concepts {
        if let Some(other) = seen_slugs.get(&concept.slug) {
            reject(
                &mut report,
                events,
                concept,
                diagnostic("lexeme_import_duplicate_slug", &[("other", other)]),
            );
            continue;
        }
        if let Some(other) = seen_qids.get(&concept.qid) {
            reject(
                &mut report,
                events,
                concept,
                diagnostic("lexeme_import_duplicate_qid", &[("other", other)]),
            );
            continue;
        }

        let (labels, sources) = match resolve_surfaces(config, concept, http, budget, &mut cached) {
            Ok(surfaces) => surfaces,
            Err(reason) => {
                reject(&mut report, events, concept, reason);
                continue;
            }
        };

        let lexeme = GroundedLexeme {
            slug: concept.slug.clone(),
            qid: concept.qid.clone(),
            labels,
            sources,
        };
        if let Err(reason) = validate(&lexeme) {
            reject(&mut report, events, concept, reason);
            continue;
        }

        seen_slugs.insert(concept.slug.clone(), concept.qid.clone());
        seen_qids.insert(concept.qid.clone(), concept.slug.clone());
        report.accepted.push(lexeme);
    }

    report.shards = shard(&report.accepted);
    report.coverage = ImportCoverage {
        requested_concepts: config.concepts.len(),
        accepted_concepts: report.accepted.len(),
        expected_surfaces: config
            .concepts
            .len()
            .saturating_mul(cached_languages().len()),
        emitted_surfaces: report
            .accepted
            .iter()
            .map(|lexeme| lexeme.labels.len())
            .sum(),
    };
    report
}

fn reject(report: &mut ImportReport, events: &mut EventLog, concept: &Concept, reason: String) {
    events.append(
        "import_rejected",
        format!("{} {} {reason}", concept.slug, concept.qid),
    );
    report.rejected.push(Rejection {
        slug: concept.slug.clone(),
        qid: concept.qid.clone(),
        reason,
    });
}

/// Resolve the registered labels for `concept`, reading the committed cache or (when
/// online and within budget) fetching and populating it.
fn resolve_surfaces(
    config: &ImportConfig,
    concept: &Concept,
    http: Option<&dyn HttpClient>,
    budget: usize,
    cached: &mut usize,
) -> Result<SurfaceMaps, String> {
    let json_path = entity_json_path(&config.cache_dir, &concept.qid);
    if json_path.is_file() {
        let text = fs::read_to_string(&json_path).map_err(|error| {
            diagnostic(
                "lexeme_import_cache_unreadable",
                &[
                    ("path", &json_path.display().to_string()),
                    ("error", &error.to_string()),
                ],
            )
        })?;
        let value: Value = serde_json::from_str(&text).map_err(|error| {
            diagnostic(
                "lexeme_import_cache_invalid_json",
                &[
                    ("path", &json_path.display().to_string()),
                    ("error", &error.to_string()),
                ],
            )
        })?;
        return surfaces_from_entity(&value, &concept.qid);
    }

    if !config.online {
        return Err(diagnostic(
            "lexeme_import_cache_miss_offline",
            &[("qid", &concept.qid)],
        ));
    }
    if *cached >= budget {
        return Err(diagnostic(
            "lexeme_import_cache_budget_reached",
            &[
                ("budget", &budget.to_string()),
                ("cached", &cached.to_string()),
                ("qid", &concept.qid),
            ],
        ));
    }
    let client =
        http.ok_or_else(|| String::from("online mode requested without an HTTP client"))?;
    let labels = fetch_and_cache(&config.cache_dir, &concept.qid, client)?;
    *cached += 1;
    Ok(labels)
}

/// Extract the project-language surfaces from a trimmed Wikidata entity.
///
/// The document is `{entities: {<qid>: {labels: …}}}`: one surface per
/// full-support language, which must resolve, and one per partial-support
/// language that does.
pub fn surfaces_from_entity(value: &Value, qid: &str) -> Result<SurfaceMaps, String> {
    let entity = value
        .get("entities")
        .and_then(|entities| entities.get(qid))
        .ok_or_else(|| diagnostic("lexeme_import_qid_absent_from_cache", &[("qid", qid)]))?;
    if entity.get("labels").is_none() {
        return Err(diagnostic(
            "lexeme_import_no_labels_in_cache",
            &[("qid", qid)],
        ));
    }
    let mut out = BTreeMap::new();
    let mut sources = BTreeMap::new();
    let mut keep = |language: &str, (surface, field): (String, String)| {
        out.insert(language.to_string(), surface);
        sources.insert(
            language.to_string(),
            SurfaceSource {
                record_id: qid.to_string(),
                field,
            },
        );
    };
    for language in import_languages() {
        keep(language, entity_surface(entity, qid, language)?);
    }
    for language in optional_import_languages() {
        if let Ok(chosen) = entity_surface(entity, qid, language) {
            keep(language, chosen);
        }
    }
    Ok((out, sources))
}

/// The surface `entity` supplies for `language` and the field it came from:
/// the label when it is a clean surface in the language's script, else the
/// first alias that is.
fn entity_surface(entity: &Value, qid: &str, language: &str) -> Result<(String, String), String> {
    let label = entity
        .get("labels")
        .and_then(|labels| labels.get(language))
        .and_then(|label| label.get("value"))
        .and_then(Value::as_str)
        .ok_or_else(|| {
            diagnostic(
                "lexeme_import_missing_cached_label",
                &[("qid", qid), ("language", language)],
            )
        })?;
    if usable_surface(label, language) {
        return Ok((label.to_string(), format!("labels.{language}.value")));
    }
    entity
        .get("aliases")
        .and_then(|aliases| aliases.get(language))
        .and_then(Value::as_array)
        .and_then(|aliases| {
            aliases.iter().enumerate().find_map(|(index, alias)| {
                let value = alias.get("value").and_then(Value::as_str)?;
                usable_surface(value, language).then(|| {
                    (
                        value.to_string(),
                        format!("aliases.{language}[{index}].value"),
                    )
                })
            })
        })
        .ok_or_else(|| {
            diagnostic(
                "lexeme_import_no_clean_alias",
                &[("qid", qid), ("language", language), ("label", label)],
            )
        })
}

/// Whether `surface` can stand as an imported surface of `language`: written in
/// the language's script and a clean single token.
fn usable_surface(surface: &str, language: &str) -> bool {
    surface_matches_language(surface, language) && surface_is_clean(surface)
}

/// Compatibility projection for callers that only need the chosen surfaces.
pub fn labels_from_entity(value: &Value, qid: &str) -> Result<BTreeMap<String, String>, String> {
    surfaces_from_entity(value, qid).map(|(labels, _)| labels)
}

fn surface_matches_language(surface: &str, language: &str) -> bool {
    // Issue #706: the script a language writes in is a detection-registry fact,
    // so a new language's surfaces are validated without a Unicode range here.
    crate::language::surface_matches_language(surface, language)
}

/// Fetch, trim, and cache `qid` from the live Wikidata `Special:EntityData`
/// endpoint. Writes both the trimmed `.json` snapshot and its canonical `.lino`
/// sibling so the closure tests stay satisfied. Only reached under
/// `FORMAL_AI_LIVE_API`.
fn fetch_and_cache(
    cache_dir: &Path,
    qid: &str,
    http: &dyn HttpClient,
) -> Result<SurfaceMaps, String> {
    let url = format!("https://www.wikidata.org/wiki/Special:EntityData/{qid}.json");
    let body = http.get(&url).map_err(|error| {
        diagnostic(
            "lexeme_import_fetch_failed",
            &[("qid", qid), ("error", &error.to_string())],
        )
    })?;
    let full: Value = serde_json::from_str(&body).map_err(|error| {
        diagnostic(
            "lexeme_import_fetch_invalid_json",
            &[("qid", qid), ("error", &error.to_string())],
        )
    })?;
    let trimmed = trim_entity(&full, qid)?;
    let json = serialize_trimmed(&trimmed);
    fs::create_dir_all(cache_dir).map_err(|error| {
        diagnostic(
            "lexeme_import_cannot_create",
            &[
                ("path", &cache_dir.display().to_string()),
                ("error", &error.to_string()),
            ],
        )
    })?;
    let value: Value = serde_json::from_str(&json).map_err(|error| {
        diagnostic(
            "lexeme_import_reparse_failed",
            &[("qid", qid), ("error", &error.to_string())],
        )
    })?;
    fs::write(entity_json_path(cache_dir, qid), &json).map_err(|error| {
        diagnostic(
            "lexeme_import_cannot_write_json",
            &[("qid", qid), ("error", &error.to_string())],
        )
    })?;
    fs::write(
        entity_lino_path(cache_dir, qid),
        json_cache_file(qid, &value),
    )
    .map_err(|error| {
        diagnostic(
            "lexeme_import_cannot_write_lino",
            &[("qid", qid), ("error", &error.to_string())],
        )
    })?;
    surfaces_from_entity(&value, qid)
}

/// Validate a candidate lexeme.
///
/// Labels must be clean single tokens, and the rendered block must parse back
/// into a meaning that denotes its slug and carries the part-of-speech and
/// grammatical-number facets on every surface.
pub fn validate(lexeme: &GroundedLexeme) -> Result<(), String> {
    if !is_entity_id(&lexeme.qid) {
        return Err(diagnostic(
            "lexeme_import_invalid_qid",
            &[("qid", &lexeme.qid)],
        ));
    }
    if lexeme.slug.is_empty() || !slug_is_clean(&lexeme.slug) {
        return Err(diagnostic(
            "lexeme_import_invalid_slug",
            &[("slug", &lexeme.slug)],
        ));
    }
    for language in emitted_languages(lexeme) {
        let surface = lexeme
            .labels
            .get(language)
            .ok_or_else(|| diagnostic("lexeme_import_missing_label", &[("language", language)]))?;
        if !surface_is_clean(surface) {
            return Err(diagnostic(
                "lexeme_import_invalid_surface",
                &[("language", language), ("surface", surface)],
            ));
        }
        let source = lexeme.sources.get(language).ok_or_else(|| {
            diagnostic(
                "lexeme_import_missing_provenance",
                &[("language", language)],
            )
        })?;
        let label_field = format!("labels.{language}.value");
        let alias_prefix = format!("aliases.{language}[");
        if source.record_id != lexeme.qid
            || (source.field != label_field
                && !(source.field.starts_with(&alias_prefix) && source.field.ends_with("].value")))
        {
            return Err(diagnostic(
                "lexeme_import_invalid_provenance",
                &[("language", language), ("qid", &lexeme.qid)],
            ));
        }
    }

    // Parse the rendered block back through the real loader and confirm the
    // grounded structure — this is the "validate on import" contract.
    let block = render_block(lexeme);
    let mut document = String::from(MEANINGS_HEAD);
    document.push('\n');
    document.push_str(&block);
    let lexicon = parse_lexicon_text(&document);
    let meaning = lexicon
        .meanings
        .iter()
        .find(|meaning| meaning.slug == lexeme.slug)
        .ok_or_else(|| {
            diagnostic(
                "lexeme_import_block_parse_failed",
                &[("slug", &lexeme.slug)],
            )
        })?;
    if meaning.wikidata != lexeme.qid {
        return Err(diagnostic(
            "lexeme_import_grounding_lost",
            &[("slug", &lexeme.slug)],
        ));
    }
    if !meaning.defined_by.iter().any(|target| target == DEFINED_BY) {
        return Err(diagnostic(
            "lexeme_import_wrong_genus",
            &[("slug", &lexeme.slug), ("defined_by", DEFINED_BY)],
        ));
    }
    for language in emitted_languages(lexeme) {
        let expected = &lexeme.labels[language];
        let lexeme_block = meaning
            .lexemes
            .iter()
            .find(|lexeme| lexeme.language == language)
            .ok_or_else(|| {
                diagnostic(
                    "lexeme_import_lexeme_lost",
                    &[("slug", &meaning.slug), ("language", language)],
                )
            })?;
        let form = lexeme_block.words.first().ok_or_else(|| {
            diagnostic(
                "lexeme_import_surface_missing",
                &[("slug", &meaning.slug), ("language", language)],
            )
        })?;
        if &form.text != expected {
            return Err(diagnostic(
                "lexeme_import_surface_changed",
                &[("slug", &meaning.slug), ("language", language)],
            ));
        }
        if !form.denotations().any(|target| target == meaning.slug) {
            return Err(diagnostic(
                "lexeme_import_denotation_lost",
                &[("slug", &meaning.slug), ("language", language)],
            ));
        }
        if form.part_of_speech() != Some(PART_OF_SPEECH) {
            return Err(diagnostic(
                "lexeme_import_part_of_speech_lost",
                &[("slug", &meaning.slug), ("language", language)],
            ));
        }
        if form.grammatical_number() != Some(GRAMMATICAL_NUMBER) {
            return Err(diagnostic(
                "lexeme_import_number_lost",
                &[("slug", &meaning.slug), ("language", language)],
            ));
        }
    }
    Ok(())
}

/// Whether `slug` is a bare, whitespace-free identifier safe to use as a
/// top-level meaning header.
fn slug_is_clean(slug: &str) -> bool {
    !slug.is_empty()
        && slug.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        })
}

/// Whether `surface` is a clean single token: non-empty, no whitespace, and no
/// character that would need quoting or start a comment when emitted bare.
fn surface_is_clean(surface: &str) -> bool {
    !surface.is_empty()
        && !surface.chars().any(char::is_whitespace)
        && !surface.contains(['#', '"', '\'', '`', '(', ')'])
}

/// Render the `meanings`-child block for one lexeme (indent 2 header).
#[must_use]
pub fn render_block(lexeme: &GroundedLexeme) -> String {
    let mut block = String::new();
    let _ = writeln!(block, "  {}", lexeme.slug);
    let _ = writeln!(block, "    grounded-in {}", lexeme.qid);
    let _ = writeln!(block, "    defined-by {DEFINED_BY}");
    for language in emitted_languages(lexeme) {
        let surface = &lexeme.labels[language];
        let _ = writeln!(block, "    lexeme {language}");
        let _ = writeln!(block, "      surface");
        let source = &lexeme.sources[language];
        let _ = writeln!(
            block,
            "        text {surface} # source {} {}",
            source.record_id, source.field
        );
        let _ = writeln!(block, "        part_of_speech {PART_OF_SPEECH}");
        let _ = writeln!(block, "        grammatical_number {GRAMMATICAL_NUMBER}");
    }
    block
}

/// Split the accepted lexemes into shard files.
///
/// Each shard carries the header and as many whole blocks as fit in
/// [`SHARD_LINE_BUDGET`] lines (a single block longer than the budget still gets
/// a shard of its own). Shards are named `meanings-lexicon-import` with a
/// zero-padded, one-based index when there is more than one.
#[must_use]
pub fn shard(accepted: &[GroundedLexeme]) -> Vec<Shard> {
    let header_lines = SHARD_HEADER.lines().count();
    let mut bodies: Vec<String> = Vec::new();
    let mut body_lines = 0;
    for lexeme in accepted {
        let block = render_block(lexeme);
        let block_lines = block.lines().count();
        let full = header_lines + body_lines + block_lines > SHARD_LINE_BUDGET;
        if bodies.is_empty() || (body_lines > 0 && full) {
            bodies.push(String::new());
            body_lines = 0;
        }
        if let Some(body) = bodies.last_mut() {
            body.push_str(&block);
        }
        body_lines += block_lines;
    }
    let single = bodies.len() == 1;
    bodies
        .into_iter()
        .enumerate()
        .map(|(index, body)| {
            let file_name = if single {
                String::from("meanings-lexicon-import.lino")
            } else {
                format!("meanings-lexicon-import-{:02}.lino", index + 1)
            };
            let mut content = String::from(SHARD_HEADER);
            content.push_str(&body);
            Shard { file_name, content }
        })
        .collect()
}

/// Project the import event stream into the portable memory wire format.
///
/// The CLI persists this document beside the generated shards, so rejected
/// entries remain replayable after the importing process exits.
#[must_use]
pub fn render_import_events(events: &EventLog) -> String {
    let mut out = String::from("demo_memory\n");
    for event in events.events() {
        let _ = writeln!(out, "  event \"{}\"", escape_lino(&event.id));
        let _ = writeln!(out, "    kind \"{}\"", escape_lino(event.kind));
        write_event_field(&mut out, "role", "assistant");
        write_event_field(&mut out, "intent", "lexeme_import");
        let _ = writeln!(out, "    content \"{}\"", escape_lino(&event.payload));
        write_event_field(&mut out, "conversationId", "issue-660");
        write_event_field(&mut out, "writeCount", "1");
    }
    out
}

fn write_event_field(out: &mut String, field: &str, value: &str) {
    let _ = writeln!(out, "    {field} \"{value}\"");
}

/// Replace the complete generated shard set without leaving stale files.
///
/// Each new file is staged in the destination directory before being renamed,
/// then obsolete importer-owned shards are removed. Unrelated seed files are
/// never touched.
pub fn write_shards(directory: &Path, shards: &[Shard]) -> Result<(), String> {
    fs::create_dir_all(directory).map_err(|error| {
        diagnostic(
            "lexeme_import_cannot_create",
            &[
                ("path", &directory.display().to_string()),
                ("error", &error.to_string()),
            ],
        )
    })?;

    for shard in shards {
        let staged = directory.join(format!(".{}.staged", shard.file_name));
        fs::write(&staged, &shard.content).map_err(|error| {
            diagnostic(
                "lexeme_import_cannot_stage",
                &[
                    ("path", &staged.display().to_string()),
                    ("error", &error.to_string()),
                ],
            )
        })?;
        fs::rename(&staged, directory.join(&shard.file_name)).map_err(|error| {
            diagnostic(
                "lexeme_import_cannot_install",
                &[("file", &shard.file_name), ("error", &error.to_string())],
            )
        })?;
    }

    let retained: std::collections::BTreeSet<&str> = shards
        .iter()
        .map(|shard| shard.file_name.as_str())
        .collect();
    let entries = fs::read_dir(directory).map_err(|error| {
        diagnostic(
            "lexeme_import_cannot_list",
            &[
                ("path", &directory.display().to_string()),
                ("error", &error.to_string()),
            ],
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            diagnostic(
                "lexeme_import_cannot_inspect",
                &[("error", &error.to_string())],
            )
        })?;
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        if file_name.starts_with("meanings-lexicon-import")
            && Path::new(file_name)
                .extension()
                .is_some_and(|ext| ext == "lino")
            && !retained.contains(file_name)
        {
            fs::remove_file(entry.path()).map_err(|error| {
                diagnostic(
                    "lexeme_import_cannot_remove_stale",
                    &[("file", file_name), ("error", &error.to_string())],
                )
            })?;
        }
    }
    Ok(())
}

fn escape_lino(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}
