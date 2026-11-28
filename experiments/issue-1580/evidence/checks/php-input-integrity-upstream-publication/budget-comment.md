Additional bounded evidence at current source
b61e334703d93a15e08bf1a278e0df0a9d6e98a8: the wrapper blob remains
71d906ad4a716902bafdf02423d8fde27f0081c2, identical to our pinned comparison.

[Seven finite process-group observations](https://github.com/link-assistant/formal-ai/blob/ac9cd6a55e3ea7d8738e6122f623d7676666630d/experiments/issue-1580/evidence/checks/php-budget-parent-exit/verification.json)
all see a live worker when this template wrapper exits. Expired parent-success
returns 0 and parent-failure returns 7 instead of the expected 124; a TERM-resistant
worker survives too. The positive short-worker/warning-only status controls also
exit while their worker is still live. Unavailable process inspection is retained
as a separate case. The current Formal AI counterpart passes the same seven
unchanged controls.

The [probe](https://github.com/link-assistant/formal-ai/blob/ac9cd6a55e3ea7d8738e6122f623d7676666630d/experiments/issue-1580/probe-budget-process-lifetime.py) uses a
Linux subreaper, finite four-second workers, a ten-second observation boundary,
bounded stack/address space, and cleanup of only its own process group. It checks
liveness before cleanup, so cleanup cannot create a false success. No sudo or
privileged process is launched; these results do not qualify Windows or EPERM
behavior. stdout/stderr, original status and source hashes are retained per case.

The existing PipelineShellScriptsTest expects zero while a background worker is
still live and kills the worker itself afterward; it does not enforce the stronger
ownership contract. These controls supplement the original report without
duplicating the closed issue. They do not establish an upstream repair.
Context: https://github.com/link-assistant/formal-ai/pull/1581.
