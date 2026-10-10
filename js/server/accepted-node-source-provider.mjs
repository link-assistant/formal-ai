import { serverMessage } from "./messages.mjs";
import { deriveCompleteSourceRequest } from "../agentic/module_function/complete-source-preflight.mjs";
import { readPolicyBlocksPlan } from "../agentic/file_read/ownership.mjs";
import { planOne } from "../agentic/plan.mjs";
import { readArguments } from "../agentic/workspace_change.mjs";
import { observedCallableRequest } from "../agentic/module_function.mjs";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { pathToFileURL } from "node:url";
import { tokenizeWithSpans } from "../agentic/crate/es_tokenizer.mjs";
import { observeSourceCallables } from "../agentic/module_function/callable_catalog.mjs";
const witnessed = new WeakMap();
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
function decodeSourceBytes(bytes, field) {
  try {
    return new TextDecoder('utf-8', {
      fatal: true,
      ignoreBOM: true
    }).decode(bytes);
  } catch {
    const error = new Error('IncompleteUtf8SourceOperation');
    error.byteReceipt = Object.freeze({
      field,
      bytesBase64: Buffer.from(bytes).toString('base64'),
      bytesCount: bytes.length
    });
    throw error;
  }
}
function readBounded(file) {
  const fileStatus = fs.statSync(file);
  assert.ok(fileStatus.isFile(), serverMessage("source-operation-non-file-source"));
  const size = fileStatus.size;
  assert.ok(size <= 1024 * 1024, 'oversized source');
  const bytes = fs.readFileSync(file);
  assert.equal(bytes.length, size, serverMessage("source-operation-source-changed-during-read"));
  return bytes;
}
/** Host policy is constructor-owned; planner messages cannot supply or widen it. */
export function createAcceptedNodeSourceProvider(configuration) {
  const requestSeed = configuration.requestSeed;
  const policy = Object.freeze({
    workspace: configuration.workspace,
    acceptanceOperand: configuration.acceptanceOperand,
    request: configuration.request,
    acceptedCommands: Object.freeze([...configuration.acceptedCommands]),
    sources: Object.freeze(configuration.sources.map(source => Object.freeze({
      ...source
    })))
  });
  assert.ok(deriveCompleteSourceRequest(policy.request, requestSeed), 'UnboundProviderRequest');
  const requestedOperands = [policy.acceptanceOperand, ...policy.sources.map(source => source.path)];
  for (const operand of requestedOperands) {
    assert.equal(typeof operand, 'string', 'UnknownReadOperand');
    assert.equal(readPolicyBlocksPlan(policy.request, planOne('read', readArguments(operand))), false, 'SourceReadAuthorityRefused');
  }
  const owner = Object.freeze({});
  assert.equal(process.env.NODE_TEST_CONTEXT, undefined, serverMessage("source-operation-nested-node-test-operation-unknown"));
  assert.equal(process.env.NODE_OPTIONS, undefined, serverMessage("source-operation-extra-node-options"));
  assert.equal(process.env.NODE_PATH, undefined, serverMessage("source-operation-foreign-module-path"));
  const environmentIdentity = hash(JSON.stringify(Object.entries(process.env).sort()));
  const nodePath = (process.env.PATH ?? '').split(path.delimiter).map(folder => path.resolve(folder, 'node')).find(file => {
    try {
      fs.accessSync(file, fs.constants.X_OK);
      return fs.statSync(file).isFile();
    } catch {
      return false;
    }
  });
  assert.ok(nodePath, serverMessage("source-operation-missing-command-executable"));
  assert.equal(fs.realpathSync(nodePath), fs.realpathSync(process.execPath), serverMessage("source-operation-different-accepted-node-executable"));
  const workspace = fs.realpathSync(policy.workspace),
    acceptance = path.resolve(workspace, policy.acceptanceOperand);
  assert.ok(/^\/[A-Za-z0-9_./-]+$/u.test(workspace) && /^\/[A-Za-z0-9_./-]+$/u.test(acceptance), serverMessage("source-operation-unknown-module-url-byte-profile"));
  assert.equal(path.dirname(acceptance), workspace, serverMessage("source-operation-unsupported-acceptance-location"));
  assert.equal(fs.realpathSync(acceptance), acceptance, 'acceptance symlink');
  const command = 'node --test ' + policy.acceptanceOperand;
  assert.ok(policy.acceptedCommands.includes(command), serverMessage("source-operation-operation-not-accepted"));
  assert.ok(policy.request.includes(command), serverMessage("source-operation-command-not-in-original-request"));
  const requestIdentity = hash(policy.request),
    acceptanceIdentity = hash(readBounded(acceptance));
  const declared = observedCallableRequest(policy.request);
  const destination = declared === null ? null : path.resolve(workspace, declared.destination);
  if (destination !== null) {
    assert.ok(!path.isAbsolute(declared.destination), 'AbsoluteCandidateOperand');
    assert.ok(destination.startsWith(workspace + path.sep), 'OutsideCandidateWorkspace');
    assert.ok(!declared.destination.split(/[\\/]/u).includes('..'), 'CandidateParentTraversal');
  }
  const directoryIdentity = directory => {
    try {
      const status = fs.lstatSync(directory, { bigint: true });
      assert.ok(status.isDirectory() && !status.isSymbolicLink(), 'UnownedCandidateParent');
      assert.equal(fs.realpathSync(directory), directory, 'CandidateParentSymlink');
      return [status.dev, status.ino, status.birthtimeNs].join(':');
    } catch (error) {
      if (error.code === 'ENOENT') return null;
      throw error;
    }
  };
  const workspaceAncestors = [];
  let ancestor = workspace;
  for (;;) {
    workspaceAncestors.push({ path: ancestor, identity: directoryIdentity(ancestor) });
    const parent = path.dirname(ancestor);
    if (parent === ancestor) break;
    ancestor = parent;
  }
  const parentDirectories = [];
  if (destination !== null) {
    let directory = path.dirname(destination);
    while (directory !== workspace) {
      assert.ok(directory.startsWith(workspace + path.sep), 'OutsideCandidateParent');
      parentDirectories.unshift({ path: directory, identity: directoryIdentity(directory) });
      directory = path.dirname(directory);
    }
  }
  const verifyParentDirectories = () => {
    for (const ancestor of workspaceAncestors) {
      assert.equal(directoryIdentity(ancestor.path), ancestor.identity, 'WorkspaceRootDrift');
    }
    for (const directory of parentDirectories) {
      assert.equal(directoryIdentity(directory.path), directory.identity, 'CandidateParentDrift');
    }
  };
  function pathResolveCandidate(operand) {
    return path.resolve(workspace, operand);
  }
  function candidateIdentity() {
    verifyParentDirectories();
    if (destination === null) return null;
    try {
      const fileStatus = fs.lstatSync(destination);
      assert.ok(fileStatus.isFile() && !fileStatus.isSymbolicLink(), serverMessage("source-operation-unowned-candidate-file"));
      return hash(readBounded(destination));
    } catch (error) {
      if (error.code === 'ENOENT') return null;
      throw error;
    }
  }
  const copies = policy.sources.map(source => {
    const full = path.resolve(workspace, source.path);
    assert.ok(full.startsWith(workspace + path.sep));
    assert.equal(fs.realpathSync(full), full, 'copy symlink');
    const bytes = readBounded(full);
    assert.equal(hash(bytes), source.sha256, serverMessage("source-operation-unobserved-source-copy"));
    return {
      path: full,
      content: decodeSourceBytes(bytes, 'source'),
      sha256: hash(bytes)
    };
  });
  const catalog = observeSourceCallables(decodeSourceBytes(readBounded(acceptance), 'acceptance'), acceptance);
  const imports = catalog.imports.filter(value => path.isAbsolute(value.specifier));
  assert.ok(imports.length > 0, serverMessage("source-operation-no-canonical-imports"));
  const bindings = imports.map(imported => {
    assert.ok(/^\/[A-Za-z0-9_./-]+$/u.test(imported.specifier), serverMessage("source-operation-ambiguous-import-url-profile"));
    assert.equal(path.extname(imported.specifier), '.mjs', serverMessage("source-operation-unknown-import-type"));
    assert.equal(fs.realpathSync(imported.specifier), imported.specifier, serverMessage("source-operation-canonical-symlink-normalization-mismatch"));
    const bytes = readBounded(imported.specifier),
      identity = hash(bytes);
    const matches = copies.filter(copy => copy.sha256 === identity);
    assert.equal(matches.length, 1, serverMessage("source-operation-canonical-bytes-not-one-observed-source"));
    const sourceCatalog = observeSourceCallables(matches[0].content, matches[0].path);
    assert.ok(imported.bindings.length > 0, serverMessage("source-operation-side-effect-import-has-no-source-call-authority"));
    for (const binding of imported.bindings) {
      const exported = sourceCatalog.exports.filter(value => value.exposed === binding.imported);
      assert.equal(exported.length, 1, serverMessage("source-operation-unknown-source-export"));
      assert.ok(sourceCatalog.declarations.some(value => value.name === exported[0].local), serverMessage("source-operation-unobserved-export-declaration"));
    }
    return Object.freeze({
      path: imported.specifier,
      moduleURL: pathToFileURL(imported.specifier).href,
      sha256: identity,
      content: decodeSourceBytes(bytes, 'source'),
      bindings: Object.freeze(imported.bindings.map(value => Object.freeze({
        ...value
      }))),
      observedCopy: matches[0].path
    });
  });
  const closure = new Map();
  function capture(file) {
    if (closure.has(file)) return;
    assert.ok(closure.size < 100, serverMessage("source-operation-unbounded-import-closure"));
    assert.equal(path.extname(file), '.mjs', serverMessage("source-operation-unknown-closure-module"));
    assert.equal(fs.realpathSync(file), file, 'closure symlink');
    const bytes = readBounded(file),
      content = decodeSourceBytes(bytes, 'source');
    closure.set(file, Object.freeze({
      path: file,
      sha256: hash(bytes),
      content
    }));
    const trees = tokenizeWithSpans(content);
    function rejectLoaders(items) {
      for (let index = 0; index < items.length; index++) {
        const node = items[index];
        if (node?.$ === 'group') rejectLoaders(node.trees);
        if (node?.kind === 'identifier' && ['import', 'require', 'eval', 'Function'].includes(node.text) && items[index + 1]?.$ === 'group' && items[index + 1].delim === 'paren') throw new Error(serverMessage("source-operation-unknown-dynamic-module-code-authority"));
        if (node?.text === 'from' && items[index + 1]?.kind === 'string') {
          const raw = items[index + 1].text;
          assert.ok(!raw.slice(1, -1).includes(String.fromCharCode(92)), 'escaped dependency');
          assert.ok(observeSourceCallables(content, file).imports.some(value => value.specifier === raw.slice(1, -1)), 'unobserved reexport');
        }
      }
    }
    rejectLoaders(trees);
    const catalog = observeSourceCallables(content, file);
    assert.ok(!catalog.gaps.some(gap => gap.reason === 'LexicalFailure'), serverMessage("source-operation-unparsed-import-closure"));
    for (const imported of catalog.imports) {
      assert.ok(imported.specifier.startsWith('.'), serverMessage("source-operation-unknown-dependency-authority"));
      assert.ok(/^[A-Za-z0-9_./-]+$/u.test(imported.specifier), serverMessage("source-operation-ambiguous-dependency-url-profile"));
      assert.ok(!imported.specifier.includes('?') && !imported.specifier.includes('#'), serverMessage("source-operation-distinct-dependency-url"));
      capture(path.resolve(path.dirname(file), imported.specifier));
    }
  }
  for (const source of bindings) capture(source.path);
  function fresh() {
    verifyParentDirectories();
    assert.equal(hash(JSON.stringify(Object.entries(process.env).sort())), environmentIdentity, 'environment drift');
    for (const source of closure.values()) assert.equal(hash(readBounded(source.path)), source.sha256, 'dependency freshness');
    assert.equal(hash(readBounded(acceptance)), acceptanceIdentity, 'acceptance freshness');
    for (const copy of copies) assert.equal(hash(readBounded(copy.path)), copy.sha256, 'copy freshness');
    for (const source of bindings) assert.equal(hash(readBounded(source.path)), source.sha256, 'canonical freshness');
  }
  return Object.freeze({
    createCandidateIO() {
      assert.ok(destination !== null, 'UnboundCandidateDestination');
      assert.equal(candidateIdentity(), null, 'ExistingCandidateDestination');
      let active = true,
        created = null;
      const createdDirectories = [];
      verifyParentDirectories();
      const fileIdentity = () => {
        try {
          const fileStatus = fs.lstatSync(destination, {
            bigint: true
          });
          if (!fileStatus.isFile() || fileStatus.isSymbolicLink()) return null;
          return [fileStatus.dev, fileStatus.ino, fileStatus.birthtimeNs, fileStatus.ctimeNs].join(':');
        } catch (error) {
          if (error.code === 'ENOENT') return null;
          throw error;
        }
      };
      const owns = () => {
        if (!active) return false;
        try { verifyParentDirectories(); return true; }
        catch { return false; }
      };
      const removeOwnedEmptyDirectories = () => {
        for (const directory of [...createdDirectories].reverse()) {
          try {
            if (directoryIdentity(directory.path) !== directory.identity) continue;
            if (fs.readdirSync(directory.path).length !== 0) continue;
            fs.rmdirSync(directory.path);
            parentDirectories.find(parent => parent.path === directory.path).identity = null;
          } catch {
            // Unknown or replaced directory ownership retains physical bytes.
          }
        }
      };
      const ownsCreatedFile = () => created !== null && fileIdentity() === created;
      const inspect = operand => {
        assert.equal(pathResolveCandidate(operand), destination, 'DifferentCandidateOperand');
        try {
          const fileStatus = fs.lstatSync(destination);
          return fileStatus.isFile() && !fileStatus.isSymbolicLink() ? {
            kind: 'file',
            identity: candidateIdentity()
          } : {
            kind: 'other'
          };
        } catch (error) {
          if (error.code === 'ENOENT') return {
            kind: 'absent'
          };
          throw error;
        }
      };
      return Object.freeze({
        ownsLease: owns,
        inspect,
        writeCandidate(source) {
          assert.ok(active && created === null, 'InactiveCandidateWriteLease');
          assert.equal(candidateIdentity(), null, 'CandidateAlreadyExists');
          verifyParentDirectories();
          try {
            for (const directory of parentDirectories) {
              if (directory.identity !== null) continue;
              fs.mkdirSync(directory.path, { mode: 0o700 });
              directory.identity = directoryIdentity(directory.path);
              createdDirectories.push({ ...directory });
            }
            fs.writeFileSync(destination, source, {
              flag: 'wx',
              mode: 0o600
            });
            created = fileIdentity();
            assert.ok(created !== null, 'MissingOwnedCandidateFile');
          } catch (error) {
            removeOwnedEmptyDirectories();
            throw error;
          }
        },
        processDisposition(receipt) {
          const witness = witnessed.get(receipt);
          assert.ok(active && ownsCreatedFile(), 'UnownedCandidateFileReceipt');
          assert.ok(witness && witness.owner === owner, 'UnownedProcessReceipt');
          witness.fresh();
          return receipt.process.status === 0 ? 'succeeded' : 'failed';
        },
        removeUnchangedCandidate(operand, expected) {
          assert.equal(pathResolveCandidate(operand), destination, 'DifferentCandidateOperand');
          if (!owns() || !ownsCreatedFile() || candidateIdentity() !== expected) return false;
          fs.unlinkSync(destination);
          removeOwnedEmptyDirectories();
          active = false;
          return true;
        },
        release() {
          active = false;
        }
      });
    },
    observe(actualCommand) {
      assert.equal(actualCommand, command, 'different command');
      fresh();
      const candidateBefore = candidateIdentity();
      const result = spawnSync(process.execPath, ['--test', policy.acceptanceOperand], {
        cwd: workspace,
        timeout: 30000,
        maxBuffer: 4 * 1024 * 1024
      });
      fresh();
      assert.equal(result.signal, null, 'incomplete process');
      assert.equal(result.error, undefined, serverMessage("source-operation-failed-process-transport"));
      assert.ok(Number.isInteger(result.status), serverMessage("source-operation-missing-process-status"));
      let stdout, stderr;
      try {
        stdout = decodeSourceBytes(result.stdout, 'stdout');
        stderr = decodeSourceBytes(result.stderr, 'stderr');
      } catch (error) {
        error.processReceipt = Object.freeze({
          command,
          cwd: workspace,
          status: result.status,
          signal: result.signal,
          stdoutBase64: Buffer.from(result.stdout).toString('base64'),
          stderrBase64: Buffer.from(result.stderr).toString('base64')
        });
        throw error;
      }
      const receipt = Object.freeze({
        kind: 'provider-observed-accepted-import-bytes',
        requestIdentity,
        acceptanceIdentity,
        command,
        process: Object.freeze({
          executable: process.execPath,
          argv: Object.freeze(['--test', policy.acceptanceOperand]),
          cwd: workspace,
          status: result.status,
          signal: result.signal
        }),
        sources: Object.freeze(bindings),
        dependencies: Object.freeze([...closure.values()]),
        stdout,
        stderr,
        authority: 'same accepted Node import source scope; provider byte witness only',
        moduleEffects: 'inherited approved operation; not purity',
        runtimeLoadedByteRaceProof: false
      });
      witnessed.set(receipt, {
        owner,
        fresh,
        requestIdentity,
        command,
        acceptanceIdentity,
        destination,
        candidateBefore,
        candidateAfter: candidateIdentity()
      });
      return receipt;
    },
    candidateStatus(receipt, request, actualCommand, actualWorkspace, path, identity) {
      this.validate(receipt, request, actualCommand);
      const proof = witnessed.get(receipt);
      assert.equal(actualWorkspace, workspace, 'foreign workspace');
      assert.equal(proof.destination, pathResolveCandidate(path), 'foreign candidate');
      assert.equal(proof.candidateBefore, identity, serverMessage("source-operation-operation-predates-candidate"));
      assert.equal(proof.candidateAfter, identity, serverMessage("source-operation-candidate-changed-during-operation"));
      assert.equal(candidateIdentity(), identity, serverMessage("source-operation-candidate-drift-after-operation"));
      return receipt.process.status;
    },
    validate(receipt, request, actualCommand) {
      const proof = witnessed.get(receipt);
      assert.ok(proof, serverMessage("source-operation-caller-metadata-is-not-provider-receipt"));
      assert.equal(proof.owner, owner, serverMessage("source-operation-foreign-provider-receipt"));
      assert.equal(proof.requestIdentity, hash(request));
      assert.equal(proof.command, actualCommand);
      assert.equal(proof.acceptanceIdentity, acceptanceIdentity);
      proof.fresh();
      return receipt.sources;
    }
  });
}
export function acceptedOperationSourceBytes(receipt, request, command, workspace) {
  const proof = witnessed.get(receipt);
  assert.ok(proof, serverMessage("source-operation-not-provider-issued"));
  assert.equal(proof.requestIdentity, hash(request), 'request authority');
  assert.equal(proof.command, command, 'command authority');
  assert.equal(receipt.process.cwd, workspace, 'workspace authority');
  proof.fresh();
  return receipt.sources;
}
