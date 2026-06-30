// Worker module 14 of 21. Loaded by ../formal_ai_worker.js.
function codingOracleAnswer(taskSlug, language) {
  const snippet = codingOracleLookup(taskSlug, language);
  if (!snippet) return null;
  const sourceMeta = KNOWLEDGE_SOURCES[snippet.source];
  const content =
    `Here is a minimal ${snippet.languageLabel} program (${snippet.taskSlug.replace(/_/g, " ")}):\n\n` +
    "```" +
    `${snippet.languageSlug}\n${snippet.code}\n` +
    "```" +
    `\n\nOutput:\n` +
    "```text\n" +
    `${snippet.expectedOutput}\n` +
    "```" +
    `\nSource: ${sourceMeta.displayName} (${snippet.sourceUrl}), cached locally as a popular example.`;
  return {
    intent: `write_program_oracle_${snippet.taskSlug}_${snippet.languageSlug}`,
    content,
    confidence: 1.0,
    evidence: [
      `response:write_program:${snippet.taskSlug}:${snippet.languageSlug}:${snippet.source}`,
      `knowledge_source:${snippet.source}`,
      `knowledge_source_url:${snippet.sourceUrl}`,
      "execution_status:not run (cached external snippet)",
      "execution_environment:no compile/run sandbox configured for cached external snippets",
      `program_parameter:language:${snippet.languageSlug}`,
      `program_parameter:task:${snippet.taskSlug}`,
    ],
    steps: undefined,
    trace: undefined,
  };
}

