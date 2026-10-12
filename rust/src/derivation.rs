//! White-box derivation record for every answer (issue #1184, E148).
//!
//! The umbrella's acceptance item asks that every answer carry its derivation
//! as links — the search queries issued, the fetched URLs with hashes, the
//! formalized page fragments, the parts decomposed from each example, the
//! recomposition, the rendering, and the verification output — and that
//! `formal-ai explain <answer-id>` print it afterwards.
//!
//! The projection is deliberately the *same* [`EventLog`] that
//! `thinking.rs` narrates: [`Derivation::record_for`] reads
//! `EventLog::events()` and nothing else, so the `--thinking` trace and the
//! derivation record cannot disagree (requirement R6). Stages a route never
//! populated stay empty and are printed as "not recorded" — never fabricated
//! (R5).
//!
//! Durability (R3) follows the `cache_path data/cache/...` convention of
//! `data/seed/sources-registry.lino`: each record persists as one Links
//! Notation file under `data/cache/derivations/<answer_id>.lino`, keyed by
//! the content-addressed answer id, so a later `explain` does not replay the
//! original request.
//!
//! The stage kinds the later issues append (`formalize:fragment`, `decompose:
//! part`, `recompose:bind`, `render:emit`) are named here as public
//! constants so E128/#1163, E129/#1164, E131/#1166 and E132/#1167 land on
//! one vocabulary instead of four private spellings.

use crate::engine::stable_id;
use crate::event_log::EventLog;
use crate::execution_evidence::Evidence;
use crate::links_format::push_lino_node;
use crate::seed::parser::parse_lino;
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// What a stage a route never populated is reported as (R5). Public so the
/// CLI renderer and the tests quote one spelling.
pub const NOT_RECORDED: &str = "not recorded";

/// Event kind holding the text of one issued search query.
pub const SEARCH_REQUEST_KIND: &str = "web_search:request";
/// Event kind holding one fetch trace (`SourceCapture::trace_payload` or the
/// `url=…;fetched_at=…;sha256=…;catalog_match=…` form the coding synthesis
/// runtime appends).
pub const SOURCE_HTTP_KIND: &str = "source:http";
/// Event kind a formalization stage appends per consumed page fragment
/// (E128/#1163).
pub const FORMALIZE_FRAGMENT_KIND: &str = "formalize:fragment";
/// Event kind a decomposition stage appends per part extracted from a
/// retrieved example (E129/#1164).
pub const DECOMPOSE_PART_KIND: &str = "decompose:part";
/// Event kind the recomposition stage appends once, naming the binding of
/// parts to parameters (E129/#1164, against E131/#1166 obligations).
pub const RECOMPOSE_BIND_KIND: &str = "recompose:bind";
/// Event kind the rendering stage appends once, naming the emitted target
/// language (E132/#1167).
pub const RENDER_EMIT_KIND: &str = "render:emit";
/// Event kind a verified route appends per executed check, with
/// `evidence_id=… command=… exit=…` fields (verifiable-task route).
pub const VERIFICATION_KIND: &str = "verify:evidence";

/// The record's shape and stage vocabulary as data (`data/seed/derivation-schema.lino`).
const SCHEMA: &str = include_str!("../embedded/data/seed/derivation-schema.lino");
/// The schema field a rule-applying handler's stages collect into (issue
/// #1174 R9): a register substitution, a grammar fix with its rule, a
/// style-guide clause, a summarization bound, a translation step.
const APPLIED_RULE_FIELD: &str = "rule";

/// The [`EventLog`] kinds the schema's `stage` rows declare as collecting
/// [`APPLIED_RULE_FIELD`]. A handler joins the record by naming its event
/// kind in the schema, not by an edit here.
fn applied_rule_kinds() -> &'static BTreeSet<String> {
    static KINDS: OnceLock<BTreeSet<String>> = OnceLock::new();
    KINDS.get_or_init(|| {
        parse_lino(SCHEMA)
            .children
            .iter()
            .flat_map(|schema| schema.children.iter())
            .filter(|row| row.name == "stage")
            .filter(|row| row.find_child_value("collects") == APPLIED_RULE_FIELD)
            .map(|row| row.find_child_value("kind").to_owned())
            .filter(|kind| !kind.is_empty())
            .collect()
    })
}

/// One rule a handler applied: the event kind that names the rule family
/// and the payload that names the rule and what it changed.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AppliedRule {
    pub kind: String,
    pub detail: String,
}

