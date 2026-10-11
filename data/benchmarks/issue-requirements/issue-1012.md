### Latest default-branch CI/CD runs

| Workflow | Status | Conclusion | Run |
| --- | --- | --- | --- |
| Desktop Release | completed | skipped | [run](https://github.com/link-assistant/formal-ai/actions/runs/31873152496) |
| Broken Link Checker | completed | success | [run](https://github.com/link-assistant/formal-ai/actions/runs/31871548746) |
| Security | completed | success | [run](https://github.com/link-assistant/formal-ai/actions/runs/31871548779) |
| Question necessity ratchet | completed | success | [run](https://github.com/link-assistant/formal-ai/actions/runs/31871548794) |
| Stock Rust Install | completed | success | [run](https://github.com/link-assistant/formal-ai/actions/runs/31871548725) |
| Write-Effect Ladder | completed | success | [run](https://github.com/link-assistant/formal-ai/actions/runs/31871548767) |
| Task Ladder | completed | success | [run](https://github.com/link-assistant/formal-ai/actions/runs/31871548726) |
| Agentic CLI Matrix | completed | success | [run](https://github.com/link-assistant/formal-ai/actions/runs/31871548784) |
| Coverage | completed | success | [run](https://github.com/link-assistant/formal-ai/actions/runs/31871548753) |
| CI/CD Pipeline | completed | cancelled | [run](https://github.com/link-assistant/formal-ai/actions/runs/31871548846) |

Use all the best practices from CI/CD templates (check full file tree to compare for all GitHub workflow and CI/CD scripts file), if the same issue is found in template report issue also in templates:

- https://github.com/link-foundation/rust-ai-driven-development-pipeline-template
- https://github.com/link-foundation/js-ai-driven-development-pipeline-template
- https://github.com/link-foundation/python-ai-driven-development-pipeline-template

We should compare all files, so we don't have more CI/CD errors in the future and reuse all the best practices from these templates.

Follow the CI/CD best practices collected in [https://github.com/link-assistant/hive-mind/blob/main/docs/CI-CD-BEST-PRACTICES.md](https://github.com/link-assistant/hive-mind/blob/main/docs/CI-CD-BEST-PRACTICES.md).

Please plan and execute everything in this single pull request, you have unlimited time and context, as context auto-compacts and you can continue indefinitely, until it is each and every requirement fully addressed, and everything is totally done.

---

<details>
<summary>Context collected by <code>/fix --ci-cd</code></summary>

- **Repository:** [link-assistant/formal-ai](https://github.com/link-assistant/formal-ai)
- **Default branch:** `main`
- **Latest commit:** `ac6e24d` ([commit](https://github.com/link-assistant/formal-ai/commit/ac6e24d5955a9df6a8ad3b8e24db33eb9c608a4d)) — Merge pull request #1011 from link-assistant/issue-938-fba70db6fbf5
- **CI/CD runs found:** 10 (1 not passing)

**Detected languages**

- **Rust** — 68.1%
- **JavaScript** — 20.3%
- **HTML** — 5.8%
- **Shell** — 2.6%
- **Python** — 2.4%
- **CSS** — 0.7%
- **PowerShell** — 0.0%
- **TypeScript** — 0.0%
- **Vim Snippet** — 0.0%
- **Dockerfile** — 0.0%
- **Ruby** — 0.0%
- **Jinja** — 0.0%

**Recommended CI/CD templates**

Apply the best practices from these templates, in priority order (most-used language first):

1. **Rust** — [link-foundation/rust-ai-driven-development-pipeline-template](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template) _(detected: Rust)_
2. **JavaScript / TypeScript** — [link-foundation/js-ai-driven-development-pipeline-template](https://github.com/link-foundation/js-ai-driven-development-pipeline-template) _(detected: JavaScript, TypeScript)_
3. **Python** — [link-foundation/python-ai-driven-development-pipeline-template](https://github.com/link-foundation/python-ai-driven-development-pipeline-template) _(detected: Python)_

Other detected languages without a dedicated template: HTML, Shell, CSS, PowerShell, Vim Snippet, Dockerfile, Ruby, Jinja.

</details>