function normalizeProgramPrompt(prompt) {
  return normalizePrompt(String(prompt || "").replace(/c\+\+/gi, " cpp ").replace(/c#/gi, " csharp "));
}

// CJK scripts have no inter-word spaces, so the whitespace-based phrase/token
// matchers never isolate a CJK word. When the expected alias itself contains a
// CJK ideograph we fall back to a substring test (issue #312). Mirrors
// `coding::catalog::contains_cjk` on the Rust side.
function containsCjk(text) {
  return /[㐀-䶿一-鿿豈-﫿぀-ヿ㄀-ㄯ]/.test(
    String(text || ""),
  );
}

// Devanagari (Hindi, …) is written without spaces between words, so the
// whitespace-based phrase/token matchers never isolate a Devanagari word —
// exactly as for CJK above. Mirrors `coding::catalog::contains_devanagari` on
// the Rust side (range U+0900–U+097F): lets a handler split a role's word forms
// by script (Devanagari vs. Han) straight from the seed, so the head-final
// Hindi and Chinese extraction strategies never name a raw word (issue #386).
function containsDevanagari(text) {
  return /[ऀ-ॿ]/.test(String(text || ""));
}

// Whether every character of `text` is ASCII. Mirrors Rust's `str::is_ascii`:
// the calculator-domain signal builder shapes a surface differently by script,
// matching an ASCII code on a word boundary but a non-ASCII surface as a raw
// substring (issue #386).
function isAsciiText(text) {
  return /^[\x00-\x7f]*$/.test(String(text || ""));
}

function containsProgramToken(normalized, token) {
  if (containsCjk(token)) return String(normalized || "").includes(token);
  return String(normalized || "")
    .split(/\s+/)
    .includes(token);
}

function containsProgramPhrase(normalized, phrase) {
  if (containsCjk(phrase)) return normalized.includes(phrase);
  return (
    normalized === phrase ||
    normalized.startsWith(`${phrase} `) ||
    normalized.endsWith(` ${phrase}`) ||
    normalized.includes(` ${phrase} `)
  );
}

function programTaskFromPrompt(normalized) {
  // The request phrasings live in each task's `program_task_<slug>` meaning, not
  // inline on WRITE_PROGRAM_TASKS — read them by slug (issue #386). Declaration
  // order (Object.keys) is the catalog priority order, identical to Rust.
  return (
    Object.keys(WRITE_PROGRAM_TASKS).find((slug) =>
      wordsForMeaning(`program_task_${slug}`).some((alias) =>
        containsProgramPhrase(normalized, alias),
      ),
    ) || null
  );
}

// Issue #386: the function words that introduce an *unknown* implementation
// language ("write a program in <name>", "на языке <name>") are seed data, not
// literals baked into the worker. Mirror src/intent_formalization.rs by sourcing
// the head-initial English/Russian surfaces of the target-preposition and
// "language" noun roles from the lexicon. The WRITE_PROGRAM_LANGUAGES catalog
// scan below already resolves every *known* language across all four supported
// languages; this fallback only reads the bare name trailing the marker, so it
// consults the two head-initial languages whose name follows the marker (the
// head-final Hindi/Chinese surfaces are carried in the seed for coverage but
// place the name before the marker, which this scan does not chase).
const ROLE_IMPLEMENTATION_LANGUAGE_PREPOSITION =
  "implementation_language_preposition";
const ROLE_IMPLEMENTATION_LANGUAGE_NOUN = "implementation_language_noun";

function programLanguageFromPrompt(normalized) {
  const tokens = normalized.split(/\s+/).filter(Boolean);
  // Each language's alias surfaces live in its `program_language_<slug>` meaning,
  // not inline on WRITE_PROGRAM_LANGUAGES — read them by slug (issue #386). Names
  // are single tokens, matched on whitespace boundaries exactly as in Rust.
  for (const slug of Object.keys(WRITE_PROGRAM_LANGUAGES)) {
    const surfaces = wordsForMeaning(`program_language_${slug}`);
    if (surfaces.some((alias) => tokens.includes(alias))) return slug;
  }
  const prepositionSurfaces = wordsForRoleInLanguages(
    ROLE_IMPLEMENTATION_LANGUAGE_PREPOSITION,
    ["en", "ru"],
  );
  const languageNounSurfaces = wordsForRoleInLanguages(
    ROLE_IMPLEMENTATION_LANGUAGE_NOUN,
    ["en", "ru"],
  );
  for (let index = 0; index < tokens.length - 1; index += 1) {
    if (!prepositionSurfaces.includes(tokens[index])) continue;
    if (languageNounSurfaces.includes(tokens[index + 1])) {
      return tokens[index + 2] || null;
    }
    return tokens[index + 1];
  }
  return null;
}

// ---------------------------------------------------------------------------
// Issue #324 R4/R7: the program-modification step as a data-driven Links
// Notation substitution pipeline. This mirrors `src/program_plan.rs` (the
// pipeline) and `src/substitution.rs` (the engine). The rule text below is
// byte-identical to `data/seed/program-plan-rules.lino`; the parity experiment
// (`experiments/issue-324-js-worker.mjs`) keeps the two copies in lockstep.
//
// Adding a new modifier transform is rule data; recognition is constrained by
// the operation slugs declared in that rule data.
// ---------------------------------------------------------------------------

// PROGRAM_PLAN_RULES_LINO is loaded from synced seed/*.lino data during loadSeed().

const TASK_NODE = "request:task";
const MODIFIER_NODE = "request:modifier";

// Issue #386: the FULL multilingual operation vocabulary, embedded inline and
// byte-identical to data/seed/operation-vocabulary.lino (regenerated by
// experiments/issue-386-sync-worker-lexicon.mjs). The JS mirror of
// OperationVocabulary in src/seed/operation_vocabulary.rs: the CODE matches
// canonical concept tokens (write / function / similar_elements / reverse_sort …),
// while the per-language surface phrases live once here in the seed data — never
// a hardcoded per-language word list in code.
// OPERATION_VOCABULARY_LINO is loaded from synced seed/*.lino data during loadSeed().

let cachedOperationVocabulary = null;
// Parse the embedded operation vocabulary into language-pooled triggers
// ({ slug, phrases, combos, inverse? }). Mirrors operation_vocabulary() in
// src/seed/operation_vocabulary.rs; phrases/combos are pooled across every
// language because operationFormMatches (below) is language-agnostic.
function operationVocabulary() {
  if (cachedOperationVocabulary) return cachedOperationVocabulary;
  const root = parseLinoTree(OPERATION_VOCABULARY_LINO);
  const container =
    root.children.find((child) => child.name === "operation_vocabulary") || root;
  const operations = [];
  for (const operationNode of container.children) {
    if (operationNode.name !== "operation") continue;
    const phrases = [];
    const combos = [];
    let inverse = null;
    for (const child of operationNode.children) {
      if (child.name === "inverse") inverse = child.value;
      else if (child.name === "language") {
        for (const form of child.children) {
          if (form.name === "phrase") phrases.push(form.value);
          else if (form.name === "combo") {
            combos.push(
              form.value
                .split("+")
                .map((token) => token.trim())
                .filter(Boolean),
            );
          }
        }
      }
    }
    const operation = { slug: operationNode.value, phrases, combos };
    if (inverse) operation.inverse = inverse;
    operations.push(operation);
  }
  cachedOperationVocabulary = operations;
  return cachedOperationVocabulary;
}

let cachedProgramModifierSlugs = null;
function literalPatternValue(node) {
  return node && node.kind === "literal" ? node.value : null;
}

function programModifierSlugs() {
  if (cachedProgramModifierSlugs) return cachedProgramModifierSlugs;
  const slugs = new Set();
  for (const rule of programPlanRules().rules) {
    for (const condition of rule.conditions || []) {
      const from = literalPatternValue(condition.from);
      const to = literalPatternValue(condition.to);
      if (from === MODIFIER_NODE && to) slugs.add(to);
    }
  }
  cachedProgramModifierSlugs = slugs;
  return slugs;
}

function operationFormMatches(normalized, operation) {
  const source = String(normalized || "");
  return (
    (operation.phrases || []).some((phrase) => source.includes(phrase)) ||
    (operation.combos || []).some((combo) =>
      combo.every((token) => source.includes(String(token || ""))),
    )
  );
}

// Issue #386: every canonical operation token whose phrasing appears in
// `normalized`, in declaration order. Mirrors OperationVocabulary::detect in
// src/seed/operation_vocabulary.rs (substring match, so a native verb still
// matches when punctuation such as the danda `।` is glued to it).
function detectOperations(normalized) {
  const detected = [];
  for (const operation of operationVocabulary()) {
    if (operationFormMatches(normalized, operation)) detected.push(operation.slug);
  }
  return detected;
}

// Issue #386: append canonical English operation tokens to a normalized prompt so
// handlers keep matching canonical concepts while accepting native verbs from
// data/seed/operation-vocabulary.lino. Mirrors
// OperationVocabulary::canonicalized_prompt — additive (never removes text), so
// English prompts are unchanged and multilingual prompts gain boundary-matchable
// tokens (e.g. a Hindi "लिखें।" yields an appended " write").
function canonicalizedPrompt(normalized) {
  const detected = detectOperations(normalized);
  if (!detected.length) return String(normalized || "");
  let out = String(normalized || "");
  for (const canonical of detected) {
    out += ` ${canonical}`;
    const phrase = canonical.replace(/_/g, " ");
    if (phrase !== canonical) out += ` ${phrase}`;
  }
  return out;
}

// Issue #386: the language-independent *meaning* lexicon — the JS mirror of
// `data/seed/meanings.lino` and `src/seed/meanings.rs`. Recognition references
// semantic *roles* (which surface words evidence a program artifact / a program
// modification, in any language), never a hardcoded per-language word list. This
// is an inline copy generated from the canonical seed by
// `scripts/migrate-meaning-seed.rs` (the same convention as
// PROGRAM_PLAN_RULES_LINO) so the worker stays self-contained when no seed has
// been fetched; parity is guarded by `experiments/issue-386-js-cancel-sort.mjs`.
// MEANINGS_LINO is loaded from synced seed/*.lino data during loadSeed().

// Semantic role: a thing a program produces that a later turn can refer back to
// (a result, an output, the program/script/code itself, an ordering).
const ROLE_PROGRAM_ARTIFACT = "program_artifact";
// Semantic role: an operation a follow-up turn can request against the active
// program (sort, reverse, cancel, change, …) — additive or subtractive.
const ROLE_PROGRAM_MODIFICATION = "program_modification";
// Semantic role: a kind of program artifact a user can ask to be authored
// (a program, a script, code, a function, a class) — the noun side of "write a <kind>".
const ROLE_PROGRAM_KIND = "program_kind";
// Semantic role: a verb that requests a program artifact be produced (write,
// create, show, generate, make, build) — the verb side of "write a <kind>".
const ROLE_PROGRAM_REQUEST = "program_request";
// Issue #386 program-synthesis roles — mirror the ROLE_PROGRAM_SYNTHESIS_*
// consts in src/seed/meanings.rs. Their surface words live in
// data/seed/meanings-program-synthesis.lino (loaded into MEANINGS_LINO).
// The subject/domain/action triple gates a synthesis request; signals
// distinguish one task from another; a task's slug is the Python function name.
const ROLE_PROGRAM_SYNTHESIS_SUBJECT = "program_synthesis_subject";
const ROLE_PROGRAM_SYNTHESIS_DOMAIN = "program_synthesis_domain";
const ROLE_PROGRAM_SYNTHESIS_ACTION = "program_synthesis_action";
const ROLE_PROGRAM_SYNTHESIS_SIGNAL = "program_synthesis_signal";
const ROLE_PROGRAM_SYNTHESIS_TASK = "program_synthesis_task";
// Issue #386 conversational-intent roles — mirror the ROLE_CLARIFICATION_REQUEST
// / ROLE_CAPABILITY_QUERY* / ROLE_SELF_FACT_QUERY / ROLE_SELF_INTRODUCTION_REQUEST
// consts in src/seed/meanings.rs. Their surface words live in
// data/seed/meanings-intent.lino (loaded into MEANINGS_LINO); the
// recognizers below ask the lexicon by meaning instead of hardcoding phrases.
const ROLE_CLARIFICATION_REQUEST = "clarification_request";
const ROLE_CAPABILITY_QUERY = "capability_query";
const ROLE_CAPABILITY_QUERY_MORE = "capability_query_more";
const ROLE_SELF_FACT_QUERY = "self_fact_query";
const ROLE_SELF_INTRODUCTION_REQUEST = "self_introduction_request";
// Issue #386 known-facts inventory roles — mirror the ROLE_KNOWLEDGE_INVENTORY_*
// / ROLE_KNOWLEDGE_POSSESSION consts in src/seed/roles.rs. Their surface words
// live in data/seed/meanings-intent.lino (the shared `fact` noun plus the
// knowledge_inventory_probe / assistant_knowing / knowledge_inventory_query
// meanings, loaded into MEANINGS_LINO); isKnownFactQuery composes these
// roles instead of hardcoding per-language phrase arrays.
const ROLE_KNOWLEDGE_INVENTORY_NOUN = "knowledge_inventory_noun";
const ROLE_KNOWLEDGE_INVENTORY_INTERROGATIVE = "knowledge_inventory_interrogative";
const ROLE_KNOWLEDGE_POSSESSION = "knowledge_possession";
const ROLE_KNOWLEDGE_INVENTORY_PHRASE = "knowledge_inventory_phrase";

// Issue #386 conversation-summary roles — mirror the
// ROLE_CONVERSATION_SUMMARY_DIRECTIVE / ROLE_CONVERSATION_REFERENCE /
// ROLE_CONVERSATION_SUMMARY_PHRASE / ROLE_CONVERSATION_SUMMARY_COURTESY consts
// in src/seed/roles.rs. Their per-language surface words live once in the
// loaded MEANINGS_LINO (data/seed/meanings-intent.lino); the
// isSummarizePrompt recogniser composes these roles instead of hardcoding
// per-language phrase / regex arrays.
const ROLE_CONVERSATION_SUMMARY_DIRECTIVE = "conversation_summary_directive";
const ROLE_CONVERSATION_REFERENCE = "conversation_reference";
const ROLE_CONVERSATION_SUMMARY_PHRASE = "conversation_summary_phrase";
const ROLE_CONVERSATION_SUMMARY_COURTESY = "conversation_summary_courtesy";
// Issue #529 previous-message recall role — mirrors
// ROLE_CONVERSATION_RECALL_PREVIOUS_MESSAGE in src/seed/roles/intent.rs. Its
// bare surface phrases ("what was written in the previous message", "что было
// написано в прошлом сообщении", …) live in data/seed/meanings-conversation.lino
// (loaded into MEANINGS_LINO); tryRecallPreviousMessage composes this role via
// lexiconMentionsRole instead of matching per-language phrases in JS.
const ROLE_CONVERSATION_RECALL_PREVIOUS_MESSAGE = "conversation_recall_previous_message";
// Issue #529 whole-memory write roles — mirror ROLE_MEMORY_* in
// src/seed/roles/intent.rs. Their surfaces live in
// data/seed/meanings-conversation.lino (loaded into MEANINGS_LINO) and drive the
// Turing-complete read+write memory primitive in tryMemoryWrite: a
// Slot::Prefix append directive ("remember …"/"запомни …"/"याद रखो …"/"记住…"),
// a memory scope phrase that distinguishes a memory rewrite from a plain coding
// request ("in memory"/"в памяти"/"स्मृति में"/"在记忆中"), the substitution
// verb ("replace"/"замени"/"बदलो"/"把"), and the old→new connector
// ("with"/"на"/"की जगह"/"换成").
const ROLE_MEMORY_APPEND_DIRECTIVE = "memory_append_directive";
const ROLE_MEMORY_SCOPE = "memory_scope";
const ROLE_MEMORY_SUBSTITUTION_CONNECTOR = "memory_substitution_connector";
const ROLE_MEMORY_SUBSTITUTION_DIRECTIVE = "memory_substitution_directive";
// Issue #386 conversation-opener role — mirrors ROLE_CONVERSATION_TOPIC_OPENER
// in src/seed/roles.rs. Its slot-marked surface words live in
// data/seed/meanings-conversation.lino (loaded into MEANINGS_LINO);
// conversationTopic asks the lexicon for these forms by meaning instead of
// hardcoding a per-language opener array.
const ROLE_CONVERSATION_TOPIC_OPENER = "conversation_topic_opener";
// Issue #386 how-cluster roles — mirror the ROLE_MECHANISM_INQUIRY /
// ROLE_PROCEDURAL_REQUEST consts in src/seed/meanings.rs. Their slot-marked
// surface words live in data/seed/meanings-how.lino (loaded into MEANINGS_LINO
// above); extractHowItWorksSubject / extractProceduralHowToTask ask the lexicon
// for these forms by meaning instead of hardcoding per-language phrase arrays.
const ROLE_MECHANISM_INQUIRY = "mechanism_inquiry";
const ROLE_PROCEDURAL_REQUEST = "procedural_request";
const ROLE_PROCEDURAL_REQUEST_ELIDED_LEAD = "procedural_request_elided_lead";
const ROLE_PROCEDURAL_ACTION_VERB = "procedural_action_verb";
// Issue #386 procedural-cluster cleanup roles — mirror the
// ROLE_PROCEDURAL_TASK_MODIFIER / ROLE_COMMON_TYPO consts in src/seed/roles.rs.
// cleanProceduralFragment / correctCommonProceduralTypos ask the lexicon for
// these forms by meaning instead of hardcoding per-language modifier and typo
// arrays.
const ROLE_PROCEDURAL_TASK_MODIFIER = "procedural_task_modifier";
// Issue #444: a follow-up that asks for the concrete steps of an active
// procedure ("give me specific instructions", "step by step", "дай конкретные
// инструкции", "具体步骤", …). Its surfaces live in data/seed/meanings-how.lino
// (loaded into MEANINGS_LINO); tryProceduralHowToFollowup asks the lexicon
// for this role by meaning instead of hardcoding per-language phrase arrays.
// Mirrors ROLE_PROCEDURAL_ELABORATION in src/seed/roles/intent.rs.
const ROLE_PROCEDURAL_ELABORATION = "procedural_elaboration";
const ROLE_COMMON_TYPO = "common_typo";
// Issue #386 mechanism-subject cleanup roles — mirror the
// ROLE_MECHANISM_PREDICATE / ROLE_DETAIL_MODIFIER / ROLE_NON_REFERENTIAL_SUBJECT
// consts in src/seed/roles.rs. stripMechanismTail / cleanMechanismSubject ask
// the lexicon for these forms by meaning instead of hardcoding per-language
// predicate, modifier, and pronoun arrays.
const ROLE_MECHANISM_PREDICATE = "mechanism_predicate";
const ROLE_DETAIL_MODIFIER = "detail_modifier";
const ROLE_NON_REFERENTIAL_SUBJECT = "non_referential_subject";

// Slot marker (U+2026 …) carried inside a surface word's text to mark the open
// subject/task position. Mirrors the `split_once('…')` slot derivation on
// WordForm in src/seed/meanings.rs (issue #386).
const SLOT_MARKER = "…";

// Build a surface form { text, action, description, slot, before, after } from a
// raw surface word, its optional canonical action, and its self-describing note.
// The slot classification and the literal text on either side are derived from
// the position of the … marker, exactly as WordForm::slot/before_slot/after_slot
// do in src/seed/meanings.rs: no marker = "bare"; trailing marker = "prefix";
// leading marker = "suffix"; a marker with text on both sides = "circumfix".
function makeWordForm(text, action, description) {
  const idx = text.indexOf(SLOT_MARKER);
  if (idx === -1) {
    return {
      text,
      action: action || "",
      description: description || "",
      slot: "bare",
      before: text,
      after: "",
    };
  }
  const before = text.slice(0, idx);
  const after = text.slice(idx + SLOT_MARKER.length);
  let slot = "bare";
  if (before && after) slot = "circumfix";
  else if (before) slot = "prefix";
  else if (after) slot = "suffix";
  return {
    text,
    action: action || "",
    description: description || "",
    slot,
    before,
    after,
  };
}

let cachedMeaningLexicon = null;
// Parse the embedded lexicon once. Each meaning keeps the semantic roles it
// plays, the surface words (across every language) that evidence it, and the
// richer per-form data (action + self-describing note + derived slot) in
// declaration order. Mirrors parse_lexicon in src/seed/meanings.rs.
function meaningLexicon() {
  if (cachedMeaningLexicon) return cachedMeaningLexicon;
  const root = parseLinoTree(MEANINGS_LINO);
  // The lexicon is split across several files (program, units, …), each
  // wrapping its records under a top-level `meanings` node. Concatenated, the
  // document therefore holds one-or-more `meanings` containers; collect the
  // records from every one. If none is present the records sit at the document
  // root (kept for robustness). Mirrors parse_lexicon in src/seed/meanings.rs.
  const containers = root.children.filter((child) => child.name === "meanings");
  const sources = containers.length ? containers : [root];
  const meanings = [];
  for (const container of sources) {
    for (const node of container.children) {
      if (node.name === "meanings") continue;
      const slug = meaningSlug(node);
      if (!slug) continue;
      const roles = [];
      const definedBy = [];
      const words = [];
      const wordForms = [];
      // Per-language word groups, so a handler can partition a role's vocabulary
      // by language (e.g. head-initial vs. head-final translation verbs) without
      // losing the language tag the flat `words` list drops. Mirrors the
      // `lexemes` field on Meaning in src/seed/meanings.rs (issue #386).
      const lexemes = [];
      if (node.name !== "meaning" && node.value) {
        definedBy.push(...node.value.split(/\s+/).filter(Boolean));
      }
      for (const child of node.children) {
        if (child.name === "role") roles.push(child.value);
        else if (child.name === "defined_by" || child.name === "defined-by") {
          definedBy.push(child.value);
        }
        else if (child.name === "lexeme") {
          const lexemeWords = [];
          for (const lexWord of child.children) {
            if (lexWord.name !== "word" && lexWord.name !== "surface") continue;
            const text = surfaceText(lexWord);
            words.push(text);
            lexemeWords.push(text);
            let action = "";
            let description = "";
            for (const attr of lexWord.children) {
              if (attr.name === "action") action = attr.value;
              else if (attr.name === "description") description = attr.value;
            }
            wordForms.push(
              makeWordForm(text, action, description || generatedWordDescription(slug, text)),
            );
          }
          lexemes.push({ language: lexemeLanguage(child), words: lexemeWords });
        } else if (child.name === "surface") {
          const text = surfaceText(child);
          words.push(text);
          let action = "";
          let description = "";
          for (const attr of child.children) {
            if (attr.name === "action") action = attr.value;
            else if (attr.name === "description") description = attr.value;
          }
          wordForms.push(
            makeWordForm(text, action, description || generatedWordDescription(slug, text)),
          );
          lexemes.push({ language: childValue(child, "language"), words: [text] });
        }
      }
      meanings.push({ slug, roles, definedBy, words, wordForms, lexemes });
    }
  }
  cachedMeaningLexicon = meanings;
  return cachedMeaningLexicon;
}

function childValue(node, name) {
  const child = (node.children || []).find((candidate) => candidate.name === name);
  return child ? child.value : "";
}

function meaningSlug(node) {
  return node.name === "meaning" ? node.value : node.name;
}

function lexemeLanguage(node) {
  return childValue(node, "language") || node.value || "";
}

function decodeCodepoints(raw) {
  return String(raw || "")
    .split(/\s+/)
    .filter(Boolean)
    .map((part) => {
      const trimmed = part.trim();
      const radix = /^0x/i.test(trimmed) ? 16 : 10;
      const parsed = parseInt(radix === 16 ? trimmed.slice(2) : trimmed, radix);
      return String.fromCodePoint(Number.isFinite(parsed) ? parsed : 0);
    })
    .join("");
}

function surfaceText(node) {
  // Issue #398: surfaces now carry a readable `text` child; the legacy
  // `codepoints` byte-dump is still decoded for backward compatibility.
  const text = childValue(node, "text");
  if (text) return text;
  const codepoints = childValue(node, "codepoints");
  return codepoints ? decodeCodepoints(codepoints) : node.value;
}

function generatedWordDescription(parentMeaning, text) {
  return text ? `${text} denotes ${parentMeaning}` : parentMeaning;
}

// Does `expected` (a surface word or multi-word phrase) appear in `normalized`?
// CJK surfaces match as substrings; everything else matches on whitespace
// boundaries (whole token or whole phrase). Mirrors surface_present in
// src/seed/meanings.rs — stricter than a raw substring, so a short surface like
// "hp" never matches inside "php" and a phrase like "each step" matches only on
// word boundaries.
function surfacePresent(normalized, expected) {
  if (!expected) return false;
  const text = String(normalized || "");
  if (containsCjk(expected)) return text.includes(expected);
  return (
    text === expected ||
    text.startsWith(`${expected} `) ||
    text.endsWith(` ${expected}`) ||
    text.includes(` ${expected} `)
  );
}

// Is any surface word (any language) of `meaning` evidenced in `normalized`?
// Mirrors Meaning::evidenced_in in src/seed/meanings.rs.
function meaningEvidencedIn(meaning, normalized) {
  return meaning.words.some((word) => surfacePresent(normalized, word));
}

// Does `normalized` mention any surface word of any meaning carrying `role`?
// Mirrors Lexicon::mentions_role in src/seed/meanings.rs — the boundary-aware,
// phrase-capable surface_present contract (CJK substring vs. whitespace token).
function lexiconMentionsRole(role, normalized) {
  return meaningsWithRole(role).some((meaning) => meaningEvidencedIn(meaning, normalized));
}

// Like lexiconMentionsRole but ignores a meaning's script-independent *value
// surfaces* — word forms with no alphabetic character, such as the operator
// symbol "+" or the numeral "10". Those forms exist so the arithmetic normalizer
// can read a meaning's machine value; they are not spelled words, so operator-
// *word* detection skips them and a bare "+" is recognised as an operator symbol
// by the symbol scan, not double-counted here. Mirrors Lexicon::mentions_role_spelled
// in src/seed/meanings.rs.
function lexiconMentionsRoleSpelled(role, normalized) {
  return meaningsWithRole(role).some((meaning) =>
    meaning.words
      .filter((word) => /\p{Alphabetic}/u.test(word))
      .some((word) => surfacePresent(normalized, word)),
  );
}

// The first meaning (declaration order) carrying `role` that is evidenced in
// `normalized`, or null. Declaration order encodes priority. Mirrors
// Lexicon::first_role_match in src/seed/meanings.rs.
function firstRoleMatch(role, normalized) {
  return meaningsWithRole(role).find((meaning) => meaningEvidencedIn(meaning, normalized)) || null;
}

// Issue #386 calendar roles — mirror the ROLE_CALENDAR_* consts in
// src/seed/meanings.rs. Their surface words live in
// data/seed/meanings-calendar.lino (loaded into MEANINGS_LINO). The
// calendar handler uses its own boundary-aware matcher (containsCalendarTerm)
// rather than lexiconMentionsRole, so these come with dedicated accessors.
const ROLE_CALENDAR_WEEKDAY = "calendar_weekday";
const ROLE_CALENDAR_DIRECTION_NEXT = "calendar_direction_next";
const ROLE_CALENDAR_DIRECTION_PREVIOUS = "calendar_direction_previous";
const ROLE_CALENDAR_TODAY = "calendar_today";
const ROLE_CALENDAR_DAY_REFERENCE = "calendar_day_reference";
const ROLE_CALENDAR_QUESTION = "calendar_question";

// Issue #404: new calendar create/schedule roles (mirror src/solver_handlers/calendar.rs
// + data/seed/meanings-calendar.lino). Use the same wordsForRole + containsCalendarTerm
// pattern as the existing weekday helpers.
const ROLE_CALENDAR_SCHEDULE_ACTION = "calendar_schedule_action";
const ROLE_CALENDAR_EVENT = "calendar_event";
const ROLE_CALENDAR_TIME = "calendar_time";
const ROLE_CALENDAR_TIMEZONE_ALIAS = "calendar_timezone_alias";
// Issue #435: relative-date words ("завтра"/"tomorrow"/"послезавтра"/…) that
// anchor a scheduled event to a day offset from today. Mirrors
// ROLE_CALENDAR_RELATIVE_DATE in src/solver_handlers/calendar.rs.
const ROLE_CALENDAR_RELATIVE_DATE = "calendar_relative_date";

// Every meaning carrying `role`, in lexicon (declaration) order. Mirrors
// Lexicon::meanings_with_role in src/seed/meanings.rs.
function meaningsWithRole(role) {
  return meaningLexicon().filter((meaning) => meaning.roles.includes(role));
}

// Every slot-marked surface form carrying `role`, flattened across all meanings
// and languages in declaration order. Recognition code buckets the result by
// form.slot ("bare" / "prefix" / "suffix" / "circumfix") to derive its
// affix-matching strategy from the data. Mirrors Lexicon::role_word_forms in
// src/seed/meanings.rs (issue #386).
function roleWordForms(role) {
  const forms = [];
  for (const meaning of meaningsWithRole(role)) {
    for (const form of meaning.wordForms) forms.push(form);
  }
  return forms;
}

// The meaning identified by `slug`, or null. Mirrors Lexicon::meaning in
// src/seed/meanings.rs.
function findMeaning(slug) {
  return meaningLexicon().find((meaning) => meaning.slug === slug) || null;
}

function wordInLanguage(meaning, language) {
  const lexeme = (meaning.lexemes || []).find((candidate) => candidate.language === language);
  return lexeme && lexeme.words.length > 0 ? lexeme.words[0] : null;
}

function dimensionLabelForUnit(unit) {
  for (const slug of unit.definedBy || []) {
    const dimension = findMeaning(slug);
    if (!dimension || !dimension.roles.includes(ROLE_PHYSICAL_DIMENSION)) continue;
    return wordInLanguage(dimension, "en") || dimension.words[0] || null;
  }
  return null;
}

function isAlphabeticCharacter(ch) {
  return !!ch && /\p{Alphabetic}/u.test(ch);
}

// Mirrors contains_unit_word in src/solver_handler_units.rs: ASCII and CJK unit
// surfaces need alphabetic boundaries so "gram" inside "program" and "克" inside
// an unrelated CJK compound do not count, while inflected alphabetic scripts can
// still match suffix forms such as Russian "килограмм" in "килограмме".
function containsUnitWord(normalized, unit) {
  if (!unit) return false;
  const text = String(normalized || "");
  if (!/[^\x00-\x7F]/.test(unit) || containsCjk(unit)) {
    let searchFrom = 0;
    while (searchFrom <= text.length) {
      const start = text.indexOf(unit, searchFrom);
      if (start < 0) return false;
      const end = start + unit.length;
      const before = Array.from(text.slice(0, start)).pop() || "";
      const after = Array.from(text.slice(end))[0] || "";
      if (!isAlphabeticCharacter(before) && !isAlphabeticCharacter(after)) {
        return true;
      }
      searchFrom = end;
    }
    return false;
  }
  return text.includes(unit);
}

function detectIncompatibleUnitPair(normalized) {
  const found = [];
  for (const unit of meaningsWithRole(ROLE_MEASUREMENT_UNIT)) {
    const dimension = dimensionLabelForUnit(unit);
    if (!dimension || found.some((entry) => entry.dimension === dimension)) {
      continue;
    }
    const matched = unit.words.find((word) => containsUnitWord(normalized, word));
    if (matched) {
      found.push({ unit: matched, dimension });
    }
  }
  if (found.length < 2) return null;
  return [found[0], found[1]];
}

function tryIncompatibleUnits(prompt, normalized) {
  const pair = detectIncompatibleUnitPair(normalized);
  if (!pair) return null;
  const [a, b] = pair;
  const content =
    `${a.unit} measures ${a.dimension}; ${b.unit} measures ${b.dimension}. ` +
    "These are different physical dimensions and cannot be converted into each other. " +
    "The incompatibility is recorded as a `unit_incompatibility` link in the network.";
  return {
    intent: "unit_incompatibility",
    content,
    confidence: 1.0,
    evidence: [
      `unit_incompatibility:${a.unit}:${a.dimension}:vs:${b.unit}:${b.dimension}`,
      "response:unit_incompatibility",
    ],
    trace: [
      `unit_incompatibility:${a.unit}:${a.dimension} vs ${b.unit}:${b.dimension}`,
    ],
    steps: [
      {
        step: "unit_incompatibility",
        detail: `${a.unit}:${a.dimension} vs ${b.unit}:${b.dimension}`,
      },
    ],
  };
}

// Distinct surface words (across all languages) carried by the meaning `slug`,
// or an empty array when no such meaning exists. The coding catalog reads each
// language's and task's alias surfaces from the `program_language_<slug>` /
// `program_task_<slug>` meanings by slug (issue #386), so the matchers name only
// the concept while the words stay self-describing seed data. Mirrors the Rust
// `coding::catalog::alias_surfaces` helper.
function wordsForMeaning(slug) {
  return findMeaning(slug)?.words || [];
}

// Distinct surface words (across all languages) for `role`, declaration order.
// Mirrors Lexicon::words_for_role in src/seed/meanings.rs.
function wordsForRole(role) {
  const seen = new Set();
  const words = [];
  for (const meaning of meaningsWithRole(role)) {
    for (const word of meaning.words) {
      if (!seen.has(word)) {
        seen.add(word);
        words.push(word);
      }
    }
  }
  return words;
}

// Issue #529 memory-write surface utilities. They mirror the boundary-aware
// span helpers in src/solver_handlers/conversation_memory/memory_write.rs so the
// browser worker recognises append + substitution requests from the same seed
// vocabulary the Rust runtime uses.

// Leftmost boundary-aware [start, end) span of `surface` within `haystack`, or
// null. Mirrors surface_span: a CJK surface matches as a substring (no
// inter-word spaces); a space-delimited surface must be a whole whitespace token
// or phrase bounded by the string ends or by spaces.
function memorySurfaceSpan(haystack, surface) {
  if (!surface) return null;
  if (containsCjk(surface)) {
    const start = haystack.indexOf(surface);
    return start === -1 ? null : [start, start + surface.length];
  }
  let search = 0;
  while (search <= haystack.length) {
    const rel = haystack.slice(search).indexOf(surface);
    if (rel === -1) return null;
    const start = search + rel;
    const end = start + surface.length;
    const leftOk = start === 0 || haystack[start - 1] === " ";
    const rightOk = end === haystack.length || haystack[end] === " ";
    if (leftOk && rightOk) return [start, end];
    search = start + 1;
  }
  return null;
}

// Leftmost boundary-aware span of any surface word of `role` in `haystack`. On a
// tie at the same start offset the longest surface wins. Mirrors
// best_surface_span in memory_write.rs.
function bestMemorySurfaceSpan(haystack, role) {
  let best = null;
  for (const surface of wordsForRole(role)) {
    const span = memorySurfaceSpan(haystack, surface);
    if (!span) continue;
    if (
      !best ||
      span[0] < best[0] ||
      (span[0] === best[0] && span[1] > best[1])
    ) {
      best = span;
    }
  }
  return best;
}

function collapseMemoryWs(text) {
  return String(text || "")
    .split(/\s+/)
    .filter(Boolean)
    .join(" ");
}

// Remove the leftmost occurrence of any surface word of `role`, returning the
// remaining text with surrounding whitespace collapsed. Mirrors
// strip_first_surface in memory_write.rs.
function stripFirstMemorySurface(haystack, role) {
  const span = bestMemorySurfaceSpan(haystack, role);
  if (!span) return null;
  return collapseMemoryWs(`${haystack.slice(0, span[0])} ${haystack.slice(span[1])}`);
}

// Split `span` once on the leftmost surface word of `role`, returning the
// [before, after] operands around the connector. Mirrors split_once_surface.
function splitOnceMemorySurface(span, role) {
  const found = bestMemorySurfaceSpan(span, role);
  if (!found) return null;
  return [span.slice(0, found[0]), span.slice(found[1])];
}

// Trim wrapping punctuation/quotes and collapse whitespace, returning null when
// nothing remains. Mirrors clean_memory_write_text in memory_write.rs.
function cleanMemoryWriteText(raw) {
  const stripped = String(raw || "").replace(
    /^[\s`"':\-_.,?!()]+|[\s`"':\-_.,?!()]+$/gu,
    "",
  );
  const text = collapseMemoryWs(stripped);
  return text ? text : null;
}

// Issue #386 translation-command role — mirrors ROLE_TRANSLATION_ACTION in
// src/seed/roles.rs. The per-language command verbs (translate, переведи/
// перевести/опиши, अनुवाद, 翻译/翻譯) live once in the loaded MEANINGS_LINO
// (data/seed/meanings-translation.lino) under the `translate` meaning; the gate
// and source-inferencer reference the concept, not the raw words.
const ROLE_TRANSLATION_ACTION = "translation_action";

// Distinct surface words for `role` limited to `languages`, declaration order.
// Mirrors Lexicon::words_for_role_in_languages in src/seed/meanings.rs (#386).
function wordsForRoleInLanguages(role, languages) {
  const out = [];
  for (const meaning of meaningsWithRole(role)) {
    for (const lexeme of meaning.lexemes) {
      if (!languages.includes(lexeme.language)) continue;
      for (const word of lexeme.words) {
        if (!out.includes(word)) out.push(word);
      }
    }
  }
  return out;
}

// The first language in `priority` whose surface word for `role` appears in
// `normalized` (raw substring), or null when none is present. Answers "which
// language did the user issue this command in?". Mirrors
// Lexicon::first_role_language in src/seed/meanings.rs (#386).
function firstRoleLanguage(role, normalized, priority) {
  for (const lang of priority) {
    const present = meaningsWithRole(role).some((meaning) =>
      meaning.lexemes.some(
        (lexeme) =>
          lexeme.language === lang &&
          lexeme.words.some((word) => normalized.includes(word)),
      ),
    );
    if (present) return lang;
  }
  return null;
}

// The first surface word `meaning` lexicalises in `language`, or null. Mirrors
// Meaning::word_in in src/seed/meanings.rs (issue #386).
function wordIn(meaning, language) {
  for (const lexeme of meaning.lexemes) {
    if (lexeme.language !== language) continue;
    for (const word of lexeme.words) {
      if (word) return word;
    }
  }
  return null;
}

// Translate `surface` from `source` to `target` through the meaning carrying
// `role` that lexicalises it: the first such meaning (declaration order) whose
// `source` lexeme lists `surface`, returning its first `target` form, or null.
// Mirrors Lexicon::role_surface_translation in src/seed/meanings.rs (issue #386).
function roleSurfaceTranslation(role, source, target, surface) {
  for (const meaning of meaningsWithRole(role)) {
    const lists = meaning.lexemes.some(
      (lexeme) => lexeme.language === source && lexeme.words.includes(surface),
    );
    if (lists) return wordIn(meaning, target);
  }
  return null;
}

// Does any meaning carrying `role` lexicalise `surface` in `language`? Mirrors
// Lexicon::role_lists_surface in src/seed/meanings.rs (issue #386).
function roleListsSurface(role, language, surface) {
  return meaningsWithRole(role).some((meaning) =>
    meaning.lexemes.some(
      (lexeme) => lexeme.language === language && lexeme.words.includes(surface),
    ),
  );
}

// Like roleSurfaceTranslation but the `source` form must also carry the per-form
// grammatical `action` tag (e.g. "genitive"). The worker keeps per-form action on
// the flat wordForms array (no language) and raw strings on each lexeme, so a
// form qualifies when the source lexeme lists `surface` and some wordForm has the
// same text and `action`. Mirrors Lexicon::role_action_surface_translation in
// src/seed/meanings.rs (issue #386).
function roleActionSurfaceTranslation(role, action, source, target, surface) {
  for (const meaning of meaningsWithRole(role)) {
    const listsInSource = meaning.lexemes.some(
      (lexeme) => lexeme.language === source && lexeme.words.includes(surface),
    );
    const tagged = meaning.wordForms.some(
      (form) => form.text === surface && form.action === action,
    );
    if (listsInSource && tagged) return wordIn(meaning, target);
  }
  return null;
}

// Issue #386 software-project follow-up roles — mirror the
// ROLE_SOFTWARE_FOLLOWUP_* consts in src/seed/meanings.rs. Their surface words
// live in data/seed/meanings-software-project.lino (loaded into MEANINGS_LINO
// above). detectSoftwareFollowUp matches them as raw substrings (not whole
// tokens), so they come with a dedicated accessor.
const ROLE_SOFTWARE_FOLLOWUP_VERIFICATION = "software_followup_verification";
const ROLE_SOFTWARE_FOLLOWUP_EXECUTION = "software_followup_execution";
const ROLE_SOFTWARE_FOLLOWUP_DEMONSTRATION = "software_followup_demonstration";

// Issue #386 behavior-rules-list roles — mirror the ROLE_RULE_LISTING_* consts
// in src/seed/roles.rs. Their surface words live in
// data/seed/meanings-behavior-rules.lino (loaded into MEANINGS_LINO).
// isBehaviorRulesList ANDs the three compositional dimensions within one
// language (subject + request + scope) and ORs the standalone phrase role.
// isBehaviorRulesCount reuses the subject/scope roles plus count-specific
// request/scope roles, matching every surface as a raw substring exactly like
// the Rust recogniser.
const ROLE_RULE_LISTING_SUBJECT = "rule_listing_subject";
const ROLE_RULE_LISTING_REQUEST = "rule_listing_request";
const ROLE_RULE_LISTING_SCOPE = "rule_listing_scope";
const ROLE_RULE_LISTING_PHRASE = "rule_listing_phrase";
const ROLE_RULE_COUNT_REQUEST = "rule_count_request";
const ROLE_RULE_COUNT_SCOPE = "rule_count_scope";
const ROLE_RULE_BRIEF_REQUEST = "rule_brief_request";

// Does `normalized` contain any surface word of any meaning carrying `role`,
// matched as a raw substring? Unlike lexiconMentionsRole (whole-token, via
// containsProgramToken), follow-up markers are multi-word phrases ("run the
// tests", "show me"), so a token-boundary match would never find them. Mirrors
// the raw `.contains()` in follow_up_kind
// (src/solver_handlers/software_project_followup.rs).
function lexiconMentionsRoleSubstring(role, normalized) {
  return meaningLexicon().some(
    (meaning) =>
      meaning.roles.includes(role) &&
      meaning.words.some((word) => word && normalized.includes(word)),
  );
}

// Does any surface form `meaning` lexicalises in one of `languages` appear in
// `normalized` as a raw substring (String.includes)? The language-restricted,
// raw-substring sibling of meaningEvidencedIn. Mirrors
// Meaning::mentions_in_languages_raw in src/seed/meanings.rs (issue #386).
function meaningMentionsInLanguagesRaw(meaning, normalized, languages) {
  return meaning.lexemes.some(
    (lexeme) =>
      languages.includes(lexeme.language) &&
      lexeme.words.some((word) => word && normalized.includes(word)),
  );
}

// The first meaning (declaration order) carrying `role` whose `languages`
// surface forms occur in `normalized` as a raw substring, or null. Declaration
// order encodes priority. Mirrors Lexicon::first_role_match_in_languages_raw in
// src/seed/meanings.rs (issue #386).
function firstRoleMatchInLanguagesRaw(role, normalized, languages) {
  return (
    meaningsWithRole(role).find((meaning) =>
      meaningMentionsInLanguagesRaw(meaning, normalized, languages),
    ) || null
  );
}

// Does any meaning carrying `role` mention one of its `languages` surface forms
// in `normalized` as a raw substring? Mirrors
// Lexicon::mentions_role_in_languages_raw in src/seed/meanings.rs (issue #386).
function mentionsRoleInLanguagesRaw(role, normalized, languages) {
  return meaningsWithRole(role).some((meaning) =>
    meaningMentionsInLanguagesRaw(meaning, normalized, languages),
  );
}

function detectedProgramModifiers(normalized) {
  const slugs = [];
  const validModifiers = programModifierSlugs();
  for (const operation of operationVocabulary()) {
    if (validModifiers.has(operation.slug) && operationFormMatches(normalized, operation)) {
      slugs.push(operation.slug);
    }
  }
  return slugs;
}

// --- Substitution engine (mirror of src/substitution.rs) -------------------

function unescapeLinoValue(value) {
  let out = "";
  for (let index = 0; index < value.length; index += 1) {
    const ch = value[index];
    if (ch === "\\" && index + 1 < value.length) {
      const next = value[index + 1];
      if (next === "n") {
        out += "\n";
        index += 1;
        continue;
      }
      if (next === '"' || next === "\\") {
        out += next;
        index += 1;
        continue;
      }
    }
    out += ch;
  }
  return out;
}

function unescapeSingleLinoValue(value) {
  let out = "";
  for (let index = 0; index < value.length; index += 1) {
    const ch = value[index];
    if (ch === "\\" && index + 1 < value.length) {
      const next = value[index + 1];
      if (next === "n") {
        out += "\n";
        index += 1;
        continue;
      }
      if (next === "\\") {
        out += "\\";
        index += 1;
        continue;
      }
      if (next === "x" && value.slice(index + 2, index + 4) === "27") {
        out += "'";
        index += 3;
        continue;
      }
    }
    out += ch;
  }
  return out;
}

function decodeRawReference(raw) {
  const value = String(raw || "").trim();
  if (value === "unformalized-raw" || value === "codepoints") return "";
  if (value.startsWith("unformalized-raw ")) {
    return decodeCodepoints(value.slice("unformalized-raw ".length));
  }
  if (value.startsWith("codepoints ")) {
    return decodeCodepoints(value.slice("codepoints ".length));
  }
  return value;
}

function stripLinoComment(line) {
  let inDoubleQuote = false;
  let escaped = false;
  let previousWasSpace = true;
  for (let index = 0; index < line.length; index += 1) {
    const ch = line[index];
    if (inDoubleQuote) {
      if (escaped) {
        escaped = false;
      } else if (ch === "\\") {
        escaped = true;
      } else if (ch === '"') {
        inDoubleQuote = false;
      }
      continue;
    }
    if (ch === '"') {
      inDoubleQuote = true;
      previousWasSpace = false;
      continue;
    }
    if (ch === "#" && previousWasSpace) return line.slice(0, index);
    previousWasSpace = /\s/.test(ch);
  }
  return line;
}

function parseLinoValue(raw) {
  const trimmed = raw.trim();
  if (trimmed.length >= 2 && trimmed.startsWith('"') && trimmed.endsWith('"')) {
    return unescapeLinoValue(trimmed.slice(1, -1));
  }
  if (trimmed.length >= 2 && trimmed.startsWith("'") && trimmed.endsWith("'")) {
    return unescapeSingleLinoValue(trimmed.slice(1, -1));
  }
  return decodeRawReference(trimmed);
}

function parseLinoTree(text) {
  const root = { name: "", value: "", depth: -1, children: [] };
  const stack = [root];
  for (const line of text.split("\n")) {
    const stripped = stripLinoComment(line);
    if (!stripped.trim()) continue;
    const indent = stripped.length - stripped.trimStart().length;
    const depth = indent / 2;
    const rest = stripped.trim();
    const colonIndex = rest.indexOf(":");
    const whitespaceIndex = rest.search(/\s/);
    let name = "";
    let value = "";
    if (colonIndex !== -1 && (whitespaceIndex === -1 || colonIndex < whitespaceIndex)) {
      name = rest.slice(0, colonIndex).trim();
      value = parseLinoValue(rest.slice(colonIndex + 1));
    } else if (whitespaceIndex === -1) {
      name = rest;
    } else {
      name = rest.slice(0, whitespaceIndex);
      value = parseLinoValue(rest.slice(whitespaceIndex + 1));
    }
    const node = { name, value, depth, children: [] };
    while (stack.length && stack[stack.length - 1].depth >= depth) stack.pop();
    stack[stack.length - 1].children.push(node);
    stack.push(node);
  }
  return root;
}

function parsePatternNode(text) {
  if (!text) throw new Error("pattern node is empty");
  if (text.startsWith("$")) return { kind: "variable", variable: text.slice(1) };
  const dollar = text.indexOf("$");
  if (dollar !== -1) {
    return { kind: "prefix", prefix: text.slice(0, dollar), variable: text.slice(dollar + 1) };
  }
  return { kind: "literal", value: text };
}

function parseLinkPattern(text) {
  const index = text.indexOf("->");
  if (index === -1) throw new Error(`expected \`from -> to\`, got \`${text}\``);
  return {
    from: parsePatternNode(text.slice(0, index).trim()),
    to: parsePatternNode(text.slice(index + 2).trim()),
  };
}

function parseCrudEvent(value) {
  const map = {
    manual: "manual",
    apply: "manual",
    create: "create",
    created: "create",
    read: "read",
    select: "read",
    query: "read",
    update: "update",
    updated: "update",
    delete: "delete",
    deleted: "delete",
  };
  const key = String(value).trim().toLowerCase();
  if (!map[key]) throw new Error(`invalid CRUD event: ${value}`);
  return map[key];
}

function parseSubstitutionRule(node) {
  const rule = { id: node.value, order: 0, events: [], conditions: [], actions: [] };
  for (const child of node.children) {
    switch (child.name) {
      case "order": {
        const parsed = parseInt(child.value, 10);
        rule.order = Number.isNaN(parsed) ? 0 : parsed;
        break;
      }
      case "event":
        rule.events.push(parseCrudEvent(child.value));
        break;
      case "when":
        rule.conditions.push(parseLinkPattern(child.value));
        break;
      case "replace": {
        const add = child.children
          .filter((grandchild) => grandchild.name === "with")
          .map((grandchild) => parseLinkPattern(grandchild.value));
        rule.actions.push({ remove: parseLinkPattern(child.value), add });
        break;
      }
      default:
        break;
    }
  }
  return rule;
}

function parseSubstitutionRules(text) {
  const tree = parseLinoTree(text.trim());
  const root = tree.children[0];
  if (!root || root.name !== "substitution_rules") {
    throw new Error("not a substitution_rules document");
  }
  const idNode = root.children.find((child) => child.name === "id");
  const id = idNode ? idNode.value : "";
  const rules = root.children
    .filter((child) => child.name === "rule")
    .map(parseSubstitutionRule);
  rules.sort((left, right) =>
    left.order - right.order ||
    (left.id < right.id ? -1 : left.id > right.id ? 1 : 0),
  );
  return { id, rules };
}

const LINK_KEY_SEPARATOR = "\u0000";
const linkKey = (link) => `${link.from}${LINK_KEY_SEPARATOR}${link.to}`;
const linkFromKey = (key) => {
  const [from, to] = key.split(LINK_KEY_SEPARATOR);
  return { from, to };
};

function sortedLinksFromSet(linkSet) {
  return Array.from(linkSet, linkFromKey).sort((left, right) =>
    left.from < right.from
      ? -1
      : left.from > right.from
        ? 1
        : left.to < right.to
          ? -1
          : left.to > right.to
            ? 1
            : 0,
  );
}

function bindVariable(bindings, variable, value) {
  if (Object.prototype.hasOwnProperty.call(bindings, variable)) {
    return bindings[variable] === value;
  }
  bindings[variable] = value;
  return true;
}

function nodeMatches(pattern, value, bindings) {
  if (pattern.kind === "literal") return pattern.value === value;
  if (pattern.kind === "variable") return bindVariable(bindings, pattern.variable, value);
  if (!value.startsWith(pattern.prefix)) return false;
  return bindVariable(bindings, pattern.variable, value.slice(pattern.prefix.length));
}

function patternMatchesLink(pattern, link, bindings) {
  return (
    nodeMatches(pattern.from, link.from, bindings) &&
    nodeMatches(pattern.to, link.to, bindings)
  );
}

function instantiateNode(node, bindings) {
  if (node.kind === "literal") return node.value;
  if (node.kind === "variable") {
    return Object.prototype.hasOwnProperty.call(bindings, node.variable)
      ? bindings[node.variable]
      : null;
  }
  const value = bindings[node.variable];
  return value === undefined ? null : node.prefix + value;
}

function instantiatePattern(pattern, bindings) {
  const from = instantiateNode(pattern.from, bindings);
  const to = instantiateNode(pattern.to, bindings);
  if (from === null || to === null) return null;
  return { from, to };
}

function findBindings(links, patterns, index, bindings) {
  if (index >= patterns.length) return bindings;
  const pattern = patterns[index];
  for (const link of links) {
    const candidate = Object.assign({}, bindings);
    if (patternMatchesLink(pattern, link, candidate)) {
      const found = findBindings(links, patterns, index + 1, candidate);
      if (found) return found;
    }
  }
  return null;
}
