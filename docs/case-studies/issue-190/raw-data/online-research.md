# Online Research

## GitHub Issue Query Parameters

Source: <https://docs.github.com/en/github/managing-your-work-on-github/about-automation-for-issues-and-pull-requests-with-query-parameters>

The demo uses a generated GitHub issue URL with `title`, `body`, and `labels` query parameters. GitHub's official documentation confirms this prefill pattern for issue and pull-request creation pages.

Design implication for issue 190: the report-link generator should continue using query parameters, but it must keep the URL below the practical browser/GitHub limit. The fitter therefore preserves the last two messages first, adds earlier context only while budget remains, and truncates the boundary message instead of dropping all earlier dialog.
