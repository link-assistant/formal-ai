---
bump: minor
---

### Changed
- The self-development release gate now counts a merged pull request for the work Formal AI did in it, instead of demanding that *every* commit it introduced carry the attribution trailers. The old rule measured the composition of a pull request rather than the authorship of the work, and it had one practical consequence: a self-authored change could never ride along inside ordinary review, because a single human commit beside it erased it. Every claim the trailers make is still enforced — valid session evidence, an evidence path present in the commit, and no attributed commit naming a pull request other than the one that introduced it — and the measured share is unchanged, because it is computed per commit: an unattributed commit stays in the denominator and out of the numerator either way (issue #1069).
