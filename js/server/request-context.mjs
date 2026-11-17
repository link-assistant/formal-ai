// Request-local formalization, mirrored by rust/src/protocol/request_context.rs.
// Grammar and answer prose are shared data; no message is persisted here.
import { readRepoFile } from './lino.mjs';
import { plainText, chatPromptAndHistory } from './chat-request.mjs';
import { thinkingStep } from './thinking.mjs';

const rules = JSON.parse(readRepoFile('data/meta/request-context.json'));
const re = pattern => new RegExp(pattern, 'iu');
const captures = (name, text) => rules[name].map(pattern => re(pattern).exec(text)).find(Boolean);
const languageOf = value => Object.keys(rules.languages).find(code => rules.languages[code].includes(value));

export function formalizeRequest(messages) {
  const { prompt, history } = chatPromptAndHistory(messages);
  const constraints = [];
  messages.forEach((message, index) => {
    const role = message.role.toLowerCase();
    if (!['system', 'developer'].includes(role)) return;
    for (const text of plainText(message.content).split(re(rules.clauseSeparator)).map(text => text.trim()).filter(Boolean)) {
      const add = (kind, value) => constraints.push({ role, index, text, kind, value, status: 'pending' });
      if (re(rules.negated).test(text) && (re(rules.outputCue).test(text) || ['language', 'format', 'labels', 'length'].some(kind => captures(kind, text)))) {
        add('output', text);
        continue;
      }
      let recognized = false;
      const covered = [];
      for (const kind of ['language', 'format', 'labels', 'length']) {
        const match = captures(kind, text);
        if (!match) continue;
        let value = match[1].toLowerCase();
        if (kind === 'language' && rules.formats.includes(value)) continue;
        covered.push([match.index, match.index + match[0].length]);
        if (kind === 'labels') value += `|${match[2].toLowerCase()}`;
        if (kind === 'length') value += `|${rules.numberWords[match[2].toLowerCase()] ?? match[2]}`;
        add(kind, value);
        recognized = true;
      }
      const residue = text.split('').map((char, index) => covered.some(([start, end]) => index >= start && index < end) ? ' ' : char).join('');
      if ((!recognized && re(rules.outputCue).test(text)) || (recognized && !re(rules.ignorableResidue).test(residue))) add('output', text);
    }
  });
  // System outranks developer; the last instruction at the same rank wins.
  for (const item of constraints) {
    const winner = constraints.filter(other => other.kind === item.kind)
      .sort((a, b) => Number(a.role === 'system') - Number(b.role === 'system') || a.index - b.index || constraints.indexOf(a) - constraints.indexOf(b)).at(-1);
    if (winner !== item) item.status = 'superseded';
  }
  const active = kind => constraints.find(item => item.kind === kind && item.status !== 'superseded');
  const format = active('format')?.value;
  const preserveAnswerBody = Boolean(active('labels') || active('length') || (format && format !== rules.plainFormat));
  const answerText = format === rules.supportedFormat ? body => JSON.parse(body)[rules.jsonAnswerKey] : body => body;
  return { prompt, history, constraints, active, preserveAnswerBody, answerText, language: languageOf(active('language')?.value) };
}

function subject(text) {
  return text.toLowerCase().split(/\s+/u).map(word => rules.subjectAliases[word] ?? word)
    .filter(word => !rules.subjectModifiers.includes(word)).join(' ');
}

/** Short-circuit only a request with grounded dialogue data or a binary task. */
export function contextualAnswer(context) {
  const recall = captures('recall', context.prompt);
  if (recall) {
    for (let index = context.history.length - 1; index >= 0; index--) {
      const turn = context.history[index];
      if (turn.role !== 'user') continue;
      for (const sentence of turn.content.trim().split(re(rules.factSeparator)).reverse()) {
        const fact = re(rules.assertion).exec(sentence.trim());
        if (fact && subject(fact[1]) === subject(recall[1])) {
          return answer(fact[2], 'dialogue_fact', [`user[${index}]:${fact[1]}=${fact[2]}`]);
        }
      }
    }
    return answer(rules.missingFact, 'dialogue_fact_missing');
  }
  const labels = context.active('labels');
  if (labels?.value === rules.binaryLabels.join('|')) {
    const match = re(rules.comparison).exec(context.prompt);
    if (match) {
      const comparison = compareDecimals(match[1], match[3]);
      const relation = rules.relations[match[2].toLowerCase()];
      const result = relation === 'gt' ? comparison > 0 : relation === 'lt' ? comparison < 0 : comparison === 0;
      const symbolic = answer(rules.binaryLabels[result ? 0 : 1], 'request_comparison');
      symbolic.thinking_steps.push(thinkingStep(0, 'request_comparison', `${match[1]}:${relation}:${match[3]}:${result}`, 'high', 'request_context'));
      return symbolic;
    }
  }
  return null;
}

