// Worker module 10 of 21. Loaded by ../formal_ai_worker.js.
function pushDefinitionFragment(fragments, language, summary, source, sourceKind) {
  const cleanSummary = String(summary || "").trim();
  if (!cleanSummary) return;
  const duplicate = fragments.some(
    (fragment) =>
      fragment.language === language &&
      normalizeDefinitionFact(fragment.summary) === normalizeDefinitionFact(cleanSummary),
  );
  if (duplicate) return;
  fragments.push({
    language: String(language || "en"),
    summary: cleanSummary,
    source: String(source || "").trim(),
    sourceKind: String(sourceKind || "").trim(),
  });
}

function definitionFragments(record) {
  const fragments = [];
  pushDefinitionFragment(
    fragments,
    inferredSourceLanguage(record && record.source),
    record && record.summary,
    record && record.source,
    record && record.sourceKind,
  );
  for (const localized of Array.isArray(record && record.localized) ? record.localized : []) {
    pushDefinitionFragment(
      fragments,
      localized && localized.language,
      localized && localized.summary,
      localized && localized.source,
      localized && localized.sourceKind,
    );
  }
  return fragments;
}

function sourceLanguages(fragments) {
  const languages = [];
  for (const fragment of fragments) {
    if (!languages.includes(fragment.language)) languages.push(fragment.language);
  }
  return languages;
}

function sourceUrls(fragments) {
  const sources = [];
  for (const fragment of fragments) {
    if (!fragment.source || sources.includes(fragment.source)) continue;
    sources.push(fragment.source);
  }
  return sources;
}

function splitDefinitionSentences(summary) {
  const sentences = [];
  let current = "";
  for (const character of String(summary || "")) {
    current += character;
    if ([".", "!", "?", "।", "。"].includes(character)) {
      const sentence = current.trim();
      if (sentence) sentences.push(sentence);
      current = "";
    }
  }
  const tail = current.trim();
  if (tail) sentences.push(tail);
  return sentences;
}

function mergedDefinitionFacts(fragments) {
  const facts = [];
  const seen = new Set();
  for (const fragment of fragments) {
    for (const sentence of splitDefinitionSentences(fragment.summary)) {
      const key = normalizeDefinitionFact(sentence);
      if (!key || seen.has(key)) continue;
      seen.add(key);
      facts.push({ language: fragment.language, text: sentence });
    }
  }
  return facts;
}

function uniqueSourceFragments(fragments) {
  const unique = [];
  const seen = new Set();
  for (const fragment of fragments) {
    if (!fragment.source) continue;
    const key = `${fragment.language}\n${fragment.source}`;
    if (seen.has(key)) continue;
    seen.add(key);
    unique.push(fragment);
  }
  return unique;
}

function renderDefinitionMerge(record, fragments, facts) {
  const english = localizedConceptFor(record, "en");
  const displayTerm = (english && english.term) || record.term;
  const anchor = record.wikidata ? ` [${record.wikidata}]` : "";
  const lines = [
    `Merged definition of ${displayTerm}${anchor}`,
    `Source languages: ${sourceLanguages(fragments).join(", ")}`,
    "",
    "Facts:",
  ];
  for (const fact of facts) {
    lines.push(`- [${fact.language}] ${fact.text}`);
  }
  lines.push("Sources:");
  for (const fragment of uniqueSourceFragments(fragments)) {
    lines.push(
      `- [${fragment.language}] ${renderSourceLink(fragment.source)} (${fragment.sourceKind})`,
    );
  }
  return lines.join("\n");
}

function tryDefinitionMerge(prompt, options) {
  const opts = options || {};
  const term = extractDefinitionMergeTerm(prompt, Boolean(opts.allowPlainConcept));
  if (!term) return null;
  const evidence = [`definition_merge:request:${term}`];
  if (opts.allowPlainConcept) evidence.push("definition_merge:mode:auto");
  const lookup = lookupConceptQuery({ term, context: null });
  if (!lookup) return null;
  const record = lookup.record;
  const fragments = definitionFragments(record);
  if (fragments.length === 0) return null;
  evidence.push(`definition_merge:hit:${record.slug}`);
  if (record.wikidata) evidence.push(`wikidata:${record.wikidata}`);
  for (const language of sourceLanguages(fragments)) {
    evidence.push(`definition_merge:language:${language}`);
  }
  for (const source of sourceUrls(fragments)) {
    evidence.push(`source:${humanizeUrl(source)}`);
  }
  const facts = mergedDefinitionFacts(fragments);
  evidence.push(`definition_merge:facts:${facts.length}`);
  return {
    intent: "definition_merge",
    content: renderDefinitionMerge(record, fragments, facts),
    confidence: 0.9,
    evidence,
  };
}

// Known person name corrections for typo suggestions. Each entry maps a
// canonical name to a list of common misspellings (all lowercase).
const KNOWN_PERSON_VARIANTS = [
  { canonical: "Elon Musk", variants: ["elon musk", "elon mask", "elon muск"] },
  { canonical: "Donald Trump", variants: ["donald trump", "donald tramp", "donald tromp"] },
  { canonical: "Joe Biden", variants: ["joe biden", "joe bidan", "joe bidon"] },
  { canonical: "Barack Obama", variants: ["barack obama", "barak obama", "barrack obama"] },
  { canonical: "Vladimir Putin", variants: ["vladimir putin", "vladimir puting", "vladmir putin"] },
  { canonical: "Albert Einstein", variants: ["albert einstein", "albert einstien", "albert enstien"] },
  { canonical: "Isaac Newton", variants: ["isaac newton", "isaak newton", "issac newton"] },
  { canonical: "Nikola Tesla", variants: ["nikola tesla", "nicolas tesla", "nikolai tesla"] },
];

