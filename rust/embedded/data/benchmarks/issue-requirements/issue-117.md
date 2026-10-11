And also we should prefer nested example and multiline quoted strings as in https://github.com/link-foundation/lino-i18n project README.md, and we should add it as our dependency.

Also we should report any issues at https://github.com/link-foundation/lino-i18n if we don't have enough features to fully implement existing translation with new library.

We also need to double check, that all our features and translations in each single language we currently support also fully in all other languages we support. So all languages have the same features and translations supported at all times, we also should have CI/CD check or rule to enforce that for any future pull request, so it is impossible to merge if only single language version of any feature or translation is updated.

We need to collect data related about the issue to this repository, make sure we compile that data to `./docs/case-studies/issue-{id}` folder, and use it to do deep case study analysis (also make sure to search online for additional facts and data), list of each and all requirements from the issue, and propose possible solutions and solution plans for each requirement (we should also check known existing components/libraries, that solve similar problem or can help in solutions).

Please plan and execute everything in a single pull request, you have unlimited time and context, as context auto-compacts and you can continue indefinitely, until it is each and every requirement fully addressed, and everything is totally done.
