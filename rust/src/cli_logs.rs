//! Lossless, redacting legacy JSONL and canonical typed-Lino log conversion.
use clap::{Args, ValueEnum};
use formal_ai::dialog_log::structured_log;
use serde_json::Value;
use std::io::{self, Read, Write};
use std::path::PathBuf;
const MAXIMUM_INPUT: u64 = 16 * 1024 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum LogFormat {
    Lino,
    Json,
    Pretty,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum InputFormat {
    Lino,
    Jsonl,
}
#[derive(Debug, Args)]
pub struct LogsArgs {
    /// Existing diagnostic/dialog file; conversion redacts before output.
    file: PathBuf,
    #[arg(long, value_enum, default_value_t=LogFormat::Lino)]
    format: LogFormat,
    /// Override extension-based source detection.
    #[arg(long = "from", value_enum)]
    source: Option<InputFormat>,
}
fn convert(input: &str, from: InputFormat, to: LogFormat) -> io::Result<String> {
    if input.len() as u64 > MAXIMUM_INPUT || !input.is_empty() && !input.ends_with('\n') {
        return Err(io::Error::other("unterminated_or_oversized_log"));
    }
    let mut records = Vec::<Value>::new();
    if !input.is_empty() {
        for line in input
            .strip_suffix('\n')
            .expect("terminal newline checked")
            .split('\n')
        {
            let record = match from {
                InputFormat::Lino => structured_log::decode_dialog(line)?,
                InputFormat::Jsonl => serde_json::from_str(line)
                    .map_err(|_| io::Error::other("invalid_legacy_jsonl"))?,
            };
            records.push(structured_log::redact(&record));
        }
    }
    match to {
        LogFormat::Lino => {
            let mut output = String::new();
            for record in records {
                output.push_str(&structured_log::encode_dialog(&record)?);
                output.push('\n');
            }
            Ok(output)
        }
        LogFormat::Json => serde_json::to_string(&records)
            .map(|value| value + "\n")
            .map_err(io::Error::other),
        LogFormat::Pretty => serde_json::to_string_pretty(&records)
            .map(|value| value + "\n")
            .map_err(io::Error::other),
    }
}
fn read_and_convert(arguments: &LogsArgs) -> io::Result<String> {
    let file = std::fs::File::open(&arguments.file)?;
    if file.metadata()?.len() > MAXIMUM_INPUT {
        return Err(io::Error::other("log_size_limit"));
    }
    let mut input = String::new();
    file.take(MAXIMUM_INPUT + 1).read_to_string(&mut input)?;
    let from = arguments.source.unwrap_or_else(|| {
        if arguments
            .file
            .extension()
            .is_some_and(|extension| extension == "jsonl")
        {
            InputFormat::Jsonl
        } else {
            InputFormat::Lino
        }
    });
    convert(&input, from, arguments.format)
}
/// Both runtimes refuse malformed/truncated conversion with an empty stdout and exit2.
pub fn run_logs(arguments: LogsArgs) {
    let result = read_and_convert(&arguments)
        .and_then(|output| io::stdout().lock().write_all(output.as_bytes()));
    if result.is_err() {
        formal_ai::diagnostic!("error"; "log_conversion_refused");
        std::process::exit(2);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maintained_javascript_goldens_match_canonical_native_conversion() {
        let fixtures: Value = serde_json::from_str(include_str!(
            "../tests/fixtures/logs-cli-runtime-parity.json"
        ))
        .unwrap();
        for fixture in fixtures["cases"].as_array().unwrap() {
            let input = fixture["input"].as_str().unwrap();
            let expected = fixture["lino"].as_str().unwrap();
            assert_eq!(
                convert(input, InputFormat::Jsonl, LogFormat::Lino).unwrap(),
                expected
            );
            assert_eq!(
                convert(expected, InputFormat::Lino, LogFormat::Lino).unwrap(),
                expected
            );
            let json = convert(expected, InputFormat::Lino, LogFormat::Json).unwrap();
            let pretty = convert(expected, InputFormat::Lino, LogFormat::Pretty).unwrap();
            assert_eq!(
                serde_json::from_str::<Value>(&json).unwrap(),
                serde_json::from_str::<Value>(&pretty).unwrap()
            );
        }
    }
    #[test]
    fn malformed_or_partial_streams_never_return_a_prefix() {
        for input in ["{}", "{}\nnot-json\n", "\n", "[\n"] {
            assert!(convert(input, InputFormat::Jsonl, LogFormat::Lino).is_err());
        }
        assert!(
            convert(
                "(formal-ai-log-v1 (null))",
                InputFormat::Lino,
                LogFormat::Json
            )
            .is_err()
        );
        assert!(
            convert(
                "(formal-ai-log-v1 (null))\r\n",
                InputFormat::Lino,
                LogFormat::Json
            )
            .is_err()
        );
    }
    #[test]
    fn empty_stream_and_large_integer_and_float_types_survive() {
        assert_eq!(convert("", InputFormat::Lino, LogFormat::Lino).unwrap(), "");
        assert_eq!(
            convert("", InputFormat::Jsonl, LogFormat::Json).unwrap(),
            "[]\n"
        );
        let line = "{\"integer\":18446744073709551615,\"float\":1.5,\"negative_zero\":-0.0}\n";
        let encoded = convert(line, InputFormat::Jsonl, LogFormat::Lino).unwrap();
        assert_eq!(
            convert(&encoded, InputFormat::Lino, LogFormat::Lino).unwrap(),
            encoded
        );
    }
}
