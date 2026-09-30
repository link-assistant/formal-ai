import React, { useEffect, useState } from "react";
const { createElement: h } = React;

// All content comes from public memory records; selection/poll state is presentation only.
export function DebuggerView({ messages = [], apiBase = "" }) {
  const [events, setEvents] = useState([]);
  const [selection, setSelection] = useState(0);
  const [problem, setProblem] = useState("");
  const [rawLinks, setRawLinks] = useState("");
  useEffect(() => {
    let active = true;
    let timer;
    let cursor = "";
    const records = new Map();
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
        for (const event of [...local, ...remote]) records.set(event.id || JSON.stringify(event), event);
        if (active) { setEvents([...records.values()]); setProblem(""); }
      } catch (error) { if (active) setProblem(String(error.message || error)); }
      if (active) timer = setTimeout(poll, 1500);
    }
    poll();
    return () => { active = false; clearTimeout(timer); abort.abort(); };
  }, [apiBase]);
  const selected = events[Math.min(selection, Math.max(0, events.length - 1))] || {};
  const diagram = selected.mermaid || selected.diagram ||
    (selected.kind === "recipe_diagram" || selected.kind === "mermaid" ? selected.content : "");
  const source = selected.sourceLocation || selected.source_location ||
    (selected.kind === "source_location" ? selected.content : "");
  const links = rawLinks || (window.FormalAiMemory ? window.FormalAiMemory.exportLinksNotation(events) : "");
  return <section className="debugger-view" data-testid="debugger-view" aria-label="Formal AI Debugger">
    <header><strong>Formal AI Debugger</strong><span role="status">{problem || `${events.length} recorded events`}</span>
      <button type="button" disabled title="Stepping is unavailable in this session">Pause</button>
      <button type="button" disabled title="Stepping is unavailable in this session">Next stage</button>
    </header>
    <div className="debugger-panes">
      <section aria-label="Conversation"><h2>Conversation</h2>{messages.map(message => <p key={message.id}><strong>{message.role}</strong> {message.content}</p>)}</section>
      <section aria-label="Event links"><h2>Event links</h2><label>Recorded event <select value={Math.min(selection, Math.max(0, events.length - 1))} onChange={event => setSelection(Number(event.target.value))}>{events.map((event, index) => <option key={event.id || index} value={index}>{event.id || index}: {event.kind || event.role || "event"}</option>)}</select></label><pre>{links || "No links recorded"}</pre></section>
      <section aria-label="Recipe diagram"><h2>Recipe diagram</h2><pre data-format="mermaid">{diagram || "No recipe diagram recorded"}</pre></section>
      <section aria-label="Executing source"><h2>Executing source</h2><pre>{source || "No source location recorded"}</pre><pre>{JSON.stringify(selected, null, 2)}</pre></section>
    </div>
  </section>;
}
