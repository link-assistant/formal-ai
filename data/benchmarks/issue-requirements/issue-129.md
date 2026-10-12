In one of previous issues I asked to make https://link-assistant.github.io/formal-ai/tests page containing interactive tests for CORS free connections possibility to search engines and public knowledge databases, we need to expand the list the top most popular services, so it will be possible to visually see what works, and what does not.

We should test web pages access via fetch and also api access via fetch, we also need to have ability to expand the iframe to test if iframe actually works and so on.

So we will be able to test the connectivity.

We also should be able to switch to web-capture as a proxy (that we can start locally using formal'as cli or may be we will have such proxy on our server later, so users will have access to it).

We need to download all logs and data related about the issue to this repository, make sure we compile that data to `./docs/case-studies/issue-{id}` folder, and use it to do deep case study analysis (also make sure to search online for additional facts and data), in which we will reconstruct timeline/sequence of events, list of each and all requirements from the issue, find root causes of the each problem, and propose possible solutions and solution plans for each requirement (we should also check known existing components/libraries, that solve similar problem or can help in solutions).

If there is not enough data to find actual root cause, add debug output and verbose mode if not present, that will allow us to find root cause on next iteration.

If issue related to any other repository/project, where we can report issues on GitHub, please do so. Each issue must contain reproducible examples, workarounds and suggestions for fix the issue in code.

Please plan and execute everything in this single pull request, you have unlimited time and context, as context auto-compacts and you can continue indefinitely, until it is each and every requirement fully addressed, and everything is totally done.
