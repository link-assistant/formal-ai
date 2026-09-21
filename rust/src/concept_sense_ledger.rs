//! Content-addressed, forgettable ledger of resolved senses (issue #1138,
//! plan 01 L12).
//!
//! The ledger keeps the *recipe*, not the payload: a forgotten sense is
//! rediscovered from the same committed captures and must reproduce the same
//! [`ConceptSense::content_id`]. A record whose stored digest no longer matches
//! its bytes is rejected and re-derived rather than trusted.
//!
//! It lives under `FORMAL_AI_CACHE_DIR`, never in `data/`. Two reasons, and the
//! second is the one that decides it: a remembered sense is a cache of somebody
//! else's copyrighted gloss, and `docs/case-studies/issue-710/plans/06:232-239`
//! requires the locator and the identity to be what is retained, so a ledger
//! that is deleted loses nothing that the captures cannot re-derive.
//!
//! **Recorded deviation from plan 01's §"Cache layout and content addressing".**
//! The plan says the ledger stores "the locator and identity, never the
//! disposable payload", i.e. no gloss. `tests/unit/concept_sense_ledger.rs`,
//! written in wave T, tampers with a record by replacing the gloss text inside
//! the stored file and requires the tampered record to be refused — which only
//! has meaning if the gloss is in the file. The test is the contract, so the
//! gloss is stored and the directory stays ignored and re-derivable.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::concept_lookup::ConceptSense;
use crate::links_format::push_lino_node;
use crate::seed::parser::{LinoNode, parse_lino};
use crate::source_fetch::sha256_hex;

/// The file the ledger is written as, inside the cache directory.
const LEDGER_FILE: &str = "concept-senses.lino";

/// Root node of the ledger projection.
const ROOT: &str = "concept_senses";

/// Content-addressed store of resolved senses under `FORMAL_AI_CACHE_DIR`.
#[derive(Debug, Clone)]
pub struct ConceptSenseLedger {
    path: PathBuf,
}

impl ConceptSenseLedger {
    /// The ledger inside `cache_directory`, created on the first write.
    #[must_use]
    pub fn new(cache_directory: impl AsRef<Path>) -> Self {
        Self {
            path: cache_directory.as_ref().join(LEDGER_FILE),
        }
    }

    /// The file the ledger is stored in.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Record one resolved sense under its content id.
    ///
    /// Remembering the same sense twice replaces the record rather than
    /// appending a second one: the content id *is* the identity.
    ///
    /// # Errors
    ///
    /// Returns the underlying I/O error when the ledger cannot be written.
    pub fn remember(&self, sense: &ConceptSense) -> io::Result<()> {
        let mut records = self.records()?;
        let content_id = sense.content_id();
        let record = Record {
            content_id: content_id.clone(),
            sense: sense.clone(),
        };
        match records
            .iter()
            .position(|stored| stored.content_id == content_id)
        {
            Some(index) => records[index] = record,
            None => records.push(record),
        }
        self.write(&records)
    }

    /// Read one sense back. A record whose bytes no longer hash to its declared
    /// digest is rejected, so a tampered ledger yields `Ok(None)`.
    ///
    /// # Errors
    ///
    /// Returns the underlying I/O error when the ledger cannot be read.
    pub fn recall(&self, content_id: &str) -> io::Result<Option<ConceptSense>> {
        Ok(self
            .records()?
            .into_iter()
            .find(|record| record.content_id == content_id)
            .map(|record| record.sense))
    }

    /// Delete one sense by content id, so the rediscovery path can be proven.
    ///
    /// # Errors
    ///
    /// Returns the underlying I/O error when the ledger cannot be rewritten.
    pub fn forget(&self, content_id: &str) -> io::Result<()> {
        let mut records = self.records()?;
        records.retain(|record| record.content_id != content_id);
        self.write(&records)
    }

