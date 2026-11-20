import { serverMessage } from '../messages.mjs';
import { closeSync, lstatSync, openSync } from 'node:fs';
import { DatabaseSync } from 'node:sqlite';

import { boundedManifest, namedExport, packageLinks } from './packages.mjs';
import { analyzeRecipe, compileRecipe, readRecipe, renderRecipe, runRecipe } from './recipe.mjs';
import { linkContext, parseLinkSubstitution, runCascade, snapshot } from './substitutions.mjs';

const STATE_LIMIT = 4 * 1024 * 1024;
const SAMPLE_WINDOW = 64;
const MINIMUM_SAMPLE = 16;
const PROMOTION_THRESHOLD = 8;
const DEMOTION_THRESHOLD = 1;

export class MetaRequestError extends Error {
  constructor(status, message) {
    super(message);
    this.status = status;
  }
}

export function requireApproval(request, actor, { admin = false } = {}) {
  if (admin && actor.admin !== true) throw new MetaRequestError(403, serverMessage('dynamic-meta-admin-required'));
  if (request.confirm !== true) throw new MetaRequestError(409, serverMessage('dynamic-meta-confirmation-required'));
}

function openDatabase(path) {
  if (path !== ':memory:') {
    try {
      closeSync(openSync(path, 'wx', 0o600));
    } catch (error) {
      if (error.code !== 'EEXIST') throw error;
    }
    if (!lstatSync(path).isFile()) throw new Error(serverMessage('dynamic-meta-regular-store'));
  }
  const database = new DatabaseSync(path);
  database.exec(`
    PRAGMA busy_timeout = 1000;
    CREATE TABLE IF NOT EXISTS meta_state (
      id INTEGER PRIMARY KEY CHECK (id = 1),
      document TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS meta_receipts (
      revision INTEGER PRIMARY KEY,
      receipt TEXT NOT NULL
    );
  `);
  return database;
}

function emptyState() {
  return {
    schema: 1,
    revision: 0,
    graph: { links: [], names: [] },
    packages: [],
    settings: { selfModification: null },
    ruleOverrides: {},
    compiled: {},
    usage: [],
    review: [],
  };
}

function recipes(packages) {
  return packages.flatMap((pkg) => pkg.recipes.map(({ id }) => `${pkg.id}/${id}`)).sort();
}

function equilibrium(state) {
  if (state.usage.length < MINIMUM_SAMPLE) return [];
  const frequency = new Map();
  for (const name of state.usage) frequency.set(name, (frequency.get(name) ?? 0) + 1);
  return recipes(state.packages).flatMap((name) => {
    const uses = frequency.get(name) ?? 0;
    if (!Object.hasOwn(state.compiled, name) && uses >= PROMOTION_THRESHOLD) {
      return [{ command: `Compile recipe ${name}`, uses, window: state.usage.length }];
    }
    if (Object.hasOwn(state.compiled, name) && uses <= DEMOTION_THRESHOLD) {
      return [{ command: `Interpret recipe ${name}`, uses, window: state.usage.length }];
    }
    return [];
  });
}

function command(state, input) {
  if (typeof input !== 'string' || input.length > 180) throw new Error(serverMessage('dynamic-meta-invalid-command'));
  const text = input.trim();
  const selfModification = /^(enable|disable) (?:dynamic )?self-modification\.?$/iu.exec(text);
  if (selfModification) {
    const enabled = selfModification[1].toLowerCase() === 'enable';
    state.settings.selfModification = enabled;
    return { selfModification: enabled };
  }
  const activation = /^(enable|disable) rule (\S+)$/iu.exec(text);
  if (activation) {
    const enabled = activation[1].toLowerCase() === 'enable';
    const rule = activation[2];
    namedExport(state.packages, 'rules', rule);
    state.ruleOverrides[rule] = enabled;
    return { rule, enabled };
  }
  const compilation = /^(compile|interpret) recipe (\S+)$/iu.exec(text);
  if (!compilation) throw new Error(serverMessage('dynamic-meta-unknown-command'));
  const name = compilation[2];
  const { program } = namedExport(state.packages, 'recipes', name);
  const compiled = compilation[1].toLowerCase() === 'compile';
  if (compiled) state.compiled[name] = compileRecipe(program);
  else delete state.compiled[name];
  return { recipe: name, implementation: compiled ? 'compiled' : 'dynamic' };
}