function editDistance(a, b) {
  const left = Array.from(String(a || ""));
  const right = Array.from(String(b || ""));
  const m = left.length, n = right.length;
  const dp = Array.from({ length: m + 1 }, (_, i) =>
    Array.from({ length: n + 1 }, (_, j) => (i === 0 ? j : j === 0 ? i : 0))
  );
  for (let i = 1; i <= m; i++) {
    for (let j = 1; j <= n; j++) {
      dp[i][j] = left[i - 1] === right[j - 1]
        ? dp[i - 1][j - 1]
        : 1 + Math.min(dp[i - 1][j - 1], dp[i - 1][j], dp[i][j - 1]);
      if (
        i > 1 &&
        j > 1 &&
        left[i - 1] === right[j - 2] &&
        left[i - 2] === right[j - 1]
      ) {
        dp[i][j] = Math.min(dp[i][j], dp[i - 2][j - 2] + 1);
      }
    }
  }
  return dp[m][n];
}

function isCloseTokenTypo(actual, expected) {
  const left = String(actual || "").toLowerCase();
  const right = String(expected || "").toLowerCase();
  const leftLength = Array.from(left).length;
  const rightLength = Array.from(right).length;
  return Math.min(leftLength, rightLength) >= 4 && editDistance(left, right) === 1;
}

function leadingTokenSpans(value, limit) {
  const text = String(value || "");
  const spans = [];
  const pattern = /\S+/gu;
  let match;
  while ((match = pattern.exec(text)) !== null && spans.length < limit) {
    spans.push({
      start: match.index,
      end: match.index + match[0].length,
      text: match[0],
    });
  }
  return spans;
}

function fuzzyPrefixMatch(value, prefix) {
  const words = String(prefix || "").trim().split(/\s+/u).filter(Boolean);
  if (words.length === 0) return null;
  const spans = leadingTokenSpans(value, words.length);
  if (spans.length !== words.length) return null;
  let typoCount = 0;
  for (let i = 0; i < words.length; i += 1) {
    const actual = spans[i].text;
    const expected = words[i];
    if (actual.toLowerCase() === expected.toLowerCase()) continue;
    if (!isCloseTokenTypo(actual, expected)) return null;
    typoCount += 1;
  }
  if (typoCount !== 1) return null;
  const end = spans[spans.length - 1].end;
  return {
    typoCount,
    end,
    interpretation: {
      original: String(value || "").slice(0, end),
      corrected: String(prefix || "").trim(),
    },
  };
}

function stripKnownPrefix(value, prefixes) {
  const text = String(value || "");
  const lower = text.toLowerCase();
  for (const prefix of prefixes) {
    if (lower.startsWith(prefix)) {
      return { value: text.slice(prefix.length).trimStart(), interpretation: null };
    }
  }
  const matches = prefixes
    .map((prefix) => fuzzyPrefixMatch(text, prefix))
    .filter(Boolean)
    .sort((left, right) =>
      left.typoCount - right.typoCount || right.end - left.end,
    );
  const best = matches[0];
  if (!best) return null;
  const next = matches[1];
  if (next && next.typoCount === best.typoCount && next.end === best.end) {
    return null;
  }
  return {
    value: text.slice(best.end).trimStart(),
    interpretation: best.interpretation,
  };
}

function suggestNameCorrection(term) {
  const lower = term.toLowerCase();
  for (const { canonical, variants } of KNOWN_PERSON_VARIANTS) {
    if (variants.includes(lower)) return canonical;
  }
  for (const { canonical, variants } of KNOWN_PERSON_VARIANTS) {
    const canonicalLower = canonical.toLowerCase();
    if (
      variants.some((v) => editDistance(lower, v) === 1) ||
      editDistance(lower, canonicalLower) === 1
    ) {
      return canonical;
    }
  }
  return null;
}

function isWhoIsPrompt(normalized) {
  // "who is …" detection reasons over the who_question meaning: a language
  // whose marker leads the name occupies the who_question_lead prefix slot
  // (English who is …, Russian кто такой …), while one whose marker trails it
  // occupies the who_question_tail suffix slot (Hindi … कौन है, Chinese …是谁).
  return (
    prefixLiterals(ROLE_WHO_QUESTION_LEAD).some((lead) => normalized.startsWith(lead)) ||
    suffixLiterals(ROLE_WHO_QUESTION_TAIL).some((tail) => normalized.endsWith(tail))
  );
}

function tryWhoIsQuestion(prompt) {
  const normalized = prompt.toLowerCase().trim();
  if (!isWhoIsPrompt(normalized)) return null;
  const query = extractConceptQuery(prompt);
  if (!query) return null;
  const term = query.term;
  const suggestion = suggestNameCorrection(term);
  const content = suggestion
    ? `I don't have a Links Notation fact for "${term}" yet. Did you mean "${suggestion}"? Add a fact or rule in Links Notation and run the request again.`
    : `I don't have a Links Notation fact for "${term}" yet. Add a fact or rule in Links Notation and run the request again.`;
  return {
    intent: "who_is_question",
    content,
    confidence: 0.5,
    evidence: [`concept_lookup:miss:${term}`, "response:who_is_question"],
  };
}

