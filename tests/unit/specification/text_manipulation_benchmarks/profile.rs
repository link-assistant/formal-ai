use lino_objects_codec::format::parse_indented;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const TEXT_EDIT_PROFILE_FIXTURE: &str = "data/benchmarks/text-manipulation-suite.lino";

#[derive(Debug)]
struct LinoRecord {
    kind: String,
    id: String,
    fields: Vec<(String, String)>,
}

#[derive(Debug)]
pub(super) struct TextEditSource {
    pub(super) id: String,
    pub(super) title: String,
    pub(super) group: String,
    pub(super) domain: String,
    pub(super) primary_url: String,
    pub(super) local_profile: String,
}

#[derive(Debug)]
pub(super) struct TextEditSuite {
    pub(super) sources: BTreeMap<String, TextEditSource>,
    pub(super) minimum_pass_count: usize,
    pub(super) minimum_pass_count_per_source: usize,
    pub(super) local_ten_percent_floor_per_source: usize,
    pub(super) sources_required: usize,
    pub(super) additional_sources_required: usize,
    pub(super) variations_per_source: usize,
    pub(super) ratchet_policy: String,
    pub(super) upstream_payload_policy: String,
}

#[derive(Debug)]
pub(super) struct ProfileCase {
    pub(super) source: String,
    pub(super) prompt: String,
    pub(super) answer: String,
    pub(super) rule: &'static str,
}

pub(super) fn load_text_edit_suite() -> TextEditSuite {
    let text =
        fs::read_to_string(repo_root().join(TEXT_EDIT_PROFILE_FIXTURE)).expect("benchmark fixture");
    validate_lino_syntax(&text);
    parse_text_edit_suite(&text)
}

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn validate_lino_syntax(text: &str) {
    for record in split_records(text) {
        parse_indented(&record).expect("benchmark record should be valid Links Notation");
    }
}

fn parse_text_edit_suite(text: &str) -> TextEditSuite {
    let mut sources = BTreeMap::new();
    let mut minimum_pass_count = 0usize;
    let mut minimum_pass_count_per_source = 0usize;
    let mut local_ten_percent_floor_per_source = 0usize;
    let mut sources_required = 0usize;
    let mut additional_sources_required = 0usize;
    let mut variations_per_source = 0usize;
    let mut ratchet_policy = String::new();
    let mut upstream_payload_policy = String::new();

    for record in parse_records(text) {
        match record.kind.as_str() {
            "text_manipulation_suite" => {
                minimum_pass_count =
                    parse_usize_field(&record.fields, "minimum_pass_count").unwrap_or(0);
                minimum_pass_count_per_source =
                    parse_usize_field(&record.fields, "minimum_pass_count_per_source").unwrap_or(0);
                local_ten_percent_floor_per_source =
                    parse_usize_field(&record.fields, "local_10_percent_floor_per_source")
                        .unwrap_or(0);
                sources_required =
                    parse_usize_field(&record.fields, "sources_required").unwrap_or(0);
                additional_sources_required =
                    parse_usize_field(&record.fields, "additional_sources_required").unwrap_or(0);
                variations_per_source =
                    parse_usize_field(&record.fields, "variations_per_source").unwrap_or(0);
                ratchet_policy = field_value(&record.fields, "ratchet_policy");
                upstream_payload_policy = field_value(&record.fields, "upstream_payload_policy");
            }
            "text_manipulation_source" => {
                let source = TextEditSource {
                    id: record.id,
                    title: field_value(&record.fields, "title"),
                    group: field_value(&record.fields, "group"),
                    domain: field_value(&record.fields, "domain"),
                    primary_url: field_value(&record.fields, "primary_url"),
                    local_profile: field_value(&record.fields, "local_profile"),
                };
                sources.insert(source.id.clone(), source);
            }
            _ => {}
        }
    }

    TextEditSuite {
        sources,
        minimum_pass_count,
        minimum_pass_count_per_source,
        local_ten_percent_floor_per_source,
        sources_required,
        additional_sources_required,
        variations_per_source,
        ratchet_policy,
        upstream_payload_policy,
    }
}

