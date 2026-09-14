# The crates.io `/me` probe is a false positive

Measured 2026-09-08 from a workstation, no token involved:

```
$ curl -s -o /dev/null -w "%{http_code}\n" -A formal-ai-ci-check https://crates.io/api/v1/me
403   {"errors":[{"detail":"this action requires authentication"}]}
$ curl -s -o /dev/null -w "%{http_code}\n" -A formal-ai-ci-check -H "Authorization: cioAAAAbogus" https://crates.io/api/v1/me
403   {"errors":[{"detail":"authentication failed"}]}
```

crates.io source, `src/controllers/user/me.rs`, handler for `GET /api/v1/me`:
`AuthCheck::only_cookie()`. The endpoint accepts cookie sessions only; every API
token is answered 403 whether it is valid or not.

Run 34149311523 (push of `f971b8205`, the merge of #1082, 2026-09-07 17:51 UTC),
job `Release Preflight` (101828020707):

```
Release preflight (release mode): 1 verified, 1 failed, 0 unknown
  OK: GHCR push -- https://ghcr.io opened a blob upload session for link-assistant/formal-ai
##[error]BLOCKED: crates.io publish token -- crates.io rejected the token (HTTP 403); it is revoked, expired or misscoped
##[error]1 release credential(s) cannot publish; not spending a build on a release that cannot happen
```

The same secret had published `formal-ai 0.347.0` on 2026-09-05 (crates.io
`max_version` 0.347.0, `updated_at` 2026-09-05T09:22:48Z; run 33955786226 step
`Publish to Crates.io: success`).