// Issue #513 (visible fix for #511): recognize a request to run a shell/terminal
// command. The detection rules are mirrored in the Rust solver
// (`src/solver_terminal.rs`, `try_terminal_command`) so both engines stay at
// parity. A recognized command returns an `agent_suggestion` intent that names
// the detected command, explains agent mode, and — when agent mode is off —
// offers to switch it on and grant the `shell` capability.
// Issue #513: the terminal-command trigger vocabulary is loaded from the
// synced terminal-commands.lino seed during loadSeed(). This is the JS mirror
// of TerminalCommandVocabulary in src/seed/terminal_commands.rs: the
// per-language surface phrases / verbs / shell tokens live once in seed data,
// never as hardcoded per-language word lists in worker code.

let cachedTerminalCommandVocabulary = null;
// Parse the loaded terminal-command vocabulary into pooled trigger lists.
// Mirrors terminal_command_vocabulary() in src/seed/terminal_commands.rs;
// phrases/verbs are pooled across every language because detection is
// language-agnostic.
function terminalCommandVocabulary() {
  if (cachedTerminalCommandVocabulary) return cachedTerminalCommandVocabulary;
  const root = parseLinoTree(TERMINAL_COMMANDS_LINO);
  const container =
    root.children.find((child) => child.name === "terminal_commands") || root;
  const vocab = {
    terminalPhrases: [],
    runVerbs: new Set(),
    cjkRunVerbs: [],
    shellTokens: new Set(),
  };
  const languageValues = (group, childName) => {
    const out = [];
    for (const lang of group.children) {
      if (lang.name !== "language") continue;
      for (const child of lang.children) {
        if (child.name === childName) out.push(child.value);
      }
    }
    return out;
  };
  const directValues = (group, childName) =>
    group.children.filter((c) => c.name === childName).map((c) => c.value);
  for (const group of container.children) {
    if (group.name === "terminal_phrases") {
      vocab.terminalPhrases = languageValues(group, "phrase");
    } else if (group.name === "run_verbs") {
      vocab.runVerbs = new Set(languageValues(group, "verb"));
    } else if (group.name === "cjk_run_verbs") {
      vocab.cjkRunVerbs = directValues(group, "verb");
    } else if (group.name === "shell_tokens") {
      vocab.shellTokens = new Set(directValues(group, "token"));
    }
  }
  cachedTerminalCommandVocabulary = vocab;
  return cachedTerminalCommandVocabulary;
}

function extractBacktickCommand(prompt) {
  const first = prompt.indexOf("`");
  if (first < 0) return null;
  let start = first;
  while (start < prompt.length && prompt[start] === "`") start += 1;
  let end = start;
  while (end < prompt.length && prompt[end] !== "`") end += 1;
  if (end <= start) return null;
  const command = prompt.slice(start, end).trim();
  return command || null;
}

function leadingShellCommand(prompt) {
  const trimmed = prompt.trim().replace(/^`+|`+$/g, "").trim();
  const first = trimmed.split(/\s+/)[0] || "";
  const normalized = (first.match(/^[A-Za-z0-9_-]+/) || [""])[0].toLowerCase();
  return terminalCommandVocabulary().shellTokens.has(normalized) ? trimmed : null;
}

function detectTerminalCommand(prompt) {
  const vocab = terminalCommandVocabulary();
  const lower = prompt.toLowerCase();
  const hasPhrase = vocab.terminalPhrases.some((p) => lower.includes(p));
  const tokens = lower.split(/[^\p{L}\p{N}_]+/u).filter(Boolean);
  const tokenSet = new Set(tokens);
  const hasVerb =
    [...vocab.runVerbs].some((v) => tokenSet.has(v)) ||
    vocab.cjkRunVerbs.some((v) => lower.includes(v));
  const backtick = extractBacktickCommand(prompt);
  const leading = leadingShellCommand(prompt);
  if (backtick && (hasVerb || hasPhrase)) return backtick;
  if (hasPhrase && hasVerb) return backtick || leading;
  if (leading) return leading;
  return null;
}

// The natural-language prose lives in data/seed/multilingual-responses.lino
// under the `agent_suggestion` (Chat mode) and `agent_suggestion_active` (Agent
// mode on) intents, with a `{command}` placeholder. This mirror only looks the
// template up via answerFor() and fills in the detected command, so no
// per-language wording is hardcoded in the worker. Parity with the Rust solver
// (src/solver_terminal.rs, terminal_body) is kept by both engines reading the
// same seed intent.
function terminalCommandBody(command, language, agentModeOn) {
  const intent = agentModeOn ? "agent_suggestion_active" : "agent_suggestion";
  return answerFor(intent, language).split("{command}").join(command);
}

function tryTerminalCommand(prompt, language, preferences) {
  const command = detectTerminalCommand(prompt);
  if (!command) return null;
  const agentModeOn = Boolean(preferences && preferences.agentMode);
  return {
    intent: "agent_suggestion",
    content: terminalCommandBody(command, language, agentModeOn),
    confidence: 0.6,
    evidence: [
      `terminal:command:${command}`,
      "terminal:agent_suggestion:shell",
      "response:agent_suggestion",
    ],
  };
}

