// The loopback-only step-through debug session (issue #667, R383): `serve
// --debug-session` holds every solved turn before its derivation record is
// persisted and its answer returned, and hands its stages out one at a time,
// each `POST /v1/debug/advance` revealing the next. Every stage event carries
// the turn's recipe diagram and the routed method's Rust and JavaScript source
// locations (debug-stage.mjs). Mirrors rust/src/server/debug_session.rs.
//
// Protocol (docs/vscode/debugger.md):
//   - the session token comes from FORMAL_AI_DEBUG_SESSION_TOKEN, else it is
//     generated and printed once on stderr; every debug request carries it in
//     its JSON body (`token`), beside the server's ordinary bearer gate;
//   - stepping starts off: `pause` turns it on, so the next solved turn stops
//     at its first stage (`stage_paused`); `advance {turn, stage}` records
//     `stage_advanced` for exactly the paused stage and pauses at the next, a
//     stale or repeated advance answers 409 and records nothing; after the
//     last stage the turn is released (`turn_released`, reason `completed`);
//   - `release` (one turn, or every turn and stepping off) and a client that
//     disconnects (`disconnected`) release a paused turn, so no solve stays
//     paused once nobody can advance it;
//   - only solves inside an HTTP request scope are gated: background work
//     (dreaming, the CLI) never pauses.

import { AsyncLocalStorage } from 'node:async_hooks';
import { randomBytes, timingSafeEqual } from 'node:crypto';

import { describeTurn, locationFields, stageDiagram } from './debug-stage.mjs';
import { sortedKeys } from './json.mjs';
import { stableId } from './ids.mjs';
import { jsonResponse, messageError } from './response.mjs';

/** The environment variable a launcher hands the session token through. */
export const DEBUG_TOKEN_ENV = 'FORMAL_AI_DEBUG_SESSION_TOKEN';

const STAGE_PAUSED = 'stage_paused';
const STAGE_ADVANCED = 'stage_advanced';
const TURN_RELEASED = 'turn_released';
const LOOPBACK_NAME = 'localhost';
const IPV6_LOOPBACK = '::1';

/**
 * Mirrors `fn is_loopback_host`: `localhost`, `::1` or an address in
 * 127.0.0.0/8 — the only binds a debug session accepts.
 * @param {string} host
 */
export function isLoopbackHost(host) {
  const value = String(host ?? '').trim().toLowerCase().replace(/^\[(.*)\]$/, '$1');
  if (value === LOOPBACK_NAME || value === IPV6_LOOPBACK) return true;
  const parts = value.split('.');
  return parts.length === 4 && parts[0] === '127'
    && parts.every((part) => /^\d{1,3}$/.test(part) && Number(part) <= 255);
}

/** A fresh session token: 24 random bytes as hex. */
export function generateDebugToken() {
  return randomBytes(24).toString('hex');
}

/**
 * Mirrors `fn debug_token_from_env`: the launcher's token, else a generated
 * one (`generated` is then true and the banner prints it).
 * @returns {{token: string, generated: boolean}}
 */
export function debugTokenFrom(env = process.env) {
  const given = String(env?.[DEBUG_TOKEN_ENV] ?? '').trim();
  return given ? { token: given, generated: false } : { token: generateDebugToken(), generated: true };
}

/**
 * Mirrors `fn debug_session_banner`: the stderr line naming the session, with
 * the token only when the server generated it (a launcher already knows its own).
 * @param {DebugSession} session
 */
export function debugSessionBanner(session) {
  return session.generated ? `debug_session=${session.id};token=${session.token}` : `debug_session=${session.id}`;
}

/**
 * Mirrors `fn stage_fields`: what an event says about one stage — the step,
 * the turn's recipe diagram with this stage highlighted, and the source
 * locations of the method the turn's route resolves to (debug-stage.mjs).
 */
function stageFields(turn, index) {
  const { stages, view } = turn;
  const stage = stages[index] || {};
  return {
    stage: index,
    stages: stages.length,
    step: String(stage.step ?? ''),
    detail: String(stage.detail ?? ''),
    source: String(stage.source_event ?? ''),
    method: view.method,
    mermaid: stageDiagram(stages, index),
    ...locationFields('rust', view.rust),
    ...locationFields('js', view.js),
  };
}

/** Mirrors `struct DebugSession`. */
export class DebugSession {
  /**
   * @param {string | {token: string, generated?: boolean}} token
   * @param {{describe?: typeof describeTurn}} [options] how a turn's method
   *   and source locations are found (debug-stage.mjs `describeTurn`)
   */
  constructor(token, { describe = describeTurn } = {}) {
    this.describe = describe;
    const given = typeof token === 'object' && token !== null ? token : { token, generated: false };
    this.token = String(given.token);
    this.generated = Boolean(given.generated);
    this.id = stableId('debug_session', this.token);
    this.stepping = false;
    this.events = [];
    this.turns = new Map();
    this.turnCount = 0;
  }

