// Resolve a documentation reference through the publisher's own Sphinx pages.
function documentationMethodProvider(project) {
  const [root, path] = crateModule("crate/seed.mjs").splitPipeList(handlerRulesTableValue("documentation-method-providers", project) || "");
  return root && path ? { root, path } : null;
}

function documentationMethodSymbol(value, minimumParts = 1) {
  return value.length <= 160 && value.split(".").length >= minimumParts
    && value.split(".").every(part => /^[a-z_][a-z0-9_]*$/iu.test(part));
}

function documentationMethodCandidates(text, provider, project, selector) {
  const wrapper = /^Search\.setIndex\(([\s\S]*)\);?$/u.exec(text.trim());
  if (!wrapper) return [];
  let docnames;
  try {
    ({ docnames } = JSON.parse(wrapper[1]));
  } catch {
    return [];
  }
  if (!Array.isArray(docnames) || docnames.some(name => typeof name !== "string")) return [];
  const prefix = provider.path + "/";
  const lower = selector.toLowerCase();
  const qualified = selector.includes(".");
  const matches = new Set();
  for (const name of docnames) {
    if (!name.startsWith(prefix)) continue;
    const symbol = name.slice(prefix.length);
    if (!symbol.startsWith(project + ".") || !documentationMethodSymbol(symbol, 2)) continue;
    const normalized = symbol.toLowerCase();
    if (qualified ? normalized === lower : normalized.endsWith("." + lower)) matches.add(symbol);
    if (matches.size === 5) break;
  }
  return [...matches].sort();
}

function documentationMethodDetails(text, symbol) {
  const short = symbol.slice(symbol.indexOf(".") + 1);
  const blocks = pageHtmlBlocks(text, pageFormalizationRules());
  const heading = blocks.findIndex(block => block.kind === "heading" && block.level === 1
    && block.text.trim().replace(/[¶#]$/u, "").trim() === symbol);
  if (heading < 0) return null;

  let signature = "";
  const summaries = [];
  let wordBudget = 24;
  for (const block of blocks.slice(heading + 1)) {
    if (block.kind === "heading") break;
    if (block.kind === "list_item") {
      const term = block.text.trim().split("[source]")[0].replace(/[¶#]$/u, "").trim();
      if ([short, symbol].some(name => term.startsWith(name + "(")) && term.length <= 360) {
        if (!signature) signature = term;
      } else if (signature) break;
    } else if (signature && block.kind === "paragraph") {
      const excerpt = block.text.trim().replace(/\s+/gu, " ");
      if (!excerpt || excerpt.length > 360) continue;
      const length = excerpt.split(" ").length;
      if (length > wordBudget) break;
      summaries.push(excerpt);
      wordBudget -= length;
      if (summaries.length === 2) break;
    }
  }
  return signature && summaries.length ? { signature, excerpt: summaries.join(" ") } : null;
}

async function tryDocumentationMethodExplanation(prompt, language, preferences = {}) {
  const hit = runHandlerRuleSet("docs_method_explanation", prompt, prompt.toLowerCase(), []);
  if (!hit) return null;
  language = FORCED_RESPONSE_LANGUAGE || crateModule("crate/language.mjs").detectProse(prompt) || language;
  const [selector, project] = claimOperandDocumentedMethod(prompt);
  const provider = documentationMethodProvider(project);
  const online = preferences.offline !== true;
  const events = [];
  const answer = (key, values, symbol = selector, confidence = 0, evidence = []) => ({
    ...hit, content: howResponse(key, language, values), confidence, formalizedObject: symbol,
    evidence: [...hit.evidence, ...evidence], solverEvents: events,
  });
  const acquire = async (url) => {
    const capture = await sourceWalkFetchCapture(url, { online, bounded: true });
    if (!capture.ok) events.push({ kind: "docs_method:source_failure", payload: capture.error });
    return capture;
  };
  const record = (capture) => {
    const provenance = `${capture.url} fetched_at=${capture.fetchedAt} sha256=${capture.sha256} cached=${capture.cached}`;
    events.push({ kind: "source:http", payload: provenance });
    if (capture.cached) events.push({ kind: "cache_hit", payload: capture.url });
  };
  const page = async (symbol) => {
    const source = `${provider.root}${provider.path}/${symbol}.html`;
    const capture = await acquire(source);
    if (!capture.ok) return null;
    const details = documentationMethodDetails(capture.text, symbol);
    if (!details) {
      events.push({ kind: "docs_method:source_failure", payload: "symbol_or_summary_mismatch" });
      return null;
    }
    record(capture);
    return answer("documentation-method-source-guidance", { ...details, method: symbol, source }, symbol, 0.9, [
      "docs_method:method:" + symbol, "docs_method:source_kind:official-docs", "source:" + source,
    ]);
  };
  if (selector.includes(".")) {
    const direct = await page(selector);
    if (direct) return direct;
  }

  const index = await acquire(provider.root + "searchindex.js");
  const matches = index.ok ? documentationMethodCandidates(index.text, provider, project, selector) : [];
  if (matches.length > 0) record(index);
  if (matches.length === 1 && matches[0] !== selector) {
    const resolved = await page(matches[0]);
    if (resolved) return resolved;
  }
  if (matches.length > 1) {
    return answer("documentation-method-ambiguous", { method: selector, candidates: matches.join(", ") },
      selector, 0, ["source:" + index.url]);
  }
  return answer("docs_method_explanation", { method: selector, project });
}
