use super::*;
use crate::coding::{program_language_by_slug, PROGRAM_LANGUAGES};

/// Test-only enumeration of every CST grammar declared in the seed. The shipped
/// crate only ever looks a single language up (`grammar_metadata`), so this
/// full-table helper lives with the tests instead of in `src/`.
fn grammar_languages() -> Vec<CstGrammar> {
    let tree = parse_lino(PROGRAM_CST_GRAMMARS_LINO);
    grammar_nodes(&tree).map(grammar_from_node).collect()
}

#[test]
fn every_catalog_language_has_cst_metadata() {
    for language in PROGRAM_LANGUAGES {
        assert!(
            grammar_metadata(language.slug).is_some(),
            "`{}` must have CST metadata",
            language.slug
        );
    }
}

#[test]
fn every_cst_metadata_entry_names_a_catalog_language() {
    for grammar in grammar_languages() {
        assert!(
            program_language_by_slug(&grammar.language_slug).is_some(),
            "`{}` metadata names an unknown language",
            grammar.language_slug
        );
    }
}

#[test]
fn every_cst_metadata_entry_uses_the_meta_language_engine() {
    for grammar in grammar_languages() {
        assert_eq!(
            grammar.engine, META_LANGUAGE_ENGINE,
            "`{}` must be validated by meta-language",
            grammar.language_slug
        );
        assert!(
            !grammar.meta_language_label.is_empty(),
            "`{}` must declare a meta-language label",
            grammar.language_slug
        );
    }
}

#[test]
fn javascript_source_parses_through_meta_language() {
    let cst = parse_program_cst(
        "javascript",
        "const numbers = [3, 1, 2];\nconsole.log(numbers.sort((a, b) => a - b));\n",
    )
    .expect("JavaScript source must produce a meta-language CST");

    assert_eq!(cst.engine(), META_LANGUAGE_ENGINE);
    assert!(cst.is_valid(), "{cst:#?}");
    assert!(!cst.has_error);
    let CstEvidence::MetaLanguage {
        syntax_link_count,
        text_preserved,
        ..
    } = cst.evidence;
    assert!(syntax_link_count > 0, "expected real grammar syntax links");
    assert!(text_preserved, "meta-language must round-trip the source");
}

#[test]
fn broken_javascript_fails_meta_language_validation() {
    let cst = parse_program_cst("javascript", "const numbers = [3, 1, 2;\nconsole.log(\n")
        .expect("parsing should still produce a CST record");
    assert!(cst.has_error, "unbalanced JavaScript must report an error");
    assert!(!cst.is_valid());
    assert!(validated_program_cst("javascript", "const numbers = [3, 1, 2;\n").is_none());
}

#[test]
fn formerly_bridged_languages_now_parse_through_meta_language() {
    // TypeScript, Go and Ruby were validated through a direct tree-sitter
    // bridge until meta-language gained grammars for them (upstream
    // meta-language#41/#42/#43). They now go through the same links network.
    let cases = [
        ("typescript", "const numbers: number[] = [3, 1, 2];\n"),
        ("go", "package main\n\nfunc main() {}\n"),
        ("ruby", "puts [3, 1, 2].sort\n"),
    ];
    for (slug, source) in cases {
        let cst = parse_program_cst(slug, source)
            .unwrap_or_else(|| panic!("`{slug}` source must produce a meta-language CST"));
        assert_eq!(cst.engine(), META_LANGUAGE_ENGINE, "{slug}");
        assert!(cst.is_valid(), "{slug}: {cst:#?}");
        let CstEvidence::MetaLanguage {
            syntax_link_count, ..
        } = cst.evidence;
        assert!(
            syntax_link_count > 0,
            "`{slug}` must hit a real meta-language grammar"
        );
    }
}

#[test]
fn meta_language_handles_every_covered_language() {
    for grammar in grammar_languages() {
        assert_eq!(
            grammar.engine, META_LANGUAGE_ENGINE,
            "{}",
            grammar.language_slug
        );
        let snippet = match grammar.language_slug.as_str() {
            "javascript" => "const x = 1;\n",
            "typescript" => "const x: number = 1;\n",
            "python" => "x = 1\n",
            "rust" => "fn main() {}\n",
            "java" | "csharp" => "class A { }\n",
            "c" => "int main(void) { return 0; }\n",
            "cpp" => "int main() { return 0; }\n",
            "go" => "package main\n\nfunc main() {}\n",
            "ruby" => "puts 1\n",
            other => panic!("no snippet for meta-language language `{other}`"),
        };
        let cst = parse_program_cst(&grammar.language_slug, snippet)
            .unwrap_or_else(|| panic!("`{}` must parse", grammar.language_slug));
        assert!(cst.is_valid(), "{}: {cst:#?}", grammar.language_slug);
        let CstEvidence::MetaLanguage {
            syntax_link_count, ..
        } = cst.evidence;
        assert!(
            syntax_link_count > 0,
            "`{}` must hit a real meta-language grammar",
            grammar.language_slug
        );
    }
}
