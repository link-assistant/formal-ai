// Unit tests for the JavaScript ports of the agentic read / web / search
// routes (js/agentic/file_read*.mjs, local_search.mjs, comparison.mjs,
// web_research.mjs, report_issue.mjs, conversation_recall.mjs, ...).
//
// Each case mirrors a Rust test; the Rust source is named beside it. The
// routes are driven directly (not through plan_chat_step), with the same
// messages and tool names the Rust test hands the planner, on turns where the
// route under test is the one that answers.

import assert from 'node:assert/strict';
import { before, describe, it } from 'node:test';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { parseChatMessage } from '../../../js/server/chat-request.mjs';
import { fileReadTaskFor, planFileReadStep, suppliedFileAnswer } from '../../../js/agentic/file_read.mjs';
import { planLocalSearchStep } from '../../../js/agentic/local_search.mjs';
import { planComparisonStep } from '../../../js/agentic/comparison.mjs';
import {
  contextualReferenceClarification, definitionFollowupClarification, definitionFollowupTopic,
  isDefinitionFollowup, planWebResearchStep, preferredUrl, urlsIn, webResearchQueryFor,
} from '../../../js/agentic/web_research.mjs';
import { isReportIntent, planReportFlow } from '../../../js/agentic/report_issue.mjs';
import { planNoteCompositionStep } from '../../../js/agentic/note_composition.mjs';
import { records } from '../../../js/agentic/transcript_evidence.mjs';
import { planSharedSolverStep } from '../../../js/agentic/conversation_recall.mjs';
import { SourceRoot, sourceTreeRequest } from '../../../js/agentic/crate/meta_translate.mjs';
import { issueTitle, reportTurn } from '../../../js/agentic/crate/issue_report_title.mjs';

const user = (content) => parseChatMessage({ role: 'user', content });
const assistant = (content) => parseChatMessage({ role: 'assistant', content });
const system = (content) => parseChatMessage({ role: 'system', content });
const callMessage = (id, tool, args) => parseChatMessage({
  role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: tool, arguments: args } }],
});
const toolResult = (id, name, content) => parseChatMessage({ role: 'tool', tool_call_id: id, name, content });

/** Mirrors the Rust tests' `answer_tool_call`: append the call and its result. */
function answerCall(messages, call, result, id = `call_${messages.length}`) {
  messages.push(callMessage(id, call.tool, call.arguments));
  messages.push(toolResult(id, call.tool, result));
}

const args = (call) => JSON.parse(call.arguments);
const single = (plan) => {
  assert.equal(plan?.kind, 'tool_calls', JSON.stringify(plan));
  assert.equal(plan.calls.length, 1, JSON.stringify(plan));
  return plan.calls[0];
};
const final = (plan) => {
  assert.equal(plan?.kind, 'final', JSON.stringify(plan));
  return plan.answer;
};

const latestTask = (messages) => [...messages].reverse().find((message) => message.role === 'user').content;
const readStep = (messages, tools) => planFileReadStep(fileReadTaskFor(latestTask(messages)), messages, tools);

before(async () => {
  await installNodeHost(new WorkerHost());
});

