// The plan reader of the obligation ledger's `derive_expectation`
// (issue #1166 R1166-4), the browser twin of the parts of
// rust/src/agentic_coding/general_planner.rs, write_request.rs,
// note_composition.rs and file_path_shape.rs that decide whether a clause
// composes a `GeneralChangePlan` and in which mode. The worker only needs that
// decision (`obligationPlanMode`): the expectation a plan derives is
// observable for the literal-file and command-output modes and underivable for
// a repository work item (data/meta/obligation-evidence-contract.lino), so the
// plan's id, steps and content are never built here. JavaScript twins:
// js/agentic/general_planner.mjs, js/agentic/write_request.mjs. Every cue is a
// seed role read through the worker lexicon (formal_ai_worker_13.js).
//
// Offsets are JavaScript string indices, produced and consumed inside one
// string, exactly as the agentic port reads them.

// --------------------------------------------------- Rust `str` semantics

function oblIsWhitespace(character) {
  return character !== undefined && /^\p{White_Space}$/u.test(character);
}

function oblIsAlphanumeric(character) {
  return character !== undefined && /^[\p{Alphabetic}\p{N}]$/u.test(character);
}

function oblIsAsciiAlphanumeric(character) {
  return character !== undefined && /^[A-Za-z0-9]$/.test(character);
}

function oblIsAsciiDigit(character) {
  return character !== undefined && /^[0-9]$/.test(character);
}

function oblTrimStartMatches(text, predicate) {
  const chars = Array.from(text);
  let start = 0;
  while (start < chars.length && predicate(chars[start])) start += 1;
  return chars.slice(start).join("");
}

function oblTrimEndMatches(text, predicate) {
  const chars = Array.from(text);
  let end = chars.length;
  while (end > 0 && predicate(chars[end - 1])) end -= 1;
  return chars.slice(0, end).join("");
}

function oblTrimMatches(text, predicate) {
  return oblTrimEndMatches(oblTrimStartMatches(text, predicate), predicate);
}

function oblCharIn(set) {
  const members = new Set(Array.from(set));
  return (character) => members.has(character);
}

const oblTrim = (text) => oblTrimMatches(text, oblIsWhitespace);
const oblTrimStart = (text) => oblTrimStartMatches(text, oblIsWhitespace);
const oblTrimEnd = (text) => oblTrimEndMatches(text, oblIsWhitespace);
const oblSplitWhitespace = (text) => text.split(/\p{White_Space}+/u).filter(Boolean);
const oblLastChar = (text) => Array.from(text).pop();
const oblUtf8Len = (text) => new TextEncoder().encode(text).length;
const oblIsAscii = (text) => /^[\x00-\x7f]*$/.test(text);

// The bare surfaces of a seed role, lowercased (`bare_surfaces`).
function oblBareSurfaces(role) {
  return roleWordForms(role).filter((form) => form.slot === "bare").map((form) => String(form.text).toLowerCase());
}

// Mirrors `Lexicon::mentions_role_raw` (`str::contains`).
function oblMentionsRoleRaw(role, normalized) {
  return meaningsWithRole(role).some((meaning) => meaning.words.some((word) => normalized.includes(String(word))));
}

// Mirrors `fn mentions_bare_role` in rust/src/agentic_coding/general_planner.rs.
function oblMentionsBareRole(text, role) {
  const lowerText = text.toLowerCase();
  return roleWordForms(role).filter((form) => form.slot === "bare").some((form) => {
    const needle = String(form.text).toLowerCase();
    const start = lowerText.indexOf(needle);
    if (start < 0) return false;
    if (!oblIsAscii(needle)) return true;
    const end = start + needle.length;
    const beforeOk = start === 0 || !oblIsAlphanumeric(oblLastChar(lowerText.slice(0, start)));
    const afterOk = end === lowerText.length || !oblIsAlphanumeric(Array.from(lowerText.slice(end, end + 2))[0]);
    return beforeOk && afterOk;
  });
}

