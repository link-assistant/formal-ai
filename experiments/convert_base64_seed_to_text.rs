// One-shot conversion utility used to migrate the seed bundle from
// `body_base64` chunks (RFC 4648, 76-char wrap) to human-readable `body`
// chunks (raw response text, JSON-style escape, 200 chars per chunk).
//
// Compile + run from the repo root:
//
//     rustc experiments/convert_base64_seed_to_text.rs -O \
//         -o experiments/convert_base64_seed_to_text
//     ./experiments/convert_base64_seed_to_text
//
// Reads every `data/seed/api-cache/*.lino` file in place, rewrites each
// `response_<short_id>` block to use the new format, and refreshes the
// `seed_metadata` header.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const SEED_DIR: &str = "data/seed/api-cache";
const SEED_BODY_CHUNK_CHARS: usize = 200;
const MAX_SEED_LINES_PER_FILE: usize = 1500;

fn main() {
    let mut paths: Vec<PathBuf> = fs::read_dir(SEED_DIR)
        .expect("read seed dir")
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("lino")
        })
        .collect();
    paths.sort_by_key(|p| seed_sort_key(p));

    // Group same-URL chunks across files within each bucket. The old
    // format may split a single body across multiple part files; we want
    // to concatenate them before re-encoding.
    let mut by_bucket: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    let mut bucket_files: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    let mut record_order: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut record_bodies: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();

    for path in &paths {
        let bucket = bucket_from_path(path);
        let contents = fs::read_to_string(path).expect("read file");
        let pairs = parse_old_seed(&contents);
        bucket_files
            .entry(bucket.clone())
            .or_default()
            .push(path.clone());
        let order = record_order.entry(bucket.clone()).or_default();
        let bodies = record_bodies.entry(bucket.clone()).or_default();
        for (url, b64) in pairs {
            let entry = bodies.entry(url.clone()).or_insert_with(|| {
                order.push(url.clone());
                String::new()
            });
            entry.push_str(&b64);
        }
    }

    for (bucket, urls) in &record_order {
        let bodies = record_bodies.get(bucket).unwrap();
        let entries = by_bucket.entry(bucket.clone()).or_default();
        for url in urls {
            let Some(b64) = bodies.get(url) else {
                continue;
            };
            let bytes = match base64_decode(b64) {
                Some(b) => b,
                None => {
                    eprintln!("WARN: bucket={bucket} url={url} base64 decode failed");
                    continue;
                }
            };
            match String::from_utf8(bytes) {
                Ok(body) => entries.push((url.clone(), body)),
                Err(error) => eprintln!(
                    "WARN: bucket={bucket} url={url} body not UTF-8 ({error})"
                ),
            }
        }
    }

    // Drop ALL existing seed files; write fresh ones based on the new
    // chunking.
    for path in &paths {
        fs::remove_file(path).expect("remove old seed file");
    }

    let seed_root = Path::new(SEED_DIR);
    for (bucket, entries) in &by_bucket {
        let header = header_block(bucket);
        write_bucket_parts(bucket, &header, entries, seed_root).expect("write bucket");
    }
}

// Matches `build.rs::seed_sort_key`. Lexicographic sort places
// `<bucket>-part10.lino` before `<bucket>-part2.lino` and pushes
// `<bucket>.lino` after every part, corrupting the chunk order when we
// concatenate split bodies.
fn seed_sort_key(path: &Path) -> (String, u32, String) {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    if let Some((bucket, suffix)) = stem.rsplit_once("-part") {
        if let Ok(n) = suffix.parse::<u32>() {
            return (bucket.to_string(), n, stem.to_string());
        }
    }
    (stem.to_string(), 0, stem.to_string())
}

fn bucket_from_path(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    if let Some((bucket, suffix)) = stem.rsplit_once("-part") {
        if suffix.chars().all(|c| c.is_ascii_digit()) {
            return bucket.to_string();
        }
    }
    stem.to_string()
}

