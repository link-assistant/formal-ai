# Out-of-box and passthrough modes

The native **out-of-box** engine runs the symbolic engine in-process, needs no
third-party agent installation, and keeps agent mode off until the user opts
in. Chat, Agent, and Full Auto are permission levels within that engine: Chat
does not execute actions, Agent gates mutations, and Full Auto permits
configured actions automatically.

**Passthrough** sends the same chat turn to an external agent engine while
keeping Formal AI's UI, local server, and shared memory boundary. The desktop
adapter uses [agent-commander](https://github.com/link-assistant/agent-commander)
(`start-agent`) rather than spawning `agent`, `codex`, or `claude` directly.
This keeps stream normalization, session resume, environment filtering, and
read/write permission policy in one controller.

Desktop detects installed Agent, Codex, and Claude executables and lists the
available passthrough engines beside **Out of the box** in its engine selector.
On first launch an installed Agent CLI is preferred; without one, the native
engine remains the default. A saved available selection wins on later launches.
Switching the selector affects the next turn without changing the conversation
or shared-memory location. This behavior shipped through PR #783 for issue
#759. Outside Desktop, `formal-ai with agent`, `codex`, or `claude` provides the
same local-server integration without a UI selector.

The Docker **Agent environment** is different from engine selection: it
installs an isolated container containing Agent and agent-commander. It never
grants host filesystem or shell access merely because the container exists.

Verify the active path in the UI status line and with a benign directory-list
request. Native mode should report the in-process/local server engine;
passthrough should create an external client session while the answer still
appears in the same conversation.
