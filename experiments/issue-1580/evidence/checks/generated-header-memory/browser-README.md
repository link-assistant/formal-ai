# Deadline descendant test fixture

The newly merged main CI log retains a browser-release assertion against the obsolete ubuntu-latest label; the branch already checks the pinned ubuntu-24.04 baseline. Rechecking its whole suite exposed a separate container fixture problem: ten tests passed, but the exact descendant PID remained observable after timeout.

The bounded experiment runs the unchanged production deadline against the original fixture. It returns the required status 124 and observes the child in Linux state Z, adopted by PID 1: exited, but not reaped. In a second run the fixture parent waits for its child's exit event after the process-group TERM, without sending a signal to the child itself. The deadline again returns 124, and the exact PID reports ESRCH.

The test uses that cooperative reaper. The production deadline, timeout limits, expected status, permission-error rejection and exact ESRCH assertion are unchanged. A runner that leaves the descendant executing cannot satisfy the assertion through the parent's handler. The full browser suite now passes all eleven cases with zero skips in 2.873 seconds. Raw failing/passing logs and both process-state observations are retained. No live descendant remains from the completed probes.

The other fresh main JavaScript failures also pass on the merged branch: quoted-passage QA 15/15, connector receipts 20/20, and policy decoding 2/2, each with zero skips. This is supplied Codex work and receives zero independent Formal AI authorship credit.