fn header_block(bucket: &str) -> String {
    let cap = if bucket == "wiktionary-pages" { "256" } else { "128" };
    format!(
        "seed_metadata\n  bucket \"{bucket}\"\n  format \"Links Notation; one `response_<short_id>` record per fetched URL\"\n  populated_by \"examples/refresh_translation_cache.rs\"\n  max_records \"{cap}\"\n  max_lines \"1500\"\n  split_marker \"{bucket}-partN.lino\"\n  body_format \"raw response text; concatenate `body` chunks; values escape one literal quote as two consecutive quotes per Links Notation\"\n",
    )
}

/// Parse the old base64-chunked seed format.
fn parse_old_seed(text: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut current_url: Option<String> = None;
    let mut current_b64 = String::new();
    let flush = |url: &mut Option<String>, b64: &mut String, out: &mut Vec<(String, String)>| {
        if let Some(url_value) = url.take() {
            if !b64.is_empty() {
                out.push((url_value, std::mem::take(b64)));
            }
        }
    };
    for raw_line in text.lines() {
        let trimmed = raw_line.trim_end_matches(['\r', '\n']);
        if trimmed.trim().is_empty() {
            continue;
        }
        let indent = trimmed.bytes().take_while(|b| *b == b' ').count();
        let content = &trimmed[indent..];
        if indent == 0 {
            flush(&mut current_url, &mut current_b64, &mut out);
            if content.starts_with("response_") {
                current_url = Some(String::new());
            }
            continue;
        }
        if current_url.is_none() {
            continue;
        }
        if let Some(value) = strip_kv(content, "url") {
            current_url = Some(value.to_owned());
        } else if let Some(value) = strip_kv(content, "body_base64") {
            current_b64.push_str(value);
        }
    }
    flush(&mut current_url, &mut current_b64, &mut out);
    out
}

fn strip_kv<'a>(content: &'a str, key: &str) -> Option<&'a str> {
    let rest = content.strip_prefix(key)?;
    let rest = rest.strip_prefix(' ')?;
    let rest = rest.strip_prefix('"')?;
    rest.strip_suffix('"')
}

fn base64_decode(input: &str) -> Option<Vec<u8>> {
    let mut out: Vec<u8> = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;
    let mut pad: u32 = 0;
    for byte in input.bytes() {
        if matches!(byte, b' ' | b'\t' | b'\r' | b'\n') {
            continue;
        }
        let value = match byte {
            b'A'..=b'Z' => u32::from(byte - b'A'),
            b'a'..=b'z' => u32::from(byte - b'a') + 26,
            b'0'..=b'9' => u32::from(byte - b'0') + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => {
                pad += 1;
                if pad > 2 {
                    return None;
                }
                buf <<= 6;
                bits += 6;
                if bits >= 8 {
                    bits -= 8;
                }
                continue;
            }
            _ => return None,
        };
        if pad > 0 {
            return None;
        }
        buf = (buf << 6) | value;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buf >> bits) & 0xff) as u8);
        }
    }
    Some(out)
}

fn escape_lino_string(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 8);
    for ch in input.chars() {
        if ch == '"' {
            out.push('"');
            out.push('"');
        } else {
            out.push(ch);
        }
    }
    out
}

fn split_body_into_chunks(body: &str, chars: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if body.is_empty() {
        return out;
    }
    let chars_vec: Vec<char> = body.chars().collect();
    let total = chars_vec.len();
    let mut start = 0usize;
    while start < total {
        let mut end = (start + chars).min(total);
        // Don't break in the middle of a run of `"` chars: the next
        // chunk would otherwise start with `"`, which collides with the
        // Links Notation opening-quote count.
        while end < total && chars_vec[end] == '"' {
            end += 1;
        }
        out.push(chars_vec[start..end].iter().collect());
        start = end;
    }
    out
}

