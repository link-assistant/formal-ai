# Specification coverage metric scope correction

The authoritative production checker observation is coordinator T2528:1173 native specification tests total,136 carried sessions executed and136 passed. This is the current production coverage metric.

The earlier scratch T2916 inventory obtained145 IDs from `specificationCases(root).cases`. That number measures parser/candidate inventory, not successful production carried-session execution. Older scratch summary fields/descriptions that called145 "current production carried" were mislabeled. The original raw inventory and request/output remain immutable evidence; this correction changes their interpretation, not their bytes or assertions.

Scratch T2916 separately parsed193 cases, blocked164 unsupported runtime identities, and genuinely executed23 new default/history sessions:21 passed,2 failed. T2928's25/25 checks cover14 scratch-reader tests plus11 source-backed compiled-skill/dispatch prototype tests. Those results do not update the production checker, its gap record, or the136/136 count. The unmodified production worker still fails the unchanged compiled-skill replay session; the genuine selected in-memory overlay passes all5 assertions.

Production compiler and typed-reader delivery remains a subsequent authorized source phase. It must rerun the actual production checker and retain native assertions, exact runtime/fixture identity and original failures. No coverage closure or production count increase is inferred from scratch candidate eligibility.
