use std::error::Error;
use std::fs;
use std::io;
use std::path::PathBuf;

use clap::Args;
use formal_ai::statement_audit::{
    audit_corpus, parse_evidence_json, AuditConfig, EvidenceCapture, RepositoryCorpus,
};
use serde_json::json;

#[derive(Debug, Clone, Args)]
pub struct StatementAuditArgs {
    /// Repository snapshot to inspect. Git-tracked files are preferred.
    #[arg(long, default_value = ".")]
    root: PathBuf,

    /// Replayable evidence JSON document. Repeat to merge captures.
    #[arg(long = "evidence", value_name = "PATH")]
    evidence: Vec<PathBuf>,

    /// Links Notation audit artifact, or `-` for standard output.
    #[arg(long, default_value = "statement-audit.lino")]
    output: PathBuf,

    /// Softmax temperature used to compare exclusive alternatives.
    #[arg(long, default_value_t = 0.7, value_parser = parse_temperature)]
    temperature: f32,
}

pub fn run_statement_audit(args: &StatementAuditArgs) -> Result<(), Box<dyn Error>> {
    let corpus = RepositoryCorpus::from_repository(&args.root)?;
    let evidence = load_evidence(&args.evidence)?;
    let audit = audit_corpus(
        &corpus,
        &evidence,
        AuditConfig {
            temperature: args.temperature,
            ..AuditConfig::default()
        },
    );
    let rendered = audit.to_links_notation();
    if args.output.as_os_str() == "-" {
        print!("{rendered}");
    } else {
        fs::write(&args.output, rendered)?;
    }
    eprintln!(
        "{}",
        serde_json::to_string(&json!({
            "statement_audit": {
                "root": args.root,
                "output": args.output,
                "statements": audit.statements.len(),
                "contradictions": audit.contradictions.len(),
                "findings": audit.findings.len(),
                "evidence_captures": evidence.len(),
                "skipped_paths": audit.skipped_paths.len(),
                "temperature": args.temperature,
            }
        }))?
    );
    Ok(())
}

fn load_evidence(paths: &[PathBuf]) -> Result<Vec<EvidenceCapture>, Box<dyn Error>> {
    let mut captures = Vec::new();
    for path in paths {
        let input = fs::read_to_string(path)?;
        let parsed = parse_evidence_json(&input).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}: {error}", path.display()),
            )
        })?;
        captures.extend(parsed);
    }
    Ok(captures)
}

fn parse_temperature(value: &str) -> Result<f32, String> {
    let parsed = value.parse::<f32>().map_err(|error| {
        render_error(
            "statement_audit_invalid_temperature",
            &[("error", &error.to_string())],
        )
    })?;
    if parsed.is_finite() && parsed > 0.0 {
        Ok(parsed)
    } else {
        Err(render_error("statement_audit_temperature_range", &[]))
    }
}

fn render_error(intent: &str, values: &[(&str, &str)]) -> String {
    let mut rendered =
        formal_ai::seed::response_for(intent, "en").unwrap_or_else(|| intent.to_owned());
    for (name, value) in values {
        rendered = rendered.replace(&format!("{{{name}}}"), value);
    }
    rendered
}
