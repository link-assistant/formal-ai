import { cached } from '../agentic/host.mjs';
import { agenticMessage } from '../agentic/messages.mjs';

const NUMBER = '([0-9]+(?:[.,][0-9]+)?)';
const DATE = '([0-9]{4}(?:-[0-9]{2}-[0-9]{2})?)';
const TOKEN = /[\p{L}\p{N}_]+(?:[.-][\p{L}\p{N}_]+)*(?:\+\+|#)?/gu;
const CYRILLIC = /^\p{Script=Cyrillic}+$/u;

function pattern(rule, replacements, flags = 'iu') {
  return cached('structured-verdict-' + rule + '-' + flags, () =>
    new RegExp(agenticMessage('structured-verdict-' + rule, replacements), flags));
}

function trimSpan(text) {
  return text.trim().replace(/[.!?;]+$/u, '').trimEnd();
}

function sections(source) {
  const markers = Array.from(source.matchAll(pattern('sections', {}, 'giu')));
  const byKind = [[], [], []];
  for (const [index, marker] of markers.entries()) {
    const kind = marker[1] ? 0 : marker[2] ? 1 : 2;
    const text = trimSpan(source.slice(marker.index + marker[0].length, markers[index + 1]?.index));
    byKind[kind].push(text);
  }
  return byKind;
}

function wordMatches(left, right) {
  if (left === right) return true;
  if (!CYRILLIC.test(left) || !CYRILLIC.test(right)) return false;
  const endings = pattern('russian-ending');
  const stem = left.replace(endings, '');
  return stem.length >= 4 && (stem === right || stem === right.replace(endings, ''));
}

function subjectSpans(text, subject) {
  const tokens = Array.from(text.matchAll(TOKEN));
  const matches = [];
  for (const [index, token] of tokens.entries()) {
    const last = tokens[index + subject.length - 1];
    if (!last || !subject.every((word, offset) =>
      wordMatches(word, tokens[index + offset][0].toLowerCase()))) continue;
    matches.push([token.index, last.index + last[0].length]);
  }
  return matches;
}

function itemMatches(item, subject) {
  const phrase = item.trim().replace(/^["'“«]+/u, '');
  return subjectSpans(phrase, subject).some(([start, end]) => {
    const tail = phrase.slice(end).trim().replace(/^["'”»]+/u, '').trimStart();
    return start === 0 && (!tail || pattern('item-tail').test(tail));
  });
}

function classify(claim, subject) {
  const cues = Array.from(claim.matchAll(pattern('use', {}, 'giu')));
  let positive = false;
  let negative = false;
  let excluded = false;
  for (const [index, cue] of cues.entries()) {
    const raw = claim.slice(cue.index + cue[0].length, cues[index + 1]?.index).trim().replace(/,$/u, '');
    const restricted = raw.replace(pattern('only'), '').trim();
    const matched = restricted.split(pattern('list')).some(item => itemMatches(item, subject));
    if (cue[1]) negative ||= matched;
    else {
      positive ||= matched;
      excluded ||= Boolean(restricted && restricted !== raw && !matched);
    }
  }
  if (!cues.length) {
    const restricted = claim.replace(pattern('only'), '').trim();
    if (restricted && restricted !== claim) {
      positive = restricted.split(pattern('list')).some(item => itemMatches(item, subject));
      excluded = !positive;
    }
    positive ||= subjectSpans(claim, subject).some(([start, end]) =>
      pattern('role-prefix').test(claim.slice(0, start))
      || (start === 0 && pattern('role').test(claim.slice(end).trimStart())));
  }
  return { positive, negative, excluded };
}

function date(text) {
  if (!text || !/^[0-9]{4}(?:-[0-9]{2}-[0-9]{2})?$/u.test(text)) return undefined;
  const [year, month, day] = text.split('-').map(Number);
  if (!year) return undefined;
  if (month === undefined) return [year];
  const leap = year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);
  const days = [31, leap ? 29 : 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
  return day >= 1 && day <= days[month - 1] ? [year, month * 100 + day] : undefined;
}

function durationVerdict(minimum, start, end) {
  if (!start || !end) return undefined;
  const years = end[0] - start[0];
  if (years < 0 || (years === 0 && end[1] < start[1])) return undefined;
  if ((start.length === 1) === (end.length === 1)) return years - Number(end[1] < start[1]) >= minimum;
  // With mixed precision, require the threshold to hold for every possible anniversary.
  const earliestStart = start[1] ?? 101;
  const latestStart = start[1] ?? 1231;
  const earliestEnd = end[1] ?? 101;
  const latestEnd = end[1] ?? 1231;
  if (years - Number(earliestEnd < latestStart) >= minimum) return true;
  if (years - Number(latestEnd < earliestStart) < minimum) return false;
  return undefined;
}

function outcome(result, rule, evidence, reference) {
  return { result, trace: { rule, evidence, ...(reference ? { reference } : {}) } };
}

function durationEvidence(claim, subject, minimum, reference) {
  const periods = Array.from(claim.matchAll(pattern('period', { date: DATE }, 'giu')));
  const results = [];
  for (const [index, period] of periods.entries()) {
    const [, englishStart, russianStart, end, since] = period;
    const previous = periods[index - 1];
    const before = claim.slice(previous ? previous.index + previous[0].length : 0, period.index).trim();
    const after = claim.slice(period.index + period[0].length, periods[index + 1]?.index).trim();
    const nearest = before.split(pattern('list')).at(-1);
    const attached = before
      ? classify(nearest, subject).positive || itemMatches(nearest, subject)
      : classify(after, subject).positive;
    if (!attached) continue;
    if (since && !reference) {
      results.push(outcome(null, 'minimum_duration_missing_reference', claim));
      continue;
    }
    const verdict = durationVerdict(minimum, date(englishStart ?? russianStart ?? since), date(end ?? reference));
    results.push(outcome(verdict ?? null, verdict === undefined ? 'minimum_duration_incomplete' : 'minimum_duration',
      claim, since ? reference : undefined));
  }
  return results;
}

export function criterionVerdict(source) {
  const [criteria, texts, references] = sections(source);
  if (criteria.length !== 1 || texts.length !== 1) return undefined;
  return assessCriterion(criteria[0], texts[0], references.length === 1 ? references[0] : undefined);
}

export function criterionReference(header) {
  const references = sections(header)[2];
  return references.length === 1 ? references[0] : undefined;
}

function subjectWords(target) {
  return Array.from(target.toLowerCase().matchAll(TOKEN), match => match[0]);
}

export function mentionsSubject(text, target) {
  const subject = subjectWords(target);
  return subject.length > 0 && subjectSpans(text, subject).length > 0;
}

export function assessCriterion(criterion, text, reference) {
  const leading = pattern('minimum-leading', { number: NUMBER }).exec(criterion);
  const trailing = pattern('minimum-trailing', { number: NUMBER }).exec(criterion);
  const minimum = (leading?.[1] ?? leading?.[2] ?? trailing?.[2])?.replace(',', '.');
  const cue = pattern('use').exec(criterion);
  const target = leading?.[3] ?? trailing?.[1] ?? (cue?.index === 0 && cue[2]
    ? criterion.slice(cue[0].length) : criterion.replace(pattern('experience-tail'), ''));
  const subject = subjectWords(target);
  if (!subject.length || (minimum !== undefined && !Number.isFinite(Number(minimum)))) return undefined;
  const claims = text.split(pattern('claims')).map(trimSpan).filter(Boolean);
  const classifications = claims.map(claim => ({ claim, ...classify(claim, subject) }));
  const negative = classifications.find(item => item.negative);
  const positive = classifications.find(item => item.positive);
  if (negative && positive) return outcome(null, 'conflicting_criterion_evidence', text);
  if (negative) return outcome(false, 'skill_negation', negative.claim);
  if (minimum !== undefined) {
    const durations = claims.flatMap(claim => durationEvidence(claim, subject, Number(minimum), reference));
    const sufficient = durations.find(item => item.result === true);
    if (sufficient) return sufficient;
    if (durations.length === 1) return durations[0];
    if (durations.length > 1) return outcome(null, 'minimum_duration_incomplete', text);
  } else if (positive) {
    return outcome(true, 'skill_use', positive.claim);
  }
  const exclusive = classifications.find(item => item.excluded);
  if (!positive && exclusive) return outcome(false, 'skill_exclusion', exclusive.claim);
  return outcome(null, 'no_criterion_evidence', text);
}
