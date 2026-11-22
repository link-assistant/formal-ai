// Generic first-person need -> registered goal -> source-backed practical guidance.
function practicalHelpPolicy(key) {
  return howPolicy("practical-help", key, {});
}

function practicalHelpValues(key) {
  const scalar = practicalHelpPolicy(key);
  const rows = handlerRulesTables()["practical-help"]?.rows ?? [];
  return scalar ? [scalar] : rows.filter(([name]) => name === key).map(([, value]) => value);
}

function practicalHelpForms(key) {
  return practicalHelpValues(key).map(normalizePrompt).filter(Boolean);
}

function practicalHelpContains(text, word) {
  if (!word) return false;
  const unspaced = /[\u3400-\u9fff]/u.test(word);
  const wordCharacter = /[\p{L}\p{N}_]/u;
  let from = 0;
  for (;;) {
    const start = text.indexOf(word, from);
    if (start < 0) return false;
    from = start + word.length;
    if (unspaced || (!wordCharacter.test(text[start - 1] || "")
        && !wordCharacter.test(text[from] || ""))) return true;
  }
}

function practicalHelpGoal(prompt) {
  if (/^\s*["'\x60\u00ab\u201c]/u.test(prompt)) return null;
  const text = normalizePrompt(prompt);
  const lead = practicalHelpForms("leads").sort((left, right) => right.length - left.length)
    .find(word => text.startsWith(word) && (/[\u3400-\u9fff]/u.test(word)
      || !/[\p{L}\p{N}_]/u.test(text[word.length] || "")));
  if (!lead) return null;
  const rest = text.slice(lead.length).trim();
  if (practicalHelpForms("negations").some(word => practicalHelpContains(rest, word))) return null;
  return practicalHelpValues("goals").find(goal => practicalHelpForms(goal + "-forms")
    .some(word => rest.startsWith(word) && practicalHelpContains(rest, word))) || null;
}
function practicalHelpTrustedSource(source) {
  try {
    const url = new URL(source);
    const hosts = practicalHelpValues("trusted-hosts");
    return url.protocol === "https:" && !url.username && !url.password
      && hosts.some(host => host.trim().toLowerCase() === url.hostname);
  } catch {
    return false;
  }
}

function practicalHelpExcerpt(text, goal) {
  return sourceSectionExcerpt(text, practicalHelpPolicy(goal + "-heading"),
    practicalHelpForms(goal + "-excerpt-terms"));
}

function sourceSectionExcerpt(text, title, terms) {
  const heading = normalizePrompt(title);
  const cues = terms.map(normalizePrompt);
  if (!heading || !cues.length || cues.some(word => !word) ||
      new TextEncoder().encode(text).length > HOW_TO_CAPTURE_MAXIMUM_BYTES) return "";
  let level = 0;
  for (const block of pageHtmlBlocks(text, pageFormalizationRules())) {
    if (block.kind === "heading") {
      if (level && block.level <= level) break;
      if (!level && normalizePrompt(block.text) === heading) level = block.level;
    } else if (level && ["paragraph", "list_item"].includes(block.kind)) {
      const excerpt = block.text.trim().replace(/\s+/gu, " ");
      if (!excerpt || [...excerpt].length > 480) continue;
      const normalized = normalizePrompt(excerpt);
      if (cues.every(word => practicalHelpContains(normalized, word))) return excerpt;
    }
  }
  return "";
}

async function practicalHelpAnswer(goal, language, preferences = {}) {
  const source = practicalHelpPolicy(goal + "-source");
  const online = preferences.offline !== true;
  let capture = { error: "untrusted_source" };
  if (practicalHelpTrustedSource(source)) {
    capture = await sourceWalkFetchCapture(source, { online, bounded: true });
    if (!capture.ok && online) {
      const recovered = await howToCaptureThroughComponent(source, []);
      if (recovered) {
        capture = { ...recovered, ok: true };
        sourceWalkCaptureCache.set(source, Object.freeze(capture));
      }
    }
  }
  const excerpt = capture.text ? practicalHelpExcerpt(capture.text, goal) : "";
  const mode = excerpt ? (capture.cached ? "cached-source" : "fetched-source") : "unavailable";
  const key = excerpt ? "practical-help-source-guidance" : practicalHelpPolicy(goal + "-unavailable");
  const evidence = ["meaning:practical_need", "practical_help:goal:" + goal, "practical_help:source_mode:" + mode];
  const solverEvents = [{ kind: "practical_help:goal", payload: goal }];
  if (excerpt) {
    const provenance = `${source} fetched_at=${capture.fetchedAt} sha256=${capture.sha256} cached=${capture.cached}`;
    solverEvents.push({ kind: "source:http", payload: provenance });
    if (capture.cached) solverEvents.push({ kind: "cache_hit", payload: source });
    solverEvents.push({ kind: "practical_help:source", payload: source });
    evidence.push("source:" + source);
  } else {
    solverEvents.push({ kind: "practical_help:source_failure", payload: capture.error || "no_matching_excerpt" });
  }
  solverEvents.push({ kind: "practical_help:source_mode", payload: mode });
  return {
    intent: "procedural_how_to", content: howResponse(key, language, { source, excerpt }),
    confidence: excerpt ? 0.8 : 0, evidence, solverEvents,
  };
}
async function tryPracticalHelp(prompt, language, preferences = {}) {
  const goal = practicalHelpGoal(prompt);
  return goal ? practicalHelpAnswer(goal, language, preferences) : null;
}

async function tryPracticalHelpFollowup(prompt, language, history, preferences = {}) {
  const goal = practicalHelpFollowupGoal(normalizePrompt(prompt), lastHistoryTurn(history, "user"));
  return goal ? practicalHelpAnswer(goal, language, preferences) : null;
}

function practicalHelpFollowupGoal(normalized, prior) {
  if (!prior || (!practicalHelpForms("followups").includes(normalized)
    && !isProceduralElaborationRequest(normalized))) return null;
  return practicalHelpGoal(prior);
}
