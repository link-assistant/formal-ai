#!/usr/bin/env node

import fs from 'node:fs/promises';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const [hiveMindRoot, workspace, taskPath, agentPath = 'agent'] = process.argv.slice(2);
if (!hiveMindRoot || !workspace || !taskPath) {
  console.error(
    'usage: run-hive-executor.mjs HIVE_MIND_ROOT WORKSPACE TASK_PATH [AGENT_PATH]',
  );
  process.exit(2);
}

const agentModuleUrl = pathToFileURL(path.join(hiveMindRoot, 'src/agent.lib.mjs'));
const { executeAgentCommand } = await import(agentModuleUrl.href);
const { $ } = await globalThis.use('command-stream');
const prompt = await fs.readFile(taskPath, 'utf8');
const log = async message => process.stdout.write(`${String(message)}\n`);
const formatAligned = (...parts) => parts.filter(Boolean).join(' ');
const getResourceSnapshot = async () => ({
  memory: 'MemTotal: 0 kB\nMemFree: 0 kB',
  load: '0.00 0.00 0.00',
});

const result = await executeAgentCommand({
  tempDir: workspace,
  branchName: 'issue-921-fixture',
  prompt,
  systemPrompt: '',
  argv: {
    acceptIncommingCommentsAsInput: false,
    dryRun: false,
    fork: false,
    model: 'formal-ai',
    onlyPrepareCommand: false,
    playwrightMcp: true,
    resume: null,
    verbose: true,
  },
  log,
  formatAligned,
  getResourceSnapshot,
  forkedRepo: null,
  feedbackLines: [],
  owner: 'link-assistant',
  repo: 'formal-ai',
  prNumber: null,
  issueNumber: 921,
  agentPath,
  $,
  calculatePricing: async modelName => ({
    modelName,
    totalCostUSD: null,
    breakdown: null,
  }),
  waitForRetryDelay: async () => {},
});

process.stdout.write(`hive-executor-result=${JSON.stringify(result)}\n`);
process.exitCode = result.success ? 0 : (result.errorInfo?.exitCode ?? 1);
