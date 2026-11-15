// Builds rust/tests/fixtures/issue-1557-attempted-issues.json: the bodies of
// the 19 issues Formal AI attempted before issue #1557 (fetched with
// `gh issue view N --json number,title,body > N.json`), each of which is an
// English issue, plus synthetic cases the issue names explicitly.
// Usage: node experiments/issue-1557-build-fixture.mjs <dir-with-N.json>
import { readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const SYNTHETIC = [
  {
    case: 'a fully Russian issue',
    title: 'Бот отвечает на английском вместо русского',
    body: 'Когда я пишу боту по-русски, он отвечает на английском языке. Ожидается, что ответ будет на том же языке, что и вопрос. Пожалуйста, исправьте определение языка.',
    expected: 'ru',
  },
  {
    case: 'a Russian body with English identifiers',
    title: 'Падение parse_input на пустой строке',
    body: 'Исправь ошибку в функции parse_input в модуле Rust: она падает на пустой строке, см. src/parser.rs и тест test_empty_input. Также проверь, что parseInput в JavaScript ведёт себя так же.',
    expected: 'ru',
  },
  {
    case: 'an English body containing "formalization", "described" and "response"',
    title: 'Formalization summary is wrong',
    body: 'The formalization described in the issue produces a response that ignores the request. Please make the formalization step keep the described constraints in the response.',
    expected: 'en',
  },
  {
    case: 'an English body quoting a Russian sentence',
    title: 'Russian greeting is answered formally',
    body: 'The bot answered "На вы? Нет, на ты." which is wrong; please fix the formalization step so the reply keeps the informal register.',
    expected: 'en',
  },
  {
    case: 'an English body with a Russian code block and blockquote',
    title: 'Log excerpt in Russian',
    body: 'The worker crashes when it prints this log:\n\n```\nОшибка: не удалось открыть файл конфигурации\n```\n\n> Это сообщение пользователя из чата\n\nPlease make the crash handler print the path instead.',
    expected: 'en',
  },
  {
    case: 'a fully Spanish issue',
    title: 'El bot responde en inglés',
    body: 'Cuando escribo al bot en español, responde en inglés. ¿Por qué? Por favor, corrijan la detección del idioma para que la respuesta use el mismo idioma que la pregunta.',
    expected: 'es',
  },
];

const dir = process.argv[2];
const attempted = readdirSync(dir)
  .filter((name) => /^\d+\.json$/.test(name))
  .map((name) => JSON.parse(readFileSync(join(dir, name), 'utf8')))
  .sort((a, b) => a.number - b.number)
  .map(({ number, title, body }) => ({ number, title, body, expected: 'en' }));
const list = (items) => items.map((item) => `    ${JSON.stringify(item)}`).join(',\n');
const out = 'rust/tests/fixtures/issue-1557-attempted-issues.json';
writeFileSync(
  out,
  `{\n  "attempted": [\n${list(attempted)}\n  ],\n  "synthetic": [\n${list(SYNTHETIC)}\n  ]\n}\n`,
);
console.log(`wrote ${attempted.length} attempted and ${SYNTHETIC.length} synthetic cases to ${out}`);