describe('file_read (rust/tests/unit/agentic-coding/file_read_tool_calls.rs)', () => {
  const TOOLS = ['read', 'bash'];

  it('direct file read prompts emit read tool calls', () => {
    for (const [prompt, path] of [
      ['read the file alpha.txt', 'alpha.txt'],
      ['show me the contents of beta.md', 'beta.md'],
      ['open alpha.txt and tell me what\'s inside', 'alpha.txt'],
      ['what does beta.md say?', 'beta.md'],
      ['please read gamma.json for me', 'gamma.json'],
      ['what is the value of gamma_marker in gamma.json?', 'gamma.json'],
      ['print the first line of alpha.txt', 'alpha.txt'],
    ]) {
      const call = single(readStep([user(prompt)], TOOLS));
      assert.equal(call.tool, 'read', prompt);
      assert.equal(args(call).filePath, path, prompt);
    }
  });

  it('cat file prompt uses the typed read capability', () => {
    const call = single(readStep([user('cat gamma.json')], TOOLS));
    assert.equal(call.tool, 'read');
    assert.equal(args(call).filePath, 'gamma.json');
  });

  it('read arguments carry the three path spellings, sorted', () => {
    const call = single(readStep([user('read the file alpha.txt')], TOOLS));
    assert.equal(call.arguments, '{"filePath":"alpha.txt","file_path":"alpha.txt","path":"alpha.txt"}');
  });

  it('list then read first file walks the tool loop to final content', () => {
    const messages = [user('list the files then read the first one alphabetically')];
    const list = single(readStep(messages, TOOLS));
    assert.equal(list.tool, 'bash');
    assert.equal(args(list).command, "find . -maxdepth 1 -type f | sed 's#^./##' | sort");
    answerCall(messages, list, 'alpha.txt\nbeta.md\ngamma.json\n');
    const read = single(readStep(messages, TOOLS));
    assert.equal(read.tool, 'read');
    assert.equal(args(read).filePath, 'alpha.txt');
    answerCall(messages, read, 'ALPHA_MARKER_11111\nsecond line\n');
    assert.match(final(readStep(messages, TOOLS)), /ALPHA_MARKER_11111/);
  });

  it('read file in named folder lists then reads nested file', () => {
    const messages = [user('read the file in the subdir folder')];
    const list = single(readStep(messages, TOOLS));
    assert.equal(args(list).command, "find subdir -maxdepth 1 -type f | sed 's#^.*/##' | sort");
    answerCall(messages, list, 'nested.md\n');
    const read = single(readStep(messages, TOOLS));
    assert.equal(args(read).filePath, 'subdir/nested.md');
    answerCall(messages, read, 'NESTED_MARKER_44444\n');
    assert.match(final(readStep(messages, TOOLS)), /NESTED_MARKER_44444/);
  });

  it('ls folder prompt reads last file from listing', () => {
    const messages = [user('ls the folder and show me the contents of the last file')];
    const list = single(readStep(messages, TOOLS));
    answerCall(messages, list, 'alpha.txt\nbeta.md\ngamma.json\n');
    const read = single(readStep(messages, TOOLS));
    assert.equal(args(read).filePath, 'gamma.json');
    answerCall(messages, read, '{"gamma_marker":"GAMMA_33333","n":42}\n');
    assert.match(final(readStep(messages, TOOLS)), /GAMMA_33333/);
  });

  it('read every file lists then reads each file', () => {
    const messages = [user('read every file here and summarize them')];
    const list = single(readStep(messages, TOOLS));
    assert.equal(list.tool, 'bash');
    answerCall(messages, list, 'alpha.txt\nbeta.md\ngamma.json\n');
    const reads = readStep(messages, TOOLS);
    assert.equal(reads.calls.length, 3);
    assert.deepEqual(reads.calls.map((call) => args(call).filePath), ['alpha.txt', 'beta.md', 'gamma.json']);
    reads.calls.forEach((call, index) => answerCall(messages, call,
      ['ALPHA_MARKER_11111\n', 'BETA_MARKER_22222\n', '{"gamma_marker":"GAMMA_33333","n":42}\n'][index]));
    const answer = final(readStep(messages, TOOLS));
    for (const marker of ['ALPHA_MARKER_11111', 'BETA_MARKER_22222', 'GAMMA_33333']) assert.match(answer, new RegExp(marker));
  });

  it('a value request answers with the extracted value', () => {
    const messages = [user('what is the value of gamma_marker in gamma.json?')];
    const read = single(readStep(messages, TOOLS));
    answerCall(messages, read, '{"gamma_marker":"GAMMA_33333","n":42}\n');
    assert.equal(final(readStep(messages, TOOLS)), 'Value of `gamma_marker` in `gamma.json`: GAMMA_33333');
  });

  it('write intent beats read intent (rust/tests/unit/agentic-coding/file_creation_writes.rs)', () => {
    assert.equal(fileReadTaskFor('create a file notes.txt with hello'), null);
  });
});

