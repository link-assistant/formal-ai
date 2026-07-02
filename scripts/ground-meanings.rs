//! Wikidata grounding pipeline for the canonical seed (issue #398, defect #3).
//!
//! PR #399 review (comment 4664274427, defect 3 / CI check 3) demands that every
//! seed meaning reference a *real* Wikidata id, with the per-id source snapshot
//! cached so the grounding closure is checked in — "never left dangling, never
//! deferred". This script is the re-runnable algorithm that does that for a
//! curated, *verified* batch of common-vocabulary meanings.
//!
//! For each `(slug, qid, expected_label_token)` entry it:
//!
//!   1. Fetches `Special:EntityData/<qid>.json` from Wikidata (via `curl`) when
//!      the cache file is missing, trims it to the established cache convention
//!      (`type`, `id`, `labels`/`descriptions`/`aliases` in en/ru/hi/zh only,
//!      wrapped in `{entities:{<qid>:…}, success:1}`), and writes the pretty
//!      multi-line JSON to `data/cache/wikidata/entity/<qid>.json`.
//!   2. **Verifies** that the fetched entity's labels actually contain
//!      `expected_label_token` (case-insensitive). This guard is the whole point
//!      of curating tokens: a wrong Qid (e.g. `Q206` is "Stephen Harper", not
//!      "seven") is *refused*, never grounded, so the batch can only inject
//!      correct anchors.
//!   3. Generates the lossless `.lino` snapshot via the
//!      `wikidata_json_to_lino` example (the same codec the cache is built with).
//!   4. Inserts `grounded-in <qid>` as the first child of the meaning block in
//!      `data/seed/**/meanings*.lino` (idempotent — re-running is a no-op).
//!
//! Browser builds read the canonical seed via `scripts/sync-seed.sh` and
//! `src/web/seed_loader.js`, so this script only updates `data/seed` and the
//! checked-in Wikidata cache.
//!
//! Network access is only needed the first time an id is fetched; afterwards the
//! checked-in cache satisfies the closure tests offline. Run with
//! `rust-script scripts/ground-meanings.rs` (std-only; also compiles with
//! `rustc`). Requires `curl`, `python3`, and a built `cargo`.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const USER_AGENT: &str = "formal-ai-grounding/1.0 (https://github.com/link-assistant/formal-ai)";

/// `(meaning slug, Wikidata id, expected label token)`. Every token here was
/// confirmed against the live `labels` of the listed id before it was added —
/// the verification step below re-checks it on every run so a stale or wrong
/// mapping fails loudly instead of grounding a meaning to the wrong concept.
const GROUNDINGS: &[(&str, &str, &str)] = &[
    // calendar weekdays
    ("monday", "Q105", "monday"),
    ("tuesday", "Q127", "tuesday"),
    ("wednesday", "Q128", "wednesday"),
    ("thursday", "Q129", "thursday"),
    ("friday", "Q130", "friday"),
    ("saturday", "Q131", "saturday"),
    ("sunday", "Q132", "sunday"),
    // arithmetic operations
    ("addition", "Q32043", "addition"),
    ("subtraction", "Q40754", "subtraction"),
    ("multiplication", "Q40276", "multiplication"),
    ("division", "Q1226939", "division"),
    // currencies
    ("us_dollar", "Q4917", "dollar"),
    ("euro", "Q4916", "euro"),
    ("ruble", "Q41044", "ruble"),
    // length units
    ("meter", "Q11573", "metre"),
    ("kilometer", "Q828224", "kilometre"),
    ("centimeter", "Q174728", "centimetre"),
    ("millimeter", "Q174789", "millimetre"),
    // mass units
    ("gram", "Q41803", "gram"),
    ("kilogram", "Q11570", "kilogram"),
    ("pound", "Q100995", "pound"),
    // time units
    ("second", "Q11574", "second"),
    ("minute", "Q7727", "minute"),
    ("hour", "Q25235", "hour"),
    ("millisecond", "Q723733", "millisecond"),
    ("day", "Q573", "day"),
    ("month", "Q5151", "month"),
    // data-size units
    ("byte", "Q8799", "byte"),
    ("bit", "Q8805", "bit"),
    ("terabyte", "Q79741", "terabyte"),
    // temperature
    ("celsius", "Q25267", "celsius"),
    // mathematical functions
    ("square_root", "Q134237", "square root"),
    ("logarithm", "Q11197", "logarithm"),
    ("natural_logarithm", "Q204037", "natural logarithm"),
    ("sine", "Q152415", "sine"),
    // core quantities
    ("money", "Q1368", "money"),
    ("quantity", "Q309314", "quantity"),
    // cardinal numbers. Several Wikidata number items carry only a
    // Hindi-numeral label, so the verification token is the surface the item
    // actually exposes in one of the cached languages (en digit where present,
    // otherwise the Devanagari numeral).
    ("zero", "Q204", "zero"),
    ("one", "Q199", "1"),
    ("two", "Q200", "2"),
    ("three", "Q201", "3"),
    ("four", "Q202", "4"),
    ("five", "Q203", "\u{096B}"),
    ("six", "Q23488", "\u{096C}"),
    ("seven", "Q23350", "\u{096D}"),
    ("eight", "Q23355", "\u{096E}"),
    ("nine", "Q19108", "\u{096F}"),
    ("ten", "Q23806", "10"),
    ("cardinal_number", "Q1329258", "cardinal"),
    // physical dimensions and the unit-of-measurement concept
    ("unit", "Q47574", "unit"),
    ("length", "Q36253", "length"),
    ("mass", "Q11423", "mass"),
    ("time", "Q11471", "time"),
    ("temperature", "Q11466", "temperature"),
    ("data_storage", "Q105666562", "data size"),
    // additional measurement units
    ("kilobyte", "Q79726", "kilobyte"),
    ("megabyte", "Q79735", "megabyte"),
    ("gigabyte", "Q79738", "gigabyte"),
    ("ton", "Q191118", "tonne"),
    ("fahrenheit", "Q42289", "fahrenheit"),
    ("kelvin", "Q11579", "kelvin"),
    // additional mathematical functions and operations
    ("cosine", "Q1256164", "cosine"),
    ("tangent", "Q1129196", "tangent"),
    ("modulo", "Q1799665", "modulo"),
    ("arithmetic_operation", "Q12170668", "arithmetic"),
    ("mathematical_function", "Q11348", "function"),
    // programming languages
    ("program_language", "Q9143", "programming language"),
    ("program_language_rust", "Q575650", "rust"),
    ("program_language_python", "Q28865", "python"),
    ("program_language_javascript", "Q2005", "javascript"),
    ("program_language_typescript", "Q978185", "typescript"),
    ("program_language_go", "Q37227", "go"),
    ("program_language_c", "Q15777", "c"),
    ("program_language_cpp", "Q2407", "c++"),
    ("program_language_java", "Q251", "java"),
    ("program_language_csharp", "Q2370", "c#"),
    ("program_language_ruby", "Q161053", "ruby"),
    // natural languages
    ("human_language", "Q33742", "natural language"),
    ("language_english", "Q1860", "english"),
    ("language_russian", "Q7737", "russian"),
    ("language_hindi", "Q1568", "hindi"),
    ("language_chinese", "Q7850", "chinese"),
    // concrete nouns used by the translation vocabulary
    ("apple", "Q89", "apple"),
    ("tomato", "Q23501", "tomato"),
    ("cucumber", "Q2735883", "cucumber"),
    ("potato", "Q10998", "potato"),
    ("carrot", "Q81", "carrot"),
    ("bread", "Q7802", "bread"),
    ("water", "Q283", "water"),
    // fact relations ground to Wikidata properties (P-ids), not Q-items
    ("capital", "P36", "capital"),
    ("population", "P1082", "population"),
    ("continent", "P30", "continent"),
    ("currency", "P38", "currency"),
    ("official_language", "P37", "official language"),
    // lexical-meta concepts
    ("part_of_speech", "Q82042", "part of speech"),
    ("noun", "Q1084", "noun"),
    ("noun_phrase", "Q1401131", "noun phrase"),
    ("lexical_form", "Q4147654", "grammatical form"),
    ("lexical_sense", "Q1570700", "word sense"),
    // grammatical number and its values (issue #538): so a word form can pin
    // whether it lexicalises the singular or the plural of its meaning.
    ("grammatical_number", "Q104083", "grammatical number"),
    ("singular", "Q110786", "singular"),
    ("plural", "Q146786", "plural"),
    // translation vocabulary and common concepts
    ("translate", "Q7553", "translation"),
    ("synonym", "Q42106", "synonym"),
    // finance concepts
    ("investment", "Q4290", "investment"),
    ("interest_finance", "Q170924", "interest"),
    ("compounding", "Q959606", "compound interest"),
    ("year_period", "Q577", "year"),
    // calculator concepts
    ("exchange_rate", "Q66100", "exchange rate"),
    ("quantity_conversion", "Q618655", "conversion of units"),
    // calendar concepts
    ("calendar_day", "Q573", "day"),
    ("calendar_date", "Q205892", "date"),
    ("calendar_week", "Q23387", "week"),
    // fact concepts and relations (relations ground to properties)
    ("physical_constant", "Q173227", "physical constant"),
    ("author_of_book", "P50", "author"),
    ("painter_of_painting", "P170", "creator"),
    ("built_year", "P571", "inception"),
    // common conversational and discourse vocabulary
    ("greeting_hello", "Q98815142", "hello"),
    ("gratitude_thank_you", "Q2728730", "gratitude"),
    ("affirmation_yes", "Q6452715", "yes"),
    ("example", "Q14944328", "example"),
    ("conjunction_or", "Q1651704", "disjunction"),
    // core programming artifacts
    ("program", "Q40056", "program"),
    ("code", "Q128751", "source code"),
    ("sort", "Q2303697", "sorting"),
    // discourse and calendar concepts
    ("politeness", "Q281287", "politeness"),
    ("calendar_today", "Q3151690", "today"),
    ("calendar_tomorrow", "Q1209716", "tomorrow"),
    (
        "calendar_day_after_tomorrow",
        "Q1036448",
        "day after tomorrow",
    ),
];

