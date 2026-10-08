DuckDuckGo search API working without CORS restrictions, so it should be set by default as search engine for all web search requests and tool calls in CLI, server, and browser only app on GitHub Pages and other UIs we have, so the behavior will be consistent.

When we have multiple search engines available it is important to by default implement combine top 10 results for all of them to relative rerank, so urls that are found at multiple search engines should bubble up.

We also should expand our tests in browser only page https://link-assistant.github.io/formal-ai/tests

We should add more popular search engines support in both tests and the codebase.
Add more code hosting providers like GitHub, GitLab, BitBucket and others from China, Russia and other locations.
Add more general knowledge databases like Wikipedia, Wikidata, Wiktionary, ontologies and so on.

Confirmation that DuckDuckGo API is working:

<img width="1824" height="1105" alt="Image" src="https://github.com/user-attachments/assets/bc10c05e-48bc-495a-86a9-cd9e7e0ef120" />

We have multiple general knowledge providers working in the browser only mode, meaning we should fully support them in codebase for reasoning about questions and coding:

<img width="1824" height="1105" alt="Image" src="https://github.com/user-attachments/assets/b55d6e66-7121-4276-8050-7f93189f0c58" />

Confirmed working locally:
- Wikipedia
- Wikidata
- Open Library
- OpenAlex
- Crossref

We should also add tests for providers of scientific papers and journals, and test if it possible to get in browser only mode articles and pdf originals of them. Only free (without paywall will work for now).

Also expand our test cases in such a way, that we actually trigger access to these external APIs for reasoning about questions or coding tasks.

Also make sure each reasoning step is recorded in exportable memory, with full request to external API, full response from external API, its interpretation to unified links format (so we can do the same request to multiple available services, and combine results), make sure we do such requests in parallel, and by default use not more than 5 databases at a time, not more 5 search engines at a time. Off course if we get CORS we should temporary disable requests to such services.

Double check that as much code as possible we use for logic compiled from Rust to WebAssembly, so we don't duplicate the code. Also JavaScript code can be only used for UI purposes, all the data processing should be done in Rust background worker compiled to webassembly, it will allow to make UI responsive, allowing for example user to work at the same time on multiple conversations, and each one processed in the background without blocking the UI or duplicating the code.

Please check existing features and test cases, and try to expand them with more specific variations tested, but implementation should be done mostly generalized, so we really reason about questions and tasks, with focus on data and coding tasks. So more generalized implementation, more specifics in tests, and less just memoization (if possible we should replace all memoization with reasoning), and we cache something we should cache also API requests and reasoning steps, so preseed data should be no different from real time cached data, so everything is compatible and will just continue working.

Also double check the entire codebase and docs are consistent with previous issues, and this one.

We need to collect data related about the issue to this repository, make sure we compile that data to `./docs/case-studies/issue-{id}` folder, and use it to do deep case study analysis (also make sure to search online for additional facts and data), list of each and all requirements from the issue, and propose possible solutions and solution plans for each requirement (we should also check known existing components/libraries, that solve similar problem or can help in solutions).

Please plan and execute everything in a single pull request, you have unlimited time and context, as context auto-compacts and you can continue indefinitely, until it is each and every requirement fully addressed, and everything is totally done.