describe('bounded file analysis (rust/tests/unit/issue_1138_bounded_file_analysis.rs)', () => {
  const AUDIT_PROMPT = 'Audit and read `notes/alpha-plan.md`, `notes/beta-plan.md`, and '
    + '`.github/workflows/check.yml`; report concrete remaining gaps.';

  it('multi-file audit collects seed-derived gap evidence before reading files', () => {
    const task = fileReadTaskFor(AUDIT_PROMPT);
    assert.equal(task.isAnalysis(), true);
    const plan = planFileReadStep(task, [user(AUDIT_PROMPT)], ['read', 'grep']);
    assert.equal(plan.calls.length, 3);
    assert.ok(plan.calls.every((call) => call.tool === 'grep'));
    assert.deepEqual(plan.calls.map((call) => args(call).path),
      ['notes/alpha-plan.md', 'notes/beta-plan.md', '.github/workflows/check.yml']);
    for (const call of plan.calls) {
      const { pattern } = args(call);
      assert.ok(pattern.includes('TODO'), pattern);
      assert.ok(pattern.includes('pending'), pattern);
      assert.ok(pattern.includes('\\['), pattern);
    }
  });

  it('seeded audit actions route to bounded analysis in all supported languages', () => {
    for (const prompt of [
      'Review and read `a.md` and `b.md` for remaining work.',
      'Проверь и прочитай `a.md` и `b.md`, найди оставшуюся работу.',
      'समीक्षा करें और `a.md` तथा `b.md` फ़ाइलें पढ़ें।',
      '审查并读取 `a.md` 和 `b.md` 中的剩余工作。',
      'Audita y lee `a.md` y `b.md` para encontrar trabajo pendiente.',
    ]) {
      const plan = readStep([user(prompt)], ['read', 'grep']);
      assert.equal(plan.calls.length, 2, prompt);
      assert.ok(plan.calls.every((call) => call.tool === 'grep'), prompt);
    }
  });

  it('multi-file audit composes findings and marks absent evidence honestly', () => {
    const messages = [user(AUDIT_PROMPT)];
    const planned = readStep(messages, ['read', 'grep']).calls;
    planned.forEach((call, index) => answerCall(messages, call, [
      'Found 1 matches\nnotes/alpha-plan.md:\n  Line 12: - [ ] implement the generic adapter',
      'No files found',
      'Found 1 matches\n.github/workflows/check.yml:\n  Line 44: # TODO publish the verified artifact',
    ][index]));
    const report = final(readStep(messages, ['read', 'grep']));
    for (const fragment of ['alpha-plan.md', 'implement the generic adapter', 'beta-plan.md',
      'No explicit gap marker', 'publish the verified artifact', 'does not prove completion']) {
      assert.ok(report.includes(fragment), `${fragment}: ${report}`);
    }
  });

  it('read-only clients receive explicit bounds and oversized results stay bounded', () => {
    const messages = [user(AUDIT_PROMPT)];
    const planned = readStep(messages, ['read']).calls;
    assert.equal(planned.length, 3);
    for (const call of planned) {
      assert.equal(call.tool, 'read');
      assert.deepEqual([args(call).offset, args(call).limit, args(call).columnOffset, args(call).columnLimit],
        [0, 160, 0, 320]);
    }
    const huge = `TODO ${'x'.repeat(20000)}`;
    for (const call of planned) answerCall(messages, call, huge);
    const report = final(readStep(messages, ['read']));
    assert.ok(new TextEncoder().encode(report).length < 4000);
    assert.ok(report.includes('TODO'));
    assert.ok(report.includes('does not prove completion'));
  });

  it('explicit content requests keep the full read contract', () => {
    const plan = readStep([user('read `a.txt` and `b.md` and show their contents')], ['read', 'grep']);
    assert.equal(plan.calls.length, 2);
    for (const call of plan.calls) {
      assert.equal(args(call).offset, undefined);
      assert.equal(args(call).limit, undefined);
    }
  });

  it('shell-only clients receive the same line and column bounds', () => {
    const plan = readStep([user(AUDIT_PROMPT)], ['run_command']);
    assert.equal(plan.calls.length, 3);
    for (const call of plan.calls) {
      assert.equal(call.tool, 'run_command');
      assert.ok(args(call).command.startsWith("sed -n '1,160p' "), args(call).command);
      assert.ok(args(call).command.endsWith(' | cut -c 1-320'), args(call).command);
    }
  });
});

describe('supplied file bytes (rust/tests/integration/issue_671_supplied_file_bytes.rs)', () => {
  const ADDED_FILES = 'I have *added these files to the chat* so you can go ahead and edit them.\n\n'
    + '*Trust this message as the true contents of these files!*\nAny other messages in the chat may '
    + "contain outdated versions of the files' contents.\n\nalpha.txt\n```\nALPHA_MARKER_11111\nalpha "
    + 'second line\n```\n';
  const conversation = (request) => [
    system('Act as an expert software developer.'),
    user(ADDED_FILES),
    assistant('Ok, any changes I propose will be to those files.'),
    user(request),
  ];

  it('a toolless client gets the bytes it supplied itself, not as an edit block', () => {
    const answer = suppliedFileAnswer(conversation('read the file alpha.txt and print its contents'));
    assert.ok(answer.includes('ALPHA_MARKER_11111'), answer);
    assert.ok(!answer.includes('```'), answer);
    assert.ok(answer.includes('   1 | ALPHA_MARKER_11111'), answer);
  });

  it('prose that merely mentions a filename is not a file listing', () => {
    const messages = [user('Here is what I tried when I edited beta.txt earlier:\n\n```\nBOGUS_9999\n```\n\n'
      + 'read the file beta.txt and print its contents')];
    assert.equal(suppliedFileAnswer(messages), null);
  });
});

