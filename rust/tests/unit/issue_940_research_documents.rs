use formal_ai::research_documents::{
    ArtifactError, ArtifactFormat, ResearchDocument, ResearchReference, ResearchStatement,
};

fn report(text: &str) -> ResearchDocument {
    ResearchDocument {
        title: "Research".into(),
        language: "en".into(),
        statements: vec![ResearchStatement {
            text: text.into(),
            source_ids: vec!["s1".into()],
        }],
        references: vec![ResearchReference {
            id: "s1".into(),
            title: "Source".into(),
            url: "https://example.org".into(),
            captured_at: "2026-09-30".into(),
        }],
    }
}

#[test]
fn missing_or_unknown_evidence_cannot_be_rendered() {
    let mut document = report("A supported statement.");
    document.statements[0].source_ids = vec!["missing".into()];
    assert_eq!(
        document.render(ArtifactFormat::Markdown, "research"),
        Err(ArtifactError::MissingEvidence)
    );
    document.statements[0].source_ids.clear();
    assert_eq!(
        document.render(ArtifactFormat::Markdown, "research"),
        Err(ArtifactError::MissingEvidence)
    );
}

#[test]
fn filenames_cannot_escape_or_inject_download_headers() {
    for stem in ["../research", "a/b", "a\\b", "a\r\nx-header", "", "."] {
        assert_eq!(
            report("Fact").render(ArtifactFormat::Markdown, stem),
            Err(ArtifactError::UnsafeFilename)
        );
    }
}

#[test]
fn localized_markdown_keeps_text_and_provenance() {
    for text in ["Исследование", "अनुसंधान", "研究"] {
        let artifact = report(text)
            .render(ArtifactFormat::Markdown, "report")
            .unwrap();
        let content = String::from_utf8(artifact.bytes).unwrap();
        assert!(content.contains(text));
        assert!(content.contains("https://example.org"));
        assert!(content.contains("2026-09-30"));
        assert_eq!(artifact.filename, "report.md");
    }
}

#[test]
fn pdf_refuses_font_and_page_overflow_instead_of_silent_loss() {
    assert_eq!(
        report("研究").render(ArtifactFormat::Pdf, "report"),
        Err(ArtifactError::PdfRequiresUnicodeFonts)
    );
    assert_eq!(
        report(&"x".repeat(81)).render(ArtifactFormat::Pdf, "report"),
        Err(ArtifactError::PdfRequiresPagination)
    );
    let mut wide_title = report("Fact");
    wide_title.title = "W".repeat(21);
    assert_eq!(
        wide_title.render(ArtifactFormat::Pdf, "report"),
        Err(ArtifactError::PdfRequiresPagination)
    );
}

#[cfg(feature = "meta-language")]
#[test]
fn upstream_containers_carry_researched_content() {
    for format in [ArtifactFormat::Pdf, ArtifactFormat::Docx] {
        let artifact = report("A researched fact.")
            .render(format, "report")
            .unwrap();
        let label = if format == ArtifactFormat::Pdf {
            "PDF"
        } else {
            "DOCX"
        };
        assert!(formal_ai::document_formats::document_package_is_recognized(
            label,
            &artifact.bytes
        ));
        assert!(
            artifact
                .bytes
                .windows(b"researched fact".len())
                .any(|window| window == b"researched fact")
        );
        assert!(
            artifact
                .bytes
                .starts_with(if format == ArtifactFormat::Pdf {
                    b"%PDF"
                } else {
                    b"PK\x03\x04"
                })
        );
    }
}

#[cfg(feature = "meta-language")]
#[test]
fn docx_keeps_unicode_and_escapes_markup() {
    let artifact = report("研究 <tag> & Исследование")
        .render(ArtifactFormat::Docx, "report")
        .unwrap();
    let content = String::from_utf8_lossy(&artifact.bytes);
    assert!(content.contains("研究"));
    assert!(content.contains("Исследование"));
    assert!(content.contains("&lt;tag&gt;"));
}
