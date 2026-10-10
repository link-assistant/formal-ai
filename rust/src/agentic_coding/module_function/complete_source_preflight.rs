//! Complete seeded request syntax; semantic goals and host authority remain separate.
use crate::agentic_coding::shell_command_policy::sentences;
use crate::seed::parser::parse_lino;
use crate::source_fetch::sha256_hex;
use serde_json::{Value, json};

pub fn source_goal_scope(source: &str) -> Option<String> {
    if crate::normal_markov::quote_fault(source).is_some() {
        return None;
    }
    let mut result = String::new();
    let mut cursor = 0;
    for span in crate::normal_markov::quoted_segment_spans(source) {
        result.push_str(&source[cursor..span.start]);
        result.push_str(&" ".repeat(span.end - span.start));
        cursor = span.end;
    }
    result.push_str(&source[cursor..]);
    Some(result)
}

pub fn derive_complete_source_request(text: &str, seed: &str) -> Option<Value> {
    let scope = source_goal_scope(text)?;
    let request = super::discovery::observed_callable_request(text)?;
    let parsed = parse_lino(seed);
    let root = parsed
        .children
        .iter()
        .find(|node| node.name == "source-request-preflight")?;
    let kinds = [
        "declaration",
        "goal",
        "source-discovery",
        "reuse-imports",
        "read-prerequisites",
        "repair-plan",
        "verification",
        "protect-inputs",
        "gap-before-write",
    ];
    let forms: Vec<_> = root
        .children
        .iter()
        .filter(|node| node.name == "form")
        .collect();
    if forms.len() != kinds.len()
        || !kinds
            .iter()
            .all(|kind| forms.iter().filter(|form| form.id == *kind).count() == 1)
    {
        return None;
    }
    let mut clauses = Vec::new();
    for sentence in sentences(&scope) {
        let mut matches = Vec::new();
        for form in &forms {
            let pattern = regex::RegexBuilder::new(form.find_child_value("pattern"))
                .case_insensitive(true)
                .build()
                .ok()?;
            let Some(captures) = pattern.captures(sentence.text) else {
                continue;
            };
            if captures.get(0)?.as_str() != sentence.text {
                continue;
            }
            let mut values = serde_json::Map::new();
            for name in pattern.capture_names().flatten() {
                if let Some(value) = captures.name(name) {
                    values.insert(name.to_owned(), json!(value.as_str()));
                }
            }
            matches.push((form.id.as_str(), values));
        }
        if matches.len() != 1 {
            return None;
        }
        let (kind, captures) = matches.pop()?;
        if kind == "declaration" {
            let signature = captures.get("signature")?.as_str()?;
            let observed = super::signature(signature)?;
            if observed.at != 0
                || !signature.ends_with(')')
                || observed.name != request.name
                || observed.parameters != request.parameters
                || captures.get("destination")?.as_str()? != request.destination
            {
                return None;
            }
        }
        if kind == "source-discovery"
            && captures.get("sources")?.as_str()?
                != request.inputs.join(root.find_child_value("source-joiner"))
        {
            return None;
        }
        if kind == "verification"
            && (request.acceptance.len() != 1
                || captures.get("acceptance")?.as_str()? != request.acceptance[0]
                || Some(captures.get("command")?.as_str()?) != request.command.as_deref())
        {
            return None;
        }
        clauses.push(json!({"kind":kind,"captures":captures,"sentence":sentence.text,
            "evidence":super::source_contract::source_evidence(text,sentence.span.start,sentence.span.end),
            "status":"requires-contract"}));
    }
    if !kinds.iter().all(|kind| {
        clauses
            .iter()
            .filter(|clause| clause["kind"] == *kind)
            .count()
            == 1
    }) {
        return None;
    }
    Some(
        json!({"request":{"name":request.name,"parameters":request.parameters,
        "destination":request.destination,"inputs":request.inputs,"acceptance":request.acceptance,
        "command":request.command},"requestIdentity":sha256_hex(text.as_bytes()),
        "seedIdentity":sha256_hex(seed.as_bytes()),"clauses":clauses,
        "syntaxCoverage":"complete-finite-seeded-forms","semanticAuthority":false,"writeAuthority":false}),
    )
}

/// Conditional discharge only; private host receipt validation still precedes every effect.
pub fn bind_complete_source_needs(
    frame: &Value,
    composition: &Value,
    preflight: &Value,
    canonical_sources: &[Value],
) -> Result<Value, &'static str> {
    if frame["requestIdentity"].as_str().is_none()
        || frame["requestIdentity"] != composition["requestIdentity"]
        || preflight["ledger"]["sourceIdentity"] != frame["requestIdentity"]
    {
        return Err("UnboundRequestContracts");
    }
    let prerequisites = preflight["prerequisites"]
        .as_array()
        .ok_or("MissingReadPrerequisites")?;
    if !prerequisites.iter().all(|value| {
        matches!(
            value["status"].as_str(),
            Some("conditional-satisfied-read" | "conditional-satisfied-absence")
        )
    }) {
        return Err("UnsolvedImmutableReadPrerequisite");
    }
    let clauses = frame["clauses"].as_array().ok_or("MissingRequestClauses")?;
    let goal = clauses
        .iter()
        .find(|value| value["kind"] == "goal")
        .ok_or("MissingSourceGoal")?;
    if goal["sentence"] != composition["returnForm"]["sentence"]
        || composition["goalClauseCoverage"] != "complete-seeded-return-form"
    {
        return Err("UnboundSourceGoal");
    }
    let imports = composition["imports"]
        .as_array()
        .ok_or("MissingSourceImports")?;
    if !imports.iter().all(|binding| {
        canonical_sources.iter().any(|source| {
            source["moduleURL"].as_str().is_some()
                && source["moduleURL"] == binding["moduleURL"]
                && source["sha256"].as_str().is_some()
                && source["sha256"] == binding["sourceIdentity"]
        })
    }) {
        return Err("UnboundCanonicalSourceReceipt");
    }
    let effects = &composition["effectPlan"];
    if effects["kind"] != "checked-source-import-composition"
        || effects["localMutation"] != false
        || effects["additionalModuleInitialization"] != false
    {
        return Err("UnboundImportedImplementationOwnership");
    }
    Ok(
        json!({"requestIdentity":frame["requestIdentity"],"clauses":clauses,
        "readPrerequisites":prerequisites,"boundedRepairPlan":{"graph":composition["graph"],
        "imports":imports,"effects":effects},"pending":[{"kind":"verification",
        "command":frame["request"]["command"]},{"kind":"protected-input-byte-recheck"}],
        "allowedDestination":frame["request"]["destination"],"allowedCommand":frame["request"]["command"],
        "candidateWriteAuthority":"conditional-reversible-draft-only","unconditionalDeliveryAuthority":false,
        "sourceEffects":"conditional","moduleInitialization":"inherited same provider-approved Node operation; not purity",
        "loadedByteRaceProof":false,"autonomousCredit":0}),
    )
}
