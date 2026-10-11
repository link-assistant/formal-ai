Everything that is parsable as expression of https://github.com/link-assistant/calculator (natural language or math expressions that looks like math expressions) should be handled by https://github.com/link-assistant/calculator, so we can reduce number of code at our side, we still need to keep support for everything that https://github.com/link-assistant/calculator does not yet support, which essentially basic calculation related, https://github.com/link-assistant/calculator should be replacement for wolfram alpha, so everything that is basic calculation and happens to be parsable by https://github.com/link-assistant/calculator should be delegated to it.

We should also report any missing features with focus on math like or purely computational expressions. And language processing should be still in the format-ai codebase.

While you at this task, double check that all cases you touch have 5-10 natural language variations in tests, and in all currently supported languages - English, Russian, Chinese, Hindi. And also everything that is `not NSFW`, should be also added to our example prompts and chat demo simulator.

We also should all tests from https://github.com/link-assistant/calculator itself, make them wider, multilanguage, so we not only test the entire format ai agent pipeline, but may be we will find edge cases and bugs in https://github.com/link-assistant/calculator, so all of them should be reported there.

We need to collect data related about the issue to this repository, make sure we compile that data to `./docs/case-studies/issue-{id}` folder, and use it to do deep case study analysis (also make sure to search online for additional facts and data), list of each and all requirements from the issue, and propose possible solutions and solution plans for each requirement (we should also check known existing components/libraries, that solve similar problem or can help in solutions).

Please plan and execute everything in a single pull request, you have unlimited time and context, as context auto-compacts and you can continue indefinitely, until it is each and every requirement fully addressed, and everything is totally done.
