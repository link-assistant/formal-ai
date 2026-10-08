// Claim operands (issue #1175 R3, the last five rows): the operand each of
// docs_method_explanation, kupi_slona, source_conflict,
// github_repository_traffic and formalization_request answers about. Twin of
// rust/src/capability_routing/claim_operands.rs.
//
// A reader returns the operands it extracts from the prompt, an empty list
// when there are none. The claim row admits on that list being non-empty
// (OPERAND_CLAIM_EVIDENCE, joined into claimEvidence()), and the rule
// interpreter captures the same operands with `value <name> operand <kind>`,
// so the row and the answer read one thing. Every surface is a seed role.

/** Marks that separate words besides whitespace (Rust `WORD_MARKS`). */
const CLAIM_OPERAND_WORD_MARKS = ",.;:!?\"'«»()[]{}<>`—–-…，。；：！？、।“”‘’";
/** Marks that end a clause (Rust `CLAUSE_MARKS`); a full stop ends one only before whitespace. */
const CLAIM_OPERAND_CLAUSE_MARKS = ",;，；。!?！？\n।";
/** The roles whose words carry no operand of their own (Rust `FRAME_ROLES`). */
const CLAIM_OPERAND_FRAME_ROLES = ["frame_filler_word", "request_function_word", "statement_function_word"];

/** Whether a surface is written in a script without spaces between words (Rust `unspaced`). */
function claimOperandUnspaced(surface) {
  return /[぀-ヿ㐀-鿿가-힯]/u.test(String(surface || ""));
}

/** The lowercased surfaces of a seed role (Rust `role_surfaces`). */
function claimOperandSurfaces(role) {
  return wordsForRole(role).map((surface) => String(surface).toLowerCase()).filter((surface) => surface !== "");
}

/** Whether a character separates words (Rust `separates`). */
function claimOperandSeparates(character) {
  return character === undefined || /\s/u.test(character) || CLAIM_OPERAND_WORD_MARKS.includes(character);
}

/** The words of `text`: runs between whitespace and word marks (Rust `words`). */
function claimOperandWords(text) {
  const words = [];
  let word = "";
  for (const character of String(text || "")) {
    if (claimOperandSeparates(character)) {
      if (word !== "") words.push(word);
      word = "";
    } else {
      word += character;
    }
  }
  if (word !== "") words.push(word);
  return words;
}

/** Whether `surface` occurs in `lower`: as a substring for an unspaced script, between separators otherwise (Rust `names_surface`). */
function claimOperandNames(lower, surface) {
  if (claimOperandUnspaced(surface)) return lower.includes(surface);
  const characters = [...lower];
  const wanted = [...surface];
  for (let start = 0; start + wanted.length <= characters.length; start += 1) {
    if (wanted.every((character, offset) => characters[start + offset] === character)
      && claimOperandSeparates(characters[start - 1]) && claimOperandSeparates(characters[start + wanted.length])) return true;
  }
  return false;
}

/**
 * Whether every word of `text` is a word of the frame roles: the surfaces of
 * unspaced scripts are removed as substrings first, longest first (Rust
 * `only_frame_words`).
 */
function onlyFrameWords(text) {
  let rest = String(text || "").toLowerCase();
  const covered = new Set();
  const unspaced = [];
  for (const surface of CLAIM_OPERAND_FRAME_ROLES.flatMap((role) => claimOperandSurfaces(role))) {
    if (claimOperandUnspaced(surface)) unspaced.push(surface);
    else claimOperandWords(surface).forEach((word) => covered.add(word));
  }
  unspaced.sort((left, right) => [...right].length - [...left].length);
  for (const surface of unspaced) rest = rest.split(surface).join(" ");
  return claimOperandWords(rest).every((word) => covered.has(word));
}

/** `text` with every surface removed, longest first (Rust `without_surfaces`). */
function claimOperandWithout(text, surfaces) {
  return surfaces.slice().sort((left, right) => [...right].length - [...left].length)
    .reduce((rest, surface) => rest.split(surface).join(" "), String(text || "").toLowerCase());
}

/**
 * The seeded documentation page whose project and method the prompt names,
 * as [page, project, docs url] (Rust `documented_method`).
 */
