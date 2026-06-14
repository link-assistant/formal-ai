### Added
- Recognize document-generation requests ("Сделай мне пдф файл …", "make me a
  PDF/document/report with …") and answer with the universal algorithm's formal
  plan — scope, gather, classify, assemble, export — localized to the prompt
  language, instead of falling through to the unknown response.
- Updated `meta-language` to 0.45.0 (raising the crate `rust-version` to 1.77)
  and exposed its document-format concept layer through the natural-language
  document workflow: TXT, Markdown, HTML, PDF, and DOCX conversion now routes
  through `LinkNetwork::reconstruct_text_as`, reports target fidelity fallbacks,
  and records DOCX package-layer evidence when the upstream OPC profile is
  available.
