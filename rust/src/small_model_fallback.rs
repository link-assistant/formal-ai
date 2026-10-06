//! Small-model formalization fallback, behind an experimental flag
//! (issue #483).
//!
//! Formalization matching — a surface like "метр" against grounded
//! Wikipedia/Wikidata candidates — is rule-first today. The issue asks
//! whether the smallest useful models (the
//! [link-assistant/model-in-browser](https://github.com/link-assistant/model-in-browser)
//! experiment runs SmolLM2-135M-Instruct fully client-side, ~270 MB of
//! weights via Candle-in-WASM) can *help* by picking the best match
//! from offered options.
//!
//! Four constraints shape this module, and each is enforced in code,
//! not prose:
//!
//! 1. **Off by default, and then some.** `SmallModelOptions::default()`
//!    has `enabled: false`; every entry point returns `None` for it.
//!    Nothing loads, nothing downloads, nothing is even parsed for it.
//! 2. **On demand only.** `download_manifest` *returns* a plan (URL,
//!    size); the module performs no I/O and no fetch. Every catalog row
//!    carries `packaged false`, asserted in tests: nothing ships inside
//!    the application package or the web UI.
//! 3. **Hardware-fit gating.** `eligible()` lists only models whose
//!    `ram_required_mb` fits the machine's stated RAM, so users are not
//!    offered options they cannot run; the list is sorted by public
//!    rating, best first (ratings are approximate public standings from
//!    the seed's `rating_source` rows — sorting material, never
//!    correctness claims).
//! 4. **Formal first; LLMs never at the steering wheel.** The model's
//!    choice is a [`ModelProposal`] — advisory, `needs_review`, never a
//!    committed match. `confirm_proposal` accepts a proposal only when
//!    the formal rule layer independently agrees; a proposal the rules
//!    reject is discarded, and the rules never consult the model.
//!
//! The selection scorer here is a deterministic lexical stand-in
//! (surface/label overlap): it keeps the contract testable offline
//! while no inference runtime is wired. When a downloaded model runs,
//! its choice replaces the stand-in behind the same signature — the
//! guards, gating, and advisory-only shape stay.
//!
//! Catalog data lives in `data/seed/small-model-catalog.lino`
//! (mirrored at rust/embedded/data/seed/). Settings surface (`.lenv`,
//! `formal-ai with`) is the maintainer's lift; this module exposes
//! `SmallModelOptions` for it.

use crate::event_log::EventLog;
use crate::seed::parser::parse_lino;

const SEED_PATH: &str = "data/seed/small-model-catalog.lino";

/// User-controlled options; the experimental flag defaults to OFF.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SmallModelOptions {
    /// The explicit user opt-in from settings. Default `false`.
    pub enabled: bool,
    /// The machine's available RAM in MB (the hardware-fit gate).
    pub available_ram_mb: u32,
    /// A rating floor: models below this public standing are not shown
    /// (keeps tiny-but-useless options out of the user's way).
    pub min_rating: u8,
}

impl Default for SmallModelOptions {
    fn default() -> Self {
        // Off, and nothing fits: the default state performs no work and
        // reports no models, even if a caller forgets to check
        // `enabled`.
        Self {
            enabled: false,
            available_ram_mb: 0,
            min_rating: 0,
        }
    }
}

impl SmallModelOptions {
    /// The options a user's explicit enable produces for a machine with
    /// `available_ram_mb` of RAM. Enabling alone changes nothing until
    /// the RAM figure is stated — there is no "run anything" mode.
    pub fn enabled_for(available_ram_mb: u32) -> Self {
        Self {
            enabled: true,
            available_ram_mb,
            min_rating: 40,
        }
    }
}

/// One `small_model` catalog row.
#[derive(Clone, Debug)]
pub struct CatalogModel {
    pub model_id: String,
    pub kind: String,
    pub family: String,
    pub params_m: u32,
    pub ram_required_mb: u32,
    pub disk_required_mb: u32,
    pub rating: u8,
    pub rating_source: String,
    pub task_affinity: String,
    pub license: String,
    /// Always false in the seed: nothing ships in the package or web UI.
    pub packaged: bool,
    pub note: String,
}

/// Look up embedded seed content by its registered path.
fn seed_text(path: &str) -> Option<&'static str> {
    crate::seed::seed_files()
        .into_iter()
        .find(|(registered, _)| *registered == path)
        .map(|(_, text)| text)
}

