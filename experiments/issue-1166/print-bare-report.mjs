// Prints the acceptance report of a finished recipe for the #1450 body whose
// program has no comments (the example in the PR description).
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { acceptanceReport } from '../../js/agentic/task_acceptance.mjs';

await installNodeHost(new WorkerHost());
const fixtures = new URL('../../rust/tests/fixtures/issue-1166/', import.meta.url);
const body = readFileSync(new URL('issue-1450-body.md', fixtures), 'utf8');
const fixture = JSON.parse(readFileSync(new URL('acceptance.json', fixtures), 'utf8'));
const workflow = fixture.candidateCases[0].candidate.files[1];
const recipe = { language: 'kotlin', path: 'Main.kt', source: 'fun main() {\n    println("Hello, World!")\n}\n', supporting_files: [workflow], commands: ['kotlinc Main.kt'] };
console.log(acceptanceReport(body, recipe, ['Hello, World!\n']));