// ------------------------------------------------------------- sentences

const OBL_SHELL_ENDS = new Set([".", "!", "?", ";", "\n", "。", "！", "？", "；", "।"]);
const OBL_PROSE_ENDS = new Set([".", "!", "?", "\n", "。", "！", "？", "।"]);

// Mirrors `fn split_sentences` in rust/src/agentic_coding/shell_command_policy.rs.
function oblSentences(prompt, ends) {
  const out = [];
  let spanStart = 0;
  let start = 0;
  let index = 0;
  for (const character of prompt) {
    const at = index;
    index += character.length;
    if (!ends.has(character)) continue;
    if (character === "." && oblIsAlphanumeric(Array.from(prompt.slice(index))[0])) continue;
    const text = oblTrim(prompt.slice(start, at));
    if (text) {
      out.push({ text, start: spanStart, end: index });
      spanStart = index;
    }
    start = index;
  }
  const tail = oblTrim(prompt.slice(start));
  if (tail) out.push({ text: tail, start: spanStart, end: prompt.length });
  return out;
}

// ------------------------------------------------------- write requests

// Mirrors `fn tokens` in rust/src/agentic_coding/write_request.rs.
function oblTokens(request) {
  const out = [];
  let start = null;
  let index = 0;
  for (const character of request) {
    const cp = character.codePointAt(0);
    const ideographic = (cp >= 0x3001 && cp <= 0x3003) || (cp >= 0x3008 && cp <= 0x3011) || (cp >= 0x3014 && cp <= 0x301f)
      || [0xff01, 0xff08, 0xff09, 0xff0c, 0xff1a, 0xff1b, 0xff1f].includes(cp);
    if (oblIsWhitespace(character) || ideographic) {
      if (start !== null) {
        out.push({ text: request.slice(start, index), start, end: index });
        start = null;
      }
    } else if (start === null) {
      start = index;
    }
    index += character.length;
  }
  if (start !== null) out.push({ text: request.slice(start), start, end: request.length });
  return out;
}

// Mirrors `fn peel_sentence_punctuation` in rust/src/agentic_coding/file_path_shape.rs.
function oblPeelSentencePunctuation(token, strip) {
  let current = token;
  for (;;) {
    const stripped = strip(current);
    const dotless = stripped.replace(/\.+$/, "");
    const next = dotless === "" || dotless.endsWith("/") ? stripped : dotless;
    if (next === current) return current;
    current = next;
  }
}

function oblCleanPathToken(word) {
  return oblPeelSentencePunctuation(word, (token) =>
    oblTrimEndMatches(oblTrimMatches(token, oblCharIn("`\"',:;")), oblCharIn("!?")));
}

function oblLooksLikeFilePath(path) {
  const dottedNumber = path.includes(".") && /[0-9]/.test(path) && /^[0-9.]*$/.test(path);
  return !path.includes("://") && !dottedNumber && path.split("/").pop().includes(".");
}

function oblCleanCueToken(word) {
  return oblTrimMatches(word, oblCharIn("`\"',:;.!?।॥")).toLowerCase();
}

function oblSafeRelativePath(path) {
  return !path.startsWith("/")
    && !path.startsWith("-")
    && !path.split("/").some((part) => part === ".." || part === "")
    && Array.from(path).every((character) => oblIsAlphanumeric(character) || "/._-".includes(character));
}

