# Online Research

## GitHub Issue Query Parameters

Source: <https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/creating-an-issue#creating-an-issue-from-a-url-query>

The demo uses a generated GitHub issue URL with `title`, `body`, and `labels` query parameters. GitHub's official documentation confirms this prefill pattern for issue creation pages and notes that oversized URLs can fail with `414 URI Too Long`.

Design implication for issue 190: the report-link generator should continue using query parameters, but it must keep the URL below the practical browser/GitHub limit. The fitter therefore preserves the last two messages first, adds earlier context only while budget remains, and truncates the boundary message instead of dropping all earlier dialog.
