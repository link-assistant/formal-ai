use std::env;
use std::fs;
use std::process;

use formal_ai::json_lino::{json_cache_file, lino_to_json};
use serde_json::Value;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("usage: wikidata_json_to_lino <id> <input.json|input.lino> <output.lino>");
        process::exit(2);
    }

    let root_id = &args[1];
    let input = fs::read_to_string(&args[2]).unwrap_or_else(|error| {
        eprintln!("failed to read {}: {error}", args[2]);
        process::exit(1);
    });
    let json: Value = serde_json::from_str(&input).unwrap_or_else(|json_error| {
        lino_to_json(&input).unwrap_or_else(|lino_error| {
            eprintln!(
                "failed to parse {} as JSON ({json_error}) or LiNo cache ({lino_error})",
                args[2]
            );
            process::exit(1);
        })
    });
    let lino = json_cache_file(root_id, &json);
    fs::write(&args[3], lino).unwrap_or_else(|error| {
        eprintln!("failed to write {}: {error}", args[3]);
        process::exit(1);
    });
}
