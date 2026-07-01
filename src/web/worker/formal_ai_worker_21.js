// Relative-meta-logic mirror of `src/relative_meta_logic.rs` and
// `src/statement_verification.rs`. Issue #535 asks us to weigh each statement
// with a relative probability: assume it true, raise it with trusted original
// first sources, ignore reposts. The web worker replays the exact same
// deterministic offline plan into its evidence log so the website matches the
// Rust `FormalAiEngine` byte-for-byte.
//
// This lives in its own module (rather than inside the document-originality
// handler in worker_19) so each worker file stays under the 1500-line ceiling
// enforced by scripts/check-file-size.rs. The handler in worker_19 calls
// `pushStatementVerificationEvidence`, which is defined here and loaded before
// any message is processed.
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
