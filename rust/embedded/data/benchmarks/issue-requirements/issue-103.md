At the moment we have lots of test cases, but I want to ensure, for each test case we support 5-10 most probable inputs and outputs variations.

And these are translated in all English, Russian, Hindi, Chinese.

So each case in 5-10 most probable inputs and outputs multiplied by number of supported languages.

Also we should compare our test cases with other test cases of AI models tests and agentic CLI tools tests. For each type software we should have carefully crafted comparison of features. And we should also take most use/frequent and probable test cases from each of competitors, and make sure we support them all.

And if possible, please generalize the logic on the way.

Also make sure our codebase contains detailed ARCHITECTURE.md, and we are still evolving architecture. So at the moment I see it like this:

Last input message + all previous messages + memory + data about the user are context of our AI system.

Input message should be translated into links notation as link of statements, meaning a sequence of statements or questions.

It should also be recorded as is in memory, after that we should do formalization, for example each verb phrase should become P id from wikidata, and each noun phrase should become Q id from wikidata. Or if possible wikipedia article link or wiktionary. We also need to ensure that formalized concept actually contains the word or phrase, and matches it semantically.

Once we get multiple formalization options, we should use our temperature configuration in similar way neural networks use, to select the most probable interpretations.

If some interpretations have equal or close probability, we should ask questions to user to distinguish between them, or we could just try guessing (as per configuration). So in either case, we get to select some formalization variants, reason about how to construct the plan (with real reasoning steps fully viewable in diagnostics mode).

We also need to support nested reasoning steps, as tools like link assistant calculator we use can provide its own reasoning and calculation steps.

After collecting test cases and make sure we have them we use test driven development to actually implement them, and we should try as much as possible not to memoize answers, but actually reason through to provide answers based on reasoning. And every reasoning step should be displayed in diagnostic mode, so we can see them and debug them.

All reasoning steps and actions should be recorded in growable memory we have, so next time we need to reason on something similar we can reuse reasoning at least partially, also all requests and responses to the internet or file system should be recorded, so we also can reuse that. At the moment we should have maximum verbosity for the memory, every operation should go though real doublets-rs and doublets-web data store, with regular backups to browser (local storage, indexdb and so on) and other storages like disk, all backups into persistent memory should be done in .lino files or text representing links notation.

So instead of expensive GPU and neural networks we will use reasoning with access to internet as public database with our local memory as cache for that public human knowledge. As the result we should be able to reproduce universal problem solving algorithm in actual coding rules.

Our associative data store should support stored transformation/substitution rules,  that either fully in the data, and provide `when x do y` logic, or can be mapped in rust/js implemented handlers in our code, or can be mapped to dynamically compiled rust/js code that is stored in associative database as is. Or we can store instead natural language instructions or skills (as it is common for today's ais), and use these natural language instructions/skills as convertible on demand to actual code or associative transformation/substitution rules, to have ability to compile these to rust/js and after that to binary, or also to interpret them (execute as is one step at a time).

Also once we have full formalization of any natural language expression, it is easy to translate it to any other language, because we can get data from wikipedia, wikidata and wiktionary on how each phrase, word or even statement translates from source language to target language.

And because of formalization we can also translate between natural language and programming languages (in any direction).

Double check our requirements and vision are updated with what I tell here.

We need to collect data related about the issue to this repository, make sure we compile that data to `./docs/case-studies/issue-{id}` folder, and use it to do deep case study analysis (also make sure to search online for additional facts and data), list of each and all requirements from the issue, and propose possible solutions and solution plans for each requirement (we should also check known existing components/libraries, that solve similar problem or can help in solutions).

Please plan and execute everything in a single pull request, you have unlimited time and context, as context auto-compacts and you can continue indefinitely, until it is each and every requirement fully addressed, and everything is totally done.

