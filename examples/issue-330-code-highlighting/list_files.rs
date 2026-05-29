// list_files.rs — the program formal-ai returns for "list the files in the
// current directory" (issue #330 reproduction dialog). It accepts an optional
// path argument, mirroring the issue #324 follow-up turn.
//
// Run command:
//   rustc list_files.rs -o list_files && ./list_files
// Run against a specific directory:
//   ./list_files /etc
// Test (should print this file when pointed at its own folder):
//   ./list_files . | grep list_files.rs

use std::env;
use std::fs;

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| String::from("."));
    let mut names: Vec<String> = fs::read_dir(&path)
        .expect("failed to read directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    for name in names {
        println!("{name}");
    }
}