function run(state, request) {
  const { program } = namedExport(state.packages, 'recipes', request.recipe);
  const compiled = state.compiled[request.recipe];
  return {
    implementation: compiled ? 'compiled' : 'dynamic',
    ...runRecipe(program, request.input, { maxSteps: request.maxSteps, compiled }),
  };
}

export function createMetaRuntime({ path = ':memory:', triggers = true, selfModification = false, adapters = new Map() } = {}) {
  const database = openDatabase(path);
  const load = database.prepare('SELECT document FROM meta_state WHERE id = 1');
  const save = database.prepare('INSERT OR REPLACE INTO meta_state (id, document) VALUES (1, ?)');
  const saveReceipt = database.prepare('INSERT INTO meta_receipts VALUES (?, ?)');
  const receipt = database.prepare('SELECT receipt FROM meta_receipts WHERE revision = ?');
  const prune = database.prepare('DELETE FROM meta_receipts WHERE revision <= ?');
  const read = () => {
    const record = load.get();
    const state = record ? JSON.parse(record.document) : emptyState();
    if (state.schema !== 1) throw new Error(serverMessage('dynamic-meta-schema'));
    return state;
  };

  function transaction(action, operation, expectedRevision) {
    database.exec('BEGIN IMMEDIATE');
    try {
      const state = read();
      if (expectedRevision !== undefined && expectedRevision !== state.revision) {
        throw new MetaRequestError(409, serverMessage('dynamic-meta-revision-changed'));
      }
      const result = operation(state);
      if (result === null) {
        database.exec('COMMIT');
        return { revision: state.revision };
      }
      state.revision += 1;
      const document = JSON.stringify(state);
      if (Buffer.byteLength(document) > STATE_LIMIT) throw new Error(serverMessage('dynamic-meta-state-size'));
      const response = { revision: state.revision, ...result };
      save.run(document);
      saveReceipt.run(state.revision, JSON.stringify({ action, ...response }));
      prune.run(state.revision - 64);
      database.exec('COMMIT');
      return response;
    } catch (error) {
      database.exec('ROLLBACK');
      throw error;
    }
  }

  function substitute(state, initial) {
    const context = linkContext(state.graph);
    const rules = state.packages.flatMap((pkg) => pkg.rules.map((rule) => ({
      ...rule, id: `${pkg.id}/${rule.id}`,
    }))).sort((left, right) => left.id < right.id ? -1 : left.id > right.id ? 1 : 0);
    const trace = runCascade(context, initial, {
      rules,
      active: (id) => state.ruleOverrides[id] !== false,
      enabled: triggers,
      selfModification: state.settings.selfModification ?? selfModification,
      setRule: (id, value) => { state.ruleOverrides[id] = value; },
    });
    state.graph = snapshot(context);
    return { trace };
  }

  async function act(request, actor = {}) {
    if (!request || typeof request.action !== 'string') throw new Error(serverMessage('dynamic-meta-action-required'));
    const { action, expectedRevision } = request;
    switch (action) {
      case 'inspect': {
        const state = read();
        const active = state.packages.flatMap((pkg) => pkg.rules.map(({ id }) => {
          const rule = `${pkg.id}/${id}`;
          return [rule, state.ruleOverrides[rule] !== false];
        }));
        return {
          revision: state.revision, graph: state.graph, packages: state.packages,
          types: packageLinks(state.packages), rules: Object.fromEntries(active),
          compiled: Object.keys(state.compiled), proposals: equilibrium(state), lastReview: state.review,
          settings: { triggers, selfModification: state.settings.selfModification ?? selfModification },
        };
      }
      case 'receipt': {
        if (!Number.isSafeInteger(request.revision) || request.revision < 1) throw new Error(serverMessage('dynamic-meta-invalid-receipt'));
        const row = receipt.get(request.revision);
        if (!row) throw new MetaRequestError(404, serverMessage('dynamic-meta-receipt-unavailable'));
        return JSON.parse(row.receipt);
      }
      case 'translate':
        return { source: renderRecipe(request.to, readRecipe(request.from, request.source)) };
      case 'analyze': {
        const program = request.recipe === undefined ? readRecipe(request.from, request.source) :
          namedExport(read().packages, 'recipes', request.recipe).program;
        return analyzeRecipe(program);
      }
      case 'compare': {
        const state = read();
        const left = run(state, { ...request, recipe: request.left });
        const right = run(state, { ...request, recipe: request.right });
        const complete = [left, right].every(({ halt }) => ['no_applicable_rule', 'terminal_rule'].includes(halt.kind));
        return { left, right, observedAgreement: left.output === right.output, complete };
      }
      case 'run':
        return transaction(action, (state) => {
          const result = run(state, request);
          state.usage = [...state.usage, request.recipe].slice(-SAMPLE_WINDOW);
          return result;
        }, expectedRevision);
      case 'substitute': {
        requireApproval(request, actor);
        const initial = parseLinkSubstitution(request.query);
        return transaction(action, (state) => substitute(state, initial), expectedRevision);
      }
      case 'dataset':
        requireApproval(request, actor);
        return transaction(action, (state) =>
          substitute(state, namedExport(state.packages, 'datasets', request.dataset).substitution), expectedRevision);
      case 'install': {
        requireApproval(request, actor, { admin: true });
        const pkg = boundedManifest(request.manifest);
        if (request.sha256 !== undefined && request.sha256 !== pkg.origin.digest) {
          throw new MetaRequestError(409, serverMessage('dynamic-meta-manifest-digest'));
        }
        return transaction(action, (state) => {
          if (state.packages.some(({ id }) => id === pkg.id)) {
            throw new MetaRequestError(409, serverMessage('dynamic-meta-package-exists'));
          }
          if (state.packages.length >= 32) throw new Error(serverMessage('dynamic-meta-package-limit'));
          state.packages.push(pkg);
          return { package: pkg.id, origin: pkg.origin };
        }, expectedRevision);
      }
      case 'uninstall':
        requireApproval(request, actor, { admin: true });
        return transaction(action, (state) => {
          if (!state.packages.some(({ id }) => id === request.package)) throw new MetaRequestError(404, serverMessage('dynamic-meta-package-unavailable'));
          const prefix = `${request.package}/`;
          state.packages = state.packages.filter(({ id }) => id !== request.package);
          state.usage = state.usage.filter((name) => !name.startsWith(prefix));
          for (const key of Object.keys(state.ruleOverrides)) if (key.startsWith(prefix)) delete state.ruleOverrides[key];
          for (const key of Object.keys(state.compiled)) if (key.startsWith(prefix)) delete state.compiled[key];
          state.review = equilibrium(state);
          return { package: request.package };
        }, expectedRevision);
      case 'command':
        requireApproval(request, actor, { admin: true });
        return transaction(action, (state) => command(state, request.command), expectedRevision);
      case 'integration': {
        requireApproval(request, actor, { admin: true });
        if (expectedRevision !== undefined) throw new Error(serverMessage('dynamic-meta-adapter-revision'));
        const { adapter } = namedExport(read().packages, 'integrations', request.integration);
        const invoke = adapters.get(adapter);
        if (!invoke) throw new MetaRequestError(404, serverMessage('dynamic-meta-adapter-unavailable'));
        return { result: await invoke(request.input, { signal: actor.signal }) };
      }
      default: throw new Error(serverMessage('dynamic-meta-unknown-action'));
    }
  }

  function review() {
    return transaction('idle-review', (state) => {
      const proposals = equilibrium(state);
      if (JSON.stringify(proposals) === JSON.stringify(state.review)) return null;
      state.review = proposals;
      return { proposals };
    });
  }
  return { act, review, close: () => database.close() };
}
