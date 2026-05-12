use std::fs;
use std::path::Path;

use lino_objects_codec::format::parse_indented;
use walkdir::WalkDir;

const MAX_LINO_LINES: usize = 1_500;

#[test]
fn lino_data_files_are_parseable_human_readable_and_bounded() {
    let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
    assert!(data_dir.is_dir(), "data directory should exist");

    let mut checked_files = 0_usize;
    for entry in WalkDir::new(&data_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("lino") {
            continue;
        }

        checked_files += 1;
        let content = fs::read_to_string(path).expect("lino file should be UTF-8 text");
        let line_count = content.lines().count();
        assert!(
            line_count <= MAX_LINO_LINES,
            "{} has {line_count} lines, exceeding {MAX_LINO_LINES}",
            path.display()
        );
        assert!(
            !content.contains("(str ") && !content.contains("(object "),
            "{} should use indented human-readable Links Notation, not typed object encoding",
            path.display()
        );

        for record in split_records(&content) {
            parse_indented(record).unwrap_or_else(|error| {
                panic!(
                    "{} contains invalid Links Notation: {error}",
                    path.display()
                );
            });
        }
    }

    assert!(
        checked_files >= 3,
        "expected checked-in Links Notation seed data files"
    );
}

fn split_records(content: &str) -> Vec<&str> {
    content
        .split("\n\n")
        .map(str::trim)
        .filter(|record| !record.is_empty())
        .collect()
}
