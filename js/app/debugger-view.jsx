import React, { useEffect, useRef, useState } from "react";
const { createElement: h } = React;

// One `POST /v1/debug/<action>` of a `serve --debug-session` server (issue
// #667, R383; protocol in docs/vscode/debugger.md). The session token rides in
// the JSON body, so the request needs no header beyond the CORS-allowed ones.
export async function debugSessionCall(apiBase, debugToken, action, extra = {}) {
  const response = await fetch(`${apiBase.replace(/\/$/, "")}/v1/debug/${action}`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ token: debugToken, ...extra }),
  });
  const body = await response.json();
  if (!response.ok) throw new Error(body?.error?.message || `HTTP ${response.status}`);
  return body;
}

// All content comes from public memory records and, in a debug session, the
// session's stage events; selection/poll state is presentation only.
export function DebuggerView({ messages = [], apiBase = "", debugToken = "" }) {
  const [events, setEvents] = useState([]);
  const [selection, setSelection] = useState(0);
  const [problem, setProblem] = useState("");
  const [rawLinks, setRawLinks] = useState("");
  const [debugState, setDebugState] = useState(null);
  const debugCursor = useRef(0);
  const stepping = Boolean(apiBase && debugToken);
  useEffect(() => {
    let active = true;
    let timer;
    let cursor = "";
    const records = new Map();
    debugCursor.current = 0;
    const abort = new AbortController();
    const memory = window.FormalAiMemory;
    async function poll() {
      try {
        const local = memory ? await memory.listEvents() : [];
        let remote = [];
        if (apiBase) {
          const url = new URL(`${apiBase.replace(/\/$/, "")}/v1/memory/since`);
          if (cursor) url.searchParams.set("event", cursor);
          const response = await fetch(url, { signal: abort.signal });
          if (!response.ok) throw new Error(`Memory stream HTTP ${response.status}`);
          const text = await response.text();
          if (active) setRawLinks(text);
          remote = memory ? memory.parseLinksNotation(text) : [];
          if (remote.length && remote[remote.length - 1].id) cursor = remote[remote.length - 1].id;
        }
        let staged = [];
        if (apiBase && debugToken) {
          const state = await debugSessionCall(apiBase, debugToken, "session", { since: debugCursor.current });
          staged = state.events;
          debugCursor.current = state.next;
          if (active) setDebugState(state);
        }
        for (const event of [...local, ...remote, ...staged]) records.set(event.id || JSON.stringify(event), event);
        if (active) { setEvents([...records.values()]); setProblem(""); }
      } catch (error) { if (active) setProblem(String(error.message || error)); }
      if (active) timer = setTimeout(poll, 1500);
    }
    poll();
    return () => { active = false; clearTimeout(timer); abort.abort(); };
  }, [apiBase, debugToken]);
  async function step(action, extra) {
    try { setDebugState(await debugSessionCall(apiBase, debugToken, action, extra)); setProblem(""); }
    catch (error) { setProblem(String(error.message || error)); }
  }
  const paused = debugState?.paused?.[0];
  const selected = events[Math.min(selection, Math.max(0, events.length - 1))] || {};
  const diagram = selected.mermaid || selected.diagram ||
    (selected.kind === "recipe_diagram" || selected.kind === "mermaid" ? selected.content : "");
  const source = selected.sourceLocation || selected.source_location ||
    (selected.kind === "source_location" ? selected.content : "");
  const links = rawLinks || (window.FormalAiMemory ? window.FormalAiMemory.exportLinksNotation(events) : "");
  return <section className="debugger-view" data-testid="debugger-view" aria-label="Formal AI Debugger">
    <header><strong>Formal AI Debugger</strong><span role="status">{problem || `${events.length} recorded events`}</span>
      <button type="button" data-testid="debugger-pause" disabled={!stepping || Boolean(debugState?.stepping)} title={stepping ? "" : "Stepping is unavailable in this session"} onClick={() => step("pause")}>Pause</button>
      <button type="button" data-testid="debugger-next" disabled={!stepping || !paused} title={paused ? `${paused.turn} ${paused.stage + 1}/${paused.stages} ${paused.step}` : ""} onClick={() => step("advance", { turn: paused.turn, stage: paused.stage })}>Next stage</button>
      <button type="button" data-testid="debugger-continue" disabled={!stepping || !debugState?.stepping} onClick={() => step("release")}>Continue</button>
    </header>
    <div className="debugger-panes">
      <section aria-label="Conversation"><h2>Conversation</h2>{messages.map(message => <p key={message.id}><strong>{message.role}</strong> {message.content}</p>)}</section>
      <section aria-label="Event links"><h2>Event links</h2><label>Recorded event <select value={Math.min(selection, Math.max(0, events.length - 1))} onChange={event => setSelection(Number(event.target.value))}>{events.map((event, index) => <option key={event.id || index} value={index}>{event.id || index}: {event.kind || event.role || "event"}</option>)}</select></label><pre>{links || "No links recorded"}</pre></section>
      <section aria-label="Recipe diagram"><h2>Recipe diagram</h2><pre data-format="mermaid">{diagram || "No recipe diagram recorded"}</pre></section>
      <section aria-label="Executing source"><h2>Executing source</h2><pre>{source || "No source location recorded"}</pre><pre>{JSON.stringify(selected, null, 2)}</pre></section>
    </div>
  </section>;
}
