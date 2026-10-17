// Shared translation prompt parsing and complete lexical composition.
// Loaded with the other worker modules before requests are handled.

const ROLE_COMPOSITIONAL_LEMMA = "compositional_lemma";
const ROLE_COMPOSITIONAL_PHRASE = "compositional_phrase";
const ROLE_COMPOSITIONAL_GENITIVE_HEAD = "compositional_genitive_head";

function targetFirstTranslationSurface(trimmed, lower, markers) {
  const colon = trimmed.indexOf(":");
  if (colon !== -1) {
    const header = lower.slice(0, colon).trim();
    const targetFirst = markers.circumfixFrames.some(([prefix]) => {
      if (!header.startsWith(prefix)) return false;
      const target = header.slice(prefix.length).trim();
      return meaningsWithRole(ROLE_TRANSLATION_TARGET_MARKER).some((meaning) =>
        meaning.words.some((word) => word.toLowerCase() === target),
      );
    });
    if (targetFirst) return cleanUnquotedTranslationSurface(trimmed.slice(colon + 1));
  }
  return undefined;
}

function extractBetweenPrefixAndMarker(original, lower, prefix, marker) {
  if (!lower.startsWith(prefix)) return null;
  const afterPrefix = lower.slice(prefix.length);
  const markerIndex = afterPrefix.indexOf(marker);
  if (markerIndex === -1) return null;
  return cleanUnquotedTranslationSurface(
    original.slice(prefix.length, prefix.length + markerIndex),
  );
}

function cleanUnquotedTranslationSurface(candidate) {
  const cleaned = String(candidate || "").trim().replace(/[-–—:]\s*$/u, "").trim();
  if (!cleaned || /["'«»`“”‘’]/u.test(cleaned)) return null;
  return cleaned;
}

function normalizeComposableSurface(surface) {
  return String(surface || "")
    .trim()
    .replace(/[?!.。？！．]+$/u, "")
    .toLowerCase()
    .split(/\s+/u)
    .filter(Boolean)
    .join(" ");
}

function capitalizeAsciiFirst(surface) {
  const text = String(surface || "");
  if (!text) return "";
  return text[0].toUpperCase() + text.slice(1);
}

function translateRussianWordSequence(words) {
  const translated = [];
  for (let index = 0; index < words.length; index += 1) {
    const word = words[index];
    const next = words[index + 1];
    if (
      next &&
      roleListsSurface(ROLE_COMPOSITIONAL_GENITIVE_HEAD, "ru", word) &&
      roleActionSurfaceTranslation(ROLE_COMPOSITIONAL_LEMMA, "genitive", "ru", "en", next)
    ) {
      translated.push(
        roleSurfaceTranslation(ROLE_COMPOSITIONAL_LEMMA, "ru", "en", word),
        "of",
        roleActionSurfaceTranslation(ROLE_COMPOSITIONAL_LEMMA, "genitive", "ru", "en", next),
      );
      index += 1;
      continue;
    }
    const surface = roleSurfaceTranslation(ROLE_COMPOSITIONAL_LEMMA, "ru", "en", word);
    if (!surface) return null;
    translated.push(surface);
  }
  return capitalizeAsciiFirst(translated.join(" "));
}

function translateCompositionalSurface(surface, source, target) {
  const normalized = normalizeComposableSurface(surface);
  if (!normalized) return null;

  const direct =
    roleSurfaceTranslation(ROLE_COMPOSITIONAL_PHRASE, source, target, normalized) ||
    roleSurfaceTranslation(ROLE_COMPOSITIONAL_LEMMA, source, target, normalized);
  if (direct) return direct;

  if (source !== "ru" || target !== "en") return null;

  const phrase = roleSurfaceTranslation(ROLE_COMPOSITIONAL_PHRASE, "ru", "en", normalized);
  if (phrase) return phrase;

  const words = normalized.split(/\s+/u).filter(Boolean);
  if (words.length < 2 || words.length > 8) return null;
  return translateRussianWordSequence(words);
}

function targetTranslationSurfaceComplete(surface, target) {
  const text = String(surface || "").trim();
  const rejected=crateModule("crate/dialogue_formalization.mjs")
    .structuralRuleValues("target-surface","reject-fragment");
  if (!/\p{L}/u.test(text) || rejected.some(fragment=>text.includes(fragment))) return false;
  const cyrillic = /[\u0400-\u052f]/u.test(text);
  const devanagari = /[\u0900-\u097f]/u.test(text);
  const han = /[\u3400-\u9fff]/u.test(text);
  if (["en", "es", "fr", "de", "it", "pt"].includes(target)) return !cyrillic && !devanagari && !han;
  if (target === "ru") return cyrillic && !devanagari && !han;
  if (target === "hi") return devanagari && !cyrillic && !han;
  if (target === "zh") return han && !cyrillic && !devanagari;
  return true;
}

async function completeLexicalSpanTranslation(surface, source, target) {
  const parts = String(surface).match(/[^\s,-]+[\s,-]*/gu) || [];
  if (parts.length < 2 || parts.length > 8 || parts.join("") !== surface) return null;
  const translated = [];
  const provenance = [];
  for (let index = 0; index < parts.length;) {
    let found = false;
    for (let end = parts.length; end > index; end -= 1) {
      const span = parts.slice(index, end).join("");
      const core = span.replace(/[\s,-]+$/u, "");
      let unit = translateCompositionalSurface(core, source, target);
      let evidence = "seed:compositional";
      if (!unit && !/[\s,-]/u.test(core)) {
        const token = formalizeSurface(core, source);
        unit = token && deformalizeMeaning(token, target, core, source);
        evidence = token ? "seed:meaning:" + token : "wiktionary:" + source + ":" + core + "->" + target;
        if (!unit) {
          const observed = await discoverWiktionaryTranslation(core, source, target);
          // A component with multiple senses needs contextual evidence, not a guess.
          if (observed.candidates.length === 1) {
            unit = observed.candidates[0];
            evidence = observed.provenance.join(";");
          }
        }
      }
      if (!unit || !targetTranslationSurfaceComplete(unit, target)) continue;
      translated.push(unit + span.slice(core.length));
      provenance.push(evidence);
      index = end;
      found = true;
      break;
    }
    if (!found) return null;
  }
  return {surface: translated.join(""), provenance};
}