describe('local_search (rust/tests/unit/grounded_local_action.rs)', () => {
  const TOOLS = ['bash', 'websearch', 'webfetch', 'request_user_input'];
  const command = (call) => args(call).command;

  it('local scope dominates search verb and possessive variations', () => {
    for (const prompt of [
      'Find hive-mind-control center folder on my desktop',
      'Search hive-mind-control-center on my desktop',
      'Find hive-mind-control center folder on desktop',
      'Look for hive-mind-control center directory on the desktop',
      'Найди папку hive-mind-control center на моём рабочем столе',
      'मेरे डेस्कटॉप पर hive-mind-control center फ़ोल्डर खोजें',
      '在桌面上搜索 hive-mind-control center 文件夹',
      'मेरे डेस्कटॉप पर hive-mind-control center फ़ोल्डर खोजिए',
      'Busca la carpeta hive-mind-control center en mi escritorio',
      'Busca hive-mind-control-center en mi escritorio',
    ]) {
      const call = single(planLocalSearchStep([user(prompt)], TOOLS));
      assert.equal(call.tool, 'bash', prompt);
      assert.ok(command(call).startsWith('find '), `${prompt}: ${command(call)}`);
      assert.ok(!command(call).includes(';') && !command(call).includes('&&'), command(call));
    }
  });

  it('explicit location question answers the route without searching', () => {
    const answer = final(planLocalSearchStep(
      [user("Is the request 'Find a folder on my desktop' a local filesystem search or a web search?")],
      ['bash', 'websearch']));
    assert.ok(answer.toLowerCase().includes('local'), answer);
    assert.ok(!answer.toLowerCase().includes('web search'), answer);
  });

  it('empty exact local lookup widens instead of claiming absence', () => {
    const messages = [user('Find hive-mind-control center folder on my desktop')];
    const exact = single(planLocalSearchStep(messages, TOOLS));
    assert.equal(command(exact), 'find "${FORMAL_AI_DESKTOP_DIR:-$HOME/Desktop}" -type d -iname \'hive-mind-control-center\' -print');
    answerCall(messages, exact, '(no output)', 'exact');
    const widened = single(planLocalSearchStep(messages, TOOLS));
    assert.ok(command(widened).includes('*hive*'), command(widened));
  });

  it('quoted local name excludes trailing answer instructions', () => {
    for (const [prompt, expected] of [
      ["Is there a folder named exactly 'hive-mind-control-center' on my desktop? Answer yes or no and say what the closest match is.",
        'hive-mind-control-center'],
      ["What's inside the 'Archive' folder on my desktop?", 'archive'],
    ]) {
      const call = single(planLocalSearchStep([user(prompt)], TOOLS));
      assert.ok(command(call).includes(`-iname '${expected}'`), command(call));
      assert.ok(!command(call).includes('answer') && !command(call).includes('closest'), command(call));
    }
  });

  it('transport-wrapped local request does not treat the whole prompt as a name', () => {
    const call = single(planLocalSearchStep([user('"Find hive-mind-control center folder on my desktop"')], TOOLS));
    assert.ok(command(call).includes("-iname 'hive-mind-control-center'"), command(call));
  });

  it('unnamed tool result widens by matching its call id', () => {
    const messages = [user('Find hive-mind-control center folder on my desktop')];
    const exact = single(planLocalSearchStep(messages, TOOLS));
    messages.push(callMessage('exact', exact.tool, exact.arguments));
    messages.push(parseChatMessage({ role: 'tool', tool_call_id: 'exact', content: '(no output)' }));
    assert.ok(command(single(planLocalSearchStep(messages, TOOLS))).includes('*hive*'));
  });

  it('absence requires exact, substring and bounded inventory observations', () => {
    const messages = [user('Find zzz-nonexistent folder on my desktop')];
    const exact = single(planLocalSearchStep(messages, TOOLS));
    assert.ok(command(exact).includes("-iname 'zzz-nonexistent'"));
    answerCall(messages, exact, '(no output)', 'exact');
    const substring = single(planLocalSearchStep(messages, TOOLS));
    assert.ok(command(substring).includes("-iname '*nonexistent*'"));
    answerCall(messages, substring, '(no output)', 'substring');
    const inventory = single(planLocalSearchStep(messages, TOOLS));
    assert.ok(command(inventory).includes('-mindepth 1 -maxdepth 3'));
    answerCall(messages, inventory, '(no output)', 'inventory');
    const answer = final(planLocalSearchStep(messages, ['bash', 'websearch']));
    assert.ok(answer.includes('exact, substring, and nearby-name'), answer);
    assert.ok(answer.includes('FORMAL_AI_DESKTOP_DIR'), answer);
    assert.ok(answer.includes('No wider location was searched'), answer);
  });

  it('differently typed near match is named instead of reported as absent', () => {
    const messages = [user("Find the folder named 'only-a-file' on my desktop")];
    for (const id of ['exact', 'substring']) {
      const call = single(planLocalSearchStep(messages, TOOLS));
      assert.ok(command(call).includes('-type d'));
      answerCall(messages, call, '(no output)', id);
    }
    const inventory = single(planLocalSearchStep(messages, TOOLS));
    assert.ok(command(inventory).includes('-mindepth 1 -maxdepth 3') && !command(inventory).includes('-type d'));
    answerCall(messages, inventory, '/tmp/Desktop/only-a-file', 'inventory');
    const metadata = single(planLocalSearchStep(messages, TOOLS));
    assert.ok(command(metadata).startsWith('ls -ld -- '));
    answerCall(messages, metadata, '-rw------- 1 user user 0 Jul 24 00:00 /tmp/Desktop/only-a-file', 'metadata');
    const answer = final(planLocalSearchStep(messages, ['bash', 'websearch']));
    assert.ok(answer.includes('No folder matched only-a-file'), answer);
    assert.ok(answer.includes('non-directory'), answer);
    assert.ok(answer.includes('/tmp/Desktop/only-a-file'), answer);
  });

  it('failed local lookup is reported instead of treated as empty', () => {
    const messages = [user('Find hive-mind-control center folder on my desktop')];
    const exact = single(planLocalSearchStep(messages, TOOLS));
    answerCall(messages, exact, '{"exit_code":1,"stderr":"desktop is unavailable"}', 'failed');
    assert.ok(final(planLocalSearchStep(messages, ['bash', 'websearch'])).includes('desktop is unavailable'));
  });
});