/// The whole catalog, in seed order.
pub fn catalog() -> Vec<CatalogModel> {
    let mut out = Vec::new();
    let Some(text) = seed_text(SEED_PATH) else {
        return out;
    };
    let document = parse_lino(text);
    let Some(root) = document
        .children
        .iter()
        .find(|child| child.name == "small_model_catalog")
    else {
        return out;
    };
    for record in root.children.iter() {
        if record.name != "small_model" {
            continue;
        }
        let model_id = record.find_child_value("model_id").trim().to_owned();
        if model_id.is_empty() {
            continue;
        }
        let packaged = record.find_child_value("packaged").trim() == "true";
        out.push(CatalogModel {
            model_id,
            kind: record.find_child_value("kind").to_string(),
            family: record.find_child_value("family").to_string(),
            params_m: record
                .find_child_value("params_m")
                .trim()
                .parse()
                .unwrap_or(0),
            ram_required_mb: record
                .find_child_value("ram_required_mb")
                .trim()
                .parse()
                .unwrap_or(0),
            disk_required_mb: record
                .find_child_value("disk_required_mb")
                .trim()
                .parse()
                .unwrap_or(0),
            rating: record
                .find_child_value("rating")
                .trim()
                .parse()
                .unwrap_or(0),
            rating_source: record.find_child_value("rating_source").to_string(),
            task_affinity: record.find_child_value("task_affinity").to_string(),
            license: record.find_child_value("license").to_string(),
            packaged,
            note: record.find_child_value("note").to_string(),
        });
    }
    out
}

/// The models worth showing: only hardware-fitting rows at or above the
/// rating floor, sorted by public rating (best first; smaller params
/// break ties — among equal standings, the smaller download wins, which
/// is the on-demand-friendly order). Owned clones of the catalog rows:
/// the catalog is tiny and callers keep a plain value.
pub fn eligible(options: &SmallModelOptions) -> Vec<CatalogModel> {
    if !options.enabled {
        return Vec::new();
    }
    let mut fitting: Vec<CatalogModel> = catalog()
        .into_iter()
        .filter(|model| {
            model.ram_required_mb <= options.available_ram_mb && model.rating >= options.min_rating
        })
        .collect();
    fitting.sort_by(|a, b| {
        b.rating
            .cmp(&a.rating)
            .then_with(|| a.params_m.cmp(&b.params_m))
            .then_with(|| a.model_id.cmp(&b.model_id))
    });
    fitting
}

/// The recommended model: the best-rated hardware-fitting row. `None`
/// when the feature is off or nothing fits.
pub fn recommend(options: &SmallModelOptions) -> Option<CatalogModel> {
    eligible(options).into_iter().next()
}

/// An on-demand download plan. The module never performs it.
#[derive(Clone, Debug)]
pub struct DownloadPlan {
    pub model_id: String,
    pub disk_required_mb: u32,
    /// The source the runtime downloads from at explicit user action.
    pub source: String,
}

/// The download plan for a model, for the runtime that performs it at
/// the user's explicit request. This function builds a record; it does
/// no I/O.
pub fn download_manifest(model: &CatalogModel) -> DownloadPlan {
    DownloadPlan {
        model_id: model.model_id.clone(),
        disk_required_mb: model.disk_required_mb,
        source: format!("https://huggingface.co/models?search={}", model.family),
    }
}

/// One option the model chooses among: a grounded formalization
/// candidate (a Wikidata entity surfaced by the rules).
#[derive(Clone, Debug)]
pub struct FormalizationCandidate {
    pub qid: String,
    pub label: String,
    pub description: String,
}

/// The model's choice: advisory only, never a committed match.
#[derive(Clone, Debug)]
pub struct ModelProposal {
    pub model_id: String,
    /// Index into the offered options.
    pub candidate_index: usize,
    /// 0.0-1.0 self-assessed confidence.
    pub confidence: f32,
    /// Always true: a proposal awaits confirmation.
    pub needs_review: bool,
}

impl ModelProposal {
    /// The steering-wheel rule: proposals never commit themselves.
    pub fn is_advisory(&self) -> bool {
        self.needs_review
    }
}

/// Deterministic stand-in scorer: how well a target surface matches one
/// candidate (exact label, label-contains, token overlap). Replaced by
/// the downloaded model's choice under the same signature later.
fn lexical_affinity(target: &str, candidate: &FormalizationCandidate) -> f32 {
    let target = target.trim().to_lowercase();
    let label = candidate.label.trim().to_lowercase();
    if target.is_empty() || label.is_empty() {
        return 0.0;
    }
    if target == label {
        return 1.0;
    }
    if label.contains(&target) || target.contains(&label) {
        return 0.7;
    }
    let target_tokens: Vec<&str> = target.split_whitespace().collect();
    let overlap = target_tokens
        .iter()
        .filter(|token| {
            label.contains(*token) || candidate.description.to_lowercase().contains(*token)
        })
        .count();
    overlap as f32 / (target_tokens.len().max(1) as f32) * 0.5
}