/// Where durable derivation records live under the repository root.
pub const DERIVATIONS_DIR: &str = "data/cache/derivations";

/// One fetched URL with its SHA-256 hash and fetch timestamp.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FetchRecord {
    pub url: String,
    pub sha256: String,
    pub fetched_at: String,
}

/// The inspectable triple of one executed verification.
///
/// The full
/// [`Evidence`] record (argv, observation kind, source) stays recoverable
/// through `evidence_id` in the obligation ledger; this projection keeps
/// what an `explain` reader needs on one line.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VerificationRecord {
    pub evidence_id: String,
    pub command: String,
    pub exit_code: Option<i64>,
}

impl VerificationRecord {
    /// Project one executed observation into its inspectable triple.
    #[must_use]
    pub fn from_evidence(evidence: &Evidence) -> Self {
        Self {
            evidence_id: evidence.evidence_id.clone(),
            command: evidence.command.clone(),
            exit_code: evidence.exit_code,
        }
    }

    /// The compact `evidence_id=… command=… exit=…` payload the
    /// [`VERIFICATION_KIND`] event carries, so routes and this module agree
    /// on one spelling.
    #[must_use]
    pub fn payload(&self) -> String {
        let exit = self
            .exit_code
            .map_or_else(|| String::from("none"), |code| code.to_string());
        format!(
            "evidence_id={};command={};exit={}",
            self.evidence_id, self.command, exit
        )
    }

    /// Parse the [`VERIFICATION_KIND`] payload back. Returns `None` when the
    /// payload carries no `evidence_id=` field, so unrelated payloads are
    /// skipped rather than half-read.
    #[must_use]
    pub fn parse_payload(payload: &str) -> Option<Self> {
        let rest = payload.strip_prefix("evidence_id=")?;
        let (evidence_id, rest) = rest
            .split_once(";command=")
            .or_else(|| rest.split_once(" command="))?;
        let (command, exit) = rest
            .rsplit_once(";exit=")
            .or_else(|| rest.rsplit_once(" exit="))?;
        if evidence_id.is_empty() {
            return None;
        }
        let evidence_id = evidence_id.to_owned();
        let command = command.to_owned();
        let exit_code = exit.parse::<i64>().ok();
        Some(Self {
            evidence_id,
            command,
            exit_code,
        })
    }
}

/// The derivation of one answer, projected from the same [`EventLog`] the
/// thinking trace narrates.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Derivation {
    pub answer_id: String,
    pub search_queries: Vec<String>,
    pub fetches: Vec<FetchRecord>,
    pub formalized_fragments: Vec<String>,
    pub decomposed_parts: Vec<String>,
    pub recomposition: Option<String>,
    pub rendering: Option<String>,
    pub verification: Vec<VerificationRecord>,
    pub applied_rules: Vec<AppliedRule>,
}

/// The content-addressed id every answer carries (R1).
///
/// Computed exactly the
/// way `SymbolicAnswer::derivation_id` does: `stable_id("answer", &answer)`
/// over the answer text, using the same FNV-1a scheme that
/// `VerifiedAnswer::derivation_id` generalizes.
#[must_use]
pub fn answer_derivation_id(answer_text: &str) -> String {
    stable_id("answer", answer_text)
}

/// Complete every solver route, including early clarification/fallback returns,
/// using the same log from which its thinking trace was projected. Persistence
/// failures are exposed as evidence rather than silently claiming a durable link.
pub(crate) fn finalize_answer(answer: &mut crate::engine::SymbolicAnswer, log: &mut EventLog) {
    let answer_id = answer.derivation_id();
    // Nested solves may already have emitted their own render event. Append
    // this answer's final rendering last so the projected record names the
    // response that is actually being returned.
    log.append(
        RENDER_EMIT_KIND,
        format!("answer_id={answer_id};format=text"),
    );
    let record = Derivation::record_for(log, &answer_id);
    answer.thinking_steps = log.thinking_steps_for_answer(&answer.answer);
    // Issue #667 (R383): under `serve --debug-session` with stepping on, the
    // solved turn is held here, before its derivation record is persisted and
    // the answer returned, and its stages are handed out one per advance.
    crate::server::gate_turn(&answer.thinking_steps);
    answer
        .evidence_links
        .push(format!("derivation:{answer_id}"));
    answer.links_notation.push('\n');
    answer.links_notation.push_str(&record.to_lino());
    let persisted = std::env::current_dir().and_then(|root| record.persist(&root));
    match persisted {
        Ok(_) => answer
            .evidence_links
            .push(format!("{DERIVATIONS_DIR}/{answer_id}.lino")),
        Err(error) => answer
            .evidence_links
            .push(format!("derivation:persistence_failed:{error}")),
    }
}

