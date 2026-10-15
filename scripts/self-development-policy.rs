//! Reviewed prospective aspirations, separate from strict attribution validation.
use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresencePolicy {
    Report,
    Enforce,
}

pub fn parse(text: &str) -> Result<PresencePolicy, String> {
    let mut root = false;
    let mut version = false;
    let mut mode = None;
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty() && !line.starts_with("//")) {
        if line == "self-development-policy" {
            if root { return Err("duplicate policy root".to_owned()); }
            root = true;
            continue;
        }
        if !root { return Err("policy root required".to_owned()); }
        let fields: Vec<_> = line.split_whitespace().collect();
        match fields.as_slice() {
            ["schema-version", "\"1\""] if !version => version = true,
            ["presence-mode", value] if mode.is_none() => {
                mode = Some(match *value {
                    "\"report\"" => PresencePolicy::Report,
                    "\"enforce\"" => PresencePolicy::Enforce,
                    _ => return Err("unknown presence-mode".to_owned()),
                });
            }
            _ => return Err(format!("unknown or duplicate policy field: {line}")),
        }
    }
    if !root || !version { return Err("policy root and schema-version 1 required".to_owned()); }
    mode.ok_or_else(|| "presence-mode required".to_owned())
}

#[cfg_attr(test, allow(dead_code))]
pub fn read(repo: &Path) -> Result<PresencePolicy, String> {
    let path = repo.join("data/meta/self-development-policy.lino");
    let text = fs::read_to_string(&path).map_err(|error| format!("could not read {}: {error}", path.display()))?;
    parse(&text)
}

fn describe_unmet(reason: &str, authored_commits: u64, authored_lines: u64, floor_satisfied: bool, target_satisfied: Option<bool>) -> String {
    let target_status = target_satisfied.map_or_else(|| "notEvaluated".to_owned(), |value| value.to_string());
    format!(
            "status=ReportedUnmetAspiration; presencePolicy=report; floorSatisfied={floor_satisfied}; targetSatisfied={target_status}; \
             measuredAuthoredCommits={authored_commits}; measuredAuthoredLines={authored_lines}; \
             recipeTransportCredit=0; reason={reason}"
        )
}

impl PresencePolicy {
    /// Called only after strict measurement succeeded; this cannot consume validation errors.
    #[cfg_attr(test, allow(dead_code))]
    pub fn report_unmet(self, reason: &str, authored_commits: u64, authored_lines: u64, floor_satisfied: bool, target_satisfied: Option<bool>) -> Result<bool, String> {
        if self == Self::Enforce { return Ok(false); }
        let report = describe_unmet(reason, authored_commits, authored_lines, floor_satisfied, target_satisfied);
        println!("{report}");
        if let Some(path) = std::env::var_os("GITHUB_STEP_SUMMARY") {
            OpenOptions::new().create(true).append(true).open(path)
                .and_then(|mut file| writeln!(file, "### Self-development policy report\n\n{report}\n"))
                .map_err(|error| format!("could not write self-development summary: {error}"))?;
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checked_in_modes_are_explicit() {
        assert_eq!(parse("self-development-policy\n schema-version \"1\"\n presence-mode \"report\""), Ok(PresencePolicy::Report));
        assert_eq!(parse("self-development-policy\n schema-version \"1\"\n presence-mode \"enforce\""), Ok(PresencePolicy::Enforce));
    }
    #[test]
    fn missing_unknown_duplicate_and_malformed_policy_refuse() {
        for input in ["self_development_policy\n schema-version \"1\"\n presence-mode \"report\"", "self-development-policy\n schema_version \"1\"\n presence-mode \"report\"", "self-development-policy\n schema-version \"1\"\n presence_mode \"report\"", "", "self-development-policy", "schema-version \"1\"", "self-development-policy\n schema-version \"2\"\n presence-mode \"report\"", "self-development-policy\n schema-version \"1\"\n presence-mode \"skip\"", "self-development-policy\n schema-version \"1\"\n presence-mode \"report\"\n presence-mode \"enforce\"", "self-development-policy\n schema-version \"1\"\n presence-mode \"report\"\n branch \"special\""] {
            assert!(parse(input).is_err(), "must refuse {input:?}");
        }
    }
    #[test]
    fn reported_missing_evidence_never_becomes_a_satisfied_floor() {
        let text = describe_unmet("no evidence", 0, 0, false, None);
        for exact in ["ReportedUnmetAspiration", "floorSatisfied=false", "targetSatisfied=notEvaluated", "measuredAuthoredCommits=0", "measuredAuthoredLines=0", "recipeTransportCredit=0"] {
            assert!(text.contains(exact), "missing {exact}");
        }
    }
    #[test]
    fn numerical_target_and_presence_are_reported_independently() {
        let text = describe_unmet("target not met", 2, 47, true, Some(false));
        for exact in ["floorSatisfied=true", "targetSatisfied=false", "measuredAuthoredCommits=2", "measuredAuthoredLines=47"] {
            assert!(text.contains(exact), "missing {exact}");
        }
    }
    #[test]
    fn enforcing_policy_never_reports_away_missing_evidence() {
        assert_eq!(PresencePolicy::Enforce.report_unmet("no evidence", 0, 0, false, None), Ok(false));
    }
}
