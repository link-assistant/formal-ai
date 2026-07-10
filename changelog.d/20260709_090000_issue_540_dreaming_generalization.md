Issue #540 dreaming now learns and generalizes, not just garbage-collects. While
idle it recalculates which topics the user interacts with most, remembers the
durable requirements the user has stated on them so he never has to repeat
himself, and generalizes each requirement into a meta-algorithm amendment baked
into memory as retained, never-forgotten learning (`meta_algorithm_amendment`).
Because an amendment can reproduce the specific task/test-run records it covers,
those specifics are forgotten first under storage pressure (the new
`ForgetCoveredSpecific` action) while the generalization is kept forever. The
dreaming meta-algorithm is now recorded as grounded data in
`data/meta/dreaming-recipe.lino`, pinned to the live source by
`tests/unit/specification/dreaming_meta_algorithm.rs`.

The follow-up completes that loop: structured amendments are now read by future
chat and Responses requests; coverage requires exact replay; repeated task
structures and multilingual data cues feed learning; real filesystem pressure,
incoming bytes, and persisted consent govern minimal cleanup; and core plus
desktop workers run only while idle and yield to foreground work. A complete
Formal AI Agent CLI gap-audit session is preserved with the issue case study.
