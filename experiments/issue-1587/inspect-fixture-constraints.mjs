// Diagnose output-constraint classification on recorded agent request prefixes.
import fs from 'node:fs';
import { parseChatRequest } from '../../js/server/chat-request.mjs';
import { formalizeRequest } from '../../js/server/request-context.mjs';

const file = process.argv[2] ?? 'data/benchmarks/fixtures/issue-1558/agent-cli-request.json';
const request = parseChatRequest(fs.readFileSync(file, 'utf8'));
console.log(JSON.stringify(formalizeRequest(request.messages).constraints, null, 2));
