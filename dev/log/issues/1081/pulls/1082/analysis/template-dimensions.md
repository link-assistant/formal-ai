| dimension                         | formal-ai | csharp | js | php | python | rust |
|-----------------------------------|-----------|--------|----|-----|--------|------|
| workflow files                    | 20        | 5      | 5  | 4   | 5      | 5    |
| jobs                              | 114       | 29     | 57 | 24  | 35     | 46   |
| jobs with timeout-minutes         | 51        | 14     | 28 | 12  | 20     | 28   |
| steps with timeout-minutes        | 32        | 0      | 2  | 0   | 2      | 0    |
| TEST_BUDGET_SECONDS uses          | 31        | 0      | 0  | 0   | 2      | 4    |
| budget-wrapper invocations        | 15        | 0      | 8  | 0   | 4      | 6    |
| checkout steps                    | 48        | 13     | 27 | 11  | 18     | 26   |
|   ... persist-credentials: false  | 50        | 1      | 2  | 2   | 18     | 26   |
| actions pinned by SHA             | 0         | 0      | 4  | 0   | 3      | 19   |
| actions pinned by tag             | 167       | 36     | 61 | 31  | 48     | 56   |
| docker:// image references        | 1         | 1      | 1  | 1   | 1      | 1    |
|   ... pinned by digest            | 1         | 0      | 0  | 0   | 0      | 0    |
| zizmor invocations                | 26        | 0      | 9  | 6   | 8      | 13   |
| ships .github/zizmor.yml          | yes       | no     | yes | yes | yes    | yes  |
| ships .github/dependabot.yml      | no        | no     | no | no  | no     | no   |
| terminal status job               | yes       | no     | yes | no  | yes    | yes  |
| concurrency groups                | 44        | 5      | 27 | 4   | 14     | 23   |
|   ... cancel-in-progress: true    | 2         | 3      | 10 | 2   | 5      | 6    |