describe('comparison (rust/tests/unit/grounded_local_action.rs)', () => {
  const TOOLS = ['bash', 'websearch', 'webfetch', 'request_user_input'];
  const step = (messages, tools = TOOLS) => planComparisonStep(latestTask(messages), messages, tools);

  it('comparison is decomposed before open web research', () => {
    const messages = [user('ФБС vs ФБО')];
    const first = single(step(messages));
    assert.equal(first.tool, 'websearch');
    const left = args(first).query.toLowerCase();
    assert.ok(left.includes('фбс') && !left.includes('фбо'), left);
    answerCall(messages, first, 'ФБС evidence: seller warehouse', 'left');
    const second = single(step(messages));
    const right = args(second).query.toLowerCase();
    assert.ok(right.includes('фбо') && !right.includes('фбс'), right);
    answerCall(messages, second, 'ФБО evidence: marketplace warehouse', 'right');
    const answer = final(step(messages, ['websearch', 'webfetch']));
    assert.ok(answer.includes('ФБС evidence') && answer.includes('ФБО evidence'), answer);
  });

  it('comparison retries the missing side after a failed search result', () => {
    const messages = [user('ФБС vs ФБО')];
    answerCall(messages, single(step(messages)), 'ФБС evidence: seller warehouse', 'left');
    answerCall(messages, single(step(messages)), 'Error: MCP tool timed out before returning evidence', 'right-timeout');
    const retry = single(step(messages));
    assert.equal(args(retry).query, 'фбо');
  });
});

