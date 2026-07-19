//! Document-format conversion through link-foundation/meta-language.
//!
//! formal-ai keeps document structure and cross-format conversion behind the
//! upstream meta-language links network. This module is the local boundary used
//! by natural-language document workflows: it exposes the supported formats,
//! their fidelity profiles, and a small conversion helper over
//! `LinkNetwork::reconstruct_text_as`.

#[cfg(feature = "meta-language")]
use meta_language::{
    canonical_document_format, document_format_profile, docx_package_is_recognized,
    docx_profile_is_recognized, parse_markup_document, pdf_profile_is_recognized,
    render_docx_package, LinkNetwork, ParseConfiguration, CROSS_FORMAT_CONCEPTS, DOCUMENT_FORMATS,
};

/// The engine recorded in traces for document-format CST/concept conversion.
pub const DOCUMENT_FORMAT_ENGINE: &str = "meta_language";

/// Fidelity profile for one document format supported by meta-language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentFormatCapabilities {
    /// Canonical format label (`txt`, `Markdown`, `HTML`, `PDF`, or `DOCX`).
    pub format: String,
    /// Cross-format concepts the target profile represents natively.
    pub native_concepts: Vec<String>,
    /// Declared lossy fallbacks for concepts the target cannot represent.
    pub fallbacks: Vec<(String, String)>,
}

/// Result of reconstructing one document format as another.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentConversion {
    /// Canonical source format label.
    pub source_format: String,
    /// Canonical target format label.
    pub target_format: String,
    /// Text representation rendered by `LinkNetwork::reconstruct_text_as`.
    pub output: String,
    /// Target fidelity profile used for the conversion.
    pub target_capabilities: DocumentFormatCapabilities,
    /// Optional real package bytes when the target format has a package layer.
    ///
    /// For meta-language 0.45 this is populated for `DOCX` with a stored-entry
    /// OPC ZIP package that contains the rendered `word/document.xml` part.
    pub package_bytes: Option<Vec<u8>>,
}

/// Document formats available through the upstream cross-format layer.
#[must_use]
pub const fn supported_document_formats() -> &'static [&'static str] {
    #[cfg(feature = "meta-language")]
    {
        DOCUMENT_FORMATS
    }
    #[cfg(not(feature = "meta-language"))]
    {
        &[]
    }
}

/// Shared formatting concepts considered by the upstream fidelity profiles.
#[must_use]
pub const fn cross_format_document_concepts() -> &'static [&'static str] {
    #[cfg(feature = "meta-language")]
    {
        CROSS_FORMAT_CONCEPTS
    }
    #[cfg(not(feature = "meta-language"))]
    {
        &[]
    }
}

/// Canonicalizes a format alias to the upstream document-format label.
#[must_use]
pub fn canonical_document_format_label(format: &str) -> Option<&'static str> {
    #[cfg(feature = "meta-language")]
    {
        canonical_document_format(format)
    }
    #[cfg(not(feature = "meta-language"))]
    {
        let _ = format;
        None
    }
}

/// Returns the meta-language fidelity profile for a document format.
#[must_use]
pub fn document_format_capabilities(format: &str) -> Option<DocumentFormatCapabilities> {
    #[cfg(not(feature = "meta-language"))]
    {
        let _ = format;
        return None;
    }
    #[cfg(feature = "meta-language")]
    {
        let canonical = canonical_document_format(format)?;
        let profile = document_format_profile(canonical)?;
        let native_concepts = CROSS_FORMAT_CONCEPTS
            .iter()
            .copied()
            .filter(|concept| profile.supports_concept(concept))
            .map(str::to_owned)
            .collect();
        let fallbacks = CROSS_FORMAT_CONCEPTS
            .iter()
            .copied()
            .filter_map(|concept| {
                profile
                    .concept_fallback(concept)
                    .map(|fallback| (concept.to_owned(), fallback.to_owned()))
            })
            .collect();

        Some(DocumentFormatCapabilities {
            format: canonical.to_owned(),
            native_concepts,
            fallbacks,
        })
    }
}

/// Whether text is recognized by meta-language's constrained profile for a format.
///
/// Markdown, HTML, and txt recognition is based on parsing into a non-empty
/// concept-layer document. PDF and DOCX use the explicit profile recognizers
/// exported by meta-language.
#[must_use]
pub fn document_profile_is_recognized(format: &str, text: &str) -> bool {
    #[cfg(not(feature = "meta-language"))]
    {
        let _ = (format, text);
        return false;
    }
    #[cfg(feature = "meta-language")]
    {
        match canonical_document_format(format) {
            Some("PDF") => pdf_profile_is_recognized(text),
            Some("DOCX") => docx_profile_is_recognized(text),
            Some(canonical) => parse_markup_document(canonical, text)
                .is_some_and(|document| !document.blocks.is_empty()),
            None => false,
        }
    }
}

/// Whether package bytes are recognized by the format's package layer.
///
/// meta-language 0.45 exposes an OPC ZIP package profile for DOCX. Other
/// document formats currently have no separate package wrapper in the upstream
/// API, so they return `false`.
#[must_use]
pub fn document_package_is_recognized(format: &str, bytes: &[u8]) -> bool {
    #[cfg(not(feature = "meta-language"))]
    {
        let _ = (format, bytes);
        return false;
    }
    #[cfg(feature = "meta-language")]
    {
        match canonical_document_format(format) {
            Some("DOCX") => docx_package_is_recognized(bytes),
            _ => false,
        }
    }
}

/// Converts a document through meta-language's shared concept layer.
#[must_use]
pub fn convert_document_format(
    source_format: &str,
    target_format: &str,
    source_text: &str,
) -> Option<DocumentConversion> {
    #[cfg(not(feature = "meta-language"))]
    {
        let _ = (source_format, target_format, source_text);
        return None;
    }
    #[cfg(feature = "meta-language")]
    {
        let source = canonical_document_format(source_format)?;
        let target = canonical_document_format(target_format)?;
        let network = LinkNetwork::parse(source_text, source, ParseConfiguration::default());
        let output = network.reconstruct_text_as(target, ParseConfiguration::default());
        let target_capabilities = document_format_capabilities(target)?;
        let package_bytes = package_bytes_for_target(source, target, source_text);

        Some(DocumentConversion {
            source_format: source.to_owned(),
            target_format: target.to_owned(),
            output,
            target_capabilities,
            package_bytes,
        })
    }
}

#[cfg(feature = "meta-language")]
fn package_bytes_for_target(
    source_format: &str,
    target_format: &str,
    source_text: &str,
) -> Option<Vec<u8>> {
    if target_format != "DOCX" {
        return None;
    }
    let document = parse_markup_document(source_format, source_text)?;
    (!document.blocks.is_empty()).then(|| render_docx_package(&document))
}