impl Derivation {
    /// Project the derivation of `answer_id` from the log the answer was
    /// built from (R2, R6). Events are read in append order; stages with no
    /// events stay empty, and `explain_text` is what reports them as "not
    /// recorded" — this record never invents a value (R5).
    #[must_use]
    pub fn record_for(log: &EventLog, answer_id: &str) -> Self {
        let mut derivation = Self {
            answer_id: answer_id.to_string(),
            ..Self::default()
        };
        for event in log.events() {
            match event.kind {
                SEARCH_REQUEST_KIND => {
                    derivation.search_queries.push(event.payload.clone());
                }
                SOURCE_HTTP_KIND => {
                    if let Some(fetch) = parse_source_http(&event.payload) {
                        derivation.fetches.push(fetch);
                    }
                }
                FORMALIZE_FRAGMENT_KIND => {
                    derivation.formalized_fragments.push(event.payload.clone());
                }
                DECOMPOSE_PART_KIND => {
                    derivation.decomposed_parts.push(event.payload.clone());
                }
                RECOMPOSE_BIND_KIND => {
                    derivation.recomposition = Some(event.payload.clone());
                }
                RENDER_EMIT_KIND => {
                    derivation.rendering = Some(event.payload.clone());
                }
                VERIFICATION_KIND => {
                    if let Some(record) = VerificationRecord::parse_payload(&event.payload) {
                        derivation.verification.push(record);
                    }
                }
                kind if applied_rule_kinds().contains(kind) => {
                    derivation.applied_rules.push(AppliedRule {
                        kind: kind.to_owned(),
                        detail: event.payload.clone(),
                    });
                }
                _ => {}
            }
        }
        derivation
    }

    /// Render the canonical Links Notation record, styled like the
    /// `sources-registry` documents: absent stages are omitted entirely, so
    /// the file never carries a fabricated value (R5). Values are quoted by
    /// the notation's own encoder (`push_lino_node`), never hand-rolled.
    #[must_use]
    pub fn to_lino(&self) -> String {
        let mut out = String::new();
        push_lino_node(&mut out, 0, "derivation", None);
        push_lino_node(&mut out, 2, "answer_id", Some(&self.answer_id));
        for query in &self.search_queries {
            push_lino_node(&mut out, 2, "search_query", Some(query));
        }
        for fetch in &self.fetches {
            push_lino_node(&mut out, 2, "fetch", None);
            push_lino_node(&mut out, 4, "url", Some(&fetch.url));
            push_lino_node(&mut out, 4, "sha256", Some(&fetch.sha256));
            push_lino_node(&mut out, 4, "fetched_at", Some(&fetch.fetched_at));
        }
        for fragment in &self.formalized_fragments {
            push_lino_node(&mut out, 2, "formalized_fragment", Some(fragment));
        }
        for part in &self.decomposed_parts {
            push_lino_node(&mut out, 2, "decomposed_part", Some(part));
        }
        if let Some(recomposition) = &self.recomposition {
            push_lino_node(&mut out, 2, "recomposition", Some(recomposition));
        }
        if let Some(rendering) = &self.rendering {
            push_lino_node(&mut out, 2, "rendering", Some(rendering));
        }
        for record in &self.verification {
            push_lino_node(&mut out, 2, "verification", None);
            push_lino_node(&mut out, 4, "evidence_id", Some(&record.evidence_id));
            push_lino_node(&mut out, 4, "command", Some(&record.command));
            let exit = record
                .exit_code
                .map_or_else(|| String::from("none"), |code| code.to_string());
            push_lino_node(&mut out, 4, "exit", Some(&exit));
        }
        for rule in &self.applied_rules {
            push_lino_node(&mut out, 2, APPLIED_RULE_FIELD, None);
            push_lino_node(&mut out, 4, "kind", Some(&rule.kind));
            push_lino_node(&mut out, 4, "detail", Some(&rule.detail));
        }
        out
    }