describe('web_research (rust/tests/unit/grounded_local_action.rs)', () => {
  const TOOLS = ['bash', 'websearch', 'webfetch', 'request_user_input'];

  it('bare definition follow-up asks for its antecedent', async () => {
    const task = 'Так что это такое то?';
    assert.equal(isDefinitionFollowup(task), true);
    assert.equal(await definitionFollowupTopic([user(task)], task), null);
    const answer = definitionFollowupClarification(task);
    assert.ok(answer.toLowerCase().includes('имеете в виду'), answer);
  });

  it('same-turn definition follow-up reuses only the antecedent topic', async () => {
    const task = 'Что такое фуфломицин? Затем: так что это такое то?';
    const messages = [user(task)];
    assert.equal(isDefinitionFollowup(task), true);
    const query = await definitionFollowupTopic(messages, task);
    const call = single(planWebResearchStep(messages, TOOLS, query, true));
    assert.equal(call.tool, 'websearch');
    const searched = args(call).query;
    assert.ok(searched.includes('фуфломицин') && !searched.includes('затем') && !searched.includes('такое'), searched);
  });

  it('later definition follow-up reuses the prior user topic', async () => {
    const messages = [user('Что такое фуфломицин?'), assistant('Я проверю определение.'), user('Так что это такое то?')];
    const query = await definitionFollowupTopic(messages, 'Так что это такое то?');
    assert.ok(query.includes('фуфломицин') && !query.includes('такое'), query);
  });

  it('definition follow-up corroborates across the search rows before answering', () => {
    const messages = [user('Что такое фуфломицин? Затем: так что это такое то?')];
    answerCall(messages, { tool: 'websearch', arguments: '{"query":"фуфломицин"}' },
      'Словарное определение https://dictionary.example.test/ru/fuflomicin\nСправка о доказательности '
      + 'https://evidence.example.test/ru/unproven-medicine\nУпотребление термина https://language.example.test/ru/fuflomicin-usage',
      'search_1');
    messages.push(callMessage('fetch_1', 'webfetch', '{"url":"https://dictionary.example.test/ru/fuflomicin"}'));
    messages.push(toolResult('fetch_1', 'webfetch',
      'Фуфломицин — разговорное неодобрительное название лекарства, клиническая эффективность которого не доказана.'));
    const call = single(planWebResearchStep(messages, TOOLS, 'фуфломицин', true));
    assert.equal(call.tool, 'webfetch');
    assert.ok(call.arguments.includes('evidence.example.test'), call.arguments);
  });

  it('covering fetch ends the recipe instead of reading every result row', () => {
    const messages = [user('hi')];
    answerCall(messages, { tool: 'websearch', arguments: '{"query":"hi"}' },
      'Title: HI | English meaning\nURL: https://dictionary.cambridge.org/dictionary/english/hi\nHighlights: used as an '
      + 'informal greeting\nTitle: HI Definition & Meaning\nURL: https://www.merriam-webster.com/dictionary/hi\n'
      + 'Highlights: used especially as a greeting', 'search_hi');
    messages.push(callMessage('fetch_hi', 'webfetch',
      '{"format":"text","url":"https://dictionary.cambridge.org/dictionary/english/hi"}'));
    messages.push(toolResult('fetch_hi', 'webfetch',
      'HI | English meaning - Cambridge Dictionary\n\nhi exclamation\n\n(informal)\n\nused as an informal greeting, '
      + 'usually to people who you know:\n\nHi, there!\n\nHi, how are you doing?'));
    const answer = final(planWebResearchStep(messages, TOOLS, 'hi', false));
    assert.ok(answer.includes('https://dictionary.cambridge.org/dictionary/english/hi'), answer);
    assert.ok(!answer.includes('merriam-webster'), answer);
  });

  it('definition imperatives route to research in every supported language', async () => {
    for (const [prompt, subject] of [
      ['Define flarb in one sentence', 'flarb'],
      ['Дай определение слова фуфломицин одним предложением', 'фуфломицин'],
      ['परिभाषित करें फ्लार्ब', 'फ्लार्ब'],
      ['定义弗拉布', '弗拉布'],
    ]) {
      const messages = [user(prompt)];
      const query = await webResearchQueryFor(messages);
      assert.ok(query !== null && query.includes(subject), `${prompt}: ${query}`);
      assert.equal(single(planWebResearchStep(messages, TOOLS, query, false)).tool, 'websearch');
    }
  });

  it('contextual reference without antecedent asks for it', () => {
    assert.equal(contextualReferenceClarification('Define flarb in one sentence'), null);
  });

  it('urls are ranked with authoritative hosts first', () => {
    // `urls_in` keeps whitespace tokens that start with a scheme, trimming trailing punctuation.
    assert.deepEqual(urlsIn('see https://a.example/x. and (https://b.gov/y) https://c.example/z),'),
      ['https://a.example/x', 'https://c.example/z']);
    assert.equal(preferredUrl('https://a.example/x https://b.gov/y https://c.example'), 'https://b.gov/y');
  });
});

describe('report_issue (rust/tests/unit/grounded_local_action.rs, report_context_collection.rs)', () => {
  it('report intent is meanings-driven for the reported Russian phrase', () => {
    for (const prompt of ['Зарепорти баг', 'Сообщи об ошибке', 'Report a bug']) {
      assert.equal(isReportIntent(prompt), true, prompt);
      const call = single(planReportFlow([user(prompt)], ['bash', 'websearch', 'webfetch', 'request_user_input']));
      assert.equal(call.tool, 'request_user_input', prompt);
    }
  });

  it('an unrelated request is not a report intent', () => {
    assert.equal(isReportIntent('read the file alpha.txt'), false);
    assert.equal(planReportFlow([user('read the file alpha.txt')], ['bash']), null);
  });
});

