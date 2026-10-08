// Node authoring adapter: declared artifact bytes, local session and explicit landing.
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { fileURLToPath } from 'node:url';
import { startServer as startLocalServer } from './main.mjs';
import { runAuthoringWith } from '../agentic/crate/repository_workspace_authoring.mjs';
import { nodeRepositoryIo } from './repository-workspace.mjs';

const REPOSITORY = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

function validate(args) {
  if (args.model && !args.model.startsWith('formal-ai/')) throw new Error('authoring requires the Formal AI model');
  if (!args.task || !args.message) throw new Error('authoring task and message are required');
  if (!/^https:\/\/github\.com\/[^/]+\/[^/]+\/pull\/[1-9][0-9]*$/u.test(args.pull_request ?? '')) {
    throw new Error('a canonical GitHub pull-request URL is required');
  }
  if (!args.produces?.length || (args.into?.length ?? 0) > args.produces.length) throw new Error('declared produced artifacts and matching destinations are required');
  for (const relative of [...args.produces, ...(args.into ?? []), ...(args.context ?? []), args.evidence ?? 'authoring-evidence']) {
    if (!relative || path.isAbsolute(relative) || relative.split(/[\\/]/u).includes('..')) throw new Error('artifact paths must stay relative: ' + relative);
  }
  const repository = path.resolve(args.repository);
  const workspace = path.resolve(args.workspace);
  if (repository === workspace || workspace.startsWith(repository + path.sep)) throw new Error('authoring workspace must be outside the destination repository');
  return { repository, workspace };
}

