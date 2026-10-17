//! Preserve semantic stderr assertions across typed and legacy diagnostics.
#[must_use]
pub fn messages(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .lines()
        .map(|line| {
            if line.trim_start().starts_with("(formal-ai-log-v1 ") {
                let record = formal_ai::dialog_log::structured_log::decode_control(line.trim())
                    .or_else(|_| formal_ai::dialog_log::structured_log::decode(line.trim()))
                    .expect("canonical typed stderr evidence");
                assert!(
                    record["schema"] == "formal-ai-diagnostic-v1"
                        || record["schema"] == "formal-ai-control-v1",
                    "unexpected stderr schema"
                );
                record["message"]
                    .as_str()
                    .expect("typed stderr message")
                    .to_owned()
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