fn parse_records(text: &str) -> Vec<LinoRecord> {
    split_records(text)
        .into_iter()
        .map(|record| parse_record(&record))
        .collect()
}

fn split_records(text: &str) -> Vec<String> {
    let mut records = Vec::new();
    let mut current = Vec::new();
    for line in text.lines() {
        let line = line.trim_end();
        if line.trim().is_empty() {
            continue;
        }
        if !line.starts_with(char::is_whitespace) && !current.is_empty() {
            records.push(current.join("\n"));
            current.clear();
        }
        current.push(line.to_owned());
    }
    if !current.is_empty() {
        records.push(current.join("\n"));
    }
    records
}

fn parse_record(block: &str) -> LinoRecord {
    let mut lines = block.lines().filter(|line| !line.trim().is_empty());
    let header = lines.next().expect("record header");
    let fields = lines
        .map(parse_lino_line)
        .filter(|(name, _)| !name.is_empty())
        .collect::<Vec<_>>();
    let kind = field_value(&fields, "record_type");
    let id = field_value(&fields, "id");
    assert!(!kind.is_empty(), "record `{header}` is missing record_type");
    assert!(!id.is_empty(), "record `{header}` is missing id");
    LinoRecord { kind, id, fields }
}

fn parse_lino_line(line: &str) -> (String, String) {
    let content = line.trim();
    if let Some((name, raw_value)) = content.split_once(' ') {
        (name.to_owned(), unescape_quoted(raw_value.trim()))
    } else {
        (content.to_owned(), String::new())
    }
}

