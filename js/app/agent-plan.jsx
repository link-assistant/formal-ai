// Agent mode: decomposes a multi-step request into an ordered task plan.

import { normalizePrompt } from "./local-prompts.jsx";

// Issue #27: agent-mode task decomposition. Splits a multi-step prompt into
// sequential sub-tasks on a small, deterministic set of separators that span
// the languages the demo already supports. The split is intentionally
// conservative — if no separator is present we return [trimmedPrompt] so a
// single-step task still runs through the same code path.
const AGENT_STEP_SEPARATORS = [
  /\s*;\s+/,
  /\s*,\s+(?:and\s+then|then|next)\s+/i,
  /\s*,\s+after\s+that\s+/i,
  /\s+then(?:\s*,)?\s+/i,
  /\s+потом\s+/i,
  /\s+затем\s+/i,
  /\s+после\s+этого\s+/i,
  /\s+然后\s*/,
  /\s+接着\s*/,
];

// Issue #27: leading conjunctions ("then", "and then", "потом", "затем",
// "next", "after that", "然后", "接着") are linkers between steps, not part of
// the task itself. Strip them so each split segment is a clean instruction.
const AGENT_LEADING_CONJUNCTIONS =
  /^(?:and\s+then|then|next|after\s+that|потом|затем|после\s+этого|然后|接着)[\s,:]+/i;

function isAgentFormattingDirective(segment) {
  const normalized = normalizePrompt(segment);
  if (!normalized) return false;
  return /^(?:format|return|output|respond|write|show)\s+(?:(?:this|that|it|the result|the answer|the information|the output)\s+)?(?:as|in)\s+(?:a\s+|an\s+)?(?:json object|json|markdown table|table|csv|yaml|xml)$/.test(normalized);
}

const AGENT_QUOTED_PHRASE_PATTERN =
  /"([^"]+)"|'([^']+)'|`([^`]+)`|“([^”]+)”|«([^»]+)»/g;

function extractAgentQuotedPhrases(text) {
  const phrases = [];
  for (const match of String(text || "").matchAll(AGENT_QUOTED_PHRASE_PATTERN)) {
    const phrase = (match.slice(1).find((value) => value !== undefined) || "").trim();
    if (phrase) phrases.push(phrase);
  }
  return phrases;
}

function agentResearchCommandPrefix(segment) {
  const text = String(segment || "").trim().toLowerCase();
  if (!/^(?:search|find|look up|lookup|research)\b/.test(text)) return "";
  if (/\bwikipedia\b/.test(text)) return "Search Wikipedia for";
  if (/\bwikidata\b/.test(text)) return "Search Wikidata for";
  if (/\bwiktionary\b/.test(text)) return "Search Wiktionary for";
  return "Search the web for";
}

function agentComparisonFocus(segment) {
  const text = String(segment || "");
  if (!/\bcompare\b/i.test(text)) return "";
  const numberedTarget = text.match(
    /\bnumber\s+of\s+([A-Za-z][A-Za-z -]{0,40}?)(?:[.?!,;:]|$)/i,
  );
  if (numberedTarget) {
    const focus = numberedTarget[1]
      .replace(/\b(?:their|his|her|its|the)\b/gi, "")
      .replace(/\s+/g, " ")
      .trim();
    if (focus) return focus;
  }
  if (/\bpatents?\b/i.test(text)) return "patents";
  return "";
}

function expandAgentResearchStep(segment) {
  const commandPrefix = agentResearchCommandPrefix(segment);
  if (!commandPrefix) return [segment];
  const quotedPhrases = extractAgentQuotedPhrases(segment);
  if (quotedPhrases.length < 2) return [segment];
  const focus = agentComparisonFocus(segment);
  if (!focus) return [segment];
  const focusKey = focus.toLowerCase();
  return quotedPhrases.map((phrase) => {
    const query = phrase.toLowerCase().includes(focusKey)
      ? phrase
      : `${phrase} ${focus}`;
    return `${commandPrefix} "${query}"`;
  });
}

export function decomposeAgentTask(text) {
  const trimmed = String(text || "").trim();
  if (!trimmed) return [];
  let segments = [trimmed];
  for (const sep of AGENT_STEP_SEPARATORS) {
    const next = [];
    for (const segment of segments) {
      const parts = segment.split(sep);
      for (const part of parts) {
        const cleaned = part.trim();
        if (cleaned) next.push(cleaned);
      }
    }
    segments = next;
  }
  const cleanedSegments = segments.map((segment) =>
    segment.replace(AGENT_LEADING_CONJUNCTIONS, "").trim(),
  ).filter((segment) => segment.length > 0);
  const mergedSegments = [];
  for (const segment of cleanedSegments) {
    if (isAgentFormattingDirective(segment) && mergedSegments.length > 0) {
      const previous = mergedSegments[mergedSegments.length - 1].replace(/\s*[.。]\s*$/u, "");
      mergedSegments[mergedSegments.length - 1] = `${previous}. Then ${segment}`;
    } else {
      mergedSegments.push(segment);
    }
  }
  return mergedSegments.flatMap((segment) => expandAgentResearchStep(segment));
}
