In this case we should not use LLM model itself, only training data, if possible without fully downloading it, but just taking parts.

For example we should try to add 10 random samples, and make sure we can use it to build tests, that will increase quality of our system, by writing failing tests, and make sure our code base generalizes to solve them all.

We don't download full dataset, but take just enough + we need to have scripts to get more when needed.

Make sure to be ambitious, take into account our vision, requirements, roadmap (the latest have stronger weight), contributing guide lines, testing guidelines and so on.

Ideally this task itself must be fully and partially solvable by Formal AI connected to Agent CLI. You task to drive Formal AI to make all the actions, if something fails, you improve algorithms by generalization, so Formal AI have enough features to solve each part of this task, and also the task as the whole.

We need to collect data related about the issue to this repository, make sure we compile that data to `./docs/case-studies/issue-{id}` folder, and use it to do deep case study analysis (also make sure to search online for additional facts and data), list of each and all requirements from the issue, and propose possible solutions and solution plans for each requirement (we should also check known existing components/libraries, that solve similar problem or can help in solutions).

Please plan and execute everything in this single pull request, you have unlimited time and context, as context auto-compacts and you can continue indefinitely, until it is each and every requirement fully addressed, and everything is totally done.


