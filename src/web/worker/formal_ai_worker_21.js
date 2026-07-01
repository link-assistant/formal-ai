// Relative-meta-logic mirror of `src/relative_meta_logic.rs` and
// `src/statement_verification.rs`. Issue #535 asks us to weigh each statement
// with a relative probability: assume it true, raise it with trusted original
// first sources, ignore reposts. The web worker replays the exact same
// deterministic offline plan into its evidence log so the website matches the
// Rust `FormalAiEngine` byte-for-byte.
//
// This module is the issue #535 surface: it holds both the relative-meta-logic
// statement weigher and the document-verification handler that calls it
// (`tryDocumentOriginalityCheck`, appended below), so worker_19 stays under the
// 1500-line ceiling enforced by scripts/check-file-size.rs. Function
// declarations hoist across the concatenated worker bundle, so worker_20's
// dispatcher reaches the handler regardless of module order.
const RML_ASSUMED_TRUE_PRIOR = 0.6;
const RML_TRUSTED_SOURCE_POLICY = [
  { slug: "original_first_party", weight: 1.0 },
  { slug: "original_journalism", weight: 0.85 },
  { slug: "independent_corroboration", weight: 0.5 },
  { slug: "unoriginal", weight: 0.0 },
];
// Sentence terminators across every script the solver recognises, mirroring
// `SENTENCE_TERMINATORS` in `src/statement_verification.rs`.
const RML_SENTENCE_TERMINATORS = new Set([
  ".",
  "!",
  "?",
  "。",
  "！",
  "？",
  "।",
  "॥",
  "؟",
  "\n",
]);
const RML_MIN_STATEMENT_WORDS = 3;
const RML_MIN_STATEMENT_CHARS = 6;

function rmlDecimal(value) {
  // Match Rust's fixed 6-decimal grid so identical inputs serialise identically.
  return Number(value).toFixed(6);
}

function documentOriginalityFullTextSample(prompt) {
  for (const rawLine of String(prompt || "").split(/\r?\n/u)) {
    const line = rawLine.trim();
    for (const prefix of ["Text excerpt:", "Text sample:", "OCR text:"]) {
      if (!line.startsWith(prefix)) continue;
      const sample = line.slice(prefix.length).trim();
      if (sample) return sample;
    }
  }
  return "";
}

function extractVerificationStatements(sample) {
  const statements = [];
  let current = "";
  const pushStatement = () => {
    const trimmed = current.trim();
    current = "";
    if (!trimmed) return;
    const wordCount = trimmed.split(/\s+/u).filter(Boolean).length;
    const charCount = Array.from(trimmed).filter(
      (character) => !/\s/u.test(character),
    ).length;
    if (wordCount < RML_MIN_STATEMENT_WORDS && charCount < RML_MIN_STATEMENT_CHARS) {
      return;
    }
    statements.push(trimmed);
  };
  for (const character of Array.from(String(sample || ""))) {
    if (RML_SENTENCE_TERMINATORS.has(character)) {
      pushStatement();
    } else {
      current += character;
    }
  }
  pushStatement();
  return statements;
}

function verificationGroundingQuery(statement) {
  const condensed = String(statement || "")
    .split(/\s+/u)
    .filter(Boolean)
    .join(" ");
  return `"${condensed}" fact check source`;
}

// The deterministic offline assessment of a single statement: assumed-true
// prior with no evidence collected yet. Mirrors `StatementAssessment::assess`
// with an empty evidence slice.
function verificationAssessmentTrace() {
  const prior = RML_ASSUMED_TRUE_PRIOR;
  const support = 0;
  const contradiction = 0;
  const raised = 1 - (1 - prior) * (1 - support);
  const posterior = raised * (1 - contradiction);
  return (
    `prior=${rmlDecimal(prior)} support=${rmlDecimal(support)} ` +
    `contradiction=${rmlDecimal(contradiction)} posterior=${rmlDecimal(posterior)} ignored=0`
  );
}

// Append the relative-meta-logic verification plan into `evidence`, mirroring
// `log_statement_verification` in `src/solver_handlers/document_originality.rs`.
function pushStatementVerificationEvidence(prompt, language, evidence) {
  evidence.push(
    `relative_meta_logic:assumed_prior:${rmlDecimal(RML_ASSUMED_TRUE_PRIOR)}`,
  );
  for (const tier of RML_TRUSTED_SOURCE_POLICY) {
    evidence.push(
      `relative_meta_logic:trusted_source_tier:${tier.slug}:weight=${rmlDecimal(tier.weight)}`,
    );
  }
  evidence.push("relative_meta_logic:ignored_source_tier:unoriginal");

  const sample = documentOriginalityFullTextSample(prompt);
  if (!sample) return;
  const statements = extractVerificationStatements(sample);
  evidence.push(`statement_verification:statement_count:${statements.length}`);
  for (const statement of statements) {
    const query = verificationGroundingQuery(statement);
    evidence.push(`statement_verification:statement:${statement}`);
    evidence.push(`statement_verification:query:${query}`);
    evidence.push(`web_search:request:${query}`);
    evidence.push("web_search:query_kind:document_originality_check");
    evidence.push(`statement_verification:assessment:${verificationAssessmentTrace()}`);
  }
}

