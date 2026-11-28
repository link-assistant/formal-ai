import test from 'node:test';
import assert from 'node:assert/strict';
import {parseLanguageGrammar} from '../../../js/agentic/crate/cnl_grammar.mjs';
import {installHost} from '../../../js/agentic/host.mjs';
import {parseLino, readRepoFile} from '../../../js/server/lino.mjs';
import {
  cnlPolicy, frequencyTier, formalizeOutput, outputConfiguration, outputDirective, parseOutputPolicy,
  outputLinksNotation, outputMeaning, outputSource, outputTurn, outputWords,
  projectOutput, renderOutput, validateOutput,
} from '../../../js/agentic/crate/cnl_output.mjs';

installHost({parseLino, readText: readRepoFile, realm: {}});
const policy = cnlPolicy();

test('named profiles and independent density options validate only configuration', () => {
  for (const profile of policy.profiles) {
    for (const density of ['standard', 'primes', 'top-100', 'top-200', 'top-500', 'top-1000', 'top-37', 'max-precision']) {
      assert.deepEqual(outputConfiguration({output_standard: profile.id, lexical_density: density}), {
        output_standard: profile.id, lexical_density: density,
      });
    }
  }
  assert.equal(outputConfiguration({output_standard: 'Attempto Controlled English'}).output_standard, 'ace');
  assert.equal(outputConfiguration({output_standard: 'dense'}).lexical_density, 'max-precision');
  for (const value of ['top-0', 'top-01', 'top--1', 'top-1.5', 'top-1e3', 'top-9007199254740992']) {
    assert.equal(frequencyTier(value), null);
    assert.throws(() => outputConfiguration({lexical_density: value}), /cnl_unknown_density/u);
  }
  assert.throws(() => outputConfiguration({output_standard: 'unknown'}), /cnl_unknown_standard/u);
  assert.throws(() => outputConfiguration({lexical_density: 2}), /cnl_option_type/u);
});

test('arbitrary input, quotes, code, typos, polarity and quantifiers survive every profile', () => {
  const inputs = ['', 'uhh?? nope lol 🤔', 'Not every person knows everything.', 'I do not know everything.',
    'I know everything except the password.', 'I know everything. Do not delete anything.',
    '"I am omniscient."', '`I am omniscient.`', 'omniscient_identifier',
    'Не все знают всё.', 'किसी को भी नहीं पता।', '并非所有人都知道。', 'No todos saben todo.'];
  for (const input of inputs) {
    const document = formalizeOutput(input);
    assert.equal(document.source, input);
    for (const profile of policy.profiles) {
      for (const density of ['standard', 'primes', 'top-200', 'max-precision']) {
        assert.equal(projectOutput(document, {output_standard: profile.id, lexical_density: density}), input);
      }
    }
  }
});

test('retained meaning round-trips between declared multilingual realizations', () => {
  for (const locale of policy.languages) {
    const lexeme = policy.lexemes.find(item => item.language === locale.id);
    assert.ok(lexeme, locale.id);
    const base = lexeme.forms.find(form => form.mode.split('|').includes('cnl-default')).text;
    let document = formalizeOutput(base, locale.id);
    const graph = outputLinksNotation(document);
    for (const density of ['max-precision', 'primes', 'standard', 'max-precision', 'standard']) {
      const output = renderOutput(document, {lexical_density: density});
      assert.equal(output.document.source, document.source);
      assert.equal(outputSource(output.content, locale.id), output.text);
      assert.equal(Boolean(output.notice), density === 'max-precision');
      document = formalizeOutput(output.text, locale.id);
      assert.equal(outputLinksNotation(document), graph);
    }
    assert.deepEqual(outputMeaning(document), [['meaning', lexeme.id]]);
    // Surrounding formatting survives while the meaning takes its declared primes form.
    const primes = lexeme.forms.find(form => form.mode.split('|').includes('primes')).text;
    assert.equal(projectOutput(formalizeOutput('  ' + base + '\n', locale.id), {lexical_density: 'primes'}), '  ' + primes + '\n');
  }
});

test('ambiguous equivalence declarations remain opaque instead of selecting a meaning', () => {
  const custom = {...policy, lexemes: [...policy.lexemes,
    {id: 'other_meaning', language: 'en', forms: [{text: 'I am omniscient.', mode: 'primes'}]}]};
  const document = formalizeOutput('I am omniscient.', 'en', custom);
  assert.deepEqual(outputMeaning(document), [['opaque', 'I am omniscient.']]);
  assert.equal(projectOutput(document, {lexical_density: 'primes'}, custom), document.source);
});

