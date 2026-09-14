# Local release-ratchet precondition

Fetch annotated release tags before running the pull-request ratchet locally:

`git fetch origin --tags`

Without the latest tag, the check reports a skip instead of evaluating the
current release cycle.