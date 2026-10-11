use super::{BundleInfo, export_bundle, export_full_memory, import_full_memory};

#[test]
fn seed_bytes_survive_empty_lines_final_newlines_and_unicode() {
    let cases = [
        "",
        "\n",
        "\n\n",
        "first\n\nλ🙂\r\nlast\n\n",
        "literal \\n and \"quote\"\r\t",
    ];
    for contents in cases {
        let files = [("seed/renamed.lino", contents)];
        let full = export_full_memory(&files, &[], &[], &BundleInfo::default());
        for document in [export_bundle(&files, &[]), full] {
            assert_eq!(
                import_full_memory(&document).seed_files,
                vec![(String::from("seed/renamed.lino"), String::from(contents))]
            );
        }
    }
}

#[test]
fn legacy_crlf_bundle_stays_readable_without_the_new_encoding_marker() {
    let document = "formal_ai_bundle\r\n  seed_files\r\n    file \"seed/legacy.lino\"\r\n      legacy\r\n        value \"ok\"\r\n  demo_memory\r\n";
    assert_eq!(
        import_full_memory(document).seed_files,
        vec![(
            String::from("seed/legacy.lino"),
            String::from("legacy\n  value \"ok\"")
        )]
    );
}
