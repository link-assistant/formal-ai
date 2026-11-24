# Release artifact regression headroom

The maintained scanner catches the 1000-line release-publishing owner. Its two
artifact assertion bodies now live in an explicit child with ordinary imports.
Their original parent test names, order, assertions, fixture bytes, process
behavior and cleanup stay unchanged. The parent has 850 lines, its child 162;
the unchanged repository limits report 76 warnings and no violations.

The complete existing release-publishing owner, including its release-recovery
and selected-WASM-build children, is registered in the hosted workflow contract
target. This adds actual native coverage instead of relying on the JavaScript
source checks. Compilation and native execution still require hosted evidence.

Before extraction/registration, the focused suite has 11 successes and two
expected failures; afterward all 13 cases pass. All 50 native-runner controls,
18 actual selected-source-WASM JavaScript tests, 153 maintained issue cases,
three affected JavaScript gates, Rust 2024 formatting and whitespace checks
pass locally. No Rust build was run. These results do not establish full issue
acceptance or independent Formal AI authorship; supplied work receives zero
independent credit.

Evidence: `experiments/issue-1580/evidence/checks/release-artifact-contracts/`.
