# First-attempt Ubuntu mirror preference

The green hosted baseline still reports two first-attempt APT deadlines in
Layered CI 45162493244: line 9012 follows regional HTTP download stalls and
line 25085 interrupts dpkg database reading. Both eventually recover. Their
captured contexts are in [the terminal baseline](apt-repair-terminal-baseline.md).
The first warning motivates avoiding the observed regional HTTP source before
search-tool installation starts; source selection alone cannot prove the dpkg
warning is eliminated.

Search provisioning now requests the existing exact official Ubuntu HTTPS
alternate before its first package operation. Preparation remains inside the
privileged attempt deadline and uses the same owned regular-file, size, path,
content and atomic replacement guards. A runner without that mirror list keeps
its configured sources. FORMAL_AI_APT_OFFICIAL_MIRROR_FIRST=0 disables the
preference; disabling fallback also disables the default first preference.
The shared installer defaults the new flag to zero. Existing attempt counts,
deadlines, interrupted-package recovery and signed package verification remain.

## Reproduction and verification

Three focused cases fail against the unchanged production scripts: the EC2 and
Azure fixtures observe regional HTTP at the first package operation, and an
invalid new flag is accepted. After the repair, 18 search-installer, 16 official
mirror, 6 container, 9 browser isolation and 11 browser release tests pass,
60 total. Twelve affected JavaScript gates, including the secret scan of committed
source, and both shell syntax checks pass.
Hosted execution of this new source remains pending.

The first neighboring container run had five exact 15-second timeouts. Targeted
Bash tracing shows host .bashrc loading pyenv initialization before APT. The
fixture's recording rm intercepted pyenv's lock cleanup and left its rehash
waiting. The fixture now uses a temporary home, disabled startup environment
files and --noprofile --norc. Every dynamic case includes a hostile .bashrc and
simulated SSH client. The reusable
[control](../../../experiments/issue-1580/container-apt-startup-control.mjs)
removes the shell flags and records shell exit 97 before package operations;
all six isolated tests pass with their original 15-second limit. No production
clock was increased. The initial failures and both intermediate probes remain
in the receipt, including the first control's less useful missing-call-file error.

## Evidence and limits

The [sealed local receipt](../../../experiments/issue-1580/evidence/checks/apt-first-mirror-repair-20261123/manifest.json)
contains full command logs, before/after source, the original failed container
suite, targeted trace captures and the controlled startup reproduction.
Its archive has 56 members and 72238 compressed bytes; SHA-256
fdac19e84b8891346835848bf2877c78f4c56736271bb417d6074cb8a9bc7f01.
The [final committed-source gates](../../../experiments/issue-1580/evidence/checks/apt-first-mirror-final-checks-20261123/manifest.json)
add the successful secret scan and complete twelve-gate result.
Every member was read back and hash checked. This is targeted trace review,
not a literal read of every tracing line or qualification of the full issue.

[PR #1581](https://github.com/link-assistant/formal-ai/pull/1581) remains draft.
The real hosted producer rerun and later local native repairs are separate
pending qualifications. Supplied edits and conclusions earn zero independent
authorship credit. All broader unmet criteria remain open.