fn short_record_id(idx: usize, url: &str) -> String {
    let mut slug = String::with_capacity(24);
    let tail = url.rsplit_once('/').map_or(url, |(_, after)| after);
    for ch in tail.chars().take(24) {
        if ch.is_alphanumeric() || ch == '-' || ch == '_' {
            slug.push(ch);
        } else {
            slug.push('_');
        }
    }
    if slug.is_empty() {
        slug.push_str("record");
    }
    format!("{idx:04}_{slug}")
}

fn write_bucket_parts(
    bucket_name: &str,
    header_block: &str,
    entries: &[(String, String)],
    seed_root: &Path,
) -> Result<usize, String> {
    let header_lines = header_block.lines().count();
    let mut parts_written = 0usize;
    let mut part_index = 0usize;
    let mut buffer = String::new();
    buffer.push_str(header_block);
    let mut lines_in_buffer = header_lines;
    let mut wrote_records_in_part = false;

    for (idx, (url, body)) in entries.iter().enumerate() {
        let chunks = split_body_into_chunks(body, SEED_BODY_CHUNK_CHARS);
        let mut remaining: &[String] = &chunks;
        let mut split_index = 0usize;
        loop {
            if remaining.is_empty() {
                break;
            }
            let separator = usize::from(wrote_records_in_part);
            let record_header = 2;
            let used = lines_in_buffer + separator + record_header;
            let body_budget = MAX_SEED_LINES_PER_FILE
                .saturating_sub(used)
                .saturating_sub(1);
            if body_budget == 0 {
                flush_part(
                    seed_root,
                    bucket_name,
                    &mut part_index,
                    &mut buffer,
                    &mut parts_written,
                )?;
                buffer.push_str(header_block);
                lines_in_buffer = header_lines;
                wrote_records_in_part = false;
                continue;
            }
            if wrote_records_in_part {
                buffer.push('\n');
                lines_in_buffer += 1;
            }
            let short_id = if split_index == 0 {
                short_record_id(idx, url)
            } else {
                format!("{}_p{}", short_record_id(idx, url), split_index + 1)
            };
            buffer.push_str("response_");
            buffer.push_str(&short_id);
            buffer.push('\n');
            buffer.push_str("  url \"");
            buffer.push_str(&escape_lino_string(url));
            buffer.push_str("\"\n");
            lines_in_buffer += 2;
            let take = body_budget.min(remaining.len());
            for chunk in &remaining[..take] {
                buffer.push_str("  body \"");
                buffer.push_str(&escape_lino_string(chunk));
                buffer.push_str("\"\n");
                lines_in_buffer += 1;
            }
            remaining = &remaining[take..];
            split_index += 1;
            wrote_records_in_part = true;
            if !remaining.is_empty() {
                flush_part(
                    seed_root,
                    bucket_name,
                    &mut part_index,
                    &mut buffer,
                    &mut parts_written,
                )?;
                buffer.push_str(header_block);
                lines_in_buffer = header_lines;
                wrote_records_in_part = false;
            }
        }
    }
    flush_part(
        seed_root,
        bucket_name,
        &mut part_index,
        &mut buffer,
        &mut parts_written,
    )?;
    Ok(parts_written)
}

fn flush_part(
    seed_root: &Path,
    bucket_name: &str,
    part_index: &mut usize,
    buffer: &mut String,
    parts_written: &mut usize,
) -> Result<(), String> {
    let path = part_path(seed_root, bucket_name, *part_index);
    fs::write(&path, buffer.as_str()).map_err(|e| format!("write {}: {e}", path.display()))?;
    *parts_written += 1;
    *part_index += 1;
    buffer.clear();
    Ok(())
}

fn part_path(seed_root: &Path, bucket_name: &str, part_index: usize) -> PathBuf {
    if part_index == 0 {
        seed_root.join(format!("{bucket_name}.lino"))
    } else {
        seed_root.join(format!("{bucket_name}-part{part_index}.lino"))
    }
}
