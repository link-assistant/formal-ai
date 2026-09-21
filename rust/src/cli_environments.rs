//! Printing of the declared execution environments.

use formal_ai::environment_records;

pub fn run_environments() {
    for record in environment_records() {
        println!("# {}", record.id);
        println!("  label: {}", record.label);
        println!("  runtime: {}", record.runtime);
        println!("  seed_path: {}", record.seed_path);
        println!("  memory_store: {}", record.memory_store);
        println!("  memory_export: {}", record.memory_export_command);
        println!("  bundle_export: {}", record.bundle_export_command);
        println!("  bundle_import: {}", record.bundle_import_command);
        if !record.start_command.is_empty() {
            println!("  start: {}", record.start_command);
        }
        if !record.package_command.is_empty() {
            println!("  package: {}", record.package_command);
        }
        if !record.tools.is_empty() {
            println!("  tools: {}", record.tools.join(", "));
        }
        println!();
    }
}
