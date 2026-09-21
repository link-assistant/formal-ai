## Issue #1138 Formalization Depth

Unfamiliar requirement text is recursively grounded rather than preserved as
an assertion-shaped sentence. The contract is covered by
`rust/tests/unit/issue_1138_formalization_depth.rs`.

| ID | Requirement | Status / evidence |
| --- | --- | --- |
| R1138-B4-1 | Every ungrounded surface, relation, or procedure becomes an explicit need with source span and origin. | Implemented by the formalization graph; covered by `rust/tests/unit/issue_1138_formalization_depth.rs`. |
| R1138-B4-2 | A need is satisfied through the shared registry lookup, and the retrieved gloss is recursively formalized to bounded depth. | Implemented by the formalization-depth loop; covered by `rust/tests/unit/issue_1138_formalization_depth.rs`. |
| R1138-B4-3 | A grounded result is a sourced concept, predicate, entity, or procedure link, never a stored answer sentence. | Covered by `rust/tests/unit/issue_1138_formalization_depth.rs`. |
| R1138-B4-4 | Preserved text cannot satisfy an assertion primitive, and an unresolved need cannot be reported covered. | Covered by `rust/tests/unit/issue_1138_formalization_depth.rs`. |
| R1138-B4-5 | An extracted procedure enters memory only through bounded execution and named review with license metadata retained. | Covered by `rust/tests/unit/issue_1138_formalization_depth.rs`. |
| R1138-B4-6 | Equivalent English, Russian, Hindi, Chinese, and Spanish requirements produce one graph identity or a per-language gap. | Covered by `rust/tests/unit/issue_1138_formalization_depth.rs`. |
| R1138-B4-7 | Sentence segmentation is script-aware and every recorded span selects exactly its source text. | Covered by `rust/tests/unit/issue_1138_formalization_depth.rs`. |
| R1138-B4-8 | A custom agentic task is formalized instead of substituting a seeded example narrative. | Covered by `rust/tests/unit/issue_1138_formalization_depth.rs`. |
| R1138-B4-9 | One need type and status vocabulary serve the universal loop, coding path, and formalizer. | Covered by `rust/tests/unit/issue_1138_formalization_depth.rs`. |
| R1138-B4-10 | Offline capture replay reproduces the graph identity, and deleting derived graph memory loses no source evidence. | Covered by `rust/tests/unit/issue_1138_formalization_depth.rs`. |