// Wikipedia REST summary endpoint per language. Browser-friendly: CORS is
// enabled by Wikimedia for these summary endpoints, so the worker can fetch
// without a proxy from GitHub Pages.
const WIKIPEDIA_HOSTS = {
  en: "https://en.wikipedia.org/api/rest_v1/page/summary",
  ru: "https://ru.wikipedia.org/api/rest_v1/page/summary",
  hi: "https://hi.wikipedia.org/api/rest_v1/page/summary",
  zh: "https://zh.wikipedia.org/api/rest_v1/page/summary",
};

// Wikipedia full-text page search endpoint per language (CORS-enabled). Returns
// ranked page results matching a free-text query — more effective than the
// title-only search for context-aware disambiguation because the ranker scores
// body content, not just the title.
const WIKIPEDIA_SEARCH_HOSTS = {
  en: "https://en.wikipedia.org/w/rest.php/v1/search/page",
  ru: "https://ru.wikipedia.org/w/rest.php/v1/search/page",
  hi: "https://hi.wikipedia.org/w/rest.php/v1/search/page",
  zh: "https://zh.wikipedia.org/w/rest.php/v1/search/page",
};

const WIKIPEDIA_ACTION_API_HOSTS = {
  en: "https://en.wikipedia.org/w/api.php",
  ru: "https://ru.wikipedia.org/w/api.php",
  hi: "https://hi.wikipedia.org/w/api.php",
  zh: "https://zh.wikipedia.org/w/api.php",
};

const WIKTIONARY_SEARCH_HOSTS = {
  en: "https://en.wiktionary.org/w/api.php",
  ru: "https://ru.wiktionary.org/w/api.php",
  hi: "https://hi.wiktionary.org/w/api.php",
  zh: "https://zh.wiktionary.org/w/api.php",
};

const WIKINEWS_SEARCH_HOSTS = {
  en: "https://en.wikinews.org/w/api.php",
  ru: "https://ru.wikinews.org/w/api.php",
  zh: "https://zh.wikinews.org/w/api.php",
};

function wikipediaHostsFor(language) {
  // Try the detected language first, then fall back to English so a Russian
  // query for an English-only article still returns a definition.
  const ordered = [language, "en"].filter(
    (value, index, array) => value && array.indexOf(value) === index,
  );
  return ordered.map((lang) => ({
    language: lang,
    url: WIKIPEDIA_HOSTS[lang] || WIKIPEDIA_HOSTS.en,
  }));
}

function capitalizeWords(term) {
  return term
    .split(/(\s+)/)
    .map((part) =>
      /\S/.test(part) ? part.charAt(0).toUpperCase() + part.slice(1) : part,
    )
    .join("");
}

function wikipediaTermVariants(term) {
  const seen = new Set();
  const variants = [];
  const push = (value) => {
    if (!value) return;
    const slug = String(value)
      .trim()
      .replace(/\s+/g, "_")
      .replace(/_+/g, "_");
    if (!slug || seen.has(slug)) return;
    seen.add(slug);
    variants.push(slug);
  };
  const trimmed = String(term || "").trim();
  push(trimmed);
  push(capitalizeWords(trimmed));
  push(capitalizeWords(trimmed.toLowerCase()));
  push(trimmed.toLowerCase());
  // Biography titles on Wikipedia (notably ru.wikipedia.org) use the
  // "Surname, Given names" form: querying "Илон Маск" 404s, but "Маск, Илон"
  // resolves. For two-word terms try the swap in both original and
  // capitalized casing so other language hosts can match too.
  const words = trimmed.split(/\s+/).filter(Boolean);
  if (words.length === 2) {
    const swapped = `${words[1]}, ${words[0]}`;
    push(swapped);
    push(capitalizeWords(swapped.toLowerCase()));
  }
  return variants;
}

function normalizeLookupText(value) {
  return String(value || "")
    .normalize("NFKD")
    .toLowerCase()
    .replace(/\p{M}/gu, "")
    .replace(/[^\p{L}\p{N}]+/gu, " ")
    .trim();
}

function compactLookupText(value) {
  return normalizeLookupText(value).replace(/\s+/g, "");
}

function boundedEditDistance(left, right, limit) {
  if (Math.abs(left.length - right.length) > limit) return limit + 1;
  let previous = Array.from({ length: right.length + 1 }, (_, index) => index);
  for (let i = 1; i <= left.length; i += 1) {
    const current = [i];
    let rowMin = current[0];
    for (let j = 1; j <= right.length; j += 1) {
      const cost = left[i - 1] === right[j - 1] ? 0 : 1;
      const next = Math.min(
        previous[j] + 1,
        current[j - 1] + 1,
        previous[j - 1] + cost,
      );
      current[j] = next;
      rowMin = Math.min(rowMin, next);
    }
    if (rowMin > limit) return limit + 1;
    previous = current;
  }
  return previous[right.length];
}

function isNearLookupText(left, right) {
  const a = compactLookupText(left);
  const b = compactLookupText(right);
  if (!a || !b) return false;
  const maxLength = Math.max(a.length, b.length);
  const limit = maxLength <= 8 ? 1 : 2;
  return boundedEditDistance(a, b, limit) <= limit;
}

