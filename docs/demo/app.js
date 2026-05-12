const { createElement: h, useEffect, useRef, useState } = React;

function App() {
  const workerRef = useRef(null);
  const [messages, setMessages] = useState([
    { role: "assistant", content: "Hi, how may I help you?" },
  ]);
  const [prompt, setPrompt] = useState("");
  const [pending, setPending] = useState(false);
  const [workerState, setWorkerState] = useState("wasm worker");

  useEffect(() => {
    const worker = new Worker("formal_ai_worker.js");
    workerRef.current = worker;
    worker.onmessage = (event) => {
      if (event.data.kind === "ready") {
        setWorkerState(event.data.mode);
        return;
      }
      setMessages((items) => [
        ...items,
        { role: "assistant", content: event.data.content },
      ]);
      setPending(false);
    };
    return () => worker.terminate();
  }, []);

  function send() {
    const text = prompt.trim();
    if (!text || pending) {
      return;
    }
    setMessages((items) => [...items, { role: "user", content: text }]);
    setPrompt("");
    setPending(true);
    workerRef.current.postMessage({ prompt: text });
  }

  function handleKeyDown(event) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      send();
    }
  }

  return h(
    "main",
    { className: "app" },
    h(
      "header",
      { className: "topbar" },
      h("div", { className: "brand" }, h("span", { className: "mark" }, "FA"), "formal-ai"),
      h("div", { className: "status" }, workerState),
    ),
    h(
      "section",
      { className: "messages", "aria-live": "polite" },
      messages.map((message, index) =>
        h(
          "article",
          { key: index, className: `message ${message.role}` },
          message.content,
        ),
      ),
    ),
    h(
      "form",
      {
        className: "composer",
        onSubmit: (event) => {
          event.preventDefault();
          send();
        },
      },
      h(
        "div",
        { className: "composer-inner" },
        h("textarea", {
          value: prompt,
          rows: 2,
          placeholder: "Message formal-ai",
          onChange: (event) => setPrompt(event.target.value),
          onKeyDown: handleKeyDown,
        }),
        h("button", { type: "submit", disabled: pending || !prompt.trim() }, pending ? "..." : "Send"),
      ),
    ),
  );
}

ReactDOM.createRoot(document.getElementById("root")).render(h(App));
