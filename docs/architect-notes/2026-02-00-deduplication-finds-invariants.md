# Deduplication is how invariants are found

**Date:** 2026-02 (exact day not recorded) · **Source:** issue #531

> Инвариант это вид повторения, как и любая закономерность, нужно повторы
> искать. Без дедупликации тут ничего и никогда не сделать. Потому что говорят
> что интеллект это и есть алгоритм сжатия.

> Сам по себе повтор это ситуация, строго определённая в контексте связей, это
> когда множество связей используют одну и ту же связь.

Restated in the architect's own English in
[the 2026-09-11 note](2026-09-11-the-goal-is-the-meta-algorithm.md): a repeated
operation in a recorded sequence is a loop or recursion, an input-dependent
branch is an `if` or `match`, and enough similar sequences deduplicate into an
algorithm.
