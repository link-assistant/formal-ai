import assert from 'node:assert/strict';
import test from 'node:test';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {extractRequirements} from '../../../js/agentic/crate/requirement_extraction.mjs';
await installNodeHost(new WorkerHost());
const cases = [
  [
    "अपलोड फ़ॉर्म में प्रगति पट्टी जोड़ें।",
    true
  ],
  [
    "अपलोड फ़ॉर्म में प्रगति पट्टी मत जोड़ें।",
    true
  ],
  [
    "“संकेत” जोड़ें।",
    true
  ],
  [
    "उदाहरण “जोड़ें”।",
    false
  ],
  [
    "लेखक कहता है प्रगति पट्टी जोड़ें।",
    false
  ],
  [
    "अपलोड फ़ॉर्म में प्रगति पट्टी “जोड़ें।",
    false
  ],
  [
    "The source contains add.",
    false
  ],
  [
    "El texto termina en agrega.",
    false
  ],
  [
    "जोड़ें प्रगति पट्टी।",
    true
  ],
  [
    "Add a progress bar.",
    true
  ],
  [
    "Добавь полосу прогресса.",
    true
  ],
  [
    "添加进度条。",
    true
  ],
  [
    "Agrega una barra de progreso.",
    true
  ]
];
for (const [text,required] of cases) {
  test(`directive position preserves source and quote scope: ${text}`, () => {
    assert.deepEqual(extractRequirements(text), required ? [text] : []);
  });
}