describe('note_composition (rust/tests/unit/issue_1066_ladder_capability.rs:277)', () => {
  it('a document specification is composed, listing its parts as outstanding', () => {
    const task = 'Draft a vendor brief containing the contract owner, the renewal window, '
      + 'and the escalation path. Store it in `vendors/acme.md`.';
    assert.equal(final(planNoteCompositionStep(task, [user(task)])),
      'Draft a vendor brief containing the contract owner, the renewal window, and the '
      + 'escalation path\n\nRequested parts:\n- the contract owner\n- the renewal window\n- '
      + 'the escalation path\n\nObserved in this session:\n- nothing: no tool result was '
      + 'recorded before this note.\n\nNo requested part above is backed by an observation '
      + 'from this session.\n');
  });
});

describe('transcript_evidence', () => {
  it('records each current-turn tool result with its naming arguments', () => {
    const messages = [user('run it')];
    answerCall(messages, { tool: 'bash', arguments: '{"command":"ls -la"}' }, 'Exit Code: 0\nok', 'c1');
    const [record] = records(messages);
    assert.equal(record.command, 'bash ls -la');
    assert.deepEqual(record.argv, ['bash', 'ls', '-la']);
    assert.equal(record.source, 'harness');
    assert.match(record.evidence_id, /^evidence_[0-9a-f]{16}$/);
  });
});

describe('issue title (rust/tests/integration/issue_839_report_format.rs)', () => {
  const settings = { prefix: 'Formal AI: ', default_title: 'Formal AI agentic session report' };
  const conversation = (turns) => turns.flatMap((text, index) => [
    reportTurn('user', text, index === turns.length - 1), reportTurn('assistant', '…')]);

  it('two subjects are titled first plus last', () => {
    assert.equal(issueTitle(conversation(['Что такое фуфломицин?', 'Так что это такое то?', 'Зарепорти баг']), settings),
      'Formal AI: `Что такое фуфломицин?` + `Так что это такое то?`');
  });

  it('an answered report request can still be a subject', () => {
    assert.equal(issueTitle(conversation(['ФБС vs ФБО', 'Зарепорти баг', 'Report']), settings),
      'Formal AI: `ФБС vs ФБО` + `Зарепорти баг`');
  });

  it('a single subject is quoted; the default title needs an empty conversation', () => {
    assert.equal(issueTitle(conversation(['Find hive-mind on my desktop', 'report issue']), settings),
      'Formal AI: `Find hive-mind on my desktop`');
    assert.equal(issueTitle(conversation(['report issue']), settings), 'Formal AI: `report issue`');
    assert.equal(issueTitle([], settings), settings.default_title);
  });

  it('an oversize subject is truncated on a word boundary', () => {
    const long = 'Find the hive mind control center folder somewhere on my desktop '.repeat(4).trim();
    const title = issueTitle(conversation([long, 'report issue']), settings);
    assert.ok(Array.from(title).length <= 120, title);
    assert.ok(title.endsWith('…`'), title);
    const quoted = title.slice('Formal AI: `'.length, -'…`'.length);
    assert.ok(long.startsWith(quoted) && !quoted.endsWith(' '), title);
    assert.ok(long.slice(quoted.length).startsWith(' '), title);
  });
});

describe('source tree translation (rust/tests/unit/issue_1138_translation_tool.rs:120)', () => {
  it('recognizes paths and targets', () => {
    assert.deepEqual(sourceTreeRequest('Translate js/app/foo.js to TypeScript and write it'),
      { from: SourceRoot.JavaScript, to: SourceRoot.TypeScript, path: 'js/app/foo.js' });
    assert.deepEqual(sourceTreeRequest('translate ts/one.ts to javascript'),
      { from: SourceRoot.TypeScript, to: SourceRoot.JavaScript, path: 'ts/one.ts' });
    assert.equal(sourceTreeRequest('translate "apple" to russian'), null);
    assert.equal(sourceTreeRequest('translate js/a.js to javascript'), null);
    assert.equal(sourceTreeRequest('summarize js/app.js for the typescript review'), null);
  });

  // The worker solver has no `translate_source_tree` arm (rust/src/solver_handlers/mod.rs
  // routes it natively), so the JavaScript solver answers `translate_en_to_en` and the
  // route declines. Kept as a todo until the solver gains that arm.
  it('lowers a source tree request to one translate call (issue_1138_translate_agent_tool.rs:78)', {
    todo: 'worker solver lacks the translate_source_tree intent',
  }, async () => {
    const outcome = await planSharedSolverStep(
      [user('Translate js/no_such_probe_file.js to TypeScript and write it')], ['translate', 'write_file']);
    assert.equal(outcome.kind, 'ready');
    const call = single(outcome.plan);
    assert.equal(call.tool, 'translate');
    assert.equal(call.arguments, '{"from":"js","to":"ts","path":"js/no_such_probe_file.js","write":true}');
  });
});

