// Search results nominate pages. Only a matching procedure read from a page is evidence.
const indexedProcedureRankings = new Map();

function indexedProcedureCandidates(query, locale, results) {
  const key = `${locale}:${query}`;
  if (!results.length) return indexedProcedureRankings.get(key) || [];
  if (indexedProcedureRankings.size >= 64) indexedProcedureRankings.delete(indexedProcedureRankings.keys().next().value);
  indexedProcedureRankings.set(key, results);
  return results;
}

function indexedProcedureUrl(raw) {
  try {
    const url = new URL(raw);
    const host = url.hostname.replace(/\.$/u, "");
    if (url.protocol !== "https:" || url.port || url.username || url.password
        || !/^[a-z0-9.-]+\.[a-z][a-z0-9-]*$/u.test(host)
        || ["localhost", "local", "internal", "home.arpa"].some(suffix =>
          host === suffix || host.endsWith(`.${suffix}`))) return null;
    url.hash = "";
    return url;
  } catch {
    return null;
  }
}

function indexedProcedureMatches(task, title, language) {
  if (howToMatchesTask(task, title, language)) return true;
  if (language !== "en") return false;
  const infinitives = title.replace(/\b([a-z]{3,})ing\b/giu, (word, stem) => `${word} ${stem} ${stem}e`);
  return howToMatchesTask(task, infinitives, language);
}

function indexedProcedureHostAllowed(host, preferences) {
  return !sourceWalkRegistry().some(record => {
    if (!record.api || sourceWalkServiceAllowed(preferences, record)) return false;
    try {
      const publisher = new URL(record.api.replace("{language}", "en")).hostname;
      const domain = publisher.replace(new RegExp(`^(?:www|${record.apiLanguages.join("|") || "www"})\\.`, "u"), "");
      return host === domain || host.endsWith(`.${domain}`);
    } catch {
      return false;
    }
  });
}

function indexedSectionLists(html, limit) {
  const lists = [];
  let depth = 0;
  let stepStart;
  let steps = null;
  for (const tag of html.matchAll(/<(\/?)(ol|ul|li)\b[^>]*>/giu)) {
    const closing = Boolean(tag[1]);
    const kind = tag[2].toLowerCase();
    if (kind !== "li") {
      if (!closing && depth === 0) steps = kind === "ol" ? [] : null;
      depth += closing ? -1 : 1;
      if (depth < 0) return [];
      if (depth === 0 && steps?.length >= HOW_TO_MIN_ACCEPTED_STEPS) lists.push(steps);
    } else if (depth === 1 && steps) {
      if (!closing) stepStart = tag.index + tag[0].length;
      else if (stepStart !== undefined && steps.length < limit) {
        const item = html.slice(stepStart, tag.index).replace(/<svg\b[^>]*\baria-label="([^"]*)"[^>]*>[\s\S]*?<\/svg>/giu, " $1 ");
        const text = howToCompactStepText(item).replace(/\s+([,.!?])/gu, "$1");
        if (text && !steps.includes(text)) steps.push(text);
        stepStart = undefined;
      }
    }
  }
  return lists;
}

function indexedProcedureSteps(html, task, language, limit, host = "") {
  const main = html.match(/<main(?:\s[^>]*)?>(.*?)<\/main>/isu)?.[1] ?? html;
  const body = main.replace(/<!--[\s\S]*?-->|<(script|style|nav|aside|footer)\b[^>]*>[\s\S]*?<\/\1\s*>/giu, "");
  const headings = [...body.matchAll(/<h([1-6])(?:\s[^>]*)?>(.*?)<\/h[1-6]>/gisu)];
  const start = headings.findIndex(heading => heading[1] === "1");
  if (start < 0) return [];
  const action = howToTopicWords(task)[0];
  const labels = [];
  let procedure = [];
  for (let index = start; index < headings.length; index++) {
    const heading = headings[index];
    const level = Number(heading[1]);
    labels.length = level;
    labels[level - 1] = howToCompactStepText(heading[2]);
    const title = labels.join(" ");
    if (!indexedProcedureMatches(task, title, language)
        && !(action && indexedProcedureMatches(action, title, language)
          && indexedProcedureMatches(task, `${title} ${host}`, language))) continue;
    const section = body.slice(heading.index + heading[0].length, headings[index + 1]?.index);
    for (const steps of indexedSectionLists(section, limit)) {
      if (procedure.length && JSON.stringify(procedure) !== JSON.stringify(steps)) return [];
      procedure = steps;
    }
  }
  return procedure;
}

async function indexedProcedureGuide(guide, results, language, preferences) {
  const record = sourceWalkRegistry().find(item => item.id === "web_page");
  if (!record || !sourceWalkServiceAllowed(preferences, record)) return false;
  const candidates = new Map();
  for (const candidate of results) {
    const url = indexedProcedureUrl(candidate.url);
    if (url && indexedProcedureHostAllowed(url.hostname, preferences) && !candidates.has(url.href)) {
      candidates.set(url.href, url);
    }
  }
  for (const url of Array.from(candidates.values()).slice(0, guide.bounds.maxPagesPerService)) {
    const outcome = { sourceId: record.id, status: "no_steps", detail: url.href, pages: 1, steps: 0 };
    guide.outcomes.push(outcome);
    const capture = await sourceWalkFetchCapture(url.href, { indexed: true, bounded: true, online: preferences.offline !== true });
    if (!capture.ok) {
      outcome.status = "capture_failed";
      outcome.detail = `${url.href} ${capture.error}`;
      continue;
    }
    const steps = indexedProcedureSteps(capture.text, guide.task, language, guide.bounds.maxItems, url.hostname);
    if (!steps.length) continue;
    const found = [];
    howToPushSteps({ ...record, name: url.hostname }, capture, 1, steps, found);
    guide.steps = found;
    outcome.status = "contributed";
    outcome.steps = found.length;
    return true;
  }
  return false;
}
