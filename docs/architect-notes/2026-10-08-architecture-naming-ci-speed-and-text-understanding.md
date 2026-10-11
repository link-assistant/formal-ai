# Architecture, naming, notation, CI speed and text understanding

Source: the architect's 2026-10-08 instructions while continuing
[PR #1188](https://github.com/link-assistant/formal-ai/pull/1188). Every quote is copied byte for byte from
[`docs/case-studies/pull-request-1188/user-messages.md`](../case-studies/pull-request-1188/user-messages.md),
which `node experiments/formal_ai_subagent/collect-user-messages.mjs --write` regenerates from the session history.
The requirements these words became are rows R1188-U1 to R1188-U26 of
[`docs/requirements/issue-1188-user-requirements.md`](../requirements/issue-1188-user-requirements.md); this note keeps the words.

## Architecture, naming, notation and CI speed

Requirement rows: R1188-U1 to R1188-U14.

2026-10-08 (10:48 UTC), message 131 of `user-messages.md`:

> Double check that all requirements we were talking about in the issue, the pull request and this conversation (found this conversation in Claude Code sessions history and collect all my messages), make sure we have all requirements listed in docs of repository and they are fully delivered. Bulk draft undrafted requirements, try to delegate as much work to Formal AI as possible, if it fails - fix it. Do it by generalization instead of specialization, meaning we can have more specific tests, yet they all must be passed by smaller more universal code. Check https://raw.githubusercontent.com/link-foundation/code-architecture-principles/refs/heads/main/README.md, and make sure our architecture is fully perfect, and we have less file names that contain numbers or something not self explanatory. We should have nice directory and file tree structure, where files are split in meaniningful categories instead of parts or pages (where it actually possible). Double check our links notation fully human readable, yet consise and deduplicated as possible. So we spend less characters for pre-cached memory (yet it should not beat the readability, may be there is way to smartly reorganize the data network/tree so it makes it even readable and take less space). Also in code and in links notation where is our responsibility we must use full english words by default as variables, functions and any other entities expressed as a links. In links notation we should prefer `-` over `_`. Try to use as much automation for changes as possible, delegate some to Formal AI, so you can do more changes by applying rules (turing complete substitutions as per motif of our meta links theory and so on). Most clearly easy to automate requirements should be checked in CI/CD, and CI/CD should not have big steps/flows that exected for more than 15-30 minutes, so we can iterate as fast as possible, all long running tests must start first, so when executed in parallel with other tests we fit all system resources with smaller tests once all big once executed and so on. It should be done both on CI/CD level (parallel machines) and on tests level (parallel tests execution), we should also enforce that automatically. So we can iterate faster, and come closer to fully recursive self-improvement. Double check all our docs and vision all in sync and everything contradicting this latest vision is fixed.

## Text understanding

Requirement rows: R1188-U18 to R1188-U22.

2026-10-08 (11:25 UTC), message 132 of `user-messages.md`:

> Double check formal AI is capable of high quality formalization of any text in the internet, like wikipedia pages, and is capable of natural language translation (we can test for round trip translation, best translation is the one that services round trip translation in all languages), we also need to make sure our formal AI is capable of listing all exact requirements from the issues (we have alot of issues to have examples), and we also need to make sure formal AI is capable of doing summariazation, but selected the most important statements/facts which all other statements just describe or depend up on, deduplicate what was already provided in text, so we can do consise summarization algorithmically with no need to so with any LLMs and so on. Double check previous requirements and these requirements are carefully tracked in our repository, recordered and fully delivered, now 4 subagents is running, lets move to not more than 3 sub-agents, remember our contribuding guidelines, make sure we fully deliver everything I ask. A give all permissions required to what previsouly was forbidden by auto classifier, do it again. If auto classier ever tries to stop you - I give you permissions to do all things nessesary. If that does not work after 2-3 attempts wait for me I will give you permissions again.

## Local resources and Formal AI from JavaScript

Requirement rows: R1188-U24, R1188-U25.

2026-10-08 (05:59 UTC), message 129 of `user-messages.md`:

> Try use experiments folders in our repository instead of scratchpad for everything that is useful, Formal AI may use them too, if asked. We need to make sure it is possible so we use more of Formal AI as our subagent working on requirements of Formal AI itself, so the recursive self improvement will arrive faster.

2026-10-08 (07:36 UTC), message 130 of `user-messages.md`:

> Are you not careful with disk space? I asked to avoid building rust locally. Formal AI must be used from JavaScript source only locally for testing and experimenting, did we fully deliver automated translation between JavaScript and Rust versions of code via meta language?

2026-10-08 (11:41 UTC), message 139 of `user-messages.md`:

> Also be careful with disk space, we again running out of it.

2026-10-08 (11:44 UTC), message 140 of `user-messages.md`:

> We also need to minimize number of tests we are running locally, only run most critical tests where you do fixes and so on, everything else will be checked in CI/CD where you can do bulk fixing.

## Readable code

Requirement rows: R1188-U23.

2026-10-08 (11:40 UTC), message 137 of `user-messages.md`:

> I think we should not have single line files in ./ts folder of repository, everything must be human readable, so it should be multiline code in ./js, ./ts, ./rust

2026-10-08 (11:40 UTC), message 138 of `user-messages.md`:

> That must be also strict requirement. If we need some dist folders we can make exceptions for them, but not for any regular code.

## Subagents and delivery

Requirement rows: R1188-U22, R1188-U26.

2026-10-08 (11:26 UTC), message 133 of `user-messages.md`:

> I didn't ask to stop any agents, now you need fully deliver work of stopped agent too. I asked not to spawn more than 3 sub agents, you can wait them all to finish.

2026-10-08 (11:47 UTC), message 141 of `user-messages.md`:

> Double check you apply all our guide lines and actually use up to 3 subagents to bulk draft all undrafted changes, make sure on of the agents commits fixes to CI/CD as soon as results of runs are ready.

2026-10-08 (11:47 UTC), message 142 of `user-messages.md`:

> All undrafted requirements must be drafted.

2026-10-08 (12:09 UTC), message 143 of `user-messages.md`:

> Also work yourself, double check that all requirements are fully delivered, we need to fully deliver this pull request to guarantee we will have valid working and testable release.

## What changed

- Generalization is the rule for every fix: specific tests pass through smaller, more universal code.
- The architecture follows [code-architecture-principles](https://github.com/link-foundation/code-architecture-principles); [`docs/architecture/principles.md`](../architecture/principles.md) maps each principle to how this repository applies and enforces it.
- Names are full English words; file and directory names say what they hold, without numbered parts; owned links notation prefers `-` over `_`, stays readable and is deduplicated; bulk changes are made by rules.
- No CI job or step runs over 15-30 minutes; long work starts first; jobs and tests run in parallel; a gate enforces it.
- Formal AI formalizes real text, translates by round trip, lists the exact requirements of an issue and summarizes by dependency, all without an LLM.
- Every regular source file in `js/`, `ts/` and `rust/` is readable multi-line code; only distribution bundles may be exempt.
- Locally: no Rust builds, Formal AI from its JavaScript source, the fewest tests next to the change, and care for disk space; CI does the rest.
- At most three subagents at once, none stopped early, one of them committing CI fixes as soon as runs report.