function claimOperandDocumentedMethod(prompt) {
  const page = handlerRulesPolicy("docs_method_explanation", "page");
  const docsUrl = handlerRulesPolicy("docs_method_explanation", "docs_url");
  const alias = String(handlerRulesPolicy("docs_method_explanation", "class_alias") || "").toLowerCase();
  if (!page || !docsUrl) return [];
  const segments = page.split(".");
  const parts = segments.map((segment) => segment.toLowerCase());
  if (parts.length < 3) return [];
  const lower = String(prompt || "").toLowerCase();
  const identifiers = lower.split(/[^a-z0-9_]+/u).filter(Boolean);
  const named = (word) => identifiers.includes(word);
  const classNamed = parts.slice(1, -1).some(named) || (alias !== "" && named(alias))
    || lexiconMentionsRole(ROLE_CODE_METHOD_NOUN, lower);
  return named(parts[0]) && named(parts[parts.length - 1]) && classNamed ? [page, segments[0], docsUrl] : [];
}

/** The circular idiom when it is the whole utterance, framed only by frame words (Rust `idiom_utterance`). */
function claimOperandIdiomUtterance(prompt) {
  const lower = String(prompt || "").toLowerCase();
  const idiom = claimOperandSurfaces("circular_joke_phrase").find((surface) => {
    const at = lower.indexOf(surface);
    return at !== -1 && onlyFrameWords(`${lower.slice(0, at)} ${lower.slice(at + surface.length)}`);
  });
  return idiom === undefined ? [] : [idiom];
}

/** The clauses of `text`, split at clause marks and seeded clause joiners (Rust `clauses`). */
function claimOperandClauses(text) {
  const characters = [...String(text || "")];
  const lower = characters.map((character) => [...character.toLowerCase()][0] || character);
  const joiners = claimOperandSurfaces("clause_joiner").map((surface) => [...surface])
    .sort((left, right) => right.length - left.length);
  const clauses = [];
  let start = 0;
  let index = 0;
  const cut = (end, next) => {
    clauses.push(characters.slice(start, end).join(""));
    start = next;
    index = next;
  };
  while (index < characters.length) {
    const character = characters[index];
    if (CLAIM_OPERAND_CLAUSE_MARKS.includes(character) || (character === "." && claimOperandSeparates(characters[index + 1]))) {
      cut(index, index + 1);
      continue;
    }
    const joiner = joiners.find((surface) => surface.every((part, offset) => lower[index + offset] === part)
      && (claimOperandUnspaced(surface.join("")) || (claimOperandSeparates(characters[index - 1]) && claimOperandSeparates(characters[index + surface.length]))));
    if (joiner) cut(index, index + joiner.length);
    else index += 1;
  }
  clauses.push(characters.slice(start).join(""));
  const edge = (character) => /\s/u.test(character) || CLAIM_OPERAND_WORD_MARKS.includes(character);
  return clauses.map((clause) => {
    const kept = [...clause];
    while (kept.length > 0 && edge(kept[0])) kept.shift();
    while (kept.length > 0 && edge(kept[kept.length - 1])) kept.pop();
    return kept.join("");
  }).filter((clause) => clause !== "");
}

/** The text after the first colon when it is not empty, else the prompt (Rust `after_colon`). */
function claimOperandAfterColon(prompt) {
  const text = String(prompt || "");
  const colon = text.search(/[:：]/u);
  const after = colon === -1 ? "" : text.slice(colon + 1).trim();
  return after === "" ? text : after;
}

/** The first two clauses that attribute what they state to a source (Rust `attributed_alternatives`). */
function claimOperandAttributedAlternatives(prompt) {
  const markers = claimOperandSurfaces("source_attribution_marker");
  const attributed = claimOperandClauses(claimOperandAfterColon(prompt)).filter((clause) => {
    const lower = clause.toLowerCase();
    return markers.some((marker) => claimOperandNames(lower, marker) && [...clause].length > [...marker].length + 1);
  });
  return attributed.length >= 2 ? attributed.slice(0, 2) : [];
}

/** Owner/name slugs the prompt names: a GitHub URL, then each bare owner/name run (Rust `repository_slug_candidates`). */
function repositorySlugCandidates(prompt) {
  const slugs = [];
  const candidate = firstUrlCandidate(prompt);
  const fromUrl = candidate ? repositoryFromUrl(candidate.url) : null;
  if (fromUrl && fromUrl.platform.slug === "github") slugs.push(`${fromUrl.owner}/${fromUrl.name}`);
  for (const run of String(prompt || "").split(/[^A-Za-z0-9._/-]+/u)) {
    const trimmed = run.replace(/\.+$/u, "");
    const parts = trimmed.split("/");
    if (parts.length !== 2) continue;
    const owner = cleanRepositorySegment(parts[0]);
    const name = cleanRepositorySegment(parts[1]);
    if (!/^[A-Za-z0-9]/u.test(owner) || !/^[A-Za-z0-9]/u.test(name)) continue;
    if (/^\d+$/u.test(owner) && /^\d+$/u.test(name)) continue;
    slugs.push(`${owner}/${name}`);
  }
  return slugs;
}

