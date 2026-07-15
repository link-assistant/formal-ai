## Reproduction

The Python template still uses `actions/setup-python@v5` in `release.yml` and `docs.yml`, and `codecov/codecov-action@v4` in `release.yml`. Its latest audited release run emits the GitHub runner Node 20 deprecation warning:

https://github.com/link-foundation/python-ai-driven-development-pipeline-template/actions/runs/29434873566

Current releases are `actions/setup-python@v6` and `codecov/codecov-action@v7`.

## Suggested fix

Upgrade every setup-python occurrence to v6 and Codecov to v7, then add workflow policy tests that reject the old majors. This removes the runner warning instead of setting `ACTIONS_ALLOW_USE_UNSECURE_NODE_VERSION`.

Found while comparing all four pipeline templates for https://github.com/link-assistant/formal-ai/issues/717.
