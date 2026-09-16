//! The append-only toolchain ledger: the recipe is durable, the bytes are not.
//!
//! `data/meta/toolchain-ledger.lino` retains a compact reconstruction record —
//! source id, URL, content id, postcondition probe and the observed version —
//! so a forgotten toolchain is rediscovered to the same content id rather than
//! re-guessed. The installed prefix is disposable.
//!
//! This is recipe stage 8, `retain_experience`. `forget` is the one operation
//! that removes a row, and it removes the disposable bytes with it, because a
//! record without its payload is the whole point and a payload without its
//! record is unattributable.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use super::Platform;
use super::probe::ToolchainProbe;

/// Record type of one retained procedure.
const RECORD_TOOLCHAIN: &str = "toolchain_record";

/// The committed ledger of this repository.
const REPOSITORY_LEDGER: &str = "data/meta/toolchain-ledger.lino";

/// One durable record of a discovered setup procedure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolchainRecord {
    /// The program the record is about.
    pub program: String,
    /// The trusted publisher, by `sources-registry` id.
    pub source_id: String,
    /// The exact URL the procedure was read from.
    pub source_url: String,
    /// SHA-256 of the retrieved bytes.
    pub content_id: String,
    /// Platform the procedure is valid for.
    pub platform: Platform,
    /// The probe that must pass after installation.
    pub postcondition: ToolchainProbe,
    /// The URL a forgotten record is rediscovered from.
    pub rediscover: String,
    /// The version line as observed, never a hard-coded string.
    pub observed_version: String,
    /// Where the disposable bytes were installed.
    pub installed_prefix: PathBuf,
}

impl ToolchainRecord {
    /// The record's Links Notation form, exactly as it is appended.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut out = String::new();
        let name: String = self
            .program
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() {
                    character
                } else {
                    '_'
                }
            })
            .collect();
        let _ = writeln!(out, "toolchain_{name}");
        let _ = writeln!(out, "  record_type \"{RECORD_TOOLCHAIN}\"");
        let _ = writeln!(out, "  program \"{}\"", self.program);
        let _ = writeln!(out, "  source_id \"{}\"", self.source_id);
        let _ = writeln!(out, "  source_url \"{}\"", self.source_url);
        let _ = writeln!(out, "  content_id \"{}\"", self.content_id);
        let _ = writeln!(out, "  platform \"{}\"", self.platform.slug());
        let _ = writeln!(
            out,
            "  postcondition_program \"{}\"",
            self.postcondition.program
        );
        let _ = writeln!(
            out,
            "  postcondition_argv ({})",
            self.postcondition
                .argv
                .iter()
                .map(|argument| ["\"", argument.as_str(), "\""].concat())
                .collect::<Vec<String>>()
                .join(" ")
        );
        let _ = writeln!(out, "  rediscover \"{}\"", self.rediscover);
        let _ = writeln!(out, "  observed_version \"{}\"", self.observed_version);
        let _ = writeln!(
            out,
            "  installed_prefix \"{}\"",
            self.installed_prefix.display()
        );
        out
    }
}

/// The append-only ledger over `data/meta/toolchain-ledger.lino`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolchainLedger {
    /// Path of the committed ledger document.
    pub path: PathBuf,
}

impl ToolchainLedger {
    /// Open the ledger at `path`, creating nothing.
    #[must_use]
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// The committed ledger of this repository.
    #[must_use]
    pub fn from_repo() -> Self {
        Self::new(REPOSITORY_LEDGER)
    }

    /// Every record, in file order.
    #[must_use]
    pub fn records(&self) -> Vec<ToolchainRecord> {
        let Ok(text) = std::fs::read_to_string(&self.path) else {
            return Vec::new();
        };
        let root = crate::seed::parser::parse_lino(&text);
        root.children
            .iter()
            .filter(|node| node.find_child_value("record_type") == RECORD_TOOLCHAIN)
            .map(|node| ToolchainRecord {
                program: node.find_child_value("program").to_owned(),
                source_id: node.find_child_value("source_id").to_owned(),
                source_url: node.find_child_value("source_url").to_owned(),
                content_id: node.find_child_value("content_id").to_owned(),
                platform: Platform::from_slug(node.find_child_value("platform")),
                postcondition: ToolchainProbe {
                    program: node.find_child_value("postcondition_program").to_owned(),
                    argv: super::probe::split_tuple(node.find_child_value("postcondition_argv")),
                    expect: None,
                    requires: Vec::new(),
                },
                rediscover: node.find_child_value("rediscover").to_owned(),
                observed_version: node.find_child_value("observed_version").to_owned(),
                installed_prefix: PathBuf::from(node.find_child_value("installed_prefix")),
            })
            .collect()
    }

    /// The record for `program`, when one was retained.
    #[must_use]
    pub fn record_for(&self, program: &str) -> Option<ToolchainRecord> {
        self.records()
            .into_iter()
            .rev()
            .find(|record| record.program == program)
    }

    /// Append one record. The ledger is never rewritten.
    ///
    /// # Errors
    /// Propagates the write failure.
    pub fn append(&self, record: &ToolchainRecord) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut document = std::fs::read_to_string(&self.path).unwrap_or_default();
        document.push_str(&record.to_links_notation());
        std::fs::write(&self.path, document)
    }

    /// `formal-ai learn forget --toolchain <program>`: delete the record **and**
    /// the installed prefix, retaining nothing but the ability to rediscover.
    ///
    /// # Errors
    /// Propagates the write failure.
    pub fn forget(&self, program: &str) -> std::io::Result<()> {
        for record in self.records() {
            if record.program == program && record.installed_prefix.exists() {
                std::fs::remove_dir_all(&record.installed_prefix)?;
            }
        }
        let retained: String = self
            .records()
            .iter()
            .filter(|record| record.program != program)
            .map(ToolchainRecord::to_links_notation)
            .collect();
        std::fs::write(&self.path, retained)
    }

    /// Re-attach to an installed toolchain after a process restart, from the
    /// ledger alone, without re-probing the publisher.
    #[must_use]
    pub fn reattach(&self, program: &str) -> Option<super::install::WorkspaceToolchain> {
        let record = self.record_for(program)?;
        let root = self
            .path
            .parent()
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
        Some(super::install::WorkspaceToolchain {
            program: record.program.clone(),
            prefix: record.installed_prefix.clone(),
            environment: super::probe::workspace_environment(&root),
            content_id: record.content_id.clone(),
        })
    }
}
