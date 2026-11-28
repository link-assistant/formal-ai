// Repository metadata and the shared project-statement summarizer.

async function verifiedProjectDescription(project, preferences, evidence) {
  const endpoint = githubApiRepositoryUrl({ owner: project.org, name: project.name });
  const online = preferences?.offline !== true && preferences?.allowNetwork !== false;
  const capture = await sourceWalkFetchCapture(endpoint, { online, bounded: true, indexed: true, maxAgeSeconds: 86400 });
  if (!capture.ok) return null;
  let metadata;
  try { metadata = JSON.parse(capture.text); } catch { return null; }
  if (typeof metadata?.full_name !== "string" || typeof metadata.html_url !== "string" ||
      metadata.full_name.toLowerCase() !== projectRepoSlug(project).toLowerCase() ||
      metadata.html_url.toLowerCase() !== project.url.toLowerCase()) return null;
  const body = typeof metadata.description === "string" ? metadata.description.trim() : "";
  if (!body) return null;
  evidence.push(`source:http:${endpoint} fetched_at=${capture.fetchedAt} sha256=${capture.sha256} cached=${capture.cached}`);
  if (capture.cached) evidence.push(`cache_hit:${endpoint}`);
  return { body, source: endpoint };
}

function unavailableProjectDescription(language, evidence) {
  evidence.push("project:description:unavailable");
  return answerFor("project-description-unavailable", language);
}

async function summarizeCapturedProject(project, language, preferences, evidence) {
  const verified = await verifiedProjectDescription(project, preferences, evidence);
  if (!verified) return unavailableProjectDescription(language, evidence);
  const summary = crateModule("crate/summarization.mjs");
  const configuration = summary.withMode(summary.withLanguage(summary.defaultConfig(), language), "short");
  const rendered = summary.deformalize(summary.summarize(summary.formalize(verified.body), configuration));
  if (rendered) evidence.push("summarization:mode:short", `summarization:language:${language}`);
  return rendered || verified.body;
}

function describeProjectRecord(project, language, mode = "short") {
  const summary = crateModule("crate/summarization.mjs");
  const config = summary.withMode(summary.withLanguage(summary.defaultConfig(), language), mode);
  const statements = projectStatementsFor(project, language).map(summary.statementFromSeed);
  return summary.deformalize(summary.summarize(statements, config));
}

function projectSummaryResponse(intent, language, values) {
  const template = crateModule("crate/seed.mjs").localizedResponse(intent, language);
  if (template === null) return null;
  return handlerRulesFillOnce(template, values);
}