// --- Document-verification handler (issue #535) -----------------------------
// The originality/authenticity/fact-check handler lives here alongside the
// relative-meta-logic statement weigher it calls, so worker_19 stays under the
// 1500-line worker ceiling enforced by scripts/check-file-size.rs. Function
// declarations hoist across the concatenated worker bundle, so worker_20's
// dispatcher reaches tryDocumentOriginalityCheck regardless of module order.
function extractDocumentOriginalityAttachmentNames(prompt) {
  const names = [];
  let inSection = false;
  for (const rawLine of String(prompt || "").split(/\r?\n/u)) {
    const line = rawLine.trim();
    if (/^attached files:$/iu.test(line)) {
      inSection = true;
      continue;
    }
    if (!inSection || !line) continue;
    if (
      line.startsWith("OCR text:") ||
      line.startsWith("Text excerpt:") ||
      line.startsWith("Text sample:") ||
      line.startsWith("Text omitted:")
    ) {
      continue;
    }
    const match = line.match(/^\d+\.\s+(.+?)\s+\([^)]*\)$/u);
    if (match && match[1]) names.push(match[1].trim());
  }
  return names.filter(Boolean);
}

function hasDocumentOriginalityTextSample(prompt) {
  return String(prompt || "")
    .split(/\r?\n/u)
    .some((rawLine) => {
      const line = rawLine.trim();
      return (
        line.startsWith("OCR text:") ||
        line.startsWith("Text excerpt:") ||
        line.startsWith("Text sample:")
      );
    });
}

function documentOriginalityTextSample(prompt) {
  for (const rawLine of String(prompt || "").split(/\r?\n/u)) {
    const line = rawLine.trim();
    for (const prefix of ["Text excerpt:", "Text sample:", "OCR text:"]) {
      if (!line.startsWith(prefix)) continue;
      const sample = line
        .slice(prefix.length)
        .trim()
        .split(/\s+/u)
        .filter(Boolean)
        .slice(0, 14)
        .join(" ");
      if (sample) return sample;
    }
  }
  return "";
}

function documentOriginalityQuery(prompt, attachments) {
  const sample = documentOriginalityTextSample(prompt);
  if (sample) return `"${sample}" plagiarism originality`;
  if (attachments.length > 0) {
    return `${attachments[0]} plagiarism originality uniqueness`;
  }
  return "document plagiarism originality uniqueness";
}

function documentOriginalityContent(language, attachments, samplePresent) {
  const target = attachments.length > 0 ? attachments.join(", ") : "provided text";
  const templateIntent = samplePresent
    ? "document_originality_check_sample_present"
    : "document_originality_check_sample_missing";
  return answerFor(templateIntent, language).split("{target}").join(target);
}

function tryDocumentOriginalityCheck(prompt, language) {
  const normalized = normalizePrompt(prompt);
  const attachments = extractDocumentOriginalityAttachmentNames(prompt);
  const hasAction = lexiconMentionsRole(
    ROLE_DOCUMENT_ORIGINALITY_CHECK_ACTION,
    normalized,
  ) || lexiconMentionsRoleSubstring(
    ROLE_DOCUMENT_ORIGINALITY_CHECK_ACTION,
    normalized,
  );
  const hasSubject =
    lexiconMentionsRole(ROLE_DOCUMENT_ORIGINALITY_SUBJECT, normalized) ||
    lexiconMentionsRoleSubstring(
      ROLE_DOCUMENT_ORIGINALITY_SUBJECT,
      normalized,
    );
  const hasDocument =
    attachments.length > 0 ||
    lexiconMentionsRole(ROLE_DOCUMENT_ORIGINALITY_DOCUMENT, normalized) ||
    lexiconMentionsRoleSubstring(ROLE_DOCUMENT_ORIGINALITY_DOCUMENT, normalized);

  if (!(hasAction && hasSubject && hasDocument)) return null;

  const query = documentOriginalityQuery(prompt, attachments);
  const samplePresent = hasDocumentOriginalityTextSample(prompt);
  const evidence = [
    `language:${language || ""}`,
    `document_originality_check:request:${query}`,
  ];
  for (const attachment of attachments) {
    evidence.push(`document_originality_check:attachment:${attachment}`);
    evidence.push(`read_local_file:request:${attachment}`);
  }
  if (samplePresent) evidence.push("document_originality_check:text_sample:present");
  evidence.push(`web_search:request:${query}`);
  if (language) evidence.push(`web_search:language:${language}`);
  for (const provider of WEB_SEARCH_PROVIDERS) {
    evidence.push(`web_search:provider:${provider.id}`);
  }
  evidence.push(`web_search:combined:rrf:k=${webSearchRrfK()}`);
  evidence.push("web_search:query_kind:document_originality_check");

  pushStatementVerificationEvidence(prompt, language, evidence);

  return {
    intent: "document_originality_check",
    content: documentOriginalityContent(language, attachments, samplePresent),
    confidence: 0.84,
    evidence,
    query,
    attachments,
    formalizedObject: attachments.length > 0 ? attachments.join(", ") : "provided text",
  };
}