    /// Read a record [`Derivation::to_lino`] wrote. Returns `None` when the
    /// document is not a derivation record or carries no `answer_id`.
    #[must_use]
    pub fn from_lino(text: &str) -> Option<Self> {
        let root = parse_lino(text);
        let record = root
            .children
            .iter()
            .find(|child| child.name == "derivation")?;
        let answer_id = record.find_child_value("answer_id");
        if answer_id.is_empty() {
            return None;
        }
        let mut derivation = Self {
            answer_id: answer_id.to_string(),
            ..Self::default()
        };
        // Field rows sit directly under the record head; wrapper rows
        // (`fetch`, `verification`) nest their fields one level deeper.
        for child in &record.children {
            match child.name.as_str() {
                "search_query" => derivation.search_queries.push(child.id.clone()),
                "fetch" => derivation.fetches.push(FetchRecord {
                    url: child.find_child_value("url").to_string(),
                    sha256: child.find_child_value("sha256").to_string(),
                    fetched_at: child.find_child_value("fetched_at").to_string(),
                }),
                "formalized_fragment" => {
                    derivation.formalized_fragments.push(child.id.clone());
                }
                "decomposed_part" => derivation.decomposed_parts.push(child.id.clone()),
                "recomposition" => derivation.recomposition = Some(child.id.clone()),
                "rendering" => derivation.rendering = Some(child.id.clone()),
                "verification" => {
                    let exit = child.find_child_value("exit");
                    derivation.verification.push(VerificationRecord {
                        evidence_id: child.find_child_value("evidence_id").to_string(),
                        command: child.find_child_value("command").to_string(),
                        exit_code: if exit == "none" {
                            None
                        } else {
                            exit.parse::<i64>().ok()
                        },
                    });
                }
                APPLIED_RULE_FIELD => derivation.applied_rules.push(AppliedRule {
                    kind: child.find_child_value("kind").to_string(),
                    detail: child.find_child_value("detail").to_string(),
                }),
                _ => {}
            }
        }
        Some(derivation)
    }

    /// The human-readable `formal-ai explain` output (R4, R5): every stage is
    /// named, and a stage the route never populated is printed as
    /// "not recorded" rather than omitted or invented.
    #[must_use]
    pub fn explain_text(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "derivation {}", self.answer_id);
        let _ = writeln!(out, "  stage search_queries");
        if self.search_queries.is_empty() {
            let _ = writeln!(out, "    {NOT_RECORDED}");
        } else {
            for query in &self.search_queries {
                let _ = writeln!(out, "    {query}");
            }
        }
        let _ = writeln!(out, "  stage fetches");
        if self.fetches.is_empty() {
            let _ = writeln!(out, "    {NOT_RECORDED}");
        } else {
            for fetch in &self.fetches {
                let _ = writeln!(
                    out,
                    "    url {} sha256 {} fetched_at {}",
                    fetch.url, fetch.sha256, fetch.fetched_at
                );
            }
        }
        let _ = writeln!(out, "  stage formalized_fragments");
        if self.formalized_fragments.is_empty() {
            let _ = writeln!(out, "    {NOT_RECORDED}");
        } else {
            for fragment in &self.formalized_fragments {
                let _ = writeln!(out, "    {fragment}");
            }
        }
        let _ = writeln!(out, "  stage decomposed_parts");
        if self.decomposed_parts.is_empty() {
            let _ = writeln!(out, "    {NOT_RECORDED}");
        } else {
            for part in &self.decomposed_parts {
                let _ = writeln!(out, "    {part}");
            }
        }
        let _ = writeln!(
            out,
            "  stage recomposition: {}",
            self.recomposition.as_deref().unwrap_or(NOT_RECORDED)
        );
        let _ = writeln!(
            out,
            "  stage rendering: {}",
            self.rendering.as_deref().unwrap_or(NOT_RECORDED)
        );
        let _ = writeln!(out, "  stage verification");
        if self.verification.is_empty() {
            let _ = writeln!(out, "    {NOT_RECORDED}");
        } else {
            for record in &self.verification {
                let exit = record
                    .exit_code
                    .map_or_else(|| String::from("none"), |code| code.to_string());
                let _ = writeln!(out, "    {} exit={}", record.command, exit);
            }
        }
        let _ = writeln!(out, "  stage applied_rules");
        if self.applied_rules.is_empty() {
            let _ = writeln!(out, "    {NOT_RECORDED}");
        } else {
            for rule in &self.applied_rules {
                let _ = writeln!(out, "    {} {}", rule.kind, rule.detail);
            }
        }
        out
    }

    /// Persist the record under `data/cache/derivations/<answer_id>.lino`
    /// (R3), creating the directory. The path is returned so callers can
    /// cite it in evidence links.
    pub fn persist(&self, repository_root: &Path) -> std::io::Result<PathBuf> {
        let path = store_path(repository_root, &self.answer_id)
            .ok_or_else(|| std::io::Error::other("unusable derivation answer id"))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, self.to_lino())?;
        Ok(path)
    }

    /// Load a previously persisted record by answer id (R3, R4). `None` when
    /// no such record exists — an honest miss, never a fabricated one.
    #[must_use]
    pub fn load(repository_root: &Path, answer_id: &str) -> Option<Self> {
        let path = store_path(repository_root, answer_id)?;
        let text = std::fs::read_to_string(path).ok()?;
        let derivation = Self::from_lino(&text)?;
        (derivation.answer_id == answer_id).then_some(derivation)
    }
}

