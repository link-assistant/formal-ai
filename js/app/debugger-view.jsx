import React, { useEffect, useState } from "react";
import { debugSessionCall, loadMermaid, pausedEventIndex, pollDebugger, renderMermaid, stagePanes } from "../debugger-client.js";
import { withAssetVersion } from "./application-constants.js";

const { createElement: h, Fragment } = React;

// The five-pane debugger (issue #667, R383; docs/vscode/debugger.md). Polling,
// the debug-session client, the event-to-pane projection and the Mermaid
// renderer live in js/debugger-client.js; selection is presentation only.
const SourcePane = ({ label, pane, testId }) => <section aria-label={label} data-testid={testId}><h2>{label}</h2>{pane ? <><p><code>{pane.location}</code> <strong>{pane.symbol}</strong></p><pre>{pane.excerpt}</pre></> : <pre>{"No source location recorded"}</pre>}</section>;

export function DebuggerView({ messages = [], apiBase = "", debugToken = "" }) {
  const [view, setView] = useState({ events: [], rawLinks: "", state: null, problem: "" });
  const [selection, setSelection] = useState(null);
  const [svg, setSvg] = useState(null);
  useEffect(() => pollDebugger({ apiBase, debugToken, onUpdate: setView }), [apiBase, debugToken]);
  const step = (action, extra) => debugSessionCall(apiBase, debugToken, action, extra)
    .then(state => setView(current => ({ ...current, state, problem: "" })), error => setView(current => ({ ...current, problem: String(error.message || error) })));
  const { events, state, problem } = view;
  const stepping = Boolean(apiBase && debugToken);
  const paused = state?.paused?.[0];
  const index = Math.min(selection ?? pausedEventIndex(events, state, events.length - 1), Math.max(0, events.length - 1));
  const selected = events[index] || {};
  const panes = stagePanes(selected);
  useEffect(() => {
    let live = true;
    setSvg(null);
    renderMermaid(panes.diagram, () => loadMermaid(withAssetVersion("mermaid.bundle.js"))).then(markup => { if (live) setSvg(markup); });
    return () => { live = false; };
  }, [panes.diagram]);
  const links = view.rawLinks || (window.FormalAiMemory ? window.FormalAiMemory.exportLinksNotation(events) : "");
  return <section className="debugger-view" data-testid="debugger-view" aria-label="Formal AI Debugger">
    <header><strong>Formal AI Debugger</strong><span role="status">{problem || `${events.length} recorded events`}</span>
      <button type="button" data-testid="debugger-pause" disabled={!stepping || Boolean(state?.stepping)} title={stepping ? "" : "Stepping is unavailable in this session"} onClick={() => step("pause")}>Pause</button>
      <button type="button" data-testid="debugger-next" disabled={!stepping || !paused} title={paused ? `${paused.turn} ${paused.stage + 1}/${paused.stages} ${paused.step}` : ""} onClick={() => { setSelection(null); step("advance", { turn: paused.turn, stage: paused.stage }); }}>Next stage</button>
      <button type="button" data-testid="debugger-continue" disabled={!stepping || !state?.stepping} onClick={() => step("release")}>Continue</button>
    </header>
    <div className="debugger-panes">
      <section aria-label="Conversation"><h2>Conversation</h2>{messages.map(message => <p key={message.id}><strong>{message.role}</strong> {message.content}</p>)}</section>
      <section aria-label="Event links"><h2>Event links</h2><label>Recorded event <select value={index} onChange={event => setSelection(Number(event.target.value))}>
        {events.map((event, at) => <option key={event.id || at} value={at}>{event.id || at}: {event.kind || event.role || "event"}{event.step ? ` ${event.step}` : ""}</option>)}
        </select></label><pre>{links || "No links recorded"}</pre><pre>{JSON.stringify(selected, null, 2)}</pre></section>
      <section aria-label="Recipe diagram" data-testid="debugger-diagram">{svg ? <><h2>Recipe diagram</h2><div className="debugger-graph" dangerouslySetInnerHTML={{ __html: svg }} /></> : <><h2>Recipe diagram</h2><pre data-format="mermaid">{panes.diagram || "No recipe diagram recorded"}</pre></>}</section>
      <SourcePane label="Rust source" pane={panes.rust} testId="debugger-rust-source" />
      <SourcePane label="JavaScript source" pane={panes.js} testId="debugger-js-source" />
    </div>
  </section>;
}
