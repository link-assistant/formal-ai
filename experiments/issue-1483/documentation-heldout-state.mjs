// Drives the held-out renamed-paths documentation case (ribbon-core) and
// prints the requirements, checks and answer.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { drive } from '../js_dogfood/drive.mjs';
import { planChatStep } from '../../js/agentic/planner.mjs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { workItemRecordOf } from '../../js/agentic/work_item_report.mjs';
await installNodeHost(new WorkerHost({}));
const files = [
  { path: 'native/Project.toml', source: '[dependencies]\nribbon-core = {version="2.7",default-features=false,features=["parse"]}\n' },
  { path: 'web/client.mjs', source: 'import {decode as readPacket} from "ribbon-core";\nexport function load(value){return readPacket(value,{mode:"strict"});}\n' },
  { path: 'docs/runtime.md', source: '# Runtime\n\nPreserve these local notes.\n' }];
const prompt = 'Document the ribbon-core dependency in docs/runtime.md based on native/Project.toml and web/client.mjs. Preserve the existing guide and include observed declarations and imported calls.';
const root = fs.mkdtempSync(path.join(os.tmpdir(), 'docs-heldout-'));
for (const file of files) { fs.mkdirSync(path.dirname(path.join(root, file.path)), { recursive: true }); fs.writeFileSync(path.join(root, file.path), file.source); }
const result = await drive(planChatStep, root, prompt, { steps: 24, tools: ['read', 'edit'], allowedCommands: [] });
const record = result.record ?? workItemRecordOf(result);
console.log(JSON.stringify({ reason: record.reason, verified: record.verified, checks: record.checks, requirements: record.requirements }, null, 1));
console.log(result.answer);
fs.rmSync(root, { recursive: true, force: true });
