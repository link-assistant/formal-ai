We can start by generating questions using single word, and provide best possible answer to them.

After that all questions using 2 words and so on.

We can also use only 10% of most frequent word - we can take average from multiple known corpuses of texts for each language.

For 3 words and so on, we can take 5% of most frequent words.

For 4 we take 2.5% most frequent and so on.

We also should distinct between grammatically correct and incorrect.

And grammatically correct also should be split in logically meaningful.

We can also optimize our system for top asked questions: https://explodingtopics.com/blog/top-google-questions, and also find lots of similar sources, so we can merge them and re-rank, so we know that are the most popular questions.

We also need to support all variations of such questions.

So ideally we should have some function that produces infinite sequence of questions from smallest to largest, yet it should be possible to configure what counts as question or now, for example sequence of all grammatically and logically sound questions, for that we should exactly define, what these criteria actually mean.

We need to collect data related about the issue to this repository, make sure we compile that data to `./docs/case-studies/issue-{id}` folder, and use it to do deep case study analysis (also make sure to search online for additional facts and data), list of each and all requirements from the issue, and propose possible solutions and solution plans for each requirement (we should also check known existing components/libraries, that solve similar problem or can help in solutions).

Please plan and execute everything in this single pull request, you have unlimited time and context, as context auto-compacts and you can continue indefinitely, until it is each and every requirement fully addressed, and everything is totally done.




