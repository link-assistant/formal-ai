# macOS job durations on `main` (release.yml), ten runs before the fix

| Run | Build macOS test archive | Test (macos-15-intel / specification) |
| --- | --- | --- |
| 34095902681 | 19m success | 26m **failure** (1400 s budget, 93.68% sccache hits) |
| 34061511110 | 13m success | 18m success |
| 33994570842 | 25m **failure** (1400 s budget) | 17m success |
| 33955786226 | 18m success | 15m success |
| 33902725025 | 26m success | 17m success |
| 33310631522 | 11m success | 11m success |
| 33189772848 | 17m success | 22m success |
| 32740803291 | 19m success | 13m success |

Identical work varies 2.4x between runners. #1082 split the specification leg
into a 1800 s build and a 660 s run; this pull request gives the archive build
the same 1800 s under a 55 m cap (1800 + 400 = 2200 s, 66.7%).
