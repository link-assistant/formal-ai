//! Content-addressed history of verified, rediscoverable coding procedures.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::coding::composition::VerifiedDraft;
use crate::coding::concept_discovery::ConceptMap;
use crate::coding::task_spec::CodingTaskSpec;
use crate::links_format::push_lino_node;
use crate::seed::parser::{LinoNode, parse_lino};
use crate::source_fetch::sha256_hex;

const LEDGER_FILE: &str = "discovered-procedures.lino";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedurePart {
    pub id: String,
    pub kind: String,
    pub license: String,
    pub source_url: String,
    pub sha256: String,
    pub fetched_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredProcedure {
    pub id: String,
    pub spec_identity: String,
    pub language: String,
    pub needs: Vec<String>,
    pub parts: Vec<ProcedurePart>,
    pub source_urls: Vec<String>,
    pub composition: String,
    pub tests_passed: usize,
    pub source_sha256: String,
    pub verified_at: String,
    pub integrity_sha256: String,
}

impl DiscoveredProcedure {
    fn from_verified(spec: &CodingTaskSpec, concepts: &ConceptMap, draft: &VerifiedDraft) -> Self {
        let mut parts = concepts
            .needs
            .iter()
            .flat_map(|need| &need.candidates)
            .map(|candidate| ProcedurePart {
                id: candidate.id.clone(),
                kind: candidate.kind.clone(),
                license: candidate.license.clone(),
                source_url: candidate.source_url.clone(),
                sha256: candidate.sha256.clone(),
                fetched_at: candidate.fetched_at.clone(),
            })
            .collect::<Vec<_>>();
        parts.sort_by(|left, right| left.id.cmp(&right.id));
        parts.dedup_by(|left, right| left.id == right.id);
        let mut procedure = Self {
            id: String::new(),
            spec_identity: spec_identity(spec),
            language: spec.language.clone(),
            needs: concepts
                .needs
                .iter()
                .map(|need| need.phrase().to_owned())
                .collect(),
            parts,
            source_urls: draft.source_urls.clone(),
            composition: draft.composition.clone(),
            tests_passed: draft.assertion_count,
            source_sha256: sha256_hex(draft.source.as_bytes()),
            verified_at: unix_now().to_string(),
            integrity_sha256: String::new(),
        };
        procedure.id =
            crate::engine::stable_id("discovered_procedure", &procedure.identity_payload());
        procedure.integrity_sha256 = procedure.expected_integrity();
        procedure
    }

    fn identity_payload(&self) -> String {
        let mut fields = vec![
            self.spec_identity.clone(),
            self.language.clone(),
            self.needs.join("\u{0}"),
            self.composition.clone(),
            self.tests_passed.to_string(),
            self.source_sha256.clone(),
            self.source_urls.join("\u{0}"),
        ];
        fields.extend(self.parts.iter().map(|part| {
            [
                part.id.as_str(),
                part.kind.as_str(),
                part.license.as_str(),
                part.source_url.as_str(),
                part.sha256.as_str(),
                part.fetched_at.as_str(),
            ]
            .join("\u{0}")
        }));
        fields.join("\u{1}")
    }

    fn expected_integrity(&self) -> String {
        sha256_hex(
            format!(
                "{}\u{0}{}\u{0}{}",
                self.id,
                self.identity_payload(),
                self.verified_at
            )
            .as_bytes(),
        )
    }

    fn valid(&self) -> bool {
        self.id == crate::engine::stable_id("discovered_procedure", &self.identity_payload())
            && self.integrity_sha256 == self.expected_integrity()
            && !self.source_sha256.is_empty()
            && self.parts.iter().all(|part| {
                !part.id.is_empty()
                    && !part.license.is_empty()
                    && !part.source_url.is_empty()
                    && !part.sha256.is_empty()
                    && !part.fetched_at.is_empty()
            })
    }
}

#[derive(Debug, Clone)]
pub struct DiscoveredProcedureLedger {
    path: PathBuf,
}

impl DiscoveredProcedureLedger {
    #[must_use]
    pub fn new(cache_directory: impl AsRef<Path>) -> Self {
        Self {
            path: cache_directory.as_ref().join(LEDGER_FILE),
        }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn recall(&self, spec: &CodingTaskSpec) -> io::Result<Option<DiscoveredProcedure>> {
        let identity = spec_identity(spec);
        Ok(self
            .read_valid()?
            .into_iter()
            .rev()
            .find(|procedure| procedure.spec_identity == identity))
    }

    pub fn remember(
        &self,
        spec: &CodingTaskSpec,
        concepts: &ConceptMap,
        draft: &VerifiedDraft,
    ) -> io::Result<DiscoveredProcedure> {
        let procedure = DiscoveredProcedure::from_verified(spec, concepts, draft);
        let mut records = self.read_valid()?;
        records.retain(|record| record.id != procedure.id);
        records.push(procedure.clone());
        records.sort_by(|left, right| left.id.cmp(&right.id));
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.path, render_ledger(&records))?;
        Ok(procedure)
    }

    fn read_valid(&self) -> io::Result<Vec<DiscoveredProcedure>> {
        let text = match fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error),
        };
        let root = parse_lino(&text);
        Ok(root
            .children
            .iter()
            .find(|node| node.name == "discovered_procedures")
            .into_iter()
            .flat_map(|container| &container.children)
            .filter(|node| node.name == "discovered_procedure")
            .filter_map(parse_procedure)
            .filter(DiscoveredProcedure::valid)
            .collect())
    }
}

