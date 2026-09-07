# Defect D3 gate proof -- zizmor 1.29.0 narrow pedantic pass

Command: zizmor --config .github/zizmor.yml --persona pedantic --min-severity high --min-confidence high .github/workflows .github/actions

## As committed (both protections in place)
```
No findings to report. Good job! (142 ignored, 22 suppressed)
EXIT=0
```

## With the digest pin reverted and the stock-image marker removed
```
error[unpinned-images]: unpinned image references
  --> .github/workflows/stock-rust-install.yml:28:5
   |
28 |     container: rust:1.98-slim-bookworm
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ container image is not pinned to a SHA256 hash
   |
   = note: audit confidence → High
   = help: audit documentation → https://docs.zizmor.sh/audits/#unpinned-images

error[unpinned-images]: unpinned image references
  --> .github/workflows/workflows.yml:75:24
   |
75 |       - uses: docker://rhysd/actionlint:1.7.12
   |                        ^^^^^^^^^^^^^^^^^^^^^^^ container image is not pinned to a SHA256 hash
   |
   = note: audit confidence → High
   = help: audit documentation → https://docs.zizmor.sh/audits/#unpinned-images

165 findings (141 ignored, 22 suppressed): 0 informational, 0 low, 0 medium, 2 high
EXIT=14
```
