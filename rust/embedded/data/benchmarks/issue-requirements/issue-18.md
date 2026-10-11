For example now on export from https://link-assistant.github.io/formal-ai I get not full memory, but only small changes or part of memory.

```
demo_memory
  event "1"
    kind "message"
    role "user"
    content "HI"
    sentAt "2026-05-15T12:26:00.136Z"
  event "2"
    kind "reasoning"
    role "assistant"
    intent "greeting"
    content "impulse: HI"
    sentAt "2026-05-15T12:26:00.150Z"
  event "3"
    kind "reasoning"
    role "assistant"
    intent "greeting"
    content "formalize: hi"
    sentAt "2026-05-15T12:26:00.150Z"
  event "4"
    kind "reasoning"
    role "assistant"
    intent "greeting"
    content "detect_language: en"
    sentAt "2026-05-15T12:26:00.150Z"
  event "5"
    kind "reasoning"
    role "assistant"
    intent "greeting"
    content "match_rule: greeting"
    sentAt "2026-05-15T12:26:00.150Z"
  event "6"
    kind "message"
    role "assistant"
    intent "greeting"
    content "Hi, how may I help you?"
    sentAt "2026-05-15T12:26:00.150Z"
    evidence "intent:greeting|source:worker|rule:greeting|language:en|trace:impulse:HI|trace:formalization:hi|trace:language:en|trace:rule:greeting"
  event "7"
    kind "message"
    role "user"
    content "What is artificial intelligence?"
    sentAt "2026-05-15T12:26:09.445Z"
  event "8"
    kind "reasoning"
    role "assistant"
    intent "wikipedia_lookup"
    content "impulse: What is artificial intelligence?"
    sentAt "2026-05-15T12:26:10.337Z"
  event "9"
    kind "reasoning"
    role "assistant"
    intent "wikipedia_lookup"
    content "formalize: what is artificial intelligence"
    sentAt "2026-05-15T12:26:10.337Z"
  event "10"
    kind "reasoning"
    role "assistant"
    intent "wikipedia_lookup"
    content "detect_language: en"
    sentAt "2026-05-15T12:26:10.337Z"
  event "11"
    kind "reasoning"
    role "assistant"
    intent "wikipedia_lookup"
    content "invoke_tool: wikipedia_lookup"
    sentAt "2026-05-15T12:26:10.337Z"
  event "12"
    kind "reasoning"
    role "assistant"
    intent "wikipedia_lookup"
    content "dispatch_handler: tryWikipediaLookup"
    sentAt "2026-05-15T12:26:10.337Z"
  event "13"
    kind "tool_call"
    role "assistant"
    tool "wikipedia_lookup"
    inputs "{\"prompt\":\"What is artificial intelligence?\",\"language\":\"en\"}"
    outputs "{\"intent\":\"wikipedia_lookup\",\"confidence\":0.85}"
    content "tool:wikipedia_lookup"
    sentAt "2026-05-15T12:26:10.337Z"
  event "14"
    kind "message"
    role "assistant"
    intent "wikipedia_lookup"
    content "Artificial intelligence: Artificial intelligence (AI) is the capability of computational systems to perform tasks typically associated with human intelligence, such as learning, reasoning, problem-solving, perception, and decision-making. It is a field of research in engineering, mathematics and computer science that develops and studies methods and software that enable machines to perceive their environment and use learning and intelligence to take actions that maximize their chances of achieving defined goals.

Source: https://en.wikipedia.org/wiki/Artificial_intelligence (wikipedia)."
    sentAt "2026-05-15T12:26:10.337Z"
    evidence "intent:wikipedia_lookup|source:worker|wikipedia_lookup:Artificial intelligence|source:https://en.wikipedia.org/wiki/Artificial_intelligence|language:en|trace:impulse:What is artificial intelligence?|trace:formalization:what is artificial intelligence|trace:language:en|trace:handler:wikipedia_lookup"
```

I need to actually get the full memory, so that we can still support all previous version as them are, and also after import suggest user to make known data migrations, to improve the experience.

So our export/import should by default always be full and entire export, as seed may change in our repository or user may make changes to their own AI system by modifying the seed itself. Also it will by much easier and stable for debug, also with prefilled issue report we should also suggest to attach zip archive with .lino file to the issue (and also redacting any sensitive data), as GitHub issues does not support .lino files attachments yet, and it will be too big to prefill with GitHub issue report button/link command/option.

Make sure changes cover the entire codebase, docs and so on, everything should be in sync after this issue pull request will be closed.

We need to download all logs and data related about the issue to this repository, make sure we compile that data to `./docs/case-studies/issue-{id}` folder, and use it to do deep case study analysis (also make sure to search online for additional facts and data), in which we will reconstruct timeline/sequence of events, list of each and all requirements from the issue, find root causes of the each problem, and propose possible solutions and solution plans for each requirement (we should also check known existing components/libraries, that solve similar problem or can help in solutions).

If there is not enough data to find actual root cause, add debug output and verbose mode if not present, that will allow us to find root cause on next iteration.

If issue related to any other repository/project, where we can report issues on GitHub, please do so. Each issue must contain reproducible examples, workarounds and suggestions for fix the issue in code.

Please plan and execute everything in this single pull request, you have unlimited time and context, as context auto-compacts and you can continue indefinitely, until it is each and every requirement fully addressed, and everything is totally done.
