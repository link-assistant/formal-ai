#!/usr/bin/env node
// Fake gh for the offline Hello World e2e: state lives in $FAKE_GH_STATE.
const fs = require('node:fs');
const file = process.env.FAKE_GH_STATE;
const state = JSON.parse(fs.readFileSync(file, 'utf8'));
const args = process.argv.slice(2);
state.calls.push(args);
const flag = name => { const i = args.indexOf(name); return i < 0 ? null : args[i + 1]; };
const save = () => fs.writeFileSync(file, JSON.stringify(state, null, 2));
const [noun, verb] = args;
let out = '';
if (noun === 'issue' && verb === 'view') {
  const field = flag('--json');
  out = field === 'title' ? state.issue.title : field === 'body' ? state.issue.body : JSON.stringify(state.issue);
} else if (noun === 'pr' && verb === 'view') {
  const field = flag('--json');
  if (field === 'title') out = state.pr.title;
  else if (field === 'body') out = state.pr.body;
  else if (field === 'isDraft') out = String(state.pr.isDraft);
  else if (field === 'comments,reviews') {
    // Apply the caller's own jq filter to the recorded feedback, as gh does.
    const { execFileSync } = require('node:child_process');
    out = execFileSync('jq', ['-c', flag('--jq')], { input: JSON.stringify({ comments: state.pr.comments ?? [], reviews: state.pr.reviews ?? [] }), encoding: 'utf8' }).trim();
  } else if (field === 'headRefOid,headRefName,isDraft,state,url') {
    const { execFileSync } = require('node:child_process');
    const head = execFileSync('git', ['ls-remote', 'origin', 'refs/heads/' + state.branch], { encoding: 'utf8' }).split(/\s/u)[0];
    out = JSON.stringify({ headRefOid: head, headRefName: state.branch, isDraft: state.pr.isDraft, state: 'OPEN', url: state.pr.url });
  } else out = '';
} else if (noun === 'pr' && verb === 'checks') {
  // Simulated CI: one passing check per workflow in the pushed head, named as
  // the workflow names itself. Not a real CI run.
  if (flag('--json') !== null) {
    const { execFileSync } = require('node:child_process');
    execFileSync('git', ['fetch', '-q', 'origin', state.branch]);
    const files = execFileSync('git', ['ls-tree', '-r', '--name-only', 'FETCH_HEAD'], { encoding: 'utf8', env: { ...process.env } }).split('\n');
    out = JSON.stringify(files.filter(f => /^(\.github\/workflows|ci)\/[^/]+\.ya?ml$/u.test(f)).map(f => {
      const text = execFileSync('git', ['show', 'FETCH_HEAD:' + f], { encoding: 'utf8' });
      const name = /^name: (.+)$/mu.exec(text)?.[1] ?? f;
      return { bucket: 'pass', name: 'run', workflow: name };
    }));
  }
  state.checks = (state.checks ?? 0) + 1;
} else if (noun === 'pr' && verb === 'edit') {
  if (flag('--title') !== null) state.pr.title = flag('--title');
  if (flag('--body') !== null) state.pr.body = flag('--body');
} else if (noun === 'pr' && verb === 'ready') {
  state.pr.isDraft = false;
} else {
  save();
  process.stderr.write('fake gh: unsupported ' + args.join(' ') + '\n');
  process.exit(2);
}
save();
if (out) process.stdout.write(out + '\n');
