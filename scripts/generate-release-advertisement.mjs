import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import { readFileSync, writeFileSync, existsSync, mkdirSync, realpathSync, lstatSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { join, dirname, resolve, relative, posix } from 'node:path';
import { captureReleaseProtocol, verifyReleaseProtocol } from './release-protocol.mjs';
const require = createRequire(import.meta.url);
const {
  parseDocument
} = require('yaml');
const {
  parse
} = require('@babel/parser');
export const ADVERTISEMENT_SCHEMA = 'release-advertisement/v1';
export const ADVERTISEMENT_ROOTS = Object.freeze(['.github/workflows/desktop-release.yml',
   '.github/workflows/container-images.yml',
   '.github/workflows/publish-engine.yml',
   '.github/workflows/release.yml',
   'packages/formal-ai-engine/package.json',
   'rust/Cargo.toml',
   ".github/workflows/release-staged.yml",
   "desktop/package.json",
   "vscode/package.json",
   "scripts/native-release-artifact.mjs",
   "scripts/native-release-evidence.mjs",
   "scripts/native-release-source.mjs",
   "scripts/native-release-trust.mjs",
   "scripts/build-selected-wasm.mjs",
   "scripts/container-image-receipt.mjs",
   "scripts/verify-anonymous-image-manifest.mjs",
   "scripts/build-container-image.sh",
   "scripts/free-runner-disk.sh",
   "scripts/resolve-package-release.mjs",
   "scripts/publish-browser-engine.mjs",
   "scripts/browser-engine-deadline.mjs",
   "scripts/browser-engine-cold-install.mjs",
   "scripts/configure-dockerhub-publishing.sh",
   "scripts/release-image-factory.mjs",
   ".github/workflows/publish-vscode.yml",
   "scripts/release-extension-publication.mjs",
   "scripts/release-extension-transfer.mjs",
   "scripts/release-native-publication.mjs"]);
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
function workflow(bytes) {
  const doc = parseDocument(bytes.toString('utf8'), {
    uniqueKeys: true
  });
  assert.equal(doc.errors.length, 0, 'invalid or duplicate workflow fields');
  const value = doc.toJSON();
  assert.ok(value && typeof value.jobs === 'object');
  return value;
}
function rows(value, job, fields) {
  const matrix = value.jobs[job]?.strategy?.matrix;
  assert.ok(matrix && Object.keys(matrix).every(key => key === 'include'), 'unsupported publication matrix');
  assert.ok(Array.isArray(matrix.include) && matrix.include.length > 0 && matrix.include.length <= 128);
  const result = matrix.include.map(row => {
    const selected = {};
    for (const field of fields) {
      assert.equal(typeof row[field], 'string', 'missing advertised matrix field: ' + field);
      assert.match(row[field], /^[A-Za-z0-9_.-]+$/u, 'dynamic advertised matrix field');
      selected[field] = row[field];
    }
    return selected;
  });
  assert.equal(new Set(result.map(row => JSON.stringify(row))).size, result.length, 'duplicate advertised target');
  return result.sort((a, b) => JSON.stringify(a).localeCompare(JSON.stringify(b), 'en'));
}
function exportedLiteral(bytes, name) {
  const ast = parse(bytes.toString('utf8'), {
    sourceType: 'module'
  });
  const declarations = ast.program.body.filter(node => node.type === 'ExportNamedDeclaration').flatMap(node => node.declaration?.declarations ?? []).filter(node => node.id.type === 'Identifier' && node.id.name === name);
  assert.equal(declarations.length, 1, 'missing or ambiguous source registry');
  let node = declarations[0].init;
  if (node.type === 'CallExpression') {
    assert.equal(node.callee.type, 'MemberExpression');
    assert.equal(node.callee.computed, false);
    assert.equal(node.callee.object.name, 'Object');
    assert.equal(node.callee.property.name, 'freeze');
    assert.equal(node.arguments.length, 1);
    node = node.arguments[0];
  }
  function value(node) {
    if (node.type === 'StringLiteral') return node.value;
    if (node.type === 'ArrayExpression') return node.elements.map(value);
    assert.equal(node.type, 'ObjectExpression', 'unsupported source registry expression');
    const result = {};
    for (const property of node.properties) {
      assert.equal(property.type, 'ObjectProperty');
      assert.equal(property.computed, false);
      assert.equal(property.shorthand, false);
      const key = property.key.type === 'Identifier' ? property.key.name : property.key.value;
      assert.equal(typeof key, 'string');
      assert.ok(!Object.hasOwn(result, key), 'duplicate source registry key');
      result[key] = value(property.value);
    }
    return result;
  }
  return value(node);
}
function syntaxNodes(root) {
  const result = [],
    queue = [root];
  while (queue.length) {
    const node = queue.pop();
    if (!node || typeof node !== 'object') continue;
    assert.ok(result.length < 20000, 'bounded source syntax required');
    if (typeof node.type === 'string') result.push(node);
    for (const [key, value] of Object.entries(node)) {
      if (['loc', 'start', 'end', 'extra'].includes(key)) continue;
      if (Array.isArray(value)) queue.push(...value);else if (value && typeof value === 'object') queue.push(value);
    }
  }
  return result;
}
function sourceBinding(bytes, name, scopeName = null) {
  const ast = parse(bytes.toString('utf8'), {
    sourceType: 'module'
  });
  let scope = ast.program;
  if (scopeName !== null) {
    const functions = ast.program.body.map(node => node.declaration ?? node).filter(node => node.type === 'FunctionDeclaration' && node.id.name === scopeName);
    assert.equal(functions.length, 1, 'one declared source function required');
    scope = functions[0].body;
  }
  const candidates = syntaxNodes(scope).filter(node => node.type === 'VariableDeclarator' && node.id.type === 'Identifier' && node.id.name === name);
  assert.equal(candidates.length, 1, 'ambiguous or missing producer binding');
  return candidates[0].init;
}
function closedValue(node, context = {}) {
  if (node.type === 'StringLiteral' || node.type === 'BooleanLiteral') return node.value;
  if (node.type === 'ArrayExpression') return node.elements.map(item => closedValue(item, context));
  if (node.type === 'Identifier') {
    assert.ok(Object.hasOwn(context, node.name), 'unknown producer source identity');
    return context[node.name];
  }
  if (node.type === 'BinaryExpression') {
    assert.equal(node.operator, '===', 'unsupported producer comparison');
    return closedValue(node.left, context) === closedValue(node.right, context);
  }
  assert.equal(node.type, 'ConditionalExpression', 'unsupported producer policy expression');
  return closedValue(closedValue(node.test, context) ? node.consequent : node.alternate, context);
}
function extensionDefinition(sources, desktop) {
  const workflowPath = '.github/workflows/publish-vscode.yml';
  const helperPath = 'scripts/release-extension-publication.mjs';
  const producer = sources[helperPath];
  const registries = closedValue(sourceBinding(producer, 'registries'));
  const statuses = closedValue(sourceBinding(producer, 'statuses'));
  for (const values of [registries, statuses]) {
    assert.ok(Array.isArray(values) && values.length > 0 && values.length <= 128);
    assert.equal(new Set(values).size, values.length);
    for (const value of values) assert.match(value, /^[a-z][a-z0-9-]*$/u);
  }
  const result = sourceBinding(producer, 'result', 'publicationReceipt');
  assert.equal(result.type, 'ObjectExpression');
  const literal = name => {
    const fields = result.properties.filter(property => property.type === 'ObjectProperty' && !property.computed && (property.key.name ?? property.key.value) === name);
    assert.equal(fields.length, 1, 'missing or ambiguous outcome field');
    return closedValue(fields[0].value);
  };
  for (const field of ['authenticated', 'registryDigestObserved', 'publicationComplete']) {
    assert.equal(literal(field), false, 'producer outcome must not grant delivery authority');
  }
  assert.equal(literal('delivery'), 'Unknown');
  const declared = workflow(sources[workflowPath]);
  const publications = Object.entries(declared.jobs).flatMap(([job, definition]) => (definition.steps ?? []).flatMap(step => {
    const command = typeof step.run === 'string' ? step.run.trim().replace(/\s+/gu, ' ') : '';
    const arguments_ = command.split(' ');
    if (arguments_.length !== 4 || arguments_[0] !== 'node' || arguments_[1] !== '../../_protocol/' + helperPath || arguments_[2] !== 'publish') return [];
    assert.match(arguments_[3], /^[a-z][a-z0-9-]*$/u);
    return [{
      job,
      step,
      registry: arguments_[3]
    }];
  }));
  assert.equal(publications.length, registries.length, 'registry producer count drift');
  const tokenChoice = sourceBinding(producer, 'tokenName', 'main');
  const initialStatus = closedValue(sourceBinding(producer, 'status', 'main'));
  assert.equal(initialStatus, 'skipped-unconfigured');
  assert.ok(statuses.includes(initialStatus));
  const producerAst = parse(producer.toString('utf8'), {
    sourceType: 'module'
  });
  const main = producerAst.program.body.map(node => node.declaration ?? node).filter(node => node.type === 'FunctionDeclaration' && node.id.name === 'main');
  assert.equal(main.length, 1);
  const configuration = syntaxNodes(main[0].body).filter(node => node.type === 'IfStatement' &&
     node.test.type === 'MemberExpression' &&
     node.test.computed &&
     node.test.object.type === 'Identifier' &&
     node.test.object.name === 'environment' &&
     node.test.property.type === 'Identifier' &&
     node.test.property.name === 'tokenName');
  assert.equal(configuration.length, 1, 'one configured publication guard required');
  assert.equal(configuration[0].alternate, null, 'unconfigured publication path refused');
  const assignments = syntaxNodes(main[0].body).filter(node => node.type === 'AssignmentExpression' && node.left.type === 'Identifier' && node.left.name === 'status');
  assert.equal(assignments.length, 1, 'ambiguous producer outcome assignments');
  assert.ok(syntaxNodes(configuration[0].consequent).includes(assignments[0]));
  assert.equal(assignments[0].right.type, 'ConditionalExpression');
  for (const branch of [assignments[0].right.consequent, assignments[0].right.alternate]) {
    assert.ok(statuses.includes(closedValue(branch)), 'undeclared producer outcome');
  }
  const declaredRegistries = registries.map(name => {
    const selected = publications.filter(publication => publication.registry === name);
    assert.equal(selected.length, 1, 'missing or ambiguous registry producer');
    const {
      job,
      step
    } = selected[0];
    assert.equal(step.if.trim().replace(/\s+/gu, ' '), "needs.resolve.outputs.publish == 'true' && needs.publish.outputs.changed == 'true'", 'registry publication guard drift');
    const credential = closedValue(tokenChoice, {
      registry: name
    });
    assert.match(credential, /^[A-Z][A-Z0-9_]*$/u);
    assert.equal(step.env?.[credential], '$' + '{{ secrets.' + credential + ' }}');
    return {
      name,
      policy: 'required-if-configured',
      credential,
      workflow: workflowPath,
      job
    };
  });
  assert.equal(new Set(declaredRegistries.map(row => row.credential)).size, registries.length);
  const manifest = JSON.parse(sources['vscode/package.json']);
  assert.match(manifest.publisher, /^[a-z0-9][a-z0-9_.-]*$/u);
  assert.match(manifest.name, /^[a-z0-9][a-z0-9_.-]*$/u);
  const attachments = Object.values(desktop.jobs).flatMap(job => job.steps ?? []).map(step => step.with?.['subject-path']).filter(value => typeof value === 'string' && /^vscode\/[A-Za-z0-9_.-]+-\*\.vsix$/u.test(value));
  assert.equal(attachments.length, 1, 'one source-owned VSIX attestation subject required');
  const template = attachments[0].slice('vscode/'.length).replace('*', '$' + '{version}');
  assert.equal(template, manifest.name + '-$' + '{version}.vsix');
  return {
    publisher: manifest.publisher,
    package: manifest.name,
    template,
    releaseAttachment: 'required',
    registries: declaredRegistries,
    outcomes: {
      schema: literal('schema'),
      statuses,
      delivery: 'Unknown',
      publicationComplete: false
    }
  };
}
export function deriveDefinition(sources) {
  assert.ok(ADVERTISEMENT_ROOTS.every(path => Buffer.isBuffer(sources[path])), 'missing advertisement root');
  assert.ok(Object.keys(sources).length <= 128);
  const files = Object.keys(sources).sort().map(path => {
    const bytes = sources[path];
    assert.ok(Buffer.isBuffer(bytes) && bytes.length > 0 && bytes.length <= 1024 * 1024, 'missing bounded advertisement source');
    return {
      path
    };
  });
  for (const [name, bytes] of Object.entries(sources)) {
    if (!name.endsWith('.mjs') && !name.endsWith('.js')) continue;
    const ast = parse(bytes.toString('utf8'), {
      sourceType: 'module'
    });
    for (const node of ast.program.body) {
      if (!['ImportDeclaration', 'ExportNamedDeclaration', 'ExportAllDeclaration'].includes(node.type) || !node.source) continue;
      const dependency = node.source.value;
      assert.equal(dependency.startsWith('/'), false, 'absolute advertisement source import');
      if (!dependency.startsWith('.')) continue;
      const selected = posix.normalize(posix.join(posix.dirname(name), dependency));
      assert.match(selected, /^[A-Za-z0-9_.-]+(?:\/[A-Za-z0-9_.-]+)*$/u);
      assert.ok(!selected.split('/').some(part => ['.', '..', '.git'].includes(part)));
      assert.ok(Buffer.isBuffer(sources[selected]), 'missing declared advertisement source dependency: ' + selected);
    }
  }
  const desktop = workflow(sources[ADVERTISEMENT_ROOTS[0]]);
  const containers = workflow(sources[ADVERTISEMENT_ROOTS[1]]);
  const browser = workflow(sources[ADVERTISEMENT_ROOTS[2]]);
  const native = rows(desktop, 'native', ['target']);
  const registry = exportedLiteral(sources['scripts/native-release-artifact.mjs'], 'NATIVE_TARGETS');
  assert.deepEqual(native.map(row => row.target).sort(), Object.keys(registry).sort(), 'matrix and executable receipt registry disagree');
  const applications = rows(desktop, 'build', ['label', 'target', 'ebflag']);
  const signing = exportedLiteral(sources['scripts/native-release-evidence.mjs'], 'SIGNING_LABELS');
  assert.deepEqual(applications.filter(row => row.ebflag === '--mac').map(row => row.label).sort(), signing.sort(), 'signing registry differs from actual Mac producers');
  const cli = rows(desktop, 'cli', ['label', 'target', 'archive']);
  const images = rows(containers, 'publish-images', ['variant', 'arch', 'target']);
  assert.deepEqual(images, rows(containers, 'validate-images', ['variant', 'arch', 'target']), 'published platforms lack corresponding validation');
  for (const row of [...applications, ...cli, ...images]) assert.ok(native.some(n => n.target === row.target), 'advertised consumer lacks native producer');
  const packageManifest = JSON.parse(sources['packages/formal-ai-engine/package.json']);
  assert.equal(typeof packageManifest.name, 'string');
  assert.match(packageManifest.name, /^@[a-z0-9_.-]+\/[a-z0-9_.-]+$/u);
  const steps = Object.values(browser.jobs).flatMap(job => job.steps ?? []);
  const attachment = steps.filter(step => typeof step.run === 'string' && /gh release upload.*packages\/formal-ai-engine\/\*\.tgz/u.test(step.run));
  assert.equal(attachment.length, 1, 'browser package attachment source missing or ambiguous');
  const publication = steps.filter(step => step.env?.NODE_AUTH_TOKEN === '${{ secrets.NPM_TOKEN }}' && step.env?.NPM_TRUSTED_PUBLISHING === '${{ vars.FORMAL_AI_NPM_TRUSTED_PUBLISHING }}');
  assert.equal(publication.length, 1, 'npm configuration policy source missing or ambiguous');
  assert.ok(/\[ -z "\$NODE_AUTH_TOKEN" \] && \[ "\$NPM_TRUSTED_PUBLISHING" != "true" \]/u.test(publication[0].run));
  assert.ok(/status=skipped-unconfigured/u.test(publication[0].run), 'optional npm policy not explicit');
  const desktopPackage = JSON.parse(sources['desktop/package.json']);
  const platformByFlag = {
    '--linux': 'linux',
    '--mac': 'mac',
    '--win': 'win'
  };
  const extensionByBuilder = {
    AppImage: 'AppImage',
    deb: 'deb',
    'tar.gz': 'tar.gz',
    dmg: 'dmg',
    zip: 'zip',
    nsis: 'exe',
    portable: 'exe'
  };
  const desktopAssets = [];
  for (const row of applications) {
    const platform = platformByFlag[row.ebflag];
    assert.ok(platform, 'unsupported desktop producer platform');
    const specification = desktopPackage.build?.[platform];
    assert.ok(specification);
    assert.ok(Array.isArray(specification.target) && specification.target.length > 0);
    const arch = row.label.split('-').at(-1);
    assert.ok(['x64', 'arm64'].includes(arch));
    for (const builder of specification.target) {
      assert.equal(typeof builder, 'string');
      const ext = extensionByBuilder[builder];
      assert.ok(ext, 'unsupported packaging artifact target');
      const template = (desktopPackage.build[builder] ?? specification).artifactName;
      assert.equal(typeof template, 'string');
      assert.ok(template.includes('$' + '{arch}') && template.includes('$' + '{version}') && template.includes('$' + '{ext}'));
      const name = template.replaceAll('$' + '{arch}', arch).replaceAll('$' + '{ext}', ext);
      assert.match(name, /^[A-Za-z0-9_.-]+-\$\{version\}\.[A-Za-z0-9.]+$/u, 'unsupported artifact name expression');
      desktopAssets.push({
        label: row.label,
        builder,
        template: name
      });
    }
  }
  assert.equal(new Set(desktopAssets.map(asset => asset.template)).size, desktopAssets.length);
  const crateText = sources['rust/Cargo.toml'].toString('utf8');
  const packageBlock = /^\[package\]\n([\s\S]*?)(?=^\[|$)/mu.exec(crateText)?.[1];
  assert.ok(packageBlock);
  const crateName = /^name\s*=\s*"([a-z0-9_-]+)"\s*$/mu.exec(packageBlock)?.[1];
  assert.ok(crateName);
  const staged = workflow(sources['.github/workflows/release-staged.yml']);
  assert.equal(staged.env.GHCR_IMAGE, 'ghcr.io/' + '$' + '{{ github.repository }}');
  assert.equal(typeof staged.env.DOCKERHUB_IMAGE, 'string');
  const hubSteps = Object.values(staged.jobs).flatMap(job => job.steps ?? []).filter(step => typeof step.run === 'string' && step.run.includes('release-image-factory.mjs mirror'));
  assert.ok(hubSteps.length > 0);
  for (const step of hubSteps) assert.equal(step.if, "steps.dockerhub.outputs.enabled == 'true'");
  const distribution = {
    crate: {
      name: crateName,
      required: true
    },
    dockerHub: {
      repositoryExpression: staged.env.DOCKERHUB_IMAGE,
      policy: 'required-if-configured',
      policySource: 'scripts/configure-dockerhub-publishing.sh'
    },
    ghcr: {
      repositoryExpression: staged.env.GHCR_IMAGE,
      required: true
    }
  };
  return {
    schema: ADVERTISEMENT_SCHEMA,
    files,
    distribution,
    targets: {
      native,
      desktop: applications,
      desktopAssets,
      cli: cli.map(row => ({
        ...row,
        template: 'formal-ai-cli-' + row.target + '.' + row.archive
      })),
      containers: images,
      browser: {
        package: packageManifest.name,
        releaseAttachment: 'required',
        npm: 'required-if-configured'
      },
      extensions: extensionDefinition(sources, desktop)
    },
    publicationAuthority: false
  };
}
export function validateAdvertisement(record, sources, bindings) {
  assert.deepEqual(record, deriveAdvertisement(sources, bindings), 'advertisement differs from pinned source');
  return record;
}
export function deriveAdvertisement(sources, bindings) {
  assert.match(bindings.commit, /^[a-f0-9]{40}$/u);
  assert.match(bindings.tree, /^[a-f0-9]{40}$/u);
  assert.match(bindings.protocolSha256, /^[a-f0-9]{64}$/u);
  return {
    ...deriveDefinition(sources),
    ...bindings
  };
}
export function loadSources(root, overlay = null) {
  root = realpathSync(root);
  const sources = {},
    queue = [...ADVERTISEMENT_ROOTS];
  let total = 0;
  while (queue.length) {
    const path = queue.shift();
    if (Object.hasOwn(sources, path)) continue;
    assert.match(path, /^[A-Za-z0-9_.-]+(?:\/[A-Za-z0-9_.-]+)*$/u);
    assert.ok(!path.split('/').some(part => ['.', '..', '.git'].includes(part)));
    const physical = join(root, path);
    assert.ok(lstatSync(physical).isFile());
    assert.equal(relative(root, realpathSync(physical)), path, 'symlink advertisement input');
    const bytes = overlay && path === ADVERTISEMENT_ROOTS[0] ? readFileSync(overlay) : readFileSync(physical);
    assert.ok(bytes.length > 0 && bytes.length <= 1024 * 1024);
    total += bytes.length;
    assert.ok(total <= 16 * 1024 * 1024);
    sources[path] = bytes;
    assert.ok(Object.keys(sources).length <= 128);
    if (!path.endsWith('.mjs') && !path.endsWith('.js')) continue;
    const ast = parse(bytes.toString('utf8'), {
      sourceType: 'module'
    });
    for (const node of ast.program.body) {
      if (!['ImportDeclaration', 'ExportNamedDeclaration', 'ExportAllDeclaration'].includes(node.type) || !node.source) continue;
      const dependency = node.source.value;
      if (!dependency.startsWith('.')) continue;
      const selected = relative(root, resolve(dirname(physical), dependency));
      queue.push(selected);
    }
  }
  return sources;
}
export function loadDefinition(root, overlay = null) {
  const sources = loadSources(root, overlay);
  return deriveDefinition(sources);
}
if (process.argv[1] && existsSync(process.argv[1]) && realpathSync(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  assert.ok(args.length === 1 && args[0] === '--check' || args.length === 3 && args[0] === '--write' && args[1] === '--directory', 'exact check/write selector required');
  const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
  const destination = join(args[0] === '--check' ? root : resolve(args[2]), '.github/release-advertisement.json');
  const bytes = JSON.stringify(loadDefinition(root), null, 2) + '\n';
  if (args[0] === '--check') assert.equal(readFileSync(destination, 'utf8'), bytes, 'source advertisement drift');else {
    mkdirSync(dirname(destination), {
      recursive: true
    });
    writeFileSync(destination, bytes);
  }
  console.log('RELEASE_ADVERTISEMENT_GENERATOR_PASS');
}
