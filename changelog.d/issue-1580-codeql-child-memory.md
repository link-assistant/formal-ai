Resolve the read-only CodeQL macro observer's memory policy against its actual
Rust dataset before execution. Keep the 4 GiB total budget while assigning heap
through the child evaluator's resolver, and fail visibly on runtime diagnostics
that otherwise survive a successful query exit.