import { planChatStep } from '../../../js/agentic/planner.mjs';
import { ownedSourceTreeRequest } from '../../../js/agentic/crate/meta_translate.mjs';

describe('source-qualified translation ownership bridge', () => {
  const request = 'Translate js/no_such_probe_file.js to TypeScript and write it';
  it('preserves the original advertised source-tree tool operation', async () => {
    const plan = await planChatStep([user(request)], ['translate','write_file']);
    assert.equal(plan.kind,'tool_calls');
    assert.equal(plan.calls.length,1);
    assert.equal(plan.calls[0].tool,'translate');
    assert.equal(plan.calls[0].arguments,
      '{"from":"js","to":"ts","path":"js/no_such_probe_file.js","write":true}');
  });
  it('reports the original missing-source gap without output authoring', async () => {
    const plan = await planChatStep([user(request)], ['write_file']);
    assert.equal(plan.kind,'final');
    assert.ok(plan.answer.includes('js/no_such_probe_file.js'));
    assert.ok(plan.answer.includes('no source file to translate at that path'));
  });
  it('retains original path case, Unicode source bytes and consumed spans', async () => {
    const text = 'Translate js/Δelta.js to TypeScript and write it';
    const owned = ownedSourceTreeRequest(text);
    const bytes = new TextEncoder().encode(text);
    assert.equal(owned.request.path,'js/Δelta.js');
    assert.equal(owned.source_unit,'utf8');
    assert.deepEqual(owned.request_span,[0,bytes.length]);
    assert.equal(new TextDecoder().decode(bytes.slice(...owned.source_span)),owned.request.path);
    const plan = await planChatStep([user(text)], ['translate','write_file']);
    assert.equal(JSON.parse(plan.calls[0].arguments).path,'js/Δelta.js');
  });
  it('observational translation does not invent a write request', async () => {
    const text = 'Translate ts/Report.ts to JavaScript';
    const plan = await planChatStep([user(text)], ['translate']);
    assert.equal(JSON.parse(plan.calls[0].arguments).write,false);
  });
  for (const tail of [
    '. Deploy the release.', '. Read settings.json first.',
    '. Do not write files.', '. After signature verification.',
    '. Rotate the keys.',
  ]) it('refuses unconsumed independent requirements before effects '+tail, async () => {
    const text = request+tail;
    assert.equal(ownedSourceTreeRequest(text),null);
    assert.equal((await planChatStep([user(text)],['translate','write_file']))?.calls?.length ?? 0,0);
    assert.equal((await planSharedSolverStep([user(text)],['translate','write_file'])).kind,'not_ours');
  });
  it('does not claim completion after the translation provider refuses', async () => {
    const first = await planChatStep([user(request)],['translate','write_file']);
    const messages = [user(request), callMessage('translation',first.calls[0].tool,first.calls[0].arguments),
      {role:'tool',name:'translate',tool_call_id:'translation',content:'ENOENT: source unavailable',is_error:true}];
    const retry = await planChatStep(messages,['translate','write_file']);
    assert.equal(retry.kind,'tool_calls');
    assert.equal(retry.calls.length,1);
    assert.equal(retry.calls[0].tool,'translate');
    assert.equal(retry.calls[0].arguments,first.calls[0].arguments);
    messages.push(callMessage('translation-retry',retry.calls[0].tool,retry.calls[0].arguments),
      {role:'tool',name:'translate',tool_call_id:'translation-retry',content:'ENOENT: source unavailable',is_error:true});
    const plan = await planChatStep(messages,['translate','write_file']);
    assert.equal(plan.kind,'final');
    assert.ok(plan.answer.includes('ENOENT'));
    assert.equal(plan.calls,undefined);
  });
});


describe('translation source operand fidelity', () => {
  it('preserves the original case in legacy and owned source operands', () => {
    const prompt = 'Translate js/WeatherSource.js to TypeScript and write it';
    assert.equal(sourceTreeRequest(prompt).path, 'js/WeatherSource.js');
    assert.equal(ownedSourceTreeRequest(prompt).request.path, 'js/WeatherSource.js');
  });
  it('refuses non-scalar source text while retaining valid Unicode source spans', () => {
    for (const code of ['\ud800', '\udc00']) {
      const character = String.fromCharCode(Number.parseInt(code.slice(2), 16));
      assert.equal(ownedSourceTreeRequest('Translate js/'+character+'.js to TypeScript and write it'), null);
    }
    const prompt = 'Translate js/😀.js to TypeScript and write it';
    const owned = ownedSourceTreeRequest(prompt);
    assert.equal(new TextDecoder().decode(new TextEncoder().encode(prompt).slice(...owned.source_span)), owned.request.path);
  });
});
