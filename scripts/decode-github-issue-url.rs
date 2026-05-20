#!/usr/bin/env rust-script
//! Decode a prefilled GitHub issue URL into readable Markdown.
//!
//! Usage:
//!   rust-script scripts/decode-github-issue-url.rs --url '<issues/new?...>'
//!   pbpaste | rust-script scripts/decode-github-issue-url.rs --body-only

use std::env;
use std::io::{self, Read};
use std::process::exit;

#[derive(Debug, Default)]
struct Options {
    url: Option<String>,
    body_only: bool,
}

#[derive(Debug, Default)]
struct DecodedIssueUrl {
    repository: Option<String>,
    path: Option<String>,
    title: Option<String>,
    labels: Option<String>,
    body: Option<String>,
}

fn usage() -> &'static str {
    "Usage: rust-script scripts/decode-github-issue-url.rs [--url <url>] [--body-only]\n\
     If --url is omitted, the script reads the URL from stdin."
}

fn parse_args() -> Options {
    let mut options = Options::default();
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                println!("{}", usage());
                exit(0);
            }
            "--body-only" => options.body_only = true,
            "--url" => {
                let Some(value) = args.next() else {
                    eprintln!("--url requires a value\n\n{}", usage());
                    exit(2);
                };
                options.url = Some(value);
            }
            _ if options.url.is_none() => options.url = Some(arg),
            _ => {
                eprintln!("Unexpected argument: {arg}\n\n{}", usage());
                exit(2);
            }
        }
    }
    options
}

fn read_input(options: &Options) -> String {
    if let Some(url) = options.url.as_deref() {
        return url.trim().to_owned();
    }
    let mut input = String::new();
    if let Err(error) = io::stdin().read_to_string(&mut input) {
        eprintln!("Failed to read stdin: {error}");
        exit(1);
    }
    input.trim().to_owned()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn percent_decode_once(input: &str, plus_as_space: bool) -> String {
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let (Some(high), Some(low)) =
                (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
            {
                output.push((high << 4) | low);
                index += 3;
                continue;
            }
        }
        if plus_as_space && bytes[index] == b'+' {
            output.push(b' ');
        } else {
            output.push(bytes[index]);
        }
        index += 1;
    }
    String::from_utf8_lossy(&output).into_owned()
}

fn decode_form_component_repeated(input: &str) -> String {
    let mut current = percent_decode_once(input, true);
    for _ in 0..4 {
        let next = percent_decode_once(&current, false);
        if next == current {
            break;
        }
        current = next;
    }
    current
}

fn url_path(url: &str) -> Option<String> {
    let after_scheme = url.split_once("://").map_or(url, |(_, rest)| rest);
    let slash = after_scheme.find('/')?;
    let path_and_query = &after_scheme[slash..];
    let path_without_query = path_and_query
        .split_once('?')
        .map_or(path_and_query, |(path, _)| path);
    let path = path_without_query
        .split_once('#')
        .map_or(path_without_query, |(path, _)| path);
    Some(path.to_owned())
}

fn repository_from_path(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    if parts.len() >= 2 {
        Some(format!("{}/{}", parts[0], parts[1]))
    } else {
        None
    }
}

fn query_string(url: &str) -> Option<&str> {
    let (_, query_and_fragment) = url.split_once('?')?;
    Some(
        query_and_fragment
            .split_once('#')
            .map_or(query_and_fragment, |(query, _)| query),
    )
}

fn decode_issue_url(url: &str) -> DecodedIssueUrl {
    let path = url_path(url);
    let repository = path.as_deref().and_then(repository_from_path);
    let mut decoded = DecodedIssueUrl {
        repository,
        path,
        ..DecodedIssueUrl::default()
    };
    let Some(query) = query_string(url) else {
        return decoded;
    };
    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let key = percent_decode_once(key, true);
        let value = decode_form_component_repeated(value);
        match key.as_str() {
            "title" => decoded.title = Some(value),
            "labels" => decoded.labels = Some(value),
            "body" => decoded.body = Some(value),
            _ => {}
        }
    }
    decoded
}

fn print_full(decoded: &DecodedIssueUrl) {
    println!("# Decoded GitHub Issue URL\n");
    if let Some(repository) = decoded.repository.as_deref() {
        println!("- Repository: `{repository}`");
    }
    if let Some(path) = decoded.path.as_deref() {
        println!("- Path: `{path}`");
    }
    if let Some(labels) = decoded.labels.as_deref() {
        println!("- Labels: `{labels}`");
    }
    println!("\n## Title\n");
    println!("{}", decoded.title.as_deref().unwrap_or("(missing)"));
    println!("\n## Body\n");
    println!("{}", decoded.body.as_deref().unwrap_or("(missing)"));
}

fn main() {
    let options = parse_args();
    let input = read_input(&options);
    if input.is_empty() {
        eprintln!("No URL provided\n\n{}", usage());
        exit(2);
    }
    let decoded = decode_issue_url(&input);
    if options.body_only {
        println!("{}", decoded.body.as_deref().unwrap_or(""));
    } else {
        print_full(&decoded);
    }
}
