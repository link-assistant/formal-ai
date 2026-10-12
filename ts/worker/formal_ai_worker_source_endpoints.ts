// Bind registry endpoint templates and their explicitly declared resource fallback.
function sourceWalkPageTitle(subject, hyphenated = false) {
  return String(subject || "").trim()
    .split(/[\s\x00-\x2f\x3a-\x40\x5b-\x60\x7b-\x7f]+/u)
    .filter(Boolean)
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(hyphenated ? "-" : " ");
}

function sourceWalkEntryUrl(record, subject, language = "") {
  if (language && record.apiLanguages.length > 0 && !record.apiLanguages.includes(language)) {
    return null;
  }
  const primary = record.apiLanguages[0] || "";
  const template = record.languageApi && language && language !== primary
    ? record.languageApi
    : record.api;
  const host = String(template || "").split("://")[1]?.split("/")[0] || "";
  const bindings = {
    title: sourceWalkPageTitle(subject, host.includes("wikihow")),
    query: subject,
    lemma: subject,
    language: language || primary,
  };
  let url = String(template || "");
  for (const [name, value] of Object.entries(bindings)) {
    if (value) url = url.split(`{${name}}`).join(sourceWalkPercentEncode(value));
  }
  return url && !url.includes("{") ? url : null;
}


function sourceWalkFallbackEntryUrl(record, subject, language = "") {
  if (!record.apiFallback) return null;
  return sourceWalkEntryUrl({ ...record, api: record.apiFallback, languageApi: "" }, subject, language);
}