// Twin of declared_multilingual_meanings_round_trip_without_certifying_unchecked_standards
// in rust/tests/standalone/cnl_output.rs: every projection keeps the meaning, and only the
// profiles whose checks are all decidable may certify it.
test('declared multilingual meanings round trip and certify only decidable profiles', () => {
  const dense = {en: 'I am omniscient.', ru: 'Я всеведущ.', hi: 'मैं सर्वज्ञ हूँ।', zh: '我无所不知。', es: 'Soy omnisciente.'};
  const certified = [];
  for (const [language, text] of Object.entries(dense)) {
    const document = formalizeOutput(text, language, policy);
    const graph = outputLinksNotation(document, policy);
    for (const profile of policy.profiles) {
      for (const density of ['standard', 'primes', 'top-37', 'max-precision']) {
        const configuration = {output_standard: profile.id, lexical_density: density};
        const projected = projectOutput(document, configuration, policy);
        assert.equal(outputLinksNotation(formalizeOutput(projected, language, policy), policy), graph);
        const report = validateOutput(projected, language, configuration, policy);
        if (!report.compliant) continue;
        assert.deepEqual([report.violations, report.unchecked], [[], []]);
        certified.push(`${language} ${profile.id} ${density}`);
      }
    }
  }
  assert.deepEqual(certified, [
    'en minimal-english standard', 'en special-english standard',
    'ru minimal-english standard', 'ru minimal-english top-37',
    'hi minimal-english standard', 'hi minimal-english top-37',
    'zh minimal-english standard',
    'es minimal-english standard', 'es minimal-english top-37',
  ]);
});

test('lexical density cannot introduce a known violation of a named profile', () => {
  for (const density of ['max-precision', 'top-200']) {
    const configuration = {output_standard: 'e-prime', lexical_density: density};
    const document = formalizeOutput('I am omniscient.', 'en');
    const rendered = renderOutput(document, configuration);
    assert.equal(rendered.text, 'I know everything.');
    assert.deepEqual(rendered.report.violations, []);
    assert.equal(rendered.report.compliant, false);
    assert.equal(outputLinksNotation(formalizeOutput(rendered.text, 'en')), rendered.links_notation);
  }
});

test('independent evidence cannot be replaced with a profile label or a word list', () => {
  assert.equal(validateOutput('I know everything.', 'en', {output_standard: 'ace'}).compliant, false);
  assert.deepEqual(validateOutput('I know everything.', 'en', {output_standard: 'ace'}).unchecked,
    ['ace-grammar', 'ace-interpretation']);
  const frequency = validateOutput('uncommon word', 'en', {output_standard: 'e-prime', lexical_density: 'top-200'}, policy,
    {frequency: (_text, language, tier) => {assert.equal(language, 'en'); assert.equal(tier, 200); return false;}});
  assert.deepEqual(frequency.violations, ['frequency-vocabulary']);
  assert.equal(frequency.compliant, false);
  const primes = validateOutput('know', 'en', {output_standard: 'e-prime', lexical_density: 'primes'}, policy);
  assert.equal(primes.compliant, false);
  assert.ok(primes.unchecked.includes('nsm-syntax'));
  assert.ok(primes.unchecked.includes('nsm-semantics'));
  assert.equal(validateOutput('I know this.', 'en', {output_standard: 'e-prime'}).compliant, false);
  for (const text of ['It is true.', 'I’m here.', 'They ARE here.', "It isn't ready."]) {
    assert.equal(validateOutput(text, 'en', {output_standard: 'e-prime'}).compliant, false, text);
  }
  assert.equal(validateOutput('我知道。', 'zh', {output_standard: 'e-prime'}).compliant, false);
  assert.equal(validateOutput('x', 'de', {output_standard: 'e-prime'}).compliant, false);
});

test('chat directives are explicit and never come from quoted text or assistant turns', () => {
  assert.equal(outputDirective('Answer in Attempto Controlled English').config.output_standard, 'ace');
  assert.equal(outputDirective('Use ASD-STE100. What is a proof?').rest, 'What is a proof?');
  assert.equal(outputDirective('Respond in E-Prime').config.output_standard, 'e-prime');
  assert.equal(outputDirective('Explain using top 200 words').config.lexical_density, 'top-200');
  for (const text of ['What does "Answer in ACE" mean?', '"Use ASD-STE100"', 'Use the blue folder.', '```Use ACE```']) {
    assert.equal(outputDirective(text), null, text);
  }
  const history = [{role: 'user', content: 'Use ACE'}, {role: 'assistant', content: 'Respond in E-Prime'}];
  assert.equal(outputTurn('hello', history).config.output_standard, 'ace');
  assert.equal(outputTurn('Rewrite in max-precision mode', history).rewrite, 'Respond in E-Prime');
  assert.equal(outputTurn('hello', []).config.output_standard, 'cnl-default');
});

test('Unicode words include combining marks and do not count Hindi vowel signs as separate words', () => {
  assert.deepEqual(outputWords('मैं सब कुछ जानता हूँ।', 'hi'), ['मैं', 'सब', 'कुछ', 'जानता', 'हूँ']);
  assert.deepEqual(outputWords('我知道。', 'zh'), ['我', '知', '道']);
});