// Mirrors `fn first_prefix_lead_end`: `[start, end]` or null.
function oblFirstPrefixLeadEnd(lowered, role) {
  const markers = roleWordForms(role)
    .filter((form) => form.slot === "prefix" || form.slot === "circumfix")
    .map((form) => [oblTrim(form.before).toLowerCase(), oblTrim(form.after).toLowerCase()])
    .filter(([marker]) => marker !== "");
  let best = null;
  for (const [marker, closer] of markers) {
    let from = 0;
    for (;;) {
      const start = lowered.indexOf(marker, from);
      if (start < 0) break;
      const end = start + marker.length;
      const cjk = !marker.includes(" ") && !oblIsAscii(marker);
      const before = Array.from(lowered.slice(0, start)).pop();
      const after = Array.from(lowered.slice(end, end + 2))[0];
      const beforeOk = cjk || start === 0 || oblIsWhitespace(before);
      const afterOk = cjk || end === lowered.length || oblIsWhitespace(after) || /^[!-/:-@[-`{-~]$/.test(after);
      const closed = closer === "" || lowered.slice(end).includes(closer);
      if (beforeOk && afterOk && closed) {
        if (best === null || start < best[0] || (start === best[0] && end > best[1])) best = [start, end];
        break;
      }
      from = end;
    }
  }
  return best;
}

// Mirrors `fn content_lead_close`.
function oblContentLeadClose(lowered, from) {
  let best = null;
  for (const form of roleWordForms("file_write_content_lead")) {
    if (form.slot !== "circumfix") continue;
    const opener = oblTrim(form.before).toLowerCase();
    const closer = oblTrim(form.after).toLowerCase();
    if (!opener || !closer || !oblTrimEnd(lowered.slice(0, from)).endsWith(opener)) continue;
    const relative = lowered.slice(from).indexOf(closer);
    if (relative < 0) continue;
    if (best === null || from + relative < best) best = from + relative;
  }
  return best;
}

function oblCue(token, cues, fused, leading) {
  const cleaned = oblCleanCueToken(token.text);
  for (const cue of cues) {
    if (cleaned === cue) return [token.start, token.end];
    if (fused && containsCjk(cue)) {
      if (!leading && cleaned.endsWith(cue)) return [Math.max(0, token.end - cue.length), token.end];
      if (leading && cleaned.startsWith(cue)) return [token.start, token.start + cue.length];
    }
  }
  return null;
}

// Mirrors `fn ranked_bindings` over `fn write_bindings` (a stable sort by rank).
function oblRankedBindings(toks) {
  const families = [
    ["destination", oblBareSurfaces("file_write_destination_cue"), true],
    ["target", oblBareSurfaces("file_write_target_cue"), true],
    ["action", oblBareSurfaces("file_write_action_cue"), false],
  ];
  const out = [];
  toks.forEach((token, index) => {
    const cleaned = oblCleanPathToken(token.text);
    if (!oblLooksLikeFilePath(cleaned) || !oblSafeRelativePath(cleaned)) return;
    if (index > 0) {
      for (const [family, cues, fused] of families) {
        const span = oblCue(toks[index - 1], cues, fused, false);
        if (span) {
          out.push({ index, path: cleaned, cue_start: span[0], cue_end: span[1], family, cue_precedes: true });
          return;
        }
      }
    }
    const next = toks[index + 1];
    if (!next) return;
    for (const [family, cues, fused] of families.slice(0, 2)) {
      const span = oblCue(next, cues, fused, true);
      if (span) {
        out.push({ index, path: cleaned, cue_start: span[0], cue_end: span[1], family, cue_precedes: false });
        return;
      }
    }
  });
  const rank = (binding) => (binding.cue_precedes ? (binding.family === "action" ? 1 : 0) : 2);
  return out.map((binding, order) => ({ binding, order }))
    .sort((left, right) => rank(left.binding) - rank(right.binding) || left.order - right.order)
    .map(({ binding }) => binding);
}

function oblFirstActionCue(toks, from) {
  const actions = oblBareSurfaces("file_write_action_cue");
  for (const token of toks) {
    if (token.start < from) continue;
    const span = oblCue(token, actions, false, true);
    if (span) return span;
  }
  return null;
}

// Mirrors `fn clean_content`.
function oblCleanContent(raw) {
  const qualifiers = oblBareSurfaces("file_write_content_qualifier");
  let led = oblTrim(raw);
  for (;;) {
    const separated = oblTrim(oblTrimStartMatches(led, oblCharIn(":-—–")));
    const lowered = separated.toLowerCase();
    let shortened = separated;
    let shortest = null;
    for (const qualifier of qualifiers) {
      if (!lowered.startsWith(qualifier)) continue;
      const rest = oblTrimStart(separated.slice(qualifier.length));
      if (/^[:\-—–]/u.test(rest) && (shortest === null || oblUtf8Len(rest) < shortest)) {
        shortened = rest;
        shortest = oblUtf8Len(rest);
      }
    }
    if (shortened.length === led.length) break;
    led = shortened;
  }
  let result = led;
  const bytes = new TextEncoder().encode(led);
  if (bytes.length >= 6 && led.startsWith("```") && led.endsWith("```")) {
    result = oblTrim(led.slice(3, led.length - 3));
  } else if (bytes.length >= 2) {
    const first = bytes[0];
    if (first === bytes[bytes.length - 1] && (first === 0x60 || first === 0x22 || first === 0x27)) result = oblTrim(led.slice(1, -1));
  }
  return result ? result : null;
}

// Mirrors `fn composed_document_specification_span` in
// rust/src/agentic_coding/note_composition.rs.
function oblComposedDocumentSpecificationSpan(task) {
  const separators = wordsForRole("clause_continuation_marker");
  const fold = (text) => text.replace(/[A-Z]/g, (letter) => letter.toLowerCase());
  for (const sentence of oblSentences(task, OBL_SHELL_ENDS)) {
    const normalized = normalizePrompt(sentence.text);
    if (!lexiconMentionsRole("document_composition_action", normalized)
      || !lexiconMentionsRole("composed_document_kind", normalized)) continue;
    const lead = oblFirstPrefixLeadEnd(sentence.text.toLowerCase(), "file_write_content_lead");
    if (!lead || lead[1] > sentence.text.length) continue;
    const parts = sentence.text.slice(lead[1]).split(",").flatMap((span) => {
      const pieces = [""];
      for (const word of oblSplitWhitespace(span)) {
        if (separators.some((separator) => fold(word) === fold(separator))) {
          pieces.push("");
          continue;
        }
        const current = pieces[pieces.length - 1];
        pieces[pieces.length - 1] = current ? `${current} ${word}` : word;
      }
      return pieces;
    }).map((part) => oblTrim(oblTrimEndMatches(oblTrimStartMatches(oblTrim(part), oblCharIn(":-—–")), (c) => c === ".")))
      .filter((part) => part !== "");
    if (parts.length >= 2) return { start: sentence.start, end: sentence.end };
  }
  return null;
}

const oblSlice = (text, start, end) => (start <= end && end <= text.length ? text.slice(start, end) : null);

function oblEndOfStatement(request, from, limit) {
  const sentence = oblSentences(request, OBL_PROSE_ENDS).find((candidate) => from >= candidate.start && from < candidate.end);
  if (!sentence) return limit;
  const tail = oblSlice(request, from, sentence.end);
  const saysMore = tail !== null && Array.from(tail).some(oblIsAlphanumeric);
  const rest = from > request.length ? "" : request.slice(from);
  const breakAt = rest.indexOf("\n");
  const continues = breakAt >= 0 && from + breakAt < sentence.end && Array.from(rest.slice(breakAt)).some(oblIsAlphanumeric);
  return saysMore && !continues ? Math.min(sentence.end, limit) : limit;
}

function oblShareStatement(request, left, right) {
  const [from, limit] = left <= right ? [left, right] : [right, left];
  return from === limit || oblEndOfStatement(request, from, limit) === limit;
}

function oblNamesDeferredWorkProduct(content) {
  const lowered = oblTrimEnd(oblTrimEndMatches(oblTrim(content), oblCharIn(".!?。！？"))).toLowerCase();
  return roleWordForms("file_write_deferred_content_reference").some((form) => {
    if (form.slot === "bare") return lowered === String(form.text);
    if (form.slot !== "suffix") return false;
    const noun = oblTrimStart(form.after);
    if (!noun || !lowered.endsWith(noun)) return false;
    const before = lowered.slice(0, lowered.length - noun.length);
    return before === "" || !oblIsAsciiAlphanumeric(oblLastChar(before));
  });
}

const oblIsLiteralContent = (content) => Array.from(content).some(oblIsAlphanumeric);

// Mirrors `fn parse_write_request_bound`: the content the binding yields, or null.
function oblWriteRequestBound(request, toks, binding) {
  const lowered = request.toLowerCase();
  const clauseStart = binding.cue_precedes ? binding.cue_start : toks[binding.index].start;
  const specification = oblComposedDocumentSpecificationSpan(request);
  const authoritative = oblFirstPrefixLeadEnd(lowered, "file_write_authoritative_content_lead") !== null;
  const lead = oblFirstPrefixLeadEnd(lowered, "file_write_content_lead");
  if (lead) {
    const markerEnd = lead[1];
    const inside = specification && markerEnd >= specification.start && markerEnd < specification.end;
    if (!inside && oblShareStatement(request, markerEnd, clauseStart)) {
      const markerLeads = markerEnd <= clauseStart;
      const statementEnd = oblEndOfStatement(request, markerEnd, markerLeads ? clauseStart : request.length);
      const close = oblContentLeadClose(lowered, markerEnd);
      const markerSpan = oblSlice(request, markerEnd, close === null ? statementEnd : Math.min(close, statementEnd));
      if (!markerLeads || oblFirstActionCue(toks, 0) !== null) {
        const content = markerSpan === null ? null : oblCleanContent(markerSpan);
        if (content !== null && oblIsLiteralContent(content) && (!oblNamesDeferredWorkProduct(content) || authoritative)) return content;
      }
    }
  }
  let contentSpan;
  const actionEnd = oblFirstActionCue(toks, 0)?.[1] ?? null;
  if (binding.family === "destination" && binding.cue_precedes) {
    if (actionEnd === null || !(actionEnd <= clauseStart && oblShareStatement(request, actionEnd, clauseStart))) return null;
    contentSpan = oblSlice(request, actionEnd, clauseStart);
  } else if (binding.family === "destination") {
    const actionStart = oblFirstActionCue(toks, binding.cue_end)?.[0] ?? null;
    if (actionStart === null) return null;
    if (!(binding.cue_end <= actionStart && oblShareStatement(request, binding.cue_end, actionStart))) return null;
    contentSpan = oblSlice(request, binding.cue_end, actionStart);
  } else {
    const destinations = oblBareSurfaces("file_write_destination_cue");
    const valueLead = toks.slice(binding.index + 1).find((token) => destinations.includes(oblCleanCueToken(token.text)));
    if (!valueLead || actionEnd === null) return null;
    if (!(actionEnd <= clauseStart
      && oblShareStatement(request, actionEnd, clauseStart)
      && oblShareStatement(request, clauseStart, valueLead.start))) return null;
    contentSpan = request.slice(valueLead.end);
  }
  if (contentSpan === null) return null;
  const content = oblCleanContent(contentSpan);
  if (content === null) return null;
  const nonReferential = roleWordForms("non_referential_subject")
    .some((form) => form.slot === "bare" && content.toLowerCase() === String(form.text));
  return !nonReferential && !oblNamesDeferredWorkProduct(content) && oblIsLiteralContent(content) ? content : null;
}

// Mirrors `fn describes_code_to_author`: unquoted content without a content
// lead that names a code construct is code to author, not the file's bytes.
function oblDescribesCodeToAuthor(request, content) {
  return content !== ""
    && oblFirstPrefixLeadEnd(request.toLowerCase(), "file_write_content_lead") === null
    && !quotedTextSegments(request).some((segment) => segment.includes(content))
    && lexiconMentionsRole("coding_request_object", normalizePrompt(content));
}

// Mirrors `fn parse_command_output_request`: whether one is stated.
function oblCommandOutputRequest(request) {
  const toks = oblTokens(request);
  const runVerbs = terminalCommandVocabulary().runVerbs;
  const actions = oblBareSurfaces("file_write_action_cue");
  const targets = oblBareSurfaces("file_write_target_cue");
  const destinations = oblBareSurfaces("file_write_destination_cue");
  for (const run of toks.filter((token) => runVerbs.has(oblCleanCueToken(token.text)))) {
    const tail = request.slice(run.end);
    const leading = tail.length - oblTrimStart(tail).length;
    const quote = Array.from(tail.slice(leading))[0];
    if (quote === undefined) return false;
    if (quote !== "'" && quote !== "\"" && quote !== "`") continue;
    const body = tail.slice(leading + 1);
    const close = body.indexOf(quote);
    if (close < 0) continue;
    const command = oblTrim(body.slice(0, close));
    if (!command || /[\n\r\0]/.test(command)) continue;
    const suffix = request.slice(run.end + leading + 1 + close + 1);
    if (!oblMentionsBareRole(suffix, "file_write_command_output_reference")) continue;
    const suffixTokens = oblTokens(suffix);
    if (!suffixTokens.some((token) => actions.includes(oblCleanCueToken(token.text)))) continue;
    for (let index = 1; index < suffixTokens.length; index += 1) {
      const cleaned = oblCleanPathToken(suffixTokens[index].text);
      const cue = oblCleanCueToken(suffixTokens[index - 1].text);
      if (oblLooksLikeFilePath(cleaned) && oblSafeRelativePath(cleaned) && (targets.includes(cue) || destinations.includes(cue))) return true;
    }
  }
  return false;
}

// Mirrors `fn repository_work_reference`: a GitHub issue or pull URL, or null.
function oblRepositoryWorkReference(request) {
  for (const token of oblSplitWhitespace(request)) {
    const url = oblTrimMatches(token, oblCharIn("<>()[]{},;.\"'。，、；：（）「」«»।"));
    const path = url.startsWith("https://github.com/")
      ? url.slice("https://github.com/".length)
      : url.startsWith("http://github.com/") ? url.slice("http://github.com/".length) : null;
    if (path === null) continue;
    const segments = path.split("/");
    if (segments.length === 4 && segments[0] && segments[1] && (segments[2] === "issues" || segments[2] === "pull")
      && Array.from(segments[3]).every(oblIsAsciiDigit)) return url;
  }
  return null;
}

// Mirrors `fn objective_text`: the request after its line-anchored objective
// marker, trimmed; the request itself when it carries none.
function oblObjectiveText(request) {
  const lowered = request.toLowerCase();
  const lead = oblFirstPrefixLeadEnd(lowered, "request_objective_lead");
  if (!lead) return request;
  for (const character of Array.from(lowered.slice(0, lead[0])).reverse()) {
    if (character === "\n") break;
    if (!oblIsWhitespace(character)) return request;
  }
  return oblTrim(request.slice(lead[1]));
}

// The `GeneralPlanMode::slug` of the plan `compose_general_change_plan`
// composes for `fullRequest`, or null when it composes none.
function obligationPlanMode(fullRequest) {
  const request = oblObjectiveText(fullRequest);
  if (oblCommandOutputRequest(request)) return "command_output";
  const toks = oblTokens(request);
  for (const binding of oblRankedBindings(toks)) {
    const content = oblWriteRequestBound(request, toks, binding);
    if (content !== null) return oblDescribesCodeToAuthor(request, content) ? null : "literal_file";
  }
  const target = oblRepositoryWorkReference(request);
  return target !== null && oblMentionsBareRole(request, "software_authoring_action") ? "repository_work_item" : null;
}
