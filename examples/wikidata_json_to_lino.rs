use std::env;
use std::fs;
use std::process;

use formal_ai::json_lino::json_cache_file;
use serde_json::Value;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("usage: wikidata_json_to_lino <id> <input.json> <output.lino>");
        process::exit(2);
    }

    let root_id = &args[1];
    let input = fs::read_to_string(&args[2]).unwrap_or_else(|error| {
        eprintln!("failed to read {}: {error}", args[2]);
        process::exit(1);
    });
    let json: Value = serde_json::from_str(&input).unwrap_or_else(|error| {
        eprintln!("failed to parse {}: {error}", args[2]);
        process::exit(1);
    });
    let lino = json_cache_file(root_id, &json);
    fs::write(&args[3], lino).unwrap_or_else(|error| {
        eprintln!("failed to write {}: {error}", args[3]);
        process::exit(1);
    });
}
