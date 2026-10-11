We should implement formal / symbolic AI, that produces the same API as chat completions and responses of OpenAI. Meaning it should be possible to connect that formal language processor to any agentic tools like claude, codex, and our own http://github.com/link-assistant/agent

But this formal / symbolic AI must not use any GPU requiring neural networks. Yet we can try https://en.wikipedia.org/wiki/Bayesian_network, https://en.wikipedia.org/wiki/Markov_chain, but we should reimplement it on top of Links Notation (http://github.com/link-foundation/links-notation), and Links Data Store (http://github.com/link-foundation/link-cli).

Links allow us to give unique reference (like id or name) to any construct of links, that can be as complex as single item, pair, tree, sequence or fully cyclical network. And in all cases that would be 1 single link, that consists of links, and has one unique reference (id or name).

Meaning we can go beyond probability of continuation by latters, we can use words, and we also can have references for noun phrases, verb phrases, and sentences, paragraphs, even pages and documents. All should be learnable from large dataset like copy of http://wikidata.org or https://www.wikipedia.org. We can also use reinforcement learning techniques, but applied to data, not to neural network weights. Using neural networks is forbidden in this project (as they are GPU intensive and expensive).

At the same time, we are also interested in searching of Universal Problem Solving algorithm, that should work even without prior knowledge collected, meaning it should be able to learn anything on the go, for example algorithm finds a word or symbol - he has nothing to know about yet, and he collects data about every unknown as human will do, after that he reasons and formalizes everything in relation to tasks requirements, once task requirements are formalized, they should be formally translated to preferred programming language, reasoning can include translation natural language to formal statements that can be checked using provers like http://github.com/link-foundation/relative-meta-logic. Once tests are formalized, we should use more intuitive approach, using some data learned from successful code, we can use https://rosettacode.org/wiki/Rosetta_Code as dataset to learn from, and test against. So we should learn some probabilities of code constructs going after each other, so we can do essentially similar to randomized search by most probable probability of continuation (also links allow to have recursive sequences and so on). Once we generated few solutions for tests, we actually execute tests and learn from mistakes, of how these are failing, and use http://github.com/link-assistant/hive-mind/issues (for tasks descriptions) and https://github.com/link-assistant/hive-mind/pulls (pull requests with comments that contain logs from LLMs, we can learn from how they reason though each problem, and apply it to our universal algorithm). So once tests are passed using single solution we can continue to iterate by trying to simplify formally, but knowing which code expressions are essentially equivalent to per each input and output, again we can use https://rosettacode.org/wiki/Rosetta_Code as dataset, as well as https://www.wikifunctions.org/wiki/Wikifunctions, we can merge both actually, and make our own dataset stored in Links Notation in the repository, that is loadable to http://github.com/link-foundation/link-cli for fast operations and iterations. So we can work on simplification the algorithm, to find the most short possible form, without sacrifices on readability, so we should not try to make minified code like converting `item` to `i` in variables, we should not count variable names in the simplification, we need to simplify the code expression or statements themeselves.

We should focus on something similar that works, for example usually on `Hi`, `Hello`, LLMs reply with `Hi`, `Hello`, `Hi, how may I help you?` and so on at random. And for `Write me hello world program in Rust` they usually replay with markdown containing Rust code with instructions how to make it.

Instead of relying on expensive neural networks, we should use http://helloworldcollection.de as dataset, and convert it to Links Notation, that is usable via http://github.com/link-foundation/link-cli, so we have all required datasets in the repository, to response simple questions fast, and for reasoning we should do actual formal reasoning.

We should use all the best experience from:
- http://github.com/link-assistant/calculator
- https://github.com/link-assistant/human-language
- https://github.com/link-assistant/meta-expression
- https://github.com/link-foundation/relative-meta-logic

And use as dependencies:
- https://github.com/link-foundation/command-stream
- http://github.com/link-foundation/link-cli (CLI + library if available)
- https://github.com/link-foundation/lino-objects-codec
- http://github.com/link-foundation/links-notation
- http://github.com/linksplatform/doublets-rs
- http://github.com/linksplatform/doublets-web

The project should be available as library, CLI, API server (in CLI and in docker - ready microservice), and also GitHub Pages (React.js demo chat page with Rust web worker in web assembly). And also desktop application as in https://github.com/konard/vk-bot-desktop.

Make sure all requirements are listed at ./docs/REQUIREMENTS.md, we should have working minimum prototype for each API/CLI/UI and so on, so all interactions are working with users, everything is unit/integration/e2e tested, for e2e testing we should use http://github.com/link-foundation/browser-commander

We need to collect data related about the issue to this repository, make sure we compile that data to `./docs/case-studies/issue-{id}` folder, and use it to do deep case study analysis (also make sure to search online for additional facts and data), list of each and all requirements from the issue, and propose possible solutions and solution plans for each requirement (we should also check known existing components/libraries, that solve similar problem or can help in solutions).

Please plan and execute everything in a single pull request, you have unlimited time and context, as context auto-compacts and you can continue indefinitely, until it is each and every requirement fully addressed, and everything is totally done.