// Compare finite decimal strings exactly, without a float precision boundary.
function compareDecimals(left, right) {
  const parts = value => {
    const [whole, fraction = ''] = value.replace(/^-/, '').split('.');
    const integer = whole.replace(/^0+/u, '') || '0';
    const decimal = fraction.replace(/0+$/u, '');
    return { integer, decimal, negative: value.startsWith('-') && (integer !== '0' || decimal !== '') };
  };
  const a = parts(left), b = parts(right);
  if (a.negative !== b.negative) return a.negative ? -1 : 1;
  const width = Math.max(a.decimal.length, b.decimal.length);
  const compare = (a, b) => a === b ? 0 : a > b ? 1 : -1;
  const result = compare(a.integer.length, b.integer.length) || compare(a.integer, b.integer)
    || compare(a.decimal.padEnd(width, '0'), b.decimal.padEnd(width, '0'));
  return a.negative ? -result : result;
}

function answer(text, intent, facts = []) {
  return { answer: text, intent, confidence: 1, evidence_links: facts.map(fact => `request_fact:${fact}`),
    thinking_steps: facts.map((fact, index) => thinkingStep(index, 'request_fact', fact, 'high', 'request_context')),
    links_notation: `(request_context (intent ${intent}) (answer ${JSON.stringify(text)}))` };
}

export function composeFinalAnswer(messages, text, steps) {
  return composeRequestAnswer(messages, { ...answer(text, 'agentic_final'), thinking_steps: steps });
}

/** Compose the final symbolic answer, including cached/refused answers. */
export function composeRequestAnswer(messages, symbolic) {
  const context = formalizeRequest(messages);
  if (!context.constraints.length) return symbolic;
  const projection = symbolic.output_projection;
  let text = context.preserveAnswerBody && projection?.content === symbolic.answer ? projection.text : symbolic.answer;
  const languageText = text;
  if (context.active('format')?.value === rules.supportedFormat) text = JSON.stringify({ [rules.jsonAnswerKey]: text });
  const unmet = [];
  for (const item of context.constraints) {
    if (item.status === 'superseded') continue;
    let satisfied = false;
    if (item.kind === 'language') {
      // Only the seeded language override can authorize a localized answer.
      // English also rejects a handler that leaked another script's prose.
      satisfied = Boolean(languageOf(item.value)) && (context.language !== 'en' || !/[\p{Script=Cyrillic}\p{Script=Han}\p{Script=Devanagari}]/u.test(languageText))
        && (context.language !== 'ru' || /\p{Script=Cyrillic}/u.test(languageText) || !/\p{L}/u.test(languageText));
    }
    if (item.kind === 'labels') satisfied = item.value.split('|').includes(text.toLowerCase());
    if (item.kind === 'length') {
      const [mode, count] = item.value.split('|');
      const length = (text.match(new RegExp(rules.word, 'gu')) ?? []).length;
      const comparison = compareDecimals(String(length), count);
      satisfied = mode === 'exactly' ? comparison === 0 : comparison <= 0;
    }
    if (item.kind === 'format') satisfied = [rules.supportedFormat, rules.plainFormat].includes(item.value);
    item.status = satisfied ? 'applied' : 'unmet';
    if (!satisfied) unmet.push(item);
  }
  if (unmet.length) {
    const refuse = () => {
      const refusal = unmet.map(item => rules.refusal.replace('{kind}', item.kind).replace('{text}', item.text)).join(' ');
      return context.active('format')?.value === rules.supportedFormat ? JSON.stringify({ [rules.jsonAnswerKey]: refusal }) : refusal;
    };
    const labels = context.active('labels');
    if (labels?.status === 'applied') { labels.status = 'unmet'; unmet.push(labels); }
    const language = context.active('language');
    if (language?.status === 'applied' && context.language !== rules.refusalLanguage) {
      language.status = 'unmet'; unmet.push(language);
    }
    text = refuse();
    const length = context.active('length');
    if (length?.status === 'applied') {
      const [mode, count] = length.value.split('|');
      const words = (text.match(new RegExp(rules.word, 'gu')) ?? []).length;
      const comparison = compareDecimals(String(words), count);
      if (mode === 'exactly' ? comparison !== 0 : comparison > 0) {
        length.status = 'unmet'; unmet.push(length); text = refuse();
      }
    }
  }
  const steps = [...symbolic.thinking_steps];
  for (const item of context.constraints) {
    steps.push(thinkingStep(steps.length, 'request_constraint', `${item.role}[${item.index}]:${item.kind}:${item.status}:${item.text}`, 'high', 'request_context'));
  }
  return { ...symbolic, answer: text, thinking_steps: steps,
    intent: unmet.length ? 'request_constraint_refusal' : symbolic.intent,
    links_notation: `${symbolic.links_notation}\n(request_context (answer ${JSON.stringify(text)}))` };
}
