import {execFileSync, spawnSync} from 'node:child_process';
import {chmodSync, copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import path from 'node:path';
import YAML from 'yaml';

export const maintenanceWorkflow = readFileSync(new URL('../../.github/workflows/dependencies-latest.yml', import.meta.url), 'utf8');

export function resolverStep(source, kind) {
  const name = 'Re-resolve the ' + (kind === 'cargo' ? 'Rust' : 'JavaScript') + ' lockfiles from the rewritten manifests';
  const steps = Object.values(YAML.parse(source).jobs).flatMap(job => job.steps ?? []);
  const step = steps.find(candidate => candidate.name === name);
  if (typeof step?.run !== 'string') throw new Error('Complete maintained resolver step missing: ' + name);
  return step.run;
}

export function lockFixture(owners = ['', 'nested'], cargoOwners = ['rust', 'tools/registry']) {
  const directory = realpathSync(mkdtempSync(path.join(tmpdir(), 'formal-ai-literal-locks-')));
  const write = (name, text) => {
    mkdirSync(path.dirname(path.join(directory, name)), {recursive: true});
    writeFileSync(path.join(directory, name), text);
  };
  for (const owner of owners) {
    for (const name of ['package.json', 'package-lock.json', 'bun.lock']) write(path.posix.join(owner, name), '{}\n');
  }
  const activeCargo = cargoOwners.map(owner => path.posix.join(owner, 'Cargo.toml'));
  for (const name of activeCargo) write(name, '[dependencies]\n');
  write('scripts/dependency-manifests.json', JSON.stringify({schemaVersion: 1, activeCargo, preservedCargo: {}}));
  for (const name of ['dependency-manifest-inventory.mjs', 'resolve-dependency-lockfiles.mjs']) {
    const source = new URL('../../scripts/' + name, import.meta.url);
    if (existsSync(source)) copyFileSync(source, path.join(directory, 'scripts', name));
  }
  execFileSync('git', ['init', '--quiet'], {cwd: directory, timeout: 3000});
  execFileSync('git', ['add', '.'], {cwd: directory, timeout: 3000});
  const record = path.join(directory, 'calls.jsonl');
  const bin = path.join(directory, 'fixture-bin');
  mkdirSync(bin);
  const header = '#!' + process.execPath + '\n';
  for (const tool of ['npm', 'bun', 'cargo']) {
    write('fixture-bin/' + tool, header +
      "import {appendFileSync,existsSync,readFileSync} from 'node:fs';\n" +
      'const record=process.env.FORMAL_AI_LOCK_CALLS;\n' +
      'const previous=existsSync(record)?readFileSync(record,"utf8").trim().split("\\n").filter(Boolean).length:0;\n' +
      'appendFileSync(record,JSON.stringify({tool:' + JSON.stringify(tool) + ',cwd:process.cwd(),args:process.argv.slice(2)})+"\\n");\n' +
      'if(Number(process.env.FORMAL_AI_LOCK_FAIL_CALL)===previous+1)process.exit(17);\n');
    chmodSync(path.join(bin, tool), 0o755);
  }
  return {
    directory,
    activeCargo,
    run(kind, {source = maintenanceWorkflow, gitMode, failCall} = {}) {
      if (existsSync(record)) rmSync(record);
      const git = path.join(bin, 'git');
      if (existsSync(git)) rmSync(git);
      if (gitMode) {
        const observations = {
          failed: {stdout: 'package-lock.json\0', status: 23},
          truncated: {stdout: 'package-lock.json', status: 0},
          duplicate: {stdout: 'package-lock.json\0package-lock.json\0', status: 0},
          unsafe: {stdout: '../package-lock.json\0', status: 0},
          empty: {stdout: '', status: 0},
        };
        const observation = observations[gitMode];
        if (!observation) throw new Error('Unknown fixture observation');
        write('fixture-bin/git', header + 'process.stdout.write(' + JSON.stringify(observation.stdout) +
          ');process.exit(' + observation.status + ');\n');
        chmodSync(git, 0o755);
      }
      const result = spawnSync('bash', ['--noprofile', '--norc', '-c', resolverStep(source, kind)], {
        cwd: directory, encoding: 'utf8', timeout: 10000, maxBuffer: 128 * 1024,
        env: {...process.env, PATH: bin + path.delimiter + process.env.PATH,
          BASH_ENV: '/dev/null', FORMAL_AI_LOCK_CALLS: record, FORMAL_AI_LOCK_FAIL_CALL: String(failCall ?? 0)},
      });
      const calls = existsSync(record) ? readFileSync(record, 'utf8').trim().split('\n').filter(Boolean).map(JSON.parse) : [];
      return {...result, calls};
    },
    close: () => rmSync(directory, {recursive: true, force: true}),
  };
}