fn unescape_quoted(raw: &str) -> String {
    let inner = raw
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(raw);
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('"') => out.push('"'),
                Some('\\') | None => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn field_value(fields: &[(String, String)], name: &str) -> String {
    fields
        .iter()
        .find_map(|(field_name, value)| (field_name == name).then(|| value.clone()))
        .unwrap_or_default()
}

fn parse_usize_field(fields: &[(String, String)], name: &str) -> Option<usize> {
    let raw = field_value(fields, name);
    (!raw.is_empty()).then(|| {
        raw.parse::<usize>()
            .unwrap_or_else(|err| panic!("invalid {name} `{raw}`: {err}"))
    })
}

pub(super) fn profile_cases_for_source(source: &TextEditSource) -> Vec<ProfileCase> {
    assert!(
        !source.title.is_empty()
            && !source.domain.is_empty()
            && source.primary_url.starts_with("https://")
            && !source.local_profile.is_empty(),
        "source metadata should be reviewable: {source:?}"
    );

    let words = source.id.split('_').collect::<Vec<_>>();
    let plain = format!("local {}", words.join(" "));
    let title = format!("Local {}", titleize_words(&words));
    let snake = format!("local_{}", source.id);
    let kebab = format!("local-{}", source.id.replace('_', "-"));
    let camel = format!("local{}", pascalize_words(&words));
    let pascal = format!("Local{}", pascalize_words(&words));
    let line_one = source.id.clone();
    let line_two = String::from("profile");
    let source_lines = format!("{line_one}\n{line_two}\n{}", source.domain);
    let word_count_text = format!("{plain} profile");
    let word_count = word_count_text.split_whitespace().count().to_string();
    let replacement_input = format!("{plain} profile");
    let replacement_output = format!("global {} profile", words.join(" "));
    let duplicate_lines = format!("{line_one}\n{line_one}\n{line_two}");
    let unsorted_lines = format!("{line_two}\n{line_one}");
    let sorted_lines = {
        let mut lines = [line_two.as_str(), line_one.as_str()];
        lines.sort_unstable();
        lines.join("\n")
    };

    vec![
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Title case this text: \"{plain}\""),
            answer: title,
            rule: "rule_title_case",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Snake case this text: \"{plain}\""),
            answer: snake,
            rule: "rule_snake_case",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Kebab case this text: \"{plain}\""),
            answer: kebab,
            rule: "rule_kebab_case",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Camel case this text: \"{plain}\""),
            answer: camel,
            rule: "rule_camel_case",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Pascal case this text: \"{plain}\""),
            answer: pascal,
            rule: "rule_pascal_case",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Strip empty lines: \"{line_one}\n\n{line_two}\""),
            answer: format!("{line_one}\n{line_two}"),
            rule: "rule_strip_empty_lines",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Join lines: \"{line_one}\n{line_two}\""),
            answer: format!("{line_one} {line_two}"),
            rule: "rule_join_lines",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Number lines: \"{line_one}\n{line_two}\""),
            answer: format!("1. {line_one}\n2. {line_two}"),
            rule: "rule_number_lines",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Indent lines: \"{line_one}\n{line_two}\""),
            answer: format!("    {line_one}\n    {line_two}"),
            rule: "rule_indent_lines",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Outdent lines: \"    {line_one}\n\t{line_two}\""),
            answer: format!("{line_one}\n{line_two}"),
            rule: "rule_outdent_lines",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Count words: \"{word_count_text}\""),
            answer: word_count,
            rule: "rule_count_words",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Count lines: \"{source_lines}\""),
            answer: String::from("3"),
            rule: "rule_count_lines",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: String::from("Count characters: \"abcd\""),
            answer: String::from("4"),
            rule: "rule_count_characters",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!(
                "Extract URLs from this text: \"Source {} docs\"",
                source.primary_url
            ),
            answer: source.primary_url.clone(),
            rule: "rule_extract_url",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Extract numbers: \"10 {} -2 3.5\"", source.id),
            answer: String::from("10\n-2\n3.5"),
            rule: "rule_extract_number",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: String::from("Remove punctuation: \"Alpha, beta!\""),
            answer: String::from("Alpha beta"),
            rule: "rule_remove_punctuation",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Sentence case this text: \"hELLO {}\"", source.id),
            answer: format!("Hello {}", source.id),
            rule: "rule_sentence_case",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: String::from("Sort words: \"zeta alpha beta\""),
            answer: String::from("alpha beta zeta"),
            rule: "rule_sort_words",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Reverse lines: \"{line_one}\n{line_two}\""),
            answer: format!("{line_two}\n{line_one}"),
            rule: "rule_reverse_lines",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: String::from("Comment lines: \"let x = 1;\nreturn x;\""),
            answer: String::from("// let x = 1;\n// return x;"),
            rule: "rule_comment_lines",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Replace \"local\" with \"global\": \"{replacement_input}\""),
            answer: replacement_output,
            rule: "rule_replace_text",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Remove \" profile\" from \"{replacement_input}\""),
            answer: plain.clone(),
            rule: "rule_remove_text",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Append \" ready\" to \"{plain}\""),
            answer: format!("{plain} ready"),
            rule: "rule_append_text",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Prepend \"ready \" to \"{plain}\""),
            answer: format!("ready {plain}"),
            rule: "rule_prepend_text",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Normalize whitespace: \"{line_one}   {line_two}\n  ready\""),
            answer: format!("{line_one} {line_two} ready"),
            rule: "rule_normalize_whitespace",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Trim whitespace: \"  {plain}  \""),
            answer: plain.clone(),
            rule: "rule_trim_whitespace",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Deduplicate lines: \"{duplicate_lines}\""),
            answer: format!("{line_one}\n{line_two}"),
            rule: "rule_deduplicate_lines",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Sort lines: \"{unsorted_lines}\""),
            answer: sorted_lines,
            rule: "rule_sort_lines",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!("Count occurrences of \"local\": \"{plain} local\""),
            answer: String::from("2"),
            rule: "rule_count_occurrences",
        },
        ProfileCase {
            source: source.id.clone(),
            prompt: format!(
                "Extract email addresses from this text: \"Contact {}@example.test now\"",
                source.id
            ),
            answer: format!("{}@example.test", source.id),
            rule: "rule_extract_email",
        },
    ]
}

fn titleize_words(words: &[&str]) -> String {
    words
        .iter()
        .map(|word| capitalize_ascii_word(word))
        .collect::<Vec<_>>()
        .join(" ")
}

fn pascalize_words(words: &[&str]) -> String {
    words
        .iter()
        .map(|word| capitalize_ascii_word(word))
        .collect::<String>()
}

fn capitalize_ascii_word(word: &str) -> String {
    let mut chars = word.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut out = first.to_ascii_uppercase().to_string();
    out.push_str(chars.as_str());
    out
}