/**
 * The repository a traffic question names: an owner/name slug whose segments
 * are not cue or function words, else the assistant's own repository when a
 * second-person possessive names it (Rust `named_repository`).
 */
function claimOperandNamedRepository(prompt) {
  const excluded = new Set([...CLAIM_OPERAND_FRAME_ROLES, "github_repository_traffic_signal", "repository_reference", "github_repository_platform"]
    .flatMap((role) => claimOperandSurfaces(role)).flatMap((surface) => claimOperandWords(surface)));
  const slug = repositorySlugCandidates(prompt).find((candidate) => candidate.split("/").every((segment) => !excluded.has(segment.toLowerCase())));
  if (slug) return [slug];
  const lower = String(prompt || "").toLowerCase();
  const own = String((AGENT_INFO && AGENT_INFO.repository) || "").trim();
  return own !== "" && claimOperandSurfaces("second_person_possessive").some((surface) => claimOperandNames(lower, surface)) ? [own] : [];
}

/** The statement a formalization request carries beyond its cue, or the formal clause it deformalizes (Rust `formalization_statement`). */
function claimOperandFormalizationStatement(prompt) {
  const text = String(prompt || "");
  const sentence = formalSentenceUnderDiscussion(text);
  if (["∀", "∃", "¬∃"].some((symbol) => text.includes(symbol))) return [sentence];
  const phrases = formalTargetRecords().filter((record) => record.name === "cues")
    .flatMap((record) => (formalNamedChild(record, "intent") || { children: [] }).children)
    .filter((node) => node.name === "role").flatMap((node) => textTransformRecordValues(node, "phrase"))
    .concat(formalGrammar().formal.flatMap((language) => language.aliases))
    .map((surface) => surface.toLowerCase());
  return onlyFrameWords(claimOperandWithout(sentence, phrases)) ? [] : [sentence];
}

/** The roles that name the program a call runs in (Rust `PROGRAM_ROLES`). */
const CLAIM_OPERAND_PROGRAM_ROLES = ["script_or_code_artifact", "program_genus"];

/** Whether a character continues an identifier: a letter, a digit or `_` (Rust `continues_identifier`). */
function claimOperandContinuesIdentifier(character) {
  return character !== undefined && (character === "_" || /[\p{Alphabetic}\p{N}]/u.test(character));
}

/**
 * The function a coding request says its program calls when nothing defines
 * it (Rust `undefined_call`, issue 1173 R3): the request names a program
 * (a `CLAIM_OPERAND_PROGRAM_ROLES` surface) and a seeded
 * `function_call_verb`, and the first call expression `name(` that is not a
 * method call (`.name(`), not a callable of the seeded
 * `script_builtin_callable` word map (data/seed/code-task-cues.lino) and not written anywhere else in the
 * request (a definition or a second mention would name it) is the call a
 * sandbox run fails on.
 */
function claimOperandUndefinedCall(prompt) {
  const text = String(prompt || "");
  const lower = text.toLowerCase();
  const names = (role) => claimOperandSurfaces(role).some((surface) => claimOperandNames(lower, surface));
  if (!names("function_call_verb") || !CLAIM_OPERAND_PROGRAM_ROLES.some(names)) return [];
  const builtins = new Set(codeTaskWordEntries("script_builtin_callable").map((entry) => codeTaskChildValue(entry, "word").toLowerCase()));
  const words = claimOperandWords(text);
  const characters = [...text];
  for (let index = 0; index < characters.length; index += 1) {
    if (characters[index] !== "(") continue;
    let end = index;
    while (end > 0 && /\s/u.test(characters[end - 1])) end -= 1;
    let start = end;
    while (start > 0 && claimOperandContinuesIdentifier(characters[start - 1])) start -= 1;
    const name = characters.slice(start, end).join("");
    if (name === "" || /\p{N}/u.test(characters[start]) || characters[start - 1] === ".") continue;
    if (builtins.has(name.toLowerCase())) continue;
    if (words.filter((word) => word === name).length === 1) return [name];
  }
  return [];
}

const CLAIM_OPERANDS = Object.freeze({
  undefined_call: claimOperandUndefinedCall,
  documented_method: claimOperandDocumentedMethod,
  idiom_utterance: claimOperandIdiomUtterance,
  attributed_alternatives: claimOperandAttributedAlternatives,
  named_repository: claimOperandNamedRepository,
  formalization_statement: claimOperandFormalizationStatement,
});

const OPERAND_CLAIM_EVIDENCE = Object.freeze(Object.fromEntries(Object.entries(CLAIM_OPERANDS)
  .map(([kind, reader]) => [kind, (prompt) => reader(prompt).length > 0])));