  /** Constant-time token comparison. @param {unknown} token */
  authorizes(token) {
    if (typeof token !== 'string') return false;
    const expected = Buffer.from(this.token, 'utf8');
    const given = Buffer.from(token, 'utf8');
    return expected.length === given.length && timingSafeEqual(expected, given);
  }

  record(kind, turn, fields) {
    const event = sortedKeys({
      id: `debug_event_${this.events.length + 1}`,
      kind,
      session: this.id,
      turn: turn.id,
      ...fields,
    });
    this.events.push(event);
    return event;
  }

  /** Turn stepping on: the next solved turn pauses at its first stage. */
  pause() {
    this.stepping = true;
  }

  /**
   * Hold one solved turn until every stage is advanced or the turn is
   * released. Resolves at once when stepping is off or there is no stage.
   * @param {Array<object>} stages the turn's thinking steps
   * @param {AbortSignal} [signal] aborts when the requesting client disconnects
   */
  async gate(stages, signal) {
    if (!this.stepping || !Array.isArray(stages) || stages.length === 0) return;
    this.turnCount += 1;
    const turn = {
      id: `turn_${this.turnCount}`, stages, view: this.describe(stages), current: 0, released: false, resolve: null,
    };
    const done = new Promise((resolve) => {
      turn.resolve = resolve;
    });
    this.turns.set(turn.id, turn);
    this.record(STAGE_PAUSED, turn, stageFields(turn, 0));
    const onAbort = () => this.finish(turn, 'disconnected');
    signal?.addEventListener('abort', onAbort, { once: true });
    if (signal?.aborted) onAbort();
    await done;
    signal?.removeEventListener('abort', onAbort);
  }

  finish(turn, reason) {
    if (turn.released) return;
    turn.released = true;
    this.turns.delete(turn.id);
    this.record(TURN_RELEASED, turn, { reason, stage: turn.current, stages: turn.stages.length });
    turn.resolve();
  }

  /**
   * Advance the paused stage `stage` of `turnId`. False — nothing recorded —
   * when that stage is not the one paused (already advanced, unknown turn).
   */
  advance(turnId, stage) {
    const turn = this.turns.get(String(turnId ?? ''));
    if (!turn || turn.released || stage !== turn.current) return false;
    this.record(STAGE_ADVANCED, turn, stageFields(turn, turn.current));
    if (turn.current + 1 < turn.stages.length) {
      turn.current += 1;
      this.record(STAGE_PAUSED, turn, stageFields(turn, turn.current));
    } else {
      this.finish(turn, 'completed');
    }
    return true;
  }

  /** Release one turn, or — with no turn — every turn, and stop stepping. */
  release(turnId) {
    if (turnId !== undefined && turnId !== null) {
      const turn = this.turns.get(String(turnId));
      if (turn) this.finish(turn, 'released');
      return;
    }
    this.stepping = false;
    for (const turn of [...this.turns.values()]) this.finish(turn, 'released');
  }

  /** The session state and the events after the first `since`. */
  snapshot(since = 0) {
    const from = Number.isInteger(since) && since > 0 ? since : 0;
    return sortedKeys({
      object: 'debug.session',
      session: this.id,
      stepping: this.stepping,
      paused: [...this.turns.values()].map((turn) => ({ turn: turn.id, ...stageFields(turn, turn.current) })),
      events: this.events.slice(from),
      next: this.events.length,
    });
  }
}

const requestScope = new AsyncLocalStorage();

/** Run `fn` inside one HTTP request's scope; `signal` aborts on disconnect. */
export function runInRequestScope(signal, fn) {
  return requestScope.run({ signal }, fn);
}

/**
 * Mirrors `fn gate_turn`: hold a solved turn when the server runs a debug
 * session and the solve belongs to an HTTP request.
 * @param {{debugSession?: DebugSession}} ctx
 * @param {Array<object>} stages
 */
export async function gateTurn(ctx, stages) {
  const scope = requestScope.getStore();
  if (!ctx?.debugSession || !scope) return;
  await ctx.debugSession.gate(stages, scope.signal);
}

/** `POST /v1/debug/{session|pause|advance|release}`. */
export function handleDebug(ctx, request) {
  const session = ctx?.debugSession;
  if (!session) return messageError(404, 'debug_session_disabled');
  let body;
  try {
    body = JSON.parse(request.body || '');
  } catch {
    return messageError(400, 'debug_request_invalid');
  }
  if (!body || typeof body !== 'object' || Array.isArray(body)) return messageError(400, 'debug_request_invalid');
  if (!session.authorizes(body.token)) return messageError(401, 'debug_session_token_invalid');
  const action = request.params?.action;
  if (action === 'pause') session.pause();
  else if (action === 'release') session.release(body.turn);
  else if (action === 'advance' && !session.advance(body.turn, body.stage)) {
    return messageError(409, 'debug_stage_not_paused');
  }
  return jsonResponse(200, session.snapshot(body.since));
}
