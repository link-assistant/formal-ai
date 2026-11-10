# See what we do most wrong, that blocks our vision, and plan to fix that

Source: the architect's text of
[issue #1548](https://github.com/link-assistant/formal-ai/issues/1548), "We need
to see what we do most wrong, that blocks our vision and plan to fix that", as
it stood after its last edit on 2026-11-10T08:42:55Z. The whole text is quoted;
nothing is omitted and nothing is reworded. A snapshot of the issue is kept in
`docs/case-studies/issue-1548/raw-data/github/issue-1548.json`.

> We should create an issue with all open sub-issues, that block our progress on coding (so we go faster to RSI), general question answering and so on.
>
> All steps like formalization and deformalization (naturalization) with meta language should be supported.
>
> We also need to double check our meta algorithm is truly recursive as per our vision in picture of README.
>
> It should be able to discover information as it goes with reasoning, that discovery should be automatically cached, so we use trusted sources to just enough data to understand the task, be able to split it in specific requirements, and formalize it in to tests in required programming language, and reproduce (from internet) or generate new algorithms (if no algorithm on the internet exists), generation of new algorithm should use the same recursive meta algorithm, so we can split it in parts and for each part solve it saparately that may lead to reuse of parts of existing algorithms and so on.
>
> We should also be able to use source code on GitHub for reference (we only use ideas, not copies of files), and for copy of public domain licensed repository, and if user asks or allows we can use other source code respecting its license as library if possible, or copy file with respecting the file with no violation of original requirements.
>
> Our requirements are like axioms or statements with high probability which should define world model or formal system of the task context, meaning, we should derive next logical statements from that.
>
> So if after formalization we have still some unresolved details, we should try multiple ways to resolve them by formally infer missing statements/details from original requirements statements using https://github.com/link-foundation/relative-meta-logic
>
> We also collect knowledge on the internet and use previous experience for solution of similar or typical tasks to actually infer missing details.
>
> Our system always should prefer to study multiple options, but in all cases, it should calculate probability or fit to the original requirements based on relative-meta-logic and automatically select best option, once selected it may proceed with plan for implementation, and after plan for implementation is defined enough it should be able to actually go and do all the actions in the plan to actually implement it in the target repository or folder.
>
> If something contradicts this vision it should be updated. If there multiple valid options to try our in our research, we should support configuration (switching) between them.
>
> If something is still not detailed enough use this issue's pull requests to update our vision, docs, guidelines and so on, so all blocking and missing description will help us unblock the movement to actual implementation of this vision.
>
> We also must check latest results at (wait for finish if not finished, while doing other tasks):
> - https://github.com/konard/test-hello-world-019fb330-00e1-73b9-955e-f357a1600d5b/pull/2 (here I clearly see no Pull Request description was prepared, was not edited, and also CI/CD was not fixed).
> - https://github.com/konard/test-hello-world-019fb331-c107-78c7-8ff6-9f127a3c593c/issues/1 (at moment of writing the next execution task is in the queue)
> - https://github.com/konard/test-hello-world-019fb330-fa49-7c9d-a664-b7ea33bb698a/issues/1 (at moment of writing the next execution task is in the queue)
>
> We also have auto execution of issues with Formal AI via Hive Mind in this repository and also in https://github.com/link-assistant/hive-mind, so we should collect all failed attempts to solve real GitHub issues by Formal AI, and make sure it can advance them further.
>
> If we have any issues in https://github.com/link-assistant/hive-mind that blocks our Formal AI to be successfully used on the orchestrator side, that also must be reported, so we should fix them, but Formal AI should not get too much of special treatment, it should operate as any other OpenAI compatible API providing the intelligence via the API by using formal reasoning.
>
> We should replace all hardcoded solutions with solutions that actually use our meta algorithm for all the tasks we already support.
>
> We scope of this issue is documentation and issues planing, so the result here should be documentation changes and issues created/organized (double check we clearly mark all open issues as bug and features and so on), and all critical blockers should be organized in separate parent issue with all dependencies between issues clearly marked as blockers on GitHub.
>
> We need to collect data related about the issue to this repository, make sure we compile that data to `./docs/case-studies/issue-{id}` folder, and use it to do deep case study analysis (also make sure to search online for additional facts and data), list of each and all requirements from the issue, and propose possible solutions and solution plans for each requirement (we should also check known existing components/libraries, that solve similar problem or can help in solutions).
>
> Please plan and execute everything in this single pull request, you have unlimited time and context, as context auto-compacts and you can continue indefinitely, until each and every requirement fully addressed, and everything is totally done.
