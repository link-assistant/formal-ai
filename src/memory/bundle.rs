//! Full-memory bundle export / import (`formal_ai_bundle` Links Notation).
//!
//! A bundle is the self-contained `.lino` document the browser's
//! **Export memory** topbar button now writes by default — seed files,
//! UI preferences, environment metadata, and the entire append-only
//! `demo_memory` event log in a single file. The CLI mirrors the same
//! defaults (`formal-ai memory export` writes a bundle; `--events-only`
//! opts back into the legacy `demo_memory` shape).

use std::collections::BTreeMap;

use super::{
    escape_value, format_event_into, isoformat_now, parse_links_notation, parse_quoted,
    split_first_token, MemoryEvent, BUNDLE_HEADER, ROOT_HEADER,
};

/// Build a single Links Notation bundle document.
///
/// Contains the static seed plus the dynamic memory log plus arbitrary
/// environment metadata. Mirrors the browser's `FormalAiMemory.exportBundle`
/// so a user can drop the same file into any interface.
#[must_use]
pub fn export_bundle(seed_files: &[(&str, &str)], events: &[MemoryEvent]) -> String {
    let mut out = String::from(BUNDLE_HEADER);
    out.push('\n');
    out.push_str("  exported_at \"");
    out.push_str(&escape_value(&isoformat_now()));
    out.push_str("\"\n");
    if !seed_files.is_empty() {
        out.push_str("  seed_files\n");
        for (name, contents) in seed_files {
            out.push_str("    file \"");
            out.push_str(&escape_value(name));
            out.push_str("\"\n");
            for line in contents.lines() {
                if line.is_empty() {
                    continue;
                }
                out.push_str("      ");
                out.push_str(line);
                out.push('\n');
            }
        }
    }
    out.push_str("  ");
    out.push_str(ROOT_HEADER);
    out.push('\n');
    for event in events {
        // Indent each event one level deeper than the standalone memory
        // export so it nests inside the bundle.
        let mut block = String::new();
        format_event_into(event, &mut block);
        for line in block.lines() {
            if line.is_empty() {
                continue;
            }
            out.push_str("  ");
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Recover the memory log section from a `formal_ai_bundle` document.
///
/// Returns `None` if the input is not a recognised bundle. Used by
/// `formal-ai bundle import` so a user can drag the web demo's
/// `formal-ai-bundle.lino` into the CLI and get back just the events.
#[must_use]
pub fn extract_memory_from_bundle(text: &str) -> Option<Vec<MemoryEvent>> {
    if !text.trim_start().starts_with(BUNDLE_HEADER) {
        return None;
    }
    let mut inner = String::new();
    let mut inside = false;
    for line in text.lines() {
        let indent = line.chars().take_while(|c| *c == ' ').count();
        let content = &line[indent..];
        if !inside {
            if indent == 2 && content == ROOT_HEADER {
                inside = true;
                inner.push_str(content);
                inner.push('\n');
            }
            continue;
        }
        if indent <= 2 && !content.starts_with("event ") && !content.is_empty() {
            // A sibling section at the same depth as the memory header ends
            // the memory block.
            if indent == 2 {
                break;
            }
        }
        if indent < 2 {
            break;
        }
        // Strip two spaces of bundle indentation so the inner doc looks like
        // a standalone `demo_memory` file the existing parser understands.
        let stripped = line.strip_prefix("  ").unwrap_or(line);
        inner.push_str(stripped);
        inner.push('\n');
    }
    if !inside {
        return None;
    }
    Some(parse_links_notation(&inner))
}

/// Environment metadata embedded at the top of a `formal_ai_bundle` document.
///
/// Mirrors the `info` object passed by the browser to
/// `FormalAiMemory.exportBundle({ info })`: a small free-form record that lets
/// the maintainer reconstruct the runtime context in which the export was
/// produced (app version, URL, user agent, worker state, demo/manual mode).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BundleInfo {
    pub exported_at: Option<String>,
    pub version: Option<String>,
    pub url: Option<String>,
    pub user_agent: Option<String>,
    pub worker_state: Option<String>,
    pub mode: Option<String>,
}

/// Structured view of a parsed `formal_ai_bundle` document.
///
/// Returned by [`import_full_memory`]. `seed_files` and `preferences` keep
/// insertion order; `agent_info` is the parsed key/value map recovered from
/// the embedded `data/seed/agent-info.lino` (or `seed/agent-info.lino`) file,
/// when present — it powers [`suggest_migrations`].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ParsedBundle {
    pub events: Vec<MemoryEvent>,
    pub seed_files: Vec<(String, String)>,
    pub preferences: Vec<(String, String)>,
    pub info: BundleInfo,
    pub agent_info: BTreeMap<String, String>,
}

/// Materialize imported seed files as recomputable `seed_cache` events.
///
/// This is the production producer for the `seed_cache` kind (issue #494 via
/// issue #540 §4): a full-memory import copies the bundle's seed files into
/// the event log, so seed data participates in usage counting and — being
/// classified as a recomputable cache by the dreaming lexicon — is among the
/// first data reclaimed under storage pressure. Ids are stable over the file
/// name, so re-importing the same bundle never duplicates the cache.
#[must_use]
pub fn seed_cache_events(seed_files: &[(String, String)]) -> Vec<MemoryEvent> {
    seed_files
        .iter()
        .map(|(name, contents)| MemoryEvent {
            id: crate::engine::stable_id("seed_cache", name),
            kind: Some(String::from("seed_cache")),
            intent: Some(String::from("seed")),
            tool: Some(name.clone()),
            content: Some(contents.clone()),
            ..MemoryEvent::default()
        })
        .collect()
}

/// Build the canonical full-memory `.lino` document — the same shape the
/// browser's "Export memory" button now produces.
///
/// Contains, in order: bundle metadata (`exported_at`, `version`, `url`,
/// `user_agent`, `worker_state`, `mode`), every seed file, optional UI
/// preferences, then the entire `demo_memory` event log. A single file is
/// enough to replay the agent's state on any interface.
#[must_use]
pub fn export_full_memory(
    seed_files: &[(&str, &str)],
    events: &[MemoryEvent],
    preferences: &[(&str, &str)],
    info: &BundleInfo,
) -> String {
    let mut out = String::from(BUNDLE_HEADER);
    out.push('\n');
    let exported_at = info.exported_at.clone().unwrap_or_else(isoformat_now);
    out.push_str("  exported_at \"");
    out.push_str(&escape_value(&exported_at));
    out.push_str("\"\n");
    push_optional_info(&mut out, "version", info.version.as_deref());
    push_optional_info(&mut out, "url", info.url.as_deref());
    push_optional_info(&mut out, "user_agent", info.user_agent.as_deref());
    push_optional_info(&mut out, "worker_state", info.worker_state.as_deref());
    push_optional_info(&mut out, "mode", info.mode.as_deref());
    if !seed_files.is_empty() {
        out.push_str("  seed_files\n");
        for (name, contents) in seed_files {
            out.push_str("    file \"");
            out.push_str(&escape_value(name));
            out.push_str("\"\n");
            for line in contents.lines() {
                if line.is_empty() {
                    continue;
                }
                out.push_str("      ");
                out.push_str(line);
                out.push('\n');
            }
        }
    }
    if !preferences.is_empty() {
        out.push_str("  preferences\n");
        for (key, value) in preferences {
            if key.is_empty() {
                continue;
            }
            out.push_str("    ");
            out.push_str(key);
            out.push_str(" \"");
            out.push_str(&escape_value(value));
            out.push_str("\"\n");
        }
    }
    out.push_str("  ");
    out.push_str(ROOT_HEADER);
    out.push('\n');
    for event in events {
        let mut block = String::new();
        format_event_into(event, &mut block);
        for line in block.lines() {
            if line.is_empty() {
                continue;
            }
            out.push_str("  ");
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

fn push_optional_info(out: &mut String, key: &str, value: Option<&str>) {
    let Some(value) = value else { return };
    if value.is_empty() {
        return;
    }
    out.push_str("  ");
    out.push_str(key);
    out.push_str(" \"");
    out.push_str(&escape_value(value));
    out.push_str("\"\n");
}

/// Parse a bundle into its structured pieces.
///
/// The function is forgiving: unknown subsections are ignored, and a legacy
/// `demo_memory` document returns a `ParsedBundle` whose `events` are
/// populated and every other field empty (matching
/// `FormalAiMemory.importFullMemory` in the browser).
#[must_use]
pub fn import_full_memory(text: &str) -> ParsedBundle {
    let trimmed = text.trim_start();
    if !trimmed.starts_with(BUNDLE_HEADER) {
        return ParsedBundle {
            events: parse_links_notation(text),
            ..ParsedBundle::default()
        };
    }
    parse_bundle_document(text)
}

#[allow(clippy::too_many_lines)]
fn parse_bundle_document(text: &str) -> ParsedBundle {
    let mut bundle = ParsedBundle::default();
    let mut section: Option<&'static str> = None;
    let mut current_seed_file: Option<String> = None;
    let mut current_seed_body = String::new();
    let mut memory_lines: Vec<String> = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            if section == Some("seed_files") && current_seed_file.is_some() {
                current_seed_body.push('\n');
            }
            continue;
        }
        let indent = line.chars().take_while(|c| *c == ' ').count();
        let content = &line[indent..];
        if indent == 0 {
            // Top-level header; reset any open seed file before continuing.
            if let Some(name) = current_seed_file.take() {
                bundle
                    .seed_files
                    .push((name, std::mem::take(&mut current_seed_body)));
            }
            section = None;
            continue;
        }
        if indent == 2 {
            if let Some(name) = current_seed_file.take() {
                bundle
                    .seed_files
                    .push((name, std::mem::take(&mut current_seed_body)));
            }
            if content == "seed_files" {
                section = Some("seed_files");
                continue;
            }
            if content == "preferences" {
                section = Some("preferences");
                continue;
            }
            if content == ROOT_HEADER {
                section = Some("memory");
                memory_lines.push(String::from(ROOT_HEADER));
                continue;
            }
            // Free-form info field, e.g. `version "0.22.0"`.
            if let Some((key, rest)) = split_first_token(content) {
                if let Some(value) = parse_quoted(rest) {
                    match key {
                        "exported_at" => bundle.info.exported_at = Some(value),
                        "version" => bundle.info.version = Some(value),
                        "url" => bundle.info.url = Some(value),
                        "user_agent" => bundle.info.user_agent = Some(value),
                        "worker_state" => bundle.info.worker_state = Some(value),
                        "mode" => bundle.info.mode = Some(value),
                        _ => {}
                    }
                }
            }
            section = None;
            continue;
        }
        match section {
            Some("seed_files") => {
                if indent == 4 {
                    if let Some(name) = current_seed_file.take() {
                        bundle
                            .seed_files
                            .push((name, std::mem::take(&mut current_seed_body)));
                    }
                    if let Some(rest) = content.strip_prefix("file ") {
                        if let Some(value) = parse_quoted(rest) {
                            current_seed_file = Some(value);
                            current_seed_body = String::new();
                        }
                    }
                } else if current_seed_file.is_some() && indent >= 6 {
                    let body = if line.len() >= 6 { &line[6..] } else { "" };
                    if !current_seed_body.is_empty() {
                        current_seed_body.push('\n');
                    }
                    current_seed_body.push_str(body);
                }
            }
            Some("preferences") if indent == 4 => {
                if let Some((key, rest)) = split_first_token(content) {
                    if let Some(value) = parse_quoted(rest) {
                        bundle.preferences.push((key.to_string(), value));
                    }
                }
            }
            Some("memory") => {
                // Strip the 2 spaces of bundle indentation so the captured
                // block matches the standalone `demo_memory` shape parsed by
                // `parse_links_notation`.
                let stripped = if line.len() >= 2 { &line[2..] } else { line };
                memory_lines.push(stripped.to_string());
            }
            _ => {}
        }
    }
    if let Some(name) = current_seed_file.take() {
        bundle.seed_files.push((name, current_seed_body));
    }
    if !memory_lines.is_empty() {
        bundle.events = parse_links_notation(&memory_lines.join("\n"));
    }
    bundle.agent_info = extract_agent_info(&bundle.seed_files);
    bundle
}

fn extract_agent_info(seed_files: &[(String, String)]) -> BTreeMap<String, String> {
    for (name, contents) in seed_files {
        if name == "data/seed/agent-info.lino" || name == "seed/agent-info.lino" {
            return parse_agent_info(contents);
        }
    }
    BTreeMap::new()
}

fn parse_agent_info(text: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let mut current_field: Option<String> = None;
    for line in text.lines() {
        let indent = line.chars().take_while(|c| *c == ' ').count();
        let content = &line[indent..];
        if indent == 2 {
            if let Some(rest) = content.strip_prefix("field ") {
                if let Some(value) = parse_quoted(rest) {
                    current_field = Some(value);
                }
            }
        } else if indent == 4 {
            if let Some(rest) = content.strip_prefix("value ") {
                if let Some(value) = parse_quoted(rest) {
                    if let Some(key) = current_field.take() {
                        out.insert(key, value);
                    }
                }
            }
        }
    }
    out
}

/// Suggest data migrations between an imported bundle and the running app.
///
/// The first migration check covers the seed version baked into
/// `data/seed/agent-info.lino` (`field "version"`). When the version moved
/// forward, the returned suggestion tells the user where to look. A legacy
/// `demo_memory`-only import also yields a suggestion explaining that the
/// seed-of-origin is unknown. Returns an empty vector when no migration is
/// needed so the caller can branch on emptiness.
#[must_use]
pub fn suggest_migrations(
    imported: &ParsedBundle,
    current_agent_info: &BTreeMap<String, String>,
) -> Vec<String> {
    let mut out = Vec::new();
    let imported_version = imported
        .agent_info
        .get("version")
        .cloned()
        .or_else(|| imported.info.version.clone());
    let current_version = current_agent_info.get("version").cloned();
    match (imported_version.as_deref(), current_version.as_deref()) {
        (Some(imported_v), Some(current_v)) if imported_v != current_v => {
            out.push(format!(
                "Seed version {imported_v} → {current_v}: review the new entries in data/seed/ \
                 (multilingual responses, concepts, tools) — your imported memory was \
                 authored against an older seed.",
            ));
        }
        (Some(imported_v), None) => {
            out.push(format!(
                "Imported bundle was authored against seed version {imported_v} but the \
                 running app does not expose a seed version. Update the app to compare.",
            ));
        }
        _ => {}
    }
    if imported.seed_files.is_empty() && !imported.events.is_empty() {
        out.push(String::from(
            "Imported file is a legacy demo_memory log (no seed). The events were \
             imported, but the seed at the time of capture is unknown — export from \
             this session to upgrade to a full bundle.",
        ));
    }
    out
}