/// The `data/cache/derivations/<answer_id>.lino` path for one answer id.
///
/// Returns `None` when the id is not a `[A-Za-z0-9_-]` token — a caller-supplied id
/// that could traverse the tree (`../`) is refused, not normalized.
#[must_use]
pub fn store_path(repository_root: &Path, answer_id: &str) -> Option<PathBuf> {
    let safe = !answer_id.is_empty()
        && answer_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    safe.then(|| {
        repository_root
            .join(DERIVATIONS_DIR)
            .join(format!("{answer_id}.lino"))
    })
}

/// The `explain` entry (R4): load a durable record and render it,
/// reporting a miss honestly.
pub fn explain_answer(repository_root: &Path, answer_id: &str) -> Result<String, String> {
    Derivation::load(repository_root, answer_id)
        .map(|derivation| derivation.explain_text())
        .ok_or_else(|| miss_message(repository_root, answer_id))
}

/// The one miss message, shared by every `explain` surface so the CLI and
/// the API cannot disagree about what a caller should do next.
#[must_use]
pub fn miss_message(repository_root: &Path, answer_id: &str) -> String {
    let directory = repository_root.join(DERIVATIONS_DIR).display().to_string();
    crate::seed::report_text(
        "derivation_record_missing",
        &[("answer_id", answer_id), ("directory", &directory)],
    )
}

/// Parse one `source:http` payload into a fetch record. Two spellings exist
/// in the tree and both are read:
/// - `SourceCapture::trace_payload`: `<url> fetched_at=… sha256=… cached=…`
///   (the url is the leading bare token, space-separated fields),
/// - the coding synthesis runtime's `url=…;fetched_at=…;sha256=…;…`
///   (semicolon-separated `key=value` pairs, `url=` included).
///
/// A payload without a url is skipped (`None`), not guessed at.
#[must_use]
fn parse_source_http(payload: &str) -> Option<FetchRecord> {
    let trimmed = payload.trim();
    if trimmed.is_empty() {
        return None;
    }
    let (url, entries): (String, Vec<&str>) = if let Some(rest) = trimmed.strip_prefix("url=") {
        // Semicolon-separated form; the url runs to the first `;`.
        let mut parts = rest.split(';');
        let url = parts.next()?.trim().to_string();
        (url, parts.collect())
    } else {
        // trace_payload form; the url is the leading token.
        let mut parts = trimmed.split(' ');
        let url = parts.next()?.to_string();
        (url, parts.collect())
    };
    if url.is_empty() {
        return None;
    }
    let fields = parse_entries(&entries);
    let field = |name: &str| {
        fields
            .iter()
            .find(|(key, _)| *key == name)
            .map_or_else(String::new, |(_, value)| value.clone())
    };
    Some(FetchRecord {
        url,
        sha256: field("sha256"),
        fetched_at: field("fetched_at"),
    })
}

/// Split compact `key=value` entries into pairs. Empty entries collapse, so
/// `a=1;;b=2` and `a=1; b=2` read the same.
fn parse_entries(entries: &[&str]) -> Vec<(String, String)> {
    entries
        .iter()
        .filter_map(|entry| {
            let entry = entry.trim();
            if entry.is_empty() {
                return None;
            }
            entry
                .split_once('=')
                .map(|(name, value)| (name.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}