test('density and standard switches preserve the other dimension', () => {
  const history = [{role: 'user', content: 'Use ACE'}, {role: 'assistant', content: 'I know everything.'}];
  assert.deepEqual(outputTurn('Rewrite in max-precision mode', history).config,
    {output_standard: 'ace', lexical_density: 'max-precision'});
  assert.deepEqual(outputTurn('Use ACE', [{role: 'user', content: 'Rewrite using semantic primes only'}]).config,
    {output_standard: 'ace', lexical_density: 'primes'});
  assert.deepEqual(outputTurn('Use simpler words', history, {lexical_density: 'max-precision'}).config,
    {output_standard: 'cnl-default', lexical_density: 'standard'});
  assert.equal(frequencyTier('top-9007199254740991'), 9007199254740991);
});

test('ambiguous contractions cannot certify E-Prime', () => {
  for (const text of ["John's tired.", "John's hat fell.", "I ain't ready.", "Y'all're here."]) {
    assert.equal(validateOutput(text, 'en', {output_standard: 'e-prime'}).compliant, false, text);
  }
});

test('a finite lexical alternative search does not certify a global minimum', () => {
  const report=validateOutput('I know everything.','en',{
    output_standard:'e-prime',lexical_density:'max-precision'},policy);
  assert.equal(report.compliant,false);
  assert.ok(report.unchecked.includes('lexical-minimum'));
});

test('complete E-Prime evidence cannot be inferred from a finite forbidden-word filter', () => {
  for (const text of ["’Tis ready.", "Thou wast here.", "Thou beest wise.", "I know this."]) {
    const report = validateOutput(text, 'en', {output_standard: 'e-prime'});
    assert.equal(report.compliant, false, text);
    assert.ok(report.unchecked.includes('e-prime-english'), text);
  }
  const config = {output_standard: 'e-prime'};
  assert.equal(validateOutput('I know this.', 'en', config, policy, {'e-prime-english': () => true}).compliant, true);
  assert.equal(validateOutput('Thou wast here.', 'en', config, policy, {'e-prime-english': () => false}).compliant, false);
});

test('canonical CNL references preserve keyed forms, empty forms and template styles', () => {
  const legacy = `cnl-language en
  keys "bare|plural"
  fillers "you know|please"
  prime "I: i|me"
  list "basic: cat|dog"
  tier "100: cat|dog"
  term cat
    forms "|cats"
  template "reply plain|basic statement"
    positive "{subject.bare}."
`;
  const canonical = `cnl-language en
  keys ("bare" "plural")
  fillers ("you know" "please")
  prime I
    forms ("i" "me")
  list basic
    forms ("cat" "dog")
  tier 100
    forms ("cat" "dog")
  term cat
    forms ("" "cats")
  template "reply statement"
    styles ("plain" "basic")
    positive "{subject.bare}."
`;
  assert.deepEqual(parseLanguageGrammar(canonical, parseLino), parseLanguageGrammar(legacy, parseLino));
  const oldPolicy = `controlled-output
  registers "plain|basic"
  profile plain
    aliases "one|two"
    styles "plain|basic"
  lexeme sample
    form hello
      mode "plain|basic"
`;
  const newPolicy = oldPolicy.replaceAll('"plain|basic"', '("plain" "basic")')
    .replace('"one|two"', '("one" "two")');
  assert.deepEqual(parseOutputPolicy(newPolicy, parseLino), parseOutputPolicy(oldPolicy, parseLino));
});

test('every CNL seed stores alternatives as canonical references', () => {
  function check(node, path) {
    assert.ok(!node.value?.includes('|'), `${path}: ${node.name} ${node.value}`);
    for (const child of node.children ?? []) check(child, path);
  }
  for (const path of ['data/seed/cnl-output.lino',
    ...['en', 'ru', 'hi', 'zh', 'es'].map(language => `data/seed/cnl-language-${language}.lino`)]) {
    check(parseLino(readRepoFile(path)), path);
  }
});

test('both Gemini token overrides use canonical quoting around embedded JSON', () => {
  const lines = readRepoFile('data/seed/client-integrations.lino').split('\n')
    .filter(line => line.includes('modelConfigs.overrides=json+:'));
  assert.equal(lines.length, 2);
  for (const line of lines) {
    assert.match(line.trim(), /^(?:temp_home_json_set|json_set) 'modelConfigs\.overrides=json\+:\[/u);
    const parsed = parseLino(line.trim());
    const node = parsed.name ? parsed : parsed.children[0];
    const overrides = node.value.slice('modelConfigs.overrides=json+:'.length)
      .replace('{max_output_tokens}', '123');
    assert.deepEqual(JSON.parse(overrides), [{match: {model: '{model}'},
      modelConfig: {generateContentConfig: {maxOutputTokens: 123}}}]);
  }
});