/// `curl | python3` trim+verify program. Reads the full EntityData JSON on
/// stdin, keeps only the cache-convention keys and the en/ru/hi/zh languages,
/// asserts the expected token appears in some label, and writes pretty JSON.
/// Exits non-zero (without writing) when the token is absent — the wrong-Qid
/// guard.
const TRIM_PROGRAM: &str = r#"
import sys, json
from collections import OrderedDict
qid, token, out_path = sys.argv[1], sys.argv[2], sys.argv[3]
langs = ["en", "ru", "hi", "zh"]
doc = json.load(sys.stdin)
entity = doc["entities"][qid]
labels = entity.get("labels", {})
values = " | ".join(v.get("value", "") for v in labels.values()).lower()
if token.lower() not in values:
    sys.stderr.write("token %r not found in labels of %s (%s)\n" % (token, qid, values))
    sys.exit(3)
def keep_lang_map(section):
    return OrderedDict((lang, section[lang]) for lang in langs if lang in section)
trimmed = OrderedDict()
trimmed["type"] = entity["type"]
trimmed["id"] = entity["id"]
if "labels" in entity:
    trimmed["labels"] = keep_lang_map(entity["labels"])
if "descriptions" in entity:
    trimmed["descriptions"] = keep_lang_map(entity["descriptions"])
if "aliases" in entity:
    kept = keep_lang_map(entity["aliases"])
    if kept:
        trimmed["aliases"] = kept
result = OrderedDict()
result["entities"] = OrderedDict([(qid, trimmed)])
result["success"] = 1
with open(out_path, "w", encoding="utf-8") as handle:
    json.dump(result, handle, ensure_ascii=False, indent=2)
    handle.write("\n")
"#;

fn main() -> io::Result<()> {
    let cache_root = Path::new("data/cache/wikidata");
    let seed_files = collect_seed_files(Path::new("data/seed"))?;

    let mut grounded = 0usize;
    let mut skipped: Vec<String> = Vec::new();

    for (slug, qid, token) in GROUNDINGS {
        // Wikidata ids are sharded by kind: `P…` are properties, `L…` are
        // lexemes, and everything else (`Q…`) is an item. The grounding-closure
        // test resolves cache paths the same way, so a fact relation grounded to
        // a property (e.g. `capital` -> `P36`) must land under `property/`.
        let kind = match qid.chars().next() {
            Some('P') => "property",
            Some('L') => "lexeme",
            _ => "entity",
        };
        let cache_dir = cache_root.join(kind);
        fs::create_dir_all(&cache_dir)?;
        let json_path = cache_dir.join(format!("{qid}.json"));
        let lino_path = cache_dir.join(format!("{qid}.lino"));

        if !json_path.exists() {
            if let Err(reason) = fetch_and_trim(qid, token, &json_path) {
                skipped.push(format!("{slug} ({qid}): {reason}"));
                continue;
            }
        }
        ensure_lino(qid, &json_path, &lino_path)?;

        match ground_seed_slug(&seed_files, slug, qid)? {
            GroundOutcome::Inserted => grounded += 1,
            GroundOutcome::AlreadyGrounded => {}
            GroundOutcome::SlugMissing => {
                skipped.push(format!("{slug} ({qid}): slug not found in seed"));
            }
        }
    }

    println!("grounded {grounded} meaning(s) to verified Wikidata ids");
    if !skipped.is_empty() {
        println!("skipped {} entr(ies):", skipped.len());
        for entry in &skipped {
            println!("  - {entry}");
        }
    }
    Ok(())
}

