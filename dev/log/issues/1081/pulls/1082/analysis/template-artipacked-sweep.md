# `artipacked` sweep across the five pipeline templates

Measured with zizmor 1.29.0 at the template commits in
`../references/templates/*.HEAD`. Two passes per template:

    # what the template's own gate reports
    zizmor --offline --config .github/zizmor.yml \
        --persona regular --min-confidence medium .github/workflows

    # what exists, with nothing hidden by configuration
    zizmor --offline --config .github/zizmor.yml \
        --persona auditor .github/workflows

| template | checkouts | `persist-credentials: false` | artipacked (auditor) | artipacked (own gate) | confidences |
|---|---|---|---|---|---|
| rust | 26 | 26 | 2 | 0 | all Low |
| js | 27 | 2 | 25 | 0 | all Low |
| python | 18 | 18 | 1 | 0 | all Low |
| php | 11 | 2 | 9 | 0 | all Low |
| csharp | 13 | 1 | 12 | 0 | all Low |

Every finding is Low confidence, and every template's gate floors confidence at
`medium`, so no template's configured pass can report any of them. rust's 2 and
python's 1 are the release jobs that push, each carrying an explicit `token:`
or `persist-credentials: true` with a comment naming the push -- deliberate
exceptions, not unswept checkouts. js, php and csharp never set the input at
all.

Per-finding locations follow.

=== rust ===
  .github/workflows/release.yml    auto-release             L690   uses: actions/checkout@v6         with:           fetch-depth: 0      
  .github/workflows/release.yml    manual-release           L853   uses: actions/checkout@v6         with:           fetch-depth: 0      
  count: 2
=== js ===
  .github/workflows/example-app.yml android-build            L160   uses: actions/checkout@v6
  .github/workflows/example-app.yml desktop-package          L114   uses: actions/checkout@v6
  .github/workflows/example-app.yml ios-build                L205   uses: actions/checkout@v6
  .github/workflows/example-app.yml preview-regen            L247   uses: actions/checkout@v6         with:           # Regenerate against
  .github/workflows/example-app.yml web-build                L44    uses: actions/checkout@v6
  .github/workflows/links.yml      link-checker             L43    uses: actions/checkout@v6
  .github/workflows/release.yml    changeset-check          L158   uses: actions/checkout@v6         with:           fetch-depth: 0
  .github/workflows/release.yml    changeset-pr             L802   uses: actions/checkout@v6         with:           fetch-depth: 0
  .github/workflows/release.yml    check-file-line-limits   L107   uses: actions/checkout@v6         with:           fetch-depth: 0
  .github/workflows/release.yml    detect-changes           L64    uses: actions/checkout@v6         with:           fetch-depth: 0
  .github/workflows/release.yml    docker-build             L371   uses: actions/checkout@v6
  .github/workflows/release.yml    docker-publish           L748   uses: actions/checkout@v6
  .github/workflows/release.yml    docker-publish-build     L704   uses: actions/checkout@v6
  .github/workflows/release.yml    docker-publish-config    L667   uses: actions/checkout@v6
  .github/workflows/release.yml    instant-release          L580   uses: actions/checkout@v6         with:           fetch-depth: 0
  .github/workflows/release.yml    lint                     L212   uses: actions/checkout@v6         with:           # For PRs, fetch eno
  .github/workflows/release.yml    pipeline-status          L879   uses: actions/checkout@v6
  .github/workflows/release.yml    release                  L473   uses: actions/checkout@v6         with:           fetch-depth: 0
  .github/workflows/release.yml    test                     L291   uses: actions/checkout@v6         with:           # For PRs, fetch eno
  .github/workflows/release.yml    test-compilation         L88    uses: actions/checkout@v6
  .github/workflows/release.yml    validate-docs            L412   uses: actions/checkout@v6        # Documentation line limits (1500 lin
  .github/workflows/release.yml    version-check            L135   uses: actions/checkout@v6         with:           fetch-depth: 0
  .github/workflows/security.yml   codeql                   L40    uses: actions/checkout@v6
  .github/workflows/security.yml   dependency-review        L65    uses: actions/checkout@v6
  .github/workflows/security.yml   npm-audit                L85    uses: actions/checkout@v6
  count: 25
=== python ===
  .github/workflows/release.yml    manual-release           L637   uses: actions/checkout@v6         with:           persist-credentials:
  count: 1
=== php ===
  .github/workflows/docs.yml       build                    L41    uses: actions/checkout@v4
  .github/workflows/links.yml      link-checker             L29    uses: actions/checkout@v4
  .github/workflows/release.yml    auto-release             L236   uses: actions/checkout@v4         with:           fetch-depth: 0      
  .github/workflows/release.yml    build                    L203   uses: actions/checkout@v4
  .github/workflows/release.yml    changeset                L160   uses: actions/checkout@v4         with:           fetch-depth: 0
  .github/workflows/release.yml    detect-changes           L51    uses: actions/checkout@v4         with:           fetch-depth: 0
  .github/workflows/release.yml    lint                     L89    uses: actions/checkout@v4
  .github/workflows/release.yml    manual-release           L298   uses: actions/checkout@v4         with:           fetch-depth: 0      
  .github/workflows/release.yml    test                     L136   uses: actions/checkout@v4
  count: 9
=== csharp ===
  .github/workflows/docs.yml       build                    L49    uses: actions/checkout@v6
  .github/workflows/links.yml      link-checker             L43    uses: actions/checkout@v6
  .github/workflows/release.yml    build                    L293   uses: actions/checkout@v6
  .github/workflows/release.yml    changeset-check          L100   uses: actions/checkout@v6         with:           fetch-depth: 0
  .github/workflows/release.yml    changeset-pr             L730   uses: actions/checkout@v6         with:           fetch-depth: 0
  .github/workflows/release.yml    detect-changes           L54    uses: actions/checkout@v6         with:           fetch-depth: 0
  .github/workflows/release.yml    instant-release          L577   uses: actions/checkout@v6         with:           fetch-depth: 0      
  .github/workflows/release.yml    lint                     L165   uses: actions/checkout@v6
  .github/workflows/release.yml    release                  L349   uses: actions/checkout@v6         with:           fetch-depth: 0
  .github/workflows/release.yml    test                     L235   uses: actions/checkout@v6
  .github/workflows/security.yml   codeql                   L29    uses: actions/checkout@v6
  .github/workflows/security.yml   dependency-review        L45    uses: actions/checkout@v6
  count: 12
