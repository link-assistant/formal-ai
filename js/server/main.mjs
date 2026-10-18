#!/usr/bin/env node
// The JavaScript formal-ai HTTP server: `node js/server/main.mjs --port N`.
//
// Takes the flags and environment of `formal-ai serve` (rust/src/main.rs,
// rust/src/cli_local_transport.rs `ServeArgs`): `--host` / FORMAL_AI_HOST
// (127.0.0.1), `--port` / FORMAL_AI_PORT (8080), `--agent-mode` (or
// FORMAL_AI_AGENT_MODE), and the bearer token from FORMAL_AI_API_BEARER_TOKEN,
// FORMAL_AI_HTTP_BEARER_TOKEN or FORMAL_AI_API_TOKEN. The `--ws` / `--webrtc`
// transports are native-only and refused. `explain <answer-id>
// [--format text|links]` is the `formal-ai explain` request: it prints the
// derivation record a served answer persisted (js/server/derivation-store.mjs).
// `memory upgrade-status [--path P] [--format json]` and `memory migrate
// [--path P] [--backup B] [--receipt R] [--format json]` are the explicit
// persisted-memory upgrade (js/server/memory-upgrade.mjs). `--debug-session`
// (loopback binds only) holds each solved turn for step-through debugging
// (js/server/debug-session.mjs, docs/vscode/debugger.md).

import { pathToFileURL } from 'node:url';
import { createServeLogger } from './structured-log.mjs';
import { createGeneratedDebugHandoff } from './debug-session-handoff.mjs';

import { runExplain } from './derivation-store.mjs';
import { DebugSession, debugSessionBanner, debugTokenFrom, isLoopbackHost } from './debug-session.mjs';
import { startCoreDreaming } from './dreaming-runtime.mjs';
import { createServer } from './http.mjs';
import { createMemory, memoryPath } from './memory.mjs';
import { runMemoryCli } from './memory-upgrade.mjs';
import { serverMessage } from './messages.mjs';
import { setLearnedImporter } from './meta-learned.mjs';
import { createProverHost } from './prover-host.mjs';
import { WorkerHost } from './worker-host.mjs';
import {configurePrerequisiteGrant, resolvePrerequisiteGrantText} from './prerequisite-grant.mjs';

const TOKEN_ENV = ['FORMAL_AI_API_BEARER_TOKEN', 'FORMAL_AI_HTTP_BEARER_TOKEN', 'FORMAL_AI_API_TOKEN'];

/** `first_non_empty_env`. */
export function bearerTokenFromEnv(env = process.env) {
  for (const name of TOKEN_ENV) {
    const value = (env[name] || '').trim();
    if (value) return value;
  }
  return null;
}

function flagEnabled(value) {
  return ['1', 'true', 'yes', 'on'].includes(String(value || '').trim().toLowerCase());
}

/** Parse `serve` flags. @param {Array<string>} argv */
export function parseArgs(argv, env = process.env) {
  const options = {
    host: env.FORMAL_AI_HOST || '127.0.0.1',
    port: Number.parseInt(env.FORMAL_AI_PORT || '8080', 10),
    agentMode: flagEnabled(env.FORMAL_AI_AGENT_MODE),
    debugSession: false,
    logLevel: env.FORMAL_AI_LOG_LEVEL || 'debug',
    silent: flagEnabled(env.FORMAL_AI_SILENT),
    logDirectory: env.FORMAL_AI_LOG_DIR || null,
    trace: false,
    prerequisiteGrantJson: env.FORMAL_AI_PREREQUISITE_GRANT_JSON ?? null,
    unsupported: null,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const [name, inline] = arg.includes('=') ? arg.split(/=(.*)/s, 2) : [arg, null];
    const value = () => (inline !== null ? inline : argv[++index]);
    if (name === '--host') options.host = value();
    else if (name === '--port') options.port = Number.parseInt(value(), 10);
    else if (name === '--agent-mode') options.agentMode = true;
    else if (name === '--prerequisite-grant') options.prerequisiteGrantJson = value();
    else if (name === '--debug-session') options.debugSession = true;
    else if (name === '--silent') options.silent = true;
    else if (name === '--trace') options.trace = true;
    else if (name === '--debug' || name === '--verbose') options.logLevel = 'debug';
    else if (name === '--log-level') { options.logLevel = value(); if (!options.logLevel) options.unsupported = arg; }
    else if (name === '--log-dir') { options.logDirectory = value(); if (!options.logDirectory) options.unsupported = arg; }
    else if (name === 'serve' && index === 0) continue;
    else options.unsupported = arg;
  }
  options.prerequisiteGrantJson = resolvePrerequisiteGrantText(options.prerequisiteGrantJson, env.FORMAL_AI_INSTALL_GRANT);
  return options;
}

/**
 * Start a server; resolves once it listens.
 * @returns {Promise<{server: import('node:http').Server, ctx: object, url: string}>}
 */