function isPlausibleWikipediaSearchMatch(summary, term) {
  if (
    !summary ||
    (summary.matchKind !== "search" && summary.matchKind !== "context_search")
  ) {
    return true;
  }
  const termNormalized = normalizeLookupText(term);
  if (!termNormalized) return true;
  const termTokens = termNormalized.split(/\s+/).filter(Boolean);
  const candidates = [
    summary.title,
    summary.matchedTitle,
    String(summary.matchedSlug || "").replace(/_/g, " "),
    summary.extract,
  ];
  for (const candidate of candidates) {
    const normalized = normalizeLookupText(candidate);
    if (!normalized) continue;
    if (normalized === termNormalized) return true;
    const candidateTokens = new Set(normalized.split(/\s+/).filter(Boolean));
    if (
      termTokens.length > 0 &&
      termTokens.every((token) => candidateTokens.has(token))
    ) {
      return true;
    }
    if (isNearLookupText(termNormalized, normalized)) return true;
  }
  return false;
}

const LOOKUP_STEM_STOPWORDS = new Set([
  "a",
  "an",
  "and",
  "for",
  "in",
  "of",
  "on",
  "the",
  "to",
  "about",
  "sentence",
  "sentences",
  "в",
  "во",
  "и",
  "или",
  "на",
  "о",
  "об",
  "про",
]);

function hasSharedLookupStem(summary, term) {
  const normalizedTerm = normalizeLookupText(term);
  if (!normalizedTerm) return false;
  const content = normalizeLookupText(
    [
      summary && summary.title,
      summary && summary.matchedTitle,
      summary && String(summary.matchedSlug || "").replace(/_/g, " "),
      summary && summary.extract,
    ]
      .filter(Boolean)
      .join(" "),
  );
  if (!content) return false;
  const contentTokens = content.split(/\s+/).filter(Boolean);
  for (const token of normalizedTerm.split(/\s+/).filter(Boolean)) {
    if (LOOKUP_STEM_STOPWORDS.has(token) || token.length < 7) continue;
    const stemLength = Math.min(8, token.length - 2);
    const stem = token.slice(0, stemLength);
    if (stem.length >= 5 && contentTokens.some((candidate) => candidate.startsWith(stem))) {
      return true;
    }
  }
  return false;
}

function isArticleQuestionWikipediaMatch(summary, query) {
  if (!summary) return false;
  if (summary.matchKind === "direct") return true;
  if (isPlausibleWikipediaSearchMatch(summary, query.exactTerm)) return true;
  if (query.lookupTerm !== query.exactTerm && isPlausibleWikipediaSearchMatch(summary, query.lookupTerm)) {
    return true;
  }
  if (!hasSharedLookupStem(summary, query.lookupTerm || query.exactTerm)) {
    return false;
  }
  return !query.contextOriginal || hasSharedLookupStem(summary, query.contextOriginal);
}

// Resolve a context-qualified term to a Wikipedia page slug via full-text page
// search. Tries multiple query formulations (uppercase term, mixed case) on the
// detected language host then on English, returning the first match found.
// This helps disambiguate short acronyms like "KISS" or "DRY" when the user
// provides a programming/domain context.
async function searchWikipediaSlug(term, context, language) {
  if (typeof fetch !== "function") return null;
  const apiHeaders = {
    accept: "application/json",
    "api-user-agent":
      "formal-ai-demo (https://github.com/link-assistant/formal-ai)",
  };
  const upper = term.toUpperCase();
  // Build candidate queries in preference order: uppercase term with context is
  // most discriminating; plain term with context is the fallback.
  const queries = [];
  if (upper !== term) queries.push(`${upper} ${context}`.trim());
  queries.push(`${term} ${context}`.trim());
  const ordered = [language, "en"].filter(
    (value, index, array) => value && array.indexOf(value) === index,
  );
  for (const lang of ordered) {
    const base = WIKIPEDIA_SEARCH_HOSTS[lang] || WIKIPEDIA_SEARCH_HOSTS.en;
    for (const query of queries) {
      const url = `${base}?q=${encodeURIComponent(query)}&limit=5`;
      try {
        const response = await fetch(url, { headers: apiHeaders });
        if (!response || !response.ok) continue;
        const data = await response.json();
        if (!data || !Array.isArray(data.pages) || data.pages.length === 0)
          continue;
        // Return the key of the top result; callers will fetch the full summary.
        const page = data.pages[0];
        return {
          slug: page.key,
          title: page.title || page.key,
          language: lang,
          query,
        };
      } catch (_error) {
        // Ignore and try next query / language host.
      }
    }
  }
  return null;
}

function decodeHtmlEntities(value) {
  const named = {
    amp: "&",
    apos: "'",
    mdash: "—",
    ndash: "–",
    gt: ">",
    lt: "<",
    nbsp: " ",
    quot: '"',
  };
  return String(value || "")
    .replace(/&#x([0-9a-f]+);/giu, (_match, code) => {
      const parsed = Number.parseInt(code, 16);
      return Number.isFinite(parsed) ? String.fromCodePoint(parsed) : "";
    })
    .replace(/&#(\d+);/gu, (_match, code) => {
      const parsed = Number.parseInt(code, 10);
      return Number.isFinite(parsed) ? String.fromCodePoint(parsed) : "";
    })
    .replace(/&([a-z]+);/giu, (match, name) => named[name.toLowerCase()] || match);
}

function stripHtmlToText(html) {
  return decodeHtmlEntities(
    String(html || "")
      .replace(/<style\b[\s\S]*?<\/style>/giu, " ")
      .replace(/<script\b[\s\S]*?<\/script>/giu, " ")
      .replace(/<sup\b[\s\S]*?<\/sup>/giu, " ")
      .replace(/<[^>]+>/gu, " "),
  )
    .replace(/\s+([,.;:!?])/gu, "$1")
    .replace(/\s+/gu, " ")
    .trim();
}

