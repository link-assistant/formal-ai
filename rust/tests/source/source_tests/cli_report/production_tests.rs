use clap::Parser;
use serde_json::json;

use super::{DEFAULT_SURFACE, ReportAction, export_trace};
use crate::{Args, Command};

#[test]
fn body_parser_defaults_to_the_configured_surface() {
    let parsed = Args::try_parse_from(["formal-ai", "report", "body"]).expect("parse report body");
    let Some(Command::Report(report)) = parsed.command else {
        panic!("expected report command");
    };
    let ReportAction::Body(body) = report.action else {
        panic!("expected report body action");
    };
    assert_eq!(body.surface, DEFAULT_SURFACE);
}

#[test]
fn export_trace_formats_metadata_without_null_fields() {
    let context = json!({"metadata": {"source": "server", "messages": 3, "error": null}});
    let mut trace = export_trace(&context);
    trace.sort();
    assert_eq!(trace, ["messages: 3", "source: server"]);
}