export async function startServer({ host = '127.0.0.1', port = 0, agentMode = false, debugSession = false, env = process.env, worker, logger, prerequisiteGrantJson = env.FORMAL_AI_PREREQUISITE_GRANT_JSON ?? null } = {}) {
  if (debugSession && !isLoopbackHost(host)) throw new Error(`debug_session_requires_loopback:${host}`);
  const diagnostics = logger ?? createServeLogger({env});
  diagnostics.log('debug', 'server.start', {component:'server',host,port,agent_mode:agentMode,debug_session:debugSession});
  try {
    const session=debugSession ? new DebugSession(debugTokenFrom(env)) : null;
    if(session){
      diagnostics.addSecret?.(session.token);
      env={...env,FORMAL_AI_DEBUG_SESSION_TOKEN:session.token};
    }
    const prerequisiteGrant = configurePrerequisiteGrant(resolvePrerequisiteGrantText(prerequisiteGrantJson, env.FORMAL_AI_INSTALL_GRANT));
    if (prerequisiteGrant !== null && (prerequisiteGrant.schema !== "formal-ai-workspace-prerequisites-v1" || prerequisiteGrant.scope !== "workspace" || !Array.isArray(prerequisiteGrant.programs))) throw new Error("invalid_prerequisite_grant");
    const ctx = {
      prerequisiteGrant,
      worker: worker || new WorkerHost({ formalAiProverHost: createProverHost({ path: env.PATH }) }),
      memory: createMemory(env), agentMode, bearerToken: bearerTokenFromEnv(env),
      debugSession: session,
      env, logger:diagnostics, requestSequence:0,
    };
    await ctx.worker.boot();
    setLearnedImporter((statements) => ctx.worker.run('metaImportLearned(__learned)', { __learned: statements }));
    const server = createServer(ctx);
    server.once('close', () => {
      const cleanup=ctx.debugCredentialHandoff?.cleanup();
      if(cleanup && !cleanup.cleaned)diagnostics.log('warn','server.debug_credential_cleanup',{component:'server',reason:cleanup.reason,credential_file:ctx.debugCredentialHandoff.file});
      diagnostics.log('info','server.terminal',{component:'server',reason:ctx.shutdownReason??'server_closed',signal:ctx.shutdownSignal??null});
      diagnostics.close();
    });
    await new Promise((resolve, reject) => { server.once('error', reject); server.listen(port, host, resolve); });
    const address = server.address();
    return { server, ctx, url: 'http://'+host+':'+address.port };
  } catch(error) {
    diagnostics.log('error','server.terminal',{component:'server',reason:'startup_failed',error});
    diagnostics.close();
    throw error;
  }
}

async function main(argv) {
  // `formal-ai explain <answer-id> [--format text|links]` (rust/src/cli_explain.rs).
  if (argv[0] === 'logs') {
    const {runLogsCli}=await import('./logs-cli.mjs');
    const result=runLogsCli(argv.slice(1));
    process.stdout.write(result.stdout);process.stderr.write(result.stderr);return result.code;
  }
  if (argv[0] === 'explain') {
    const { code, stdout, stderr } = runExplain(argv.slice(1));
    process.stdout.write(stdout);
    process.stderr.write(stderr);
    return code;
  }
  // `formal-ai memory upgrade-status|migrate` (rust/src/cli_memory.rs).
  if (argv[0] === 'memory') {
    const { code, stdout, stderr } = await runMemoryCli(argv.slice(1));
    process.stdout.write(stdout);
    process.stderr.write(stderr);
    return code;
  }
  const options = parseArgs(argv);
  let logger;
  try { logger=createServeLogger({...options,env:process.env}); }
  catch {
    logger=createServeLogger({logLevel:'error',env:{...process.env,FORMAL_AI_LOG_LEVEL:'error'}});
    logger.log('error','server.terminal',{component:'server',reason:'invalid_log_configuration'});
    logger.close(); return 2;
  }
  if(options.unsupported || (options.debugSession && !isLoopbackHost(options.host))){
    logger.log('error','server.terminal',{component:'server',reason:options.unsupported?'unsupported_argument':'debug_session_requires_loopback',argument:options.unsupported});
    logger.close();return 2;
  }
  logger.log('debug','server.memory',{component:'memory',path:memoryPath(process.env)});
  startCoreDreaming({ env: process.env, memoryPath: memoryPath(process.env) });
  let running;
  options.logger=logger;
  try { running=await startServer(options); }catch{return 1;}
  const {url,ctx,server}=running;
  if(ctx.debugSession){
    if(ctx.debugSession.generated){
      try{ctx.debugCredentialHandoff=createGeneratedDebugHandoff(ctx.debugSession,{env:ctx.env,url});}
      catch{
        ctx.shutdownReason='debug_credential_handoff_failed';
        logger.log('error','server.debug_session_handoff_failed',{component:'server',reason:'credential_file_failed'});
        await new Promise(resolve=>server.close(resolve));
        return 1;
      }
      logger.log('info','server.debug_session',{component:'server',enabled:true,session_id:ctx.debugSession.id,generated:true,credential_source:'private_file',credential_file:ctx.debugCredentialHandoff.file});
    }else logger.log('info','server.debug_session',{component:'server',enabled:true,session_id:ctx.debugSession.id,generated:false,credential_source:'environment'});
  }
  logger.log('info','server.listening',{component:'server',url});
  const shutdown=signal=>{ctx.shutdownReason='signal';ctx.shutdownSignal=signal;server.close();};
  process.once('SIGTERM',()=>shutdown('SIGTERM'));process.once('SIGINT',()=>shutdown('SIGINT'));
  return null;
}

if (import.meta.url === pathToFileURL(process.argv[1] || '').href) {
  main(process.argv.slice(2)).then((code) => {
    if (code !== null) process.exitCode = code;
  });
}