function truncateDisambiguationHtml(html) {
  const text = String(html || "");
  let end = text.length;
  for (const marker of [
    /<h[1-6]\b[^>]*id=["'](?:См\._также|See_also|Примечания|References|Notes)["']/iu,
    /<div\b[^>]*id=["']disambig["']/iu,
  ]) {
    const match = marker.exec(text);
    if (match && match.index > 0) end = Math.min(end, match.index);
  }
  return text.slice(0, end);
}

function deduplicateTextList(values) {
  const out = [];
  const seen = new Set();
  for (const value of values) {
    const text = String(value || "").trim();
    if (!text) continue;
    const key = normalizeLookupText(text);
    if (!key || seen.has(key)) continue;
    seen.add(key);
    out.push(text);
  }
  return out;
}

function extractDisambiguationEntriesFromHtml(html) {
  const scoped = truncateDisambiguationHtml(html);
  const entries = [];
  const itemPattern = /<li\b[^>]*>([\s\S]*?)<\/li>/giu;
  let match;
  while ((match = itemPattern.exec(scoped)) !== null) {
    const text = stripHtmlToText(match[1]);
    if (!text || text.startsWith("↑")) continue;
    entries.push(text);
  }
  return deduplicateTextList(entries).slice(0, 12);
}

function extractDisambiguationEntriesFromSummary(summary) {
  const title = normalizeLookupText(summary && summary.title);
  const raw = String((summary && summary.extract) || "");
  const extract = raw.replace(
    /^([^:\n]{1,80}):\s*([«»"'“”„]?[^\n]{1,80}[»"'“”„]?\s[—–-]\s)/u,
    "$1:\n$2",
  );
  const lines = extract
    .split(/\n+/u)
    .map((line) => line.trim())
    .filter(Boolean)
    .filter((line) => {
      const normalized = normalizeLookupText(line.replace(/:$/u, ""));
      return normalized && normalized !== title;
    });
  return deduplicateTextList(lines);
}

function definitionPrefixForDisambiguationEntry(entry) {
  const text = String(entry || "").trim();
  const dash = text.search(/\s[—–-]\s/u);
  if (dash <= 0) return "";
  return normalizeLookupText(
    text
      .slice(0, dash)
      .trim()
      .replace(/^[«»"'“”„]+|[«»"'“”„]+$/gu, ""),
  );
}

function isDefinitionStyleDisambiguation(summary, requestedTerm, entries) {
  const targets = [requestedTerm, summary && summary.title]
    .map((value) => normalizeLookupText(value))
    .filter(Boolean);
  if (targets.length === 0) return false;
  return entries.some((entry) => {
    const prefix = definitionPrefixForDisambiguationEntry(entry);
    return prefix && targets.includes(prefix);
  });
}

async function fetchWikipediaDisambiguationEntries(summary) {
  if (typeof fetch !== "function" || !summary) return [];
  const base =
    WIKIPEDIA_ACTION_API_HOSTS[summary.language] || WIKIPEDIA_ACTION_API_HOSTS.en;
  const page = summary.matchedSlug || summary.title;
  if (!page) return [];
  const url = `${base}?action=parse&page=${encodeURIComponent(
    page,
  )}&prop=text&format=json&formatversion=2&redirects=1&origin=*`;
  try {
    const response = await fetch(url, {
      headers: {
        accept: "application/json",
        "api-user-agent":
          "formal-ai-demo (https://github.com/link-assistant/formal-ai)",
      },
    });
    if (!response || !response.ok) return [];
    const data = await response.json();
    const text = data && data.parse ? data.parse.text : "";
    let html = "";
    if (typeof text === "string") {
      html = text;
    } else if (text && typeof text === "object" && text["*"]) {
      html = text["*"];
    }
    return extractDisambiguationEntriesFromHtml(html);
  } catch (_error) {
    return [];
  }
}

async function buildDefinitionStyleDisambiguationSummary(
  data,
  term,
  language,
  matchedSlug,
  requestUrl,
) {
  const title = String(data.title || term);
  const pageUrl =
    (data.content_urls &&
      data.content_urls.desktop &&
      data.content_urls.desktop.page) ||
    requestUrl;
  const summary = {
    title,
    extract: String(data.extract || "").trim(),
    url: pageUrl,
    language,
    matchKind: "disambiguation",
    matchedSlug,
  };
  const summaryEntries = extractDisambiguationEntriesFromSummary(summary);
  if (!isDefinitionStyleDisambiguation(summary, term, summaryEntries)) {
    return null;
  }
  const parsedEntries = await fetchWikipediaDisambiguationEntries(summary);
  const entries = parsedEntries.length > 0 ? parsedEntries : summaryEntries;
  return {
    ...summary,
    extract: entries.join("\n"),
    disambiguationEntries: entries,
  };
}

async function fetchWikipediaSummary(term, language, context, options) {
  if (typeof fetch !== "function") return null;
  const includeDefinitionDisambiguation = Boolean(
    options && options.includeDefinitionDisambiguation,
  );
  const apiHeaders = {
    accept: "application/json",
    "api-user-agent":
      "formal-ai-demo (https://github.com/link-assistant/formal-ai)",
  };

  // When context is provided, first try a title-search to find the most
  // relevant article slug (e.g. "Kiss" + "рамках програмирования" → "KISS
  // principle"). This prevents ambiguous short terms from matching the wrong
  // article (e.g. the rock band instead of the software design principle).
  if (context) {
    const found = await searchWikipediaSlug(term, context, language);
    if (found) {
      const summaryBase =
        WIKIPEDIA_HOSTS[found.language] || WIKIPEDIA_HOSTS.en;
      const url = `${summaryBase}/${encodeURIComponent(found.slug)}`;
      try {
        const response = await fetch(url, { headers: apiHeaders });
        if (response && response.ok) {
          const data = await response.json();
          if (
            data &&
            typeof data === "object" &&
            data.type !== "disambiguation"
          ) {
            const extract = String(data.extract || "").trim();
            if (extract) {
              const title = String(data.title || term);
              const pageUrl =
                (data.content_urls &&
                  data.content_urls.desktop &&
                  data.content_urls.desktop.page) ||
                url;
              return {
                title,
                extract,
                url: pageUrl,
                language: found.language,
                matchKind: "context_search",
                matchedSlug: found.slug,
                matchedTitle: found.title || title,
                searchQuery: found.query || "",
              };
            }
          }
        }
      } catch (_error) {
        // Fall through to bare-term lookup below.
      }
    }
  }

  // Bare-term fallback: try direct slug variants without context.
  const hosts = wikipediaHostsFor(language);
  const variants = wikipediaTermVariants(term);
  for (const host of hosts) {
    for (const slug of variants) {
      const url = `${host.url}/${encodeURIComponent(slug)}`;
      try {
        const response = await fetch(url, { headers: apiHeaders });
        if (!response || !response.ok) continue;
        const data = await response.json();
        if (!data || typeof data !== "object") continue;
        if (data.type === "disambiguation") {
          if (includeDefinitionDisambiguation && !context) {
            const disambiguation = await buildDefinitionStyleDisambiguationSummary(
              data,
              term,
              host.language,
              slug,
              url,
            );
            if (disambiguation) return disambiguation;
          }
          continue;
        }
        const extract = String(data.extract || "").trim();
        if (!extract) continue;
        const title = String(data.title || term);
        const pageUrl =
          (data.content_urls &&
            data.content_urls.desktop &&
            data.content_urls.desktop.page) ||
          url;
        return {
          title,
          extract,
          url: pageUrl,
          language: host.language,
          matchKind: "direct",
          matchedSlug: slug,
        };
      } catch (_error) {
        // Swallow network/parse errors and continue to the next variant.
      }
    }
  }
  // All direct slug variants were disambiguation pages or not found. Use the
  // full-text search endpoint to find the top-ranked article for the term
  // (e.g. "tesla" → "Tesla, Inc." instead of the disambiguation page).
  const found = await searchWikipediaSlug(term, "", language);
  if (found) {
    const summaryBase = WIKIPEDIA_HOSTS[found.language] || WIKIPEDIA_HOSTS.en;
    const url = `${summaryBase}/${encodeURIComponent(found.slug)}`;
    try {
      const response = await fetch(url, { headers: apiHeaders });
      if (response && response.ok) {
        const data = await response.json();
        if (
          data &&
          typeof data === "object" &&
          data.type !== "disambiguation"
        ) {
          const extract = String(data.extract || "").trim();
          if (extract) {
            const title = String(data.title || term);
            const pageUrl =
              (data.content_urls &&
                data.content_urls.desktop &&
                data.content_urls.desktop.page) ||
              url;
            return {
              title,
              extract,
              url: pageUrl,
              language: found.language,
              matchKind: "search",
              matchedSlug: found.slug,
              matchedTitle: found.title || title,
              searchQuery: found.query || "",
            };
          }
        }
      }
    } catch (_error) {
      // Search-based fallback failed; return null below.
    }
  }
  return null;
}

function isClosestWikipediaMatch(summary) {
  return summary && summary.matchKind === "search";
}

function closestMatchNote(summary, language) {
  const title = summary && summary.title ? summary.title : "the top result";
  if (language === "ru") {
    return `Ближайшее совпадение по поиску Wikipedia: «${title}». Если это не то, уточните запрос.`;
  }
  if (language === "zh") {
    return `Wikipedia 搜索的最接近匹配是“${title}”。如果这不是你的意思，请进一步说明。`;
  }
  if (language === "hi") {
    return `Wikipedia खोज में सबसे नज़दीकी मिलान "${title}" है। अगर आपका मतलब यह नहीं था, तो कृपया स्पष्ट करें।`;
  }
  return `Closest match from Wikipedia search: "${title}". If that is not what you meant, clarify the prompt.`;
}

function wikipediaClarificationMessage(summary, language) {
  const title = summary && summary.title ? summary.title : "the top result";
  if (language === "ru") {
    return `Похоже, вы имели в виду «${title}». Уточните, отвечать по этой статье Wikipedia?`;
  }
  if (language === "zh") {
    return `你是指“${title}”吗？请确认后我再根据这篇 Wikipedia 文章回答。`;
  }
  if (language === "hi") {
    return `क्या आपका मतलब "${title}" था? Wikipedia के इस लेख से उत्तर देने से पहले कृपया स्पष्ट करें।`;
  }
  return `Did you mean "${title}"? Please clarify before I answer from that Wikipedia article.`;
}

function wikipediaDisambiguationMessage(summary, language) {
  const humanUrl = humanizeUrl(summary.url);
  const entries = Array.isArray(summary.disambiguationEntries)
    ? summary.disambiguationEntries
    : String(summary.extract || "")
        .split(/\n+/u)
        .map((line) => line.trim())
        .filter(Boolean);
  const list = entries.map((entry) => `- ${entry}`).join("\n");
  if (language === "ru") {
    return `На странице Wikipedia «${summary.title}» перечислены значения:\n\n${list}\n\nИсточник: [${humanUrl}](${summary.url}) (wikipedia).`;
  }
  if (language === "zh") {
    return `Wikipedia “${summary.title}”页面列出以下含义：\n\n${list}\n\n来源：[${humanUrl}](${summary.url}) (wikipedia).`;
  }
  if (language === "hi") {
    return `Wikipedia पृष्ठ "${summary.title}" ये अर्थ सूचीबद्ध करता है:\n\n${list}\n\nस्रोत: [${humanUrl}](${summary.url}) (wikipedia).`;
  }
  return `Wikipedia's "${summary.title}" page lists these meanings:\n\n${list}\n\nSource: [${humanUrl}](${summary.url}) (wikipedia).`;
}

function wikipediaArticleQuestionMessage(summary, query, language, exactMatch) {
  const humanUrl = humanizeUrl(summary.url);
  const source = `Source: [${humanUrl}](${summary.url}) (wikipedia).`;
  if (language === "ru") {
    const wikipediaName =
      summary.language === "ru" ? "русскоязычной Википедии" : "Wikipedia";
    if (exactMatch) {
      return `В Wikipedia есть статья «${summary.title}»: ${summary.extract}\n\nИсточник: [${humanUrl}](${summary.url}) (wikipedia).`;
    }
    return [
      `В ${wikipediaName} я не нашёл отдельной статьи с названием «${query.exactTerm}», но ближайшая подходящая страница — «${summary.title}»: ${summary.extract}`,
      `Источник: [${humanUrl}](${summary.url}) (wikipedia).`,
    ].join("\n\n");
  }
  if (language === "zh") {
    const zhSource = `来源：[${humanUrl}](${summary.url}) (wikipedia).`;
    if (exactMatch) {
      return `Wikipedia 有一篇“${summary.title}”条目：${summary.extract}\n\n${zhSource}`;
    }
    return `我没有找到标题为“${query.exactTerm}”的 Wikipedia 条目，但最接近的有用页面是“${summary.title}”：${summary.extract}\n\n${zhSource}`;
  }
  if (language === "hi") {
    const hiSource = `स्रोत: [${humanUrl}](${summary.url}) (wikipedia).`;
    if (exactMatch) {
      return `Wikipedia पर "${summary.title}" लेख है: ${summary.extract}\n\n${hiSource}`;
    }
    return `मुझे Wikipedia पर "${query.exactTerm}" शीर्षक वाला अलग लेख नहीं मिला, लेकिन सबसे नज़दीकी उपयोगी पृष्ठ "${summary.title}" है: ${summary.extract}\n\n${hiSource}`;
  }
  if (exactMatch) {
    return `Wikipedia has an article titled "${summary.title}": ${summary.extract}\n\n${source}`;
  }
  return `I did not find an exact Wikipedia article titled "${query.exactTerm}", but the closest useful page is "${summary.title}": ${summary.extract}\n\n${source}`;
}

// ---------------------------------------------------------------------------
// Wikidata-backed fact reasoning pipeline (issue #127).
//
// Rather than matching against hardcoded summaries in `data/seed/facts.lino`,
// fact questions ("what is the capital of X?", "столица X", "X की राजधानी",
// "X的首都") are parsed into a structured query
// `{ relation, subjectTerm, language, forceFresh }`. The query is then
// resolved against:
//
//   1. An in-memory cache (1-week TTL) keyed by `relation:subject:language`.
//      The cache is pre-warmed from the seed `FACTS` entries so the test
//      matrix stays deterministic offline.
//   2. Wikidata `wbsearchentities` to resolve the subject term to a Q-ID.
//   3. Wikidata `wbgetentities` to fetch the property claim (P36 = capital,
//      P1082 = population, P38 = currency, P37 = official language, P30 =
//      continent, P50 = author, P2046 = area, P35 = head of state, P6 = head
//      of government).
//   4. Wikidata `wbgetentities` again to resolve the target Q-ID to a label
//      in the user's prevailing language (and to a Wikipedia sitelink).
//
// Every step is recorded as a `fact_query:*` event so the reasoning trace
// shows the structured query, the cache decision, the Wikidata round-trips,
// and the final resolved answer. A user can force a fresh resolution by
// adding markers like "fresh", "no cache", "не из кэша", "без кеша",
// "ताज़ा", or "刷新" to the prompt.
// ---------------------------------------------------------------------------

const WIKIDATA_API = "https://www.wikidata.org/w/api.php";

const FACT_RELATIONS = [
  {
    relation: "capital",
    property: "P36",
    valueType: "entity",
  },
  {
    relation: "population",
    property: "P1082",
    valueType: "quantity",
  },
  {
    relation: "currency",
    property: "P38",
    valueType: "entity",
  },
  {
    relation: "official_language",
    property: "P37",
    valueType: "entity",
  },
  {
    relation: "continent",
    property: "P30",
    valueType: "entity",
  },
  {
    relation: "author_of_book",
    property: "P50",
    valueType: "entity",
  },
  {
    relation: "area",
    property: "P2046",
    valueType: "quantity",
  },
  {
    relation: "head_of_state",
    property: "P35",
    valueType: "entity",
  },
  {
    relation: "head_of_government",
    property: "P6",
    valueType: "entity",
  },
];
