I think hardcoded intents are dead end, it will require infinite amount of coding.

Instead we should translate message to meta language and work on it directly, on conversion detect questions, requirements, needs, and so, and make sure we address all of the in our response. If it is a task with big plan, we need to make sure we use todo tool in agentic mode, to actually plan each item, and do step by step execution. In chat mode, we should give meaningful answer, using fresh data gathered in the internet.

So we need to merge all of our specific algorithms into single general meta algorithm, which later should be also reason about itself and should be able to modify itself as just another algorithm/codebase/repository.

But at the moment we need to make core architectural update, so that our algorithm is as general as possible. And all already supported test cases should be greatly expanded, we should support not only single specific task/question in each categories, we should support entire class of each tasks we previously encountered, entire class of questions. With end goal to move to support of entire class of task for all possible tasks, and for all possible questions.

So tests are not deleted, changes are additive (can be improved if are totally wrong, but we need to make sure we preserve backward compatibility in most of previously working dialogs).

Codebase architecture can be altered as needed.

Data seed can be increased in detail if required, but the architecture of caches, overrides, meanings and style of lino files should be preserved (or we can take the best practices).

We will do everything in this pull request, but in steps, the first working session should be dedicated to detailed planing of what we exactly will do, in next working sessions we will be executing this plan. After that I either approve the plan, or will ask to change it.

Also see

<img width="2732" height="2048" alt="Image" src="https://github.com/user-attachments/assets/4c536685-7221-4935-8788-b3e7a68080e5" />

for inspiration.

It genuinely believe with all the tools we have it is actually possible to formalize the general problem solving algorithm for any question, task, requirement and so on. This algorithm does not need to know everything, it can use the internet and public services as database or API, and just get to know enough knowledge to actually solve any task.

Read most of our docs, code and so on, make sure that current code architecture is well documented before you start planing changing it.

We need to collect data related about the issue to this repository, make sure we compile that data to `./docs/case-studies/issue-{id}` folder, and use it to do deep case study analysis (also make sure to search online for additional facts and data), list of each and all requirements from the issue, and propose possible solutions and solution plans for each requirement (we should also check known existing components/libraries, that solve similar problem or can help in solutions).