/** Live Node authoring through the shared stage document; host ports may be injected for deterministic replay. */
export async function runNodeAuthoring(args, {
  io = nodeRepositoryIo(), startServer = startLocalServer, runSession = null, classifyStderr = null, steps,
} = {}) {
  const roots = validate(args);
  const evidence = args.evidence ?? 'authoring-evidence';
  const into = args.produces.map((produced, index) => args.into?.[index] ?? produced);
  const state = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-authoring-state-'));
  const model = args.model ?? 'formal-ai/javascript';
  let server = null; let stream = ''; let session = ''; let destinations = []; let commitMessage = '';
  const baseline = [];
  const readArtifact = (root, relative) => io.readBytes ? io.readBytes(root, relative) : io.read(root, relative);
  const readPrior = (root, relative) => io.readOptionalBytes ? io.readOptionalBytes(root, relative) : io.readOptional(root, relative);
  const sameBytes = (left, right) => left !== null && right !== null && Buffer.from(left).equals(Buffer.from(right));
  const writeEvidence = (relative, contents) => io.write(roots.repository, evidence + '/' + relative, contents);
  const observedCommand = async (root, program, argumentsList, policy = {}) => {
    const result = await io.run(root, program, argumentsList, { deadline_seconds: 300, network: 'denied', ...policy });
    if (result.missing || result.timed_out || result.exit_code !== 0) throw new Error(program + ' did not complete: ' + (result.stderr ?? result.exit_code));
    return result;
  };
  const stages = {
    clone: async () => {
      await io.prepare(roots.workspace);
      // Context is an explicit file set, never a recursive repository copy.
      if (args.seed) for (const relative of new Set([...args.produces, ...(args.context ?? [])])) {
        const contents = await readPrior(args.seed, relative);
        if (contents !== null) await io.write(roots.workspace, relative, contents);
      }
      await writeEvidence('task.txt', args.task + '\n');
      return 'observed';
    },
    locate: async () => {
      // Probe every destination path through the same confinement checks before editing.
      for (const destination of into) await io.readOptional(roots.repository, destination);
      return 'observed';
    },
    read: async () => {
      for (let index = 0; index < args.produces.length; index += 1) {
        baseline.push({ seed: args.seed ? await readPrior(args.seed, args.produces[index]) : null,
          destination: await readPrior(roots.repository, into[index]) });
      }
      return 'observed';
    },
    serve: async () => {
      server = await startServer({ host: '127.0.0.1', port: args.port ?? 0, agentMode: true,
        env: { ...process.env, FORMAL_AI_MEMORY_PATH: path.join(state, 'memory.lino'), FORMAL_AI_DREAMING: '0' } });
      if (!server?.url) throw new Error('local authoring server readiness was not observed');
      return 'observed';
    },
    edit: async () => {
      if (!server?.url) throw new Error('authoring session requires an observed local server');
      const configuration = { provider: { formalai: { name: 'Formal AI', npm: '@ai-sdk/openai-compatible',
        options: { baseURL: server.url + '/api/openai/v1', apiKey: 'local' }, models: { 'formal-ai': { name: 'Formal AI' } } } },
        model: 'formalai/formal-ai' };
      const result = runSession ? await runSession({ root: roots.workspace, task: args.task, url: server.url, configuration })
        : await observedCommand(roots.workspace, args.agent_executable ?? 'agent', [
          '--model', 'formalai/formal-ai', '--permission-mode', 'auto', '--no-summarize-session', '--no-generate-title',
          '--output-format', 'stream-json', '--compact-json', '--disable-stdin', '--prompt', args.task,
        ], { deadline_seconds: args.deadline_seconds ?? 300,
          env: { FORMAL_AI_API_KEY: 'local', LINK_ASSISTANT_AGENT_CONFIG_CONTENT: JSON.stringify(configuration) } });
      if (result.exit_code !== 0 || result.timed_out || result.missing) throw new Error('authoring session did not complete');
      await writeEvidence('agent-stderr.log', result.stderr ?? '');
      if (classifyStderr) {
        if (!await classifyStderr(result.stderr ?? '')) throw new Error('authoring session stderr classification failed');
      } else {
        await observedCommand(roots.repository, 'bash', [path.join(REPOSITORY, 'scripts/classify-agent-cli-stderr.sh'),
          path.join(roots.repository, evidence, 'agent-stderr.log')]);
      }
      stream = (result.stdout ?? '').split('\n').filter((line) => line.startsWith('{')).join('\n');
      await writeEvidence('agent-stream.jsonl', stream + '\n');
      return 'observed';
    },
    session: async () => {
      const find = (value) => {
        if (!value || typeof value !== 'object') return;
        if (typeof value.session_id === 'string' && /^ses_.+/u.test(value.session_id) && !session) session = value.session_id;
        for (const child of Object.values(value)) find(child);
      };
      for (const line of stream.split('\n').filter(Boolean)) find(JSON.parse(line));
      if (!session) throw new Error('authoring stream reported no resumable session');
      await writeEvidence('session-id.txt', 'formal-ai session ' + session + '\nformal-ai model ' + model + '\n');
      return 'observed';
    },
    verify: async () => {
      const contents = await Promise.all(args.produces.map((produced) => readArtifact(roots.workspace, produced)));
      for (const expected of args.contains ?? []) if (!contents.some((source) => source.includes(Buffer.from(expected)))) {
        throw new Error('no artifact contains: ' + expected);
      }
      return 'observed';
    },
    diff: async () => {
      const contents = await Promise.all(args.produces.map((produced) => readArtifact(roots.workspace, produced)));
      if (!contents.some((source, index) => !sameBytes(source, baseline[index].destination)
        && (!args.seed || !sameBytes(source, baseline[index].seed)))) throw new Error('no artifact differs from both seed and destination');
      for (let index = 0; index < contents.length; index += 1) {
        await io.write(roots.repository, into[index], contents[index]);
        if (!sameBytes(await readArtifact(roots.repository, into[index]), contents[index])) throw new Error('landed artifact bytes were not observed');
      }
      destinations = [...into];
      return 'observed';
    },
    commit: async () => {
      const trailers = 'Formal-AI-Session: ' + session + '\nFormal-AI-Model: ' + model
        + '\nFormal-AI-Evidence: ' + evidence + '\nFormal-AI-Pull-Request: ' + args.pull_request + '\n';
      await observedCommand(roots.repository, 'git', ['add', '--', ...destinations, evidence]);
      const diff = await observedCommand(roots.repository, 'git', ['diff', '--cached']);
      if (!diff.stdout) throw new Error('authoring reproduced committed bytes');
      commitMessage = args.message + '\n\n' + trailers;
      await observedCommand(roots.repository, 'git', ['commit', '-m', commitMessage]);
      return 'requested';
    },
  };
  try {
    const outcome = await runAuthoringWith(stages, { steps, commit: args.commit === true,
      writeTrace: (trace) => writeEvidence('repository-protocol.lino', trace) });
    return { ...outcome, session_id: session, model, destinations, commit_message: commitMessage };
  } finally {
    if (server?.close) await server.close();
    else if (server?.server) await new Promise((resolve, reject) => server.server.close((error) => error ? reject(error) : resolve()));
    fs.rmSync(state, { recursive: true, force: true });
  }
}
