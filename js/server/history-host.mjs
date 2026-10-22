// Node-owned history adapter; injected callbacks remain caller-trusted capabilities.
import fs from 'node:fs';
import path from 'node:path';
import {boundedCommandSync} from './bounded-command.mjs';
import {nodeHistoryIo as historyIo} from '../agentic/crate/history_store.mjs';
export function nodeHistoryIo(options = {}) {
  return historyIo({fs, path, ...options, runCommand: options.runCommand ?? boundedCommandSync});
}
