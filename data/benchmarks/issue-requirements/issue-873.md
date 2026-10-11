More dynamic behavior, less static.
More generalization, less specialization.

More reliance on external world data (internet, local computer and so on), less memorization. Everything that can be recomputed, rediscovered can be easily removed from memory to free space, yet if needed again can be collected again.

Full learning cycle means - we have versioned memory with history of changes - each state of memory should be recoverable, so if compilation of next version of itself fails for formal AI debugging continues from previous stable and tested version. Once next stable version is working, it can continue working.

Most of the tests must be immutable, so we always make sure the initial baseline capable for learning and doing tasks, coding and so on is available, if new version does not pass all the tests we don't switch to it.

We should fully automate recovery from any error.

Error just means we didn't plan for that function, if there are multiple options how to resolve the error we should put user to driving seat and ask him to resolve it. Yet it can be configurable and user can turn on full trust for the system, so based on all previous tasks we weight all options (their advantages and disadvantages) and select automatically which is the best fit for most cases we care about.

Doing so it should be impossible for the system to get stuck and fail. Yet we should put a limit on it, so it will not loop forever, if something is not solvable in 1 hour (default, can be configured), we stop provide plan to the user at its current state and ask permission to continue (as it takes lots of time). That also configurable, as we should support both full autonomous mode as well as permission for each command.

We need update previous architecture, so all logic of our Formal AI is a single general meta algorithm that can append/generalize itself.

We need to collect data related about the issue to this repository, make sure we compile that data to `./docs/case-studies/issue-{id}` folder, and use it to do deep case study analysis (also make sure to search online for additional facts and data), list of each and all requirements from the issue, and propose possible solutions and solution plans for each requirement (we should also check known existing components/libraries, that solve similar problem or can help in solutions).

Please plan and execute everything in this single pull request, you have unlimited time and context, as context auto-compacts and you can continue indefinitely, until it is each and every requirement fully addressed, and everything is totally done.