#[must_use]
pub fn spec_identity(spec: &CodingTaskSpec) -> String {
    let requirements = spec
        .requirement_sentences
        .iter()
        .map(|sentence| crate::engine::normalize_prompt(sentence))
        .collect::<Vec<_>>()
        .join("\u{0}");
    crate::engine::stable_id(
        "coding_spec",
        &format!(
            "{}\u{0}{}\u{0}{requirements}",
            spec.language,
            spec.signature_identity()
        ),
    )
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn parse_procedure(node: &LinoNode) -> Option<DiscoveredProcedure> {
    Some(DiscoveredProcedure {
        id: node.id.clone(),
        spec_identity: required(node, "spec_identity")?.to_owned(),
        language: required(node, "language")?.to_owned(),
        needs: values(node, "need"),
        parts: node
            .children
            .iter()
            .filter(|child| child.name == "part")
            .map(parse_part)
            .collect::<Option<Vec<_>>>()?,
        source_urls: values(node, "source_url"),
        composition: required(node, "composition")?.to_owned(),
        tests_passed: required(node, "tests_passed")?.parse().ok()?,
        source_sha256: required(node, "source_sha256")?.to_owned(),
        verified_at: required(node, "verified_at")?.to_owned(),
        integrity_sha256: required(node, "integrity_sha256")?.to_owned(),
    })
}

fn parse_part(node: &LinoNode) -> Option<ProcedurePart> {
    Some(ProcedurePart {
        id: node.id.clone(),
        kind: required(node, "kind")?.to_owned(),
        license: required(node, "license")?.to_owned(),
        source_url: required(node, "source_url")?.to_owned(),
        sha256: required(node, "sha256")?.to_owned(),
        fetched_at: required(node, "fetched_at")?.to_owned(),
    })
}

fn required<'a>(node: &'a LinoNode, name: &str) -> Option<&'a str> {
    node.children
        .iter()
        .find(|child| child.name == name)
        .map(|child| child.id.as_str())
        .filter(|value| !value.is_empty())
}

fn values(node: &LinoNode, name: &str) -> Vec<String> {
    node.children
        .iter()
        .filter(|child| child.name == name)
        .map(|child| child.id.clone())
        .collect()
}

fn render_ledger(records: &[DiscoveredProcedure]) -> String {
    let mut out = String::new();
    push_lino_node(&mut out, 0, "discovered_procedures", None);
    for record in records {
        push_lino_node(&mut out, 2, "discovered_procedure", Some(&record.id));
        push_lino_node(&mut out, 4, "spec_identity", Some(&record.spec_identity));
        push_lino_node(&mut out, 4, "language", Some(&record.language));
        for need in &record.needs {
            push_lino_node(&mut out, 4, "need", Some(need));
        }
        for part in &record.parts {
            push_lino_node(&mut out, 4, "part", Some(&part.id));
            push_lino_node(&mut out, 6, "kind", Some(&part.kind));
            push_lino_node(&mut out, 6, "license", Some(&part.license));
            push_lino_node(&mut out, 6, "source_url", Some(&part.source_url));
            push_lino_node(&mut out, 6, "sha256", Some(&part.sha256));
            push_lino_node(&mut out, 6, "fetched_at", Some(&part.fetched_at));
        }
        for source_url in &record.source_urls {
            push_lino_node(&mut out, 4, "source_url", Some(source_url));
        }
        push_lino_node(&mut out, 4, "composition", Some(&record.composition));
        push_lino_node(
            &mut out,
            4,
            "tests_passed",
            Some(&record.tests_passed.to_string()),
        );
        push_lino_node(&mut out, 4, "source_sha256", Some(&record.source_sha256));
        push_lino_node(&mut out, 4, "verified_at", Some(&record.verified_at));
        push_lino_node(
            &mut out,
            4,
            "integrity_sha256",
            Some(&record.integrity_sha256),
        );
    }
    out
}
