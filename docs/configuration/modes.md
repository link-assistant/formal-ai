# Out-of-box and passthrough modes

The current Desktop and VS Code default is the native **out-of-box** engine. It
runs the symbolic engine in-process, needs no third-party agent installation,
and keeps agent mode off until the user opts in. Chat, Agent, and Full Auto are
permission levels within that engine: Chat does not execute actions, Agent
gates mutations, and Full Auto permits configured actions automatically.

**Passthrough** sends the same chat turn to an external agent engine while
keeping Formal AI's UI, local server, and shared memory boundary. The desktop
adapter uses [agent-commander](https://github.com/link-assistant/agent-commander)
(`start-agent`) rather than spawning `agent`, `codex`, or `claude` directly.
This keeps stream normalization, session resume, environment filtering, and
read/write permission policy in one controller.

Selectable Agent/Codex/Claude passthrough and “default to Agent CLI when it is
installed” are implemented in PR #783 for issue #759 but are not part of the
current main release yet. Until that lands, Desktop remains out-of-box by
default; use `formal-ai with agent`, `codex`, or `claude` for an external
engine. After it lands, select the engine in Desktop settings; a saved explicit
choice wins, otherwise a detected Agent CLI becomes the default and native
Formal AI remains available in the same selector.

The Docker **Agent environment** is different from engine selection: it
installs an isolated container containing Agent and agent-commander. It never
grants host filesystem or shell access merely because the container exists.

Verify the active path in the UI status line and with a benign directory-list
request. Native mode should report the in-process/local server engine;
passthrough should create an external client session while the answer still
appears in the same conversation.