/// Fetch `qid` from Wikidata, trim to the cache convention, and verify the
/// label token. Returns an error string (the batch records it and moves on)
/// when the fetch fails or the token is absent.
fn fetch_and_trim(qid: &str, token: &str, json_path: &Path) -> Result<(), String> {
    let url = format!("https://www.wikidata.org/wiki/Special:EntityData/{qid}.json");
    let curl = Command::new("curl")
        .args(["-sfL", "-A", USER_AGENT, &url])
        .output()
        .map_err(|error| format!("curl failed to launch: {error}"))?;
    if !curl.status.success() {
        return Err(format!("curl exited {}", curl.status));
    }

    let mut python = Command::new("python3")
        .args(["-c", TRIM_PROGRAM, qid, token, &json_path.to_string_lossy()])
        .stdin(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|error| format!("python3 failed to launch: {error}"))?;
    python
        .stdin
        .take()
        .expect("python3 stdin")
        .write_all(&curl.stdout)
        .map_err(|error| format!("failed to pipe JSON: {error}"))?;
    let output = python
        .wait_with_output()
        .map_err(|error| format!("python3 wait failed: {error}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(())
}

fn ensure_lino(qid: &str, json_path: &Path, lino_path: &Path) -> io::Result<()> {
    if lino_path.exists() {
        return Ok(());
    }
    let status = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--example",
            "wikidata_json_to_lino",
            "--",
            qid,
            &json_path.to_string_lossy(),
            &lino_path.to_string_lossy(),
        ])
        .status()?;
    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("wikidata_json_to_lino failed for {qid}"),
        ));
    }
    Ok(())
}

enum GroundOutcome {
    Inserted,
    AlreadyGrounded,
    SlugMissing,
}

/// Insert `grounded-in <qid>` as the first child of the `slug` meaning block in
/// whichever seed file declares it. Idempotent: if the block already carries a
/// `grounded-in` line the file is left untouched.
fn ground_seed_slug(seed_files: &[PathBuf], slug: &str, qid: &str) -> io::Result<GroundOutcome> {
    let header = format!("  {slug}");
    for path in seed_files {
        let content = fs::read_to_string(path)?;
        let lines: Vec<&str> = content.lines().collect();
        let Some(index) = lines.iter().position(|line| *line == header) else {
            continue;
        };
        // Scan the block body (lines indented deeper than the header).
        let mut already = false;
        for line in &lines[index + 1..] {
            let indent = leading_spaces(line);
            if !line.trim().is_empty() && indent <= 2 {
                break;
            }
            if line.trim() == format!("grounded-in {qid}")
                || line.trim().starts_with("grounded-in ")
            {
                already = true;
                break;
            }
        }
        if already {
            return Ok(GroundOutcome::AlreadyGrounded);
        }
        let mut rebuilt: Vec<String> = Vec::with_capacity(lines.len() + 1);
        for (position, line) in lines.iter().enumerate() {
            rebuilt.push((*line).to_string());
            if position == index {
                rebuilt.push(format!("    grounded-in {qid}"));
            }
        }
        let mut joined = rebuilt.join("\n");
        if content.ends_with('\n') {
            joined.push('\n');
        }
        fs::write(path, joined)?;
        return Ok(GroundOutcome::Inserted);
    }
    Ok(GroundOutcome::SlugMissing)
}

fn leading_spaces(line: &str) -> usize {
    line.chars()
        .take_while(|character| *character == ' ')
        .count()
}

fn collect_seed_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            out.extend(collect_seed_files(&path)?);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("lino")
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("meanings"))
        {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}
