# Exact-session correction evidence

This directory preserves a real Formal AI controller → Agent CLI → Formal AI
chain for the maintainer follow-up on
[PR #876](https://github.com/link-assistant/formal-ai/pull/876).

All three canonical controller sessions name the same native Agent CLI session:
`ses_04e25ba4cffeibfMekv188DNLX`.

| Turn | Canonical evidence | Observed result |
| --- | --- | --- |
| Initial run | [`controller-session.json`](controller-session.json) | Agent CLI returned success, but Formal AI's answer created no requested file. This disproved the result. |
| First correction | [`corrected-session.json`](corrected-session.json) | The controller used `agent --resume … --no-fork` with the disproving evidence. The file was created, but the correction instructions contaminated its literal contents. |
| Final correction | [`final-session.json`](final-session.json) | The same native session was resumed again with the newly observed evidence. The file was reduced to the requested one-line invariant. |

The final artifact is
[`data/meta/orchestration-continuation-invariant.lino`](../../../../data/meta/orchestration-continuation-invariant.lino).
Its SHA-256 is
`1a94da7485b7f9c1e9bb0b17b745bfca30dbd8df583a8fd3c38f039c755bf0ee`.
The controller session digests form a parent chain:

```text
controller ea342dde4b4e3c9d6022d4b5ce2e05cdaa928f4af8562debd47081efcbf1fb4d
    ↓
corrected 028789b3e2e4cb2cc9942099c7c44e8aebc9eb102d5952e117823f4a5929f4a6
    ↓
final     172f1421de86a322de80cba3d0c74fb519dc09cc69d474c25c57d482e09eb9b2
```

The regression test replays the exact canonical bytes, verifies the two parent
digests and the unchanged native id, checks the outer Formal AI resume
arguments, checks Agent CLI's actual `--resume ID --no-fork -p` argv in its
captured stream, and pins the final artifact hash.

This evidence is intentionally honest about both failed attempts. It proves
that observed mistakes can become reviewable corrective feedback in the same
conversation; it does not claim that model agreement alone proves external
facts or that learning proposals promote themselves.