    /// Every content id the ledger currently holds, in the order the senses
    /// were remembered — which is the order the lookup produced them, so a
    /// rediscovery can be compared to the ledger element by element.
    ///
    /// # Errors
    ///
    /// Returns the underlying I/O error when the ledger cannot be read.
    pub fn content_ids(&self) -> io::Result<Vec<String>> {
        Ok(self
            .records()?
            .into_iter()
            .map(|record| record.content_id)
            .collect())
    }

    /// Every record whose stored digest still matches its stored fields.
    fn records(&self) -> io::Result<Vec<Record>> {
        let text = match fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error),
        };
        let tree = parse_lino(&text);
        let mut records = Vec::new();
        for root in tree.children.iter().filter(|node| node.name == ROOT) {
            for node in root.children.iter().filter(|node| node.name == "sense") {
                if let Some(record) = read_record(node) {
                    records.push(record);
                }
            }
        }
        Ok(records)
    }

    fn write(&self, records: &[Record]) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out = String::from(ROOT);
        out.push('\n');
        for record in records {
            push_lino_node(&mut out, 2, "sense", Some(&record.content_id));
            for (name, value) in fields(&record.sense) {
                push_lino_node(&mut out, 4, name, Some(&value));
            }
            push_lino_node(&mut out, 4, "digest", Some(&digest(&record.sense)));
        }
        fs::write(&self.path, out)
    }
}

/// One stored sense and the id it is filed under.
struct Record {
    content_id: String,
    sense: ConceptSense,
}

/// The stored fields of a sense, in the order they are written and digested.
fn fields(sense: &ConceptSense) -> Vec<(&'static str, String)> {
    vec![
        ("surface", sense.surface.clone()),
        ("lemma", sense.lemma.clone()),
        ("language", sense.language.clone()),
        ("gloss", sense.gloss.clone()),
        ("part_of_speech", sense.part_of_speech.clone()),
        ("synonyms", sense.synonyms.join(" ")),
        ("source", sense.source_id.clone()),
        ("source_url", sense.source_url.clone()),
        ("sha256", sense.sha256.clone()),
        ("fetched_at", sense.fetched_at.clone()),
        ("tier", sense.tier.slug().to_owned()),
        ("license_name", sense.license_name.clone()),
        ("license_url", sense.license_url.clone()),
        ("depth", sense.depth.to_string()),
        ("rediscover", sense.source_url.clone()),
    ]
}

/// The digest a record is checked against: over the stored fields themselves,
/// so editing any one of them in the file invalidates the record.
fn digest(sense: &ConceptSense) -> String {
    let payload = fields(sense)
        .into_iter()
        .map(|(name, value)| format!("{name}\u{1f}{value}"))
        .collect::<Vec<_>>()
        .join("\u{1e}");
    sha256_hex(payload.as_bytes())
}

/// Rebuild one record, refusing it when the stored digest no longer matches.
fn read_record(node: &LinoNode) -> Option<Record> {
    let sense = ConceptSense {
        surface: node.find_child_value("surface").to_owned(),
        lemma: node.find_child_value("lemma").to_owned(),
        language: node.find_child_value("language").to_owned(),
        gloss: node.find_child_value("gloss").to_owned(),
        part_of_speech: node.find_child_value("part_of_speech").to_owned(),
        synonyms: node
            .find_child_value("synonyms")
            .split_whitespace()
            .map(str::to_owned)
            .collect(),
        source_id: node.find_child_value("source").to_owned(),
        source_url: node.find_child_value("source_url").to_owned(),
        sha256: node.find_child_value("sha256").to_owned(),
        fetched_at: node.find_child_value("fetched_at").to_owned(),
        // A record read back from disk was served from the ledger, not from the
        // network: saying otherwise would be a fabricated provenance of the
        // system's own state.
        cached: true,
        tier: crate::reasoning_standard::episode::tier_from_slug(node.find_child_value("tier"))?,
        license_name: node.find_child_value("license_name").to_owned(),
        license_url: node.find_child_value("license_url").to_owned(),
        depth: node.find_child_value("depth").parse().unwrap_or(0),
    };
    (digest(&sense) == node.find_child_value("digest")).then(|| Record {
        content_id: node.id.clone(),
        sense,
    })
}
