Image of the telegram bot should be based on docker in docker using full box of https://github.com/link-foundation/box dind version as a base image for our telegram bot.

So as the result like in github.com/link-assistant/hive-mind we should have our docker image be built on top of box dind image (this will be the only supported image, we will not support image without dind at the moment).

We also should use https://github.com/link-foundation/start with launching coding tasks using `--isolation docker`, so the idea as that start command will spawn commands and will track their execution, meaning we will be able to get results as logs of execution.

That way our system will be able to actually test executed source code before providing it to the user.

And we also need to make sure our README in the root of repository is fully in sync with all the codebase and has instructions on how to start our system as telegram bot.

We need to collect data related about the issue to this repository, make sure we compile that data to `./docs/case-studies/issue-{id}` folder, and use it to do deep case study analysis (also make sure to search online for additional facts and data), list of each and all requirements from the issue, and propose possible solutions and solution plans for each requirement (we should also check known existing components/libraries, that solve similar problem or can help in solutions).

Please plan and execute everything in a single pull request, you have unlimited time and context, as context auto-compacts and you can continue indefinitely, until it is each and every requirement fully addressed, and everything is totally done.
