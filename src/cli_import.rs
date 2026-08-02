use std::error::Error;
use std::path::PathBuf;

use clap::Subcommand;
use formal_ai::event_log::EventLog;
use formal_ai::lexeme_import::{self, ImportConfig};
use formal_ai::translation::CurlClient;

#[derive(Debug, Subcommand)]
pub enum ImportAction {
    /// Import grounded lexemes from a `concepts` document (`<slug> <Qid>`
    /// pairs). Reads the committed Wikidata entity cache, validates every
    /// generated surface, and writes the batch as seed shard files. With
    /// `--offline` (the default) only the committed cache is read, so the run
    /// is deterministic and reproduces the committed batch byte-for-byte.
    Lexemes {
        /// Concepts document listing `<slug> <Qid>` pairs under a `concepts`
        /// node.
        #[arg(long, value_name = "PATH")]
        concepts: PathBuf,

        /// Directory holding the `<Qid>.json` entity cache records.
        #[arg(long, value_name = "DIR", default_value = "data/cache/wikidata/entity")]
        cache_dir: PathBuf,

        /// Directory the generated seed shard files are written to.
        #[arg(long, value_name = "DIR", default_value = "data/seed")]
        out: PathBuf,

        /// Read only the committed cache; never fetch live. This is the
        /// default; live population additionally requires `FORMAL_AI_LIVE_API`.
        #[arg(long, default_value_t = false)]
        offline: bool,

        /// Print the batch to stdout instead of writing shard files.
        #[arg(long, default_value_t = false)]
        dry_run: bool,

        /// Durable rejection-event document. Defaults beside `--concepts`
        /// with the extension `.import-events.lino` when a rejection occurs.
        #[arg(long, value_name = "PATH")]
        events: Option<PathBuf>,
    },
}

pub fn run_import(action: ImportAction) -> Result<(), Box<dyn Error>> {
    match action {
        ImportAction::Lexemes {
            concepts,
            cache_dir,
            out,
            offline,
            dry_run,
            events: event_path,
        } => {
            let text = std::fs::read_to_string(&concepts)?;
            let parsed = lexeme_import::parse_concepts(&text);
            let online = !offline && lexeme_import::live_api_enabled();
            let config = ImportConfig {
                concepts: parsed,
                cache_dir,
                online,
            };
            let client = online.then(CurlClient::default);
            let mut events = EventLog::new();
            let report = lexeme_import::run(
                &config,
                client
                    .as_ref()
                    .map(|client| client as &dyn formal_ai::translation::http::HttpClient),
                &mut events,
            );

            for rejection in &report.rejected {
                eprintln!(
                    "import_rejected {} {} — {}",
                    rejection.slug, rejection.qid, rejection.reason
                );
            }

            if !report.rejected.is_empty() {
                let event_path =
                    event_path.unwrap_or_else(|| concepts.with_extension("import-events.lino"));
                if let Some(parent) = event_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&event_path, lexeme_import::render_import_events(&events))?;
                return Err(lexeme_import::diagnostic(
                    "lexeme_import_refused_batch",
                    &[
                        ("count", &report.rejected.len().to_string()),
                        ("events", &event_path.display().to_string()),
                    ],
                )
                .into());
            }

            if dry_run {
                for shard in &report.shards {
                    print!("{}", shard.content);
                }
            } else {
                lexeme_import::write_shards(&out, &report.shards)?;
            }

            eprintln!(
                "imported {} concept(s) / {} surface(s) into {} shard(s); coverage {}‰; rejected {}",
                report.accepted.len(),
                report.coverage.emitted_surfaces,
                report.shards.len(),
                report.coverage.permille(),
                report.rejected.len()
            );
        }
    }
    Ok(())
}
