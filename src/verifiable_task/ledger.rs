//! Content-addressed memory for a *shape* of verifiable task (#1138 B8).
//!
//! The ledger stores the **derivation**, never the value: a recall replays the
//! program IR and re-runs it against this prompt's quantities, so a renumbered
//! paraphrase recalls the procedure and computes a different, correct number.
//! That is the property memorization cannot fake.
//!
//! Wave T lands the shapes only; wave I8 leaves 08-L13 and 08-L14 fill the
//! bodies in.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::VerifiableTask;
use crate::links_format::push_lino_node;
use crate::seed::parser::{LinoNode, parse_lino};
use crate::source_fetch::sha256_hex;

const LEDGER_FILE: &str = "verifiable-task-procedures.lino";

/// A verified derivation for a *shape* of task, not for a case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedProcedure {
    /// The record's own stable id.
    pub id: String,
    /// The task identity this derivation answers.
    pub task_identity: String,
    /// The expectation slug it was derived for.
    pub expectation: String,
    /// The IR content id, so the same derivation is recognizable across runs.
    pub derivation_id: String,
    /// The fragments the derivation composed.
    pub fragments: Vec<String>,
    /// Where those fragments came from.
    pub source_urls: Vec<String>,
    /// Which self-checks passed, by slug.
    pub checks: Vec<String>,
    /// When the derivation was verified.
    pub verified_at: String,
    /// Tamper detection over the record's identity payload.
    pub integrity_sha256: String,
}

impl VerifiedProcedure {
    /// The payload the integrity digest is taken over.
    #[must_use]
    pub fn identity_payload(&self) -> String {
        [
            self.task_identity.clone(),
            self.expectation.clone(),
            self.derivation_id.clone(),
            self.fragments.join("\u{0}"),
            self.source_urls.join("\u{0}"),
            self.checks.join("\u{0}"),
        ]
        .join("\u{1}")
    }

    /// The digest this record should carry.
    #[must_use]
    pub fn expected_integrity(&self) -> String {
        sha256_hex(
            [
                self.id.as_str(),
                self.identity_payload().as_str(),
                self.verified_at.as_str(),
            ]
            .join("\u{0}")
            .as_bytes(),
        )
    }

    /// Whether the record's digest matches its payload.
    #[must_use]
    pub fn valid(&self) -> bool {
        !self.task_identity.is_empty()
            && !self.expectation.is_empty()
            && !self.derivation_id.is_empty()
            && self.id == crate::engine::stable_id("verified_procedure", &self.identity_payload())
            && self.integrity_sha256 == self.expected_integrity()
    }
}

/// The on-disk ledger of verified derivations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiableTaskLedger {
    path: PathBuf,
}

impl VerifiableTaskLedger {
    /// Open the ledger in `cache_directory`.
    #[must_use]
    pub fn new(cache_directory: impl AsRef<Path>) -> Self {
        Self {
            path: cache_directory.as_ref().join(LEDGER_FILE),
        }
    }

    /// Where this ledger stores its records.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The derivation remembered for `task`'s identity, when one is retained.
    ///
    /// # Errors
    /// Propagates the read failure.
    pub fn recall(&self, task: &VerifiableTask) -> std::io::Result<Option<VerifiedProcedure>> {
        let identity = task.identity();
        Ok(self
            .read_valid()?
            .into_iter()
            .find(|procedure| procedure.task_identity == identity))
    }

    /// Remember the derivation that produced `answer` for `task`'s shape.
    ///
    /// # Errors
    /// Propagates the write failure.
    pub fn remember(
        &self,
        task: &VerifiableTask,
        derivation_id: &str,
        fragments: &[String],
    ) -> std::io::Result<VerifiedProcedure> {
        let mut procedure = VerifiedProcedure {
            id: String::new(),
            task_identity: task.identity(),
            expectation: task.expectation.slug().to_owned(),
            derivation_id: derivation_id.to_owned(),
            fragments: fragments.to_vec(),
            source_urls: Vec::new(),
            checks: Vec::new(),
            verified_at: unix_now().to_string(),
            integrity_sha256: String::new(),
        };
        procedure.id =
            crate::engine::stable_id("verified_procedure", &procedure.identity_payload());
        procedure.integrity_sha256 = procedure.expected_integrity();

        let mut records = self.read_valid()?;
        records.retain(|record| record.task_identity != procedure.task_identity);
        records.push(procedure.clone());
        records.sort_by(|left, right| left.id.cmp(&right.id));
        self.write(&records)?;
        Ok(procedure)
    }

    /// Delete the record for `task`'s identity, retaining nothing but the
    /// ability to rediscover it.
    ///
    /// # Errors
    /// Propagates the write failure.
    pub fn forget(&self, task: &VerifiableTask) -> std::io::Result<()> {
        let identity = task.identity();
        let mut records = self.read_valid()?;
        records.retain(|record| record.task_identity != identity);
        self.write(&records)
    }

    fn read_valid(&self) -> std::io::Result<Vec<VerifiedProcedure>> {
        let text = match std::fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error),
        };
        let root = parse_lino(&text);
        Ok(root
            .children
            .iter()
            .find(|node| node.name == "verifiable_task_procedures")
            .into_iter()
            .flat_map(|container| &container.children)
            .filter(|node| node.name == "verified_procedure")
            .filter_map(parse_procedure)
            .filter(VerifiedProcedure::valid)
            .collect())
    }

    fn write(&self, records: &[VerifiedProcedure]) -> std::io::Result<()> {
        crate::memory::write_locked_atomic(&self.path, &render_ledger(records))
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
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

fn parse_procedure(node: &LinoNode) -> Option<VerifiedProcedure> {
    Some(VerifiedProcedure {
        id: node.id.clone(),
        task_identity: required(node, "task_identity")?.to_owned(),
        expectation: required(node, "expectation")?.to_owned(),
        derivation_id: required(node, "derivation_id")?.to_owned(),
        fragments: values(node, "fragment"),
        source_urls: values(node, "source_url"),
        checks: values(node, "check"),
        verified_at: required(node, "verified_at")?.to_owned(),
        integrity_sha256: required(node, "integrity_sha256")?.to_owned(),
    })
}

fn render_ledger(records: &[VerifiedProcedure]) -> String {
    let mut out = String::new();
    push_lino_node(&mut out, 0, "verifiable_task_procedures", None);
    for record in records {
        push_lino_node(&mut out, 2, "verified_procedure", Some(&record.id));
        push_lino_node(&mut out, 4, "task_identity", Some(&record.task_identity));
        push_lino_node(&mut out, 4, "expectation", Some(&record.expectation));
        push_lino_node(&mut out, 4, "derivation_id", Some(&record.derivation_id));
        for fragment in &record.fragments {
            push_lino_node(&mut out, 4, "fragment", Some(fragment));
        }
        for source_url in &record.source_urls {
            push_lino_node(&mut out, 4, "source_url", Some(source_url));
        }
        for check in &record.checks {
            push_lino_node(&mut out, 4, "check", Some(check));
        }
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
