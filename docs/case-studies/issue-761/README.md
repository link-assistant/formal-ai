# Issue 761: end-to-end configuration documentation

## Scope and source of truth

Issue #761 requested one discoverable configuration section spanning the
agent-client testing round (#744–#762). The implementation was traced against:

- `data/seed/client-integrations.lino` for supported wrappers, aliases,
  protocols, arguments, environment, persistent config, sessions, and resume;
- `data/seed/environments.lino` and `data/seed/tools.lino` for the internal and
  external tool inventory and environment mapping;
- the merged implementations and tests for Cursor (#754), multilingual routing
  (#745), hosted tools (#746), Desktop web tools (#747), schema/capability and
  shell routing (#744/#749/#758), friendly output (#750), usage/context
  (#751/#752), shared memory (#756), session files (#757), Desktop passthrough
  (#759), and T3 Code (#760);
- open issue #762, whose not-yet-released OpenCode Desktop launcher is labeled
  as pending rather than presented as a current command.

## Reproducing the documentation gap

`tests/issue_761_docs.rs` is a six-part contract covering discoverability and
OS setup, every registered client, modes/tools, memory/API/output/languages,
each requested surface, and the composed task. Its first run failed all six
tests because `docs/configuration/` and the README entry point did not exist.
The full result is in `agent-cli-evidence/red-test.log`.

## Formal AI / Agent CLI authorship attempt

Following `CONTRIBUTING.md`, a release Formal AI server was started in agent
mode and the real installed Agent CLI was asked to create the red documentation
contract. Formal AI misclassified the repository change as feedback and opened
issue #789 instead of calling write/test tools. The accidental duplicate was
closed with an explanatory comment. The exact task, release build, server log,
Agent stream, and verbose HTTP trace are preserved under `agent-cli-evidence/`;
this demonstrates why the documented manual fallback was necessary rather than
claiming self-authorship.

## Result and verification

The new `docs/configuration/` index contains copy-paste macOS/Linux and Windows
PowerShell setup and links thirteen focused pages. The client and tool pages are
checked dynamically against both seed registries, so a new client or
environment tool cannot silently omit its documentation. `green-test.log`
records all six issue-specific tests passing.
