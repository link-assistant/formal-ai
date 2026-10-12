Instead of long instructions at the end on how to upload memory to gist or as zip archive to issue, we need to have docs in repository, so we can give only single link for user if he wants to upload the full memory.

Also we can do dialog recording shorter by using single code block with:

where A = agent, and U = user
```
U: Hi
A: Hello
U: 1+2
A: 3
```

So it can be shorter, and we will get less of error:

<img width="1080" height="2340" alt="Image" src="https://github.com/user-attachments/assets/6ac65bdb-8d68-4d38-b9a2-9f75e04add95" />

<img width="1280" height="218" alt="Image" src="https://github.com/user-attachments/assets/b178e722-727e-4a7f-871e-4e974ab5d2af" />

We need to download all logs and data related about the issue to this repository, make sure we compile that data to `./docs/case-studies/issue-{id}` folder, and use it to do deep case study analysis (also make sure to search online for additional facts and data), in which we will reconstruct timeline/sequence of events, list of each and all requirements from the issue, find root causes of the each problem, and propose possible solutions and solution plans for each requirement (we should also check known existing components/libraries, that solve similar problem or can help in solutions).

If there is not enough data to find actual root cause, add debug output and verbose mode if not present, that will allow us to find root cause on next iteration.

If issue related to any other repository/project, where we can report issues on GitHub, please do so. Each issue must contain reproducible examples, workarounds and suggestions for fix the issue in code.

Please plan and execute everything in this single pull request, you have unlimited time and context, as context auto-compacts and you can continue indefinitely, until it is each and every requirement fully addressed, and everything is totally done.