/// Ask the (enabled, fitting) model to pick the best match among the
/// offered options. Returns `None` unless the feature is enabled, a
/// model fits the hardware, and some candidate scores above zero — the
/// fallback stays silent rather than guessing when it cannot help.
///
/// The result is advisory: log it, show it, but only
/// `confirm_proposal` decides, with the formal rules holding the pen.
pub fn propose_best_match(
    options: &SmallModelOptions,
    target: &str,
    candidates: &[FormalizationCandidate],
    log: &mut EventLog,
) -> Option<ModelProposal> {
    let model = recommend(options)?;
    if candidates.is_empty() {
        return None;
    }
    let mut best = (0usize, 0.0f32);
    for (index, candidate) in candidates.iter().enumerate() {
        let affinity = lexical_affinity(target, candidate);
        if affinity > best.1 {
            best = (index, affinity);
        }
    }
    if best.1 <= 0.0 {
        return None;
    }
    log.append("small_model:proposal", model.model_id.clone());
    log.append("small_model:candidate", candidates[best.0].qid.clone());
    Some(ModelProposal {
        model_id: model.model_id.clone(),
        candidate_index: best.0,
        confidence: best.1,
        needs_review: true,
    })
}

/// The formal-first gate: a proposal becomes a match only when the
/// formal rule layer independently agrees (`rule_agrees`), whatever the
/// model said. The model never confirms itself.
pub fn confirm_proposal(proposal: &ModelProposal, rule_agrees: bool) -> bool {
    proposal.needs_review && rule_agrees
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_options_show_and_load_nothing() {
        assert!(eligible(&SmallModelOptions::default()).is_empty());
        assert!(recommend(&SmallModelOptions::default()).is_none());
    }

    #[test]
    fn nothing_in_the_catalog_is_packaged() {
        for model in catalog() {
            assert!(
                !model.packaged,
                "{} must not ship in the package",
                model.model_id
            );
        }
    }

    #[test]
    fn hardware_fit_hides_oversized_models_and_sorts_by_rating() {
        let small_machine = SmallModelOptions::enabled_for(1024);
        let shown = eligible(&small_machine);
        assert!(shown.iter().all(|model| model.ram_required_mb <= 1024));
        assert!(
            !shown.iter().any(|model| model.model_id == "phi_3_5_mini"),
            "a 6 GB model must not be offered on a 1 GB machine"
        );
        let ratings: Vec<u8> = shown.iter().map(|model| model.rating).collect();
        let mut sorted = ratings.clone();
        sorted.sort_unstable_by(|a, b| b.cmp(a));
        assert_eq!(ratings, sorted, "sorted by public rating, best first");
    }

    #[test]
    fn the_proposal_is_advisory_and_the_rules_hold_the_pen() {
        let options = SmallModelOptions::enabled_for(1024);
        let candidates = vec![
            FormalizationCandidate {
                qid: "Q218593".into(),
                label: "inch".into(),
                description: "unit of length".into(),
            },
            FormalizationCandidate {
                qid: "Q11573".into(),
                label: "metre".into(),
                description: "SI unit of length".into(),
            },
        ];
        let mut log = EventLog::new();
        let proposal = propose_best_match(&options, "метр-класс: metre", &candidates, &mut log)
            .expect("a fitting model proposes");
        assert!(proposal.is_advisory());
        assert!(
            !confirm_proposal(&proposal, false),
            "rules disagree: discarded"
        );
        assert!(confirm_proposal(&proposal, true), "rules agree: confirmed");
        assert!(log.first_of("small_model:proposal").is_some());
    }

    #[test]
    fn a_disabled_feature_proposes_nothing() {
        let candidates = vec![FormalizationCandidate {
            qid: "Q11573".into(),
            label: "metre".into(),
            description: "SI unit of length".into(),
        }];
        let mut log = EventLog::new();
        assert!(
            propose_best_match(
                &SmallModelOptions::default(),
                "metre",
                &candidates,
                &mut log
            )
            .is_none()
        );
    }

    #[test]
    fn zero_affinity_stays_silent_instead_of_guessing() {
        let options = SmallModelOptions::enabled_for(1024);
        let candidates = vec![FormalizationCandidate {
            qid: "Q42".into(),
            label: "Douglas Adams".into(),
            description: "English writer".into(),
        }];
        let mut log = EventLog::new();
        assert!(
            propose_best_match(&options, "zzzz qqqq", &candidates, &mut log).is_none(),
            "no overlap means no proposal, not a random one"
        );
    }
}
