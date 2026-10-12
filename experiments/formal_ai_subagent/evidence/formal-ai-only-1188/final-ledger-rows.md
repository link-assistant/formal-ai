# Formal AI only agent ledger for PR #1188

Recovered from original transcripts. All rows are FORMAL-AI-ONLY. T972, T989 and T1115 were unused. T960 has no saved transcript; T974 comprises 32 actual leaf-driver invocations. Original T1116/T1117 prompt artifacts were lost; this reconstructed delivery is not an unchanged-task retry.

| ID | Task | Result | After |
| --- | --- | --- | --- |
| T960 | Read gaps.md and plan general G108-G111 repairs, with functions and regressions; no edits | Fail: read and echoed gaps.md; no repair plan | Open G112; feature synthesis missing |
| T961 | Fix G108 in sandbox code_task.mjs, preserve scalar comma/backtick payloads, add regression | Fail: read and echoed gaps.md; source unchanged | Open G112; parent later supplied literals |
| T962 | Create r.md with exact quoted x and terminal newline | Fail: retained quote opener and dropped newline | G113 fixed by T1103 and regressions |
| T963 | Replace x with scalar sentence containing backticked prompt:, impulse:, candidate: | Partial: exact edit; answer split scalar at comma/backtick joiner | G108 fixed by T990-T999 |
| T964 | Create a.lino with exact quoted x m and terminal newline | Fail: retained opener and dropped newline | G113 fixed by T1103 |
| T965 | Copy a.lino to b.lino, then replace x/m with y/n | Fail: read nonexistent b.lino; no copy | G109 fixed by T976-T984 and T981/T982 |
| T966 | Create budget.lino whose quoted content mentions a Rust path | Fail: selected payload path and refused; budget absent | G110 fixed by T1102 |
| T967 | Create multiline rows.lino with initial indentation and terminal newline | Fail: removed indentation and terminal newline | G113 fixed by T1103 |
| T968 | Replace intended multiline seed after T967 | Pass: absent original bytes safely refused; no mutation | Fixture normalization belongs to G113 |
| T969 | Retry intended seed without terminal newline | Pass: absent original indentation safely refused | Fixture normalization belongs to G113 |
| T970 | Replace actual two-line seed with three-line seed | Pass: exact edit; answer displayed every line of both payloads | G111 not reproduced; no repair claimed |
| T971 | Feature-level sequenceSteps request for copy/move then edits | Fail: explanatory prose parsed as replacement; old absent | Open G112 synthesis misparse |
| T973 | Append G112/G113 findings to gaps.md through JavaScript Formal AI | Pass: exact two Markdown lines added | No further change needed |
| T974 | Replay all 32 Issue 1028 full leaf prompts via JS drive in isolated source-file copies | Pass: 31/32 source-effect marker, guard and changed-byte checks; floor 16 met | Proxy only; L07 failed; full CLI/native proof remains CI |
| T975 | Preserve several raw transcripts using a compound shell request | Fail: read and echoed transcripts; no copies | Open G112 compound planning; individual Copy works |
| T976 | Parent G109 JS sequence guard in scratch twin | Pass: one exact edit | No further change needed |
| T977 | Parent G109 Rust sequence guard in scratch twin | Pass: one exact edit | No further change needed |
| T978 | Apply G109 JS sequence guard | Pass: one exact repository edit | No further change needed |
| T979 | Apply G109 Rust sequence guard | Pass: one exact repository edit | Sibling qualification/layout by T983/T984 |
| T980 | Create exact source.lino x m fixture | Pass: plan and fixture written | No further change needed |
| T981 | Copy then two joined replacements after guard repair | Pass: source unchanged; destination y n | G109 copy verified |
| T982 | Move then two joined replacements | Pass: original destination removed; moved file z o | G109 move verified |
| T983 | Qualify Rust sibling helper and parent layout | Pass: one exact edit | Final formatter wrap by T984 |
| T984 | Apply Rust sequence formatter wrap | Pass: one exact edit | No further change needed |
| T985 | Copy T971 transcript into repository .log evidence | Pass: semantic cp preserved source | Ignored .log replaced by tracked .txt in T986 |
| T986 | Eight individual semantic transcript Copies | Pass: eight exact tracked .txt copies | No further change needed |
| T987 | Four evidence-reference replacements | Pass: entire partial edit refused; two old strings absent | Agent precondition mistake; no G52 regression |
| T988 | Retry four evidence references separately | Pass: two existing references updated; two absent refused | Bare references completed by T1108 |
| T990 | Parent G108 scalar/list JS helper and caller | Pass: three exact edits | No further change needed |
| T991 | Parent G108 Rust scalar/helper/caller | Partial: two edits succeeded; helper leaked into websearch | G114 fixed by T997-T999 |
| T992 | Fenced helper insertion before quoted anchor | Fail: unmatched-backtick refusal; unchanged | Open G112 authoring; whole-block T999 worked |
| T993 | Helper insertion by short signature replacement | Fail: quoted task: leaked into websearch | G114 fixed by T997-T999 |
| T994 | Create fenced helper scratch payload | Fail: unmatched-backtick refusal; no file | Open G112 authoring |
| T995 | Whole Rust function replacement with helper | Fail: quoted task: leaked into invalid grep | G114 fixed; identical T999 succeeded |
| T996 | Parent JS harness cp/mv/test-presence extension | Partial: edit decoded new-only newline escape and made JS invalid | G115 fixed by T1104/T1105; FA envelope repair T1100 |
| T997 | Quote-aware JS objective guard/import | Pass: two exact edits | G114 JS fixed |
| T998 | Quote-aware Rust objective guard | Pass: one exact edit | G114 Rust fixed |
| T999 | Identical whole-function helper retry after objective fix | Pass: one exact edit | G114 verified; G108 Rust helper complete |
| T1100 | Parent character-code joined error envelope | Pass: one exact edit; JS syntax restored | FA self-repair of T996; no manual source repair |
| T1101 | Parent Rust helper/objective formatter hunks | Pass: two exact edits | No further change needed |
| T1102 | G110 request token metadata and quoted-cue filtering | Pass: nine exact JS/Rust edits | Payload paths cannot become creation targets |
| T1103 | G113 quoted bytes and closing-quote statement boundary | Pass: four exact JS/Rust edits | Indentation and terminal newline preserved |
| T1104 | G115 JS new-only escape discrimination | Pass: two exact edits | Raw new source bytes preserved |
| T1105 | G115 Rust new-only escape discrimination | Pass: two exact edits | Rust twin repaired |
| T1106 | Parent G108/G109/G110/G113 JS regressions | Pass: one exact edit; 22 closest tests passed | No further change needed |
| T1107 | Parent G114/G115 JS regressions | Pass: one exact edit; 27 gap/quoted-content tests passed | No further change needed |
| T1108 | Gap statuses and remaining evidence references | Pass: seven exact documentation edits | G111 not reproduced; G112 open |
| T1109 | Six parent native gap regressions | Pass: one exact edit; formatter parsed | Native compile assigned to CI; not run locally |
| T1110 | Formatter-generated hunks on owned Rust files | Pass: 15 exact edits; seven owned Rust files format clean | No manual formatting edits |
| T1111 | Append G114/G115 fixed reports | Pass: one exact documentation edit | No further change needed |
| T1112 | Preserve original T996 transcript | Pass: exact semantic Copy into tracked .txt | No further change needed |
| T1113 | Parent concise Rust docs and JS test EOF cleanup | Pass: three exact edits; Rust file 997 lines | No further change needed |
| T1114 | Create parent changelog | Pass: plan and exact changelog written | No further change needed |
| T1116 | Create complete Markdown ledger using fenced payload | Fail: T981/T982 payload reference became missing file | Original failure retained; exact task artifact lost |
| T1117 | Retry complete ledger as quoted payload | Fail: No files found; no artifact | Original failure retained; exact task artifact lost |
| T1118 | Create recovered complete ledger using closed four-backtick literal | Pass: exact 55-row 8595-byte artifact; request and transcript retained | Reconstructed delivery only; original T1116/T1117 asks lost |
| T1119 | Ask FA for G125 repair plan after reading planner source | Fail: read and echoed source; no repair plan | G112 stays open; parent supplied literal guard |
| T1120 | Ask FA for family corpus identity correction plan | Fail: read and echoed en.lino; no plan | G112 stays open; canonical data substitutions reviewed |
| T1121 | Apply parent G125 composed-edit precedence guard in planner twins | Pass: three exact edits | Integrated with CI shared planner commit 95efef844 |
| T1122 | Copy reviewed production concise seed parser into frozen source-test reader | Pass: one exact whole-file replacement; production-byte equality | Source fixture semantics restored; native execution remains CI |
| T1123 | Disambiguate two metadata meaning identities using parent semantic IDs | Pass: two exact edits; global meaning definitions unique | Runtime subject, handler and intent names preserved |
| T1124 | Align family corpus expected fields and suite header with runtime names | Pass: five computed writes and one edit; 180 fields aligned; all300 prompts preserved | Native CI validation pending; separate JS family migration parity remains open |
| T1125 | Append general G125 JS and native regressions through FA | Partial: JS edit passed; native payload safely refused due unmatched nested transport mark | Native requested bytes unchanged until T1127; no manual self-repair |
| T1126 | Restore original document via FA in tiny sandbox and retry saved T1416 unchanged | Pass: exact old document created; identical original prompt produced exact intended document | Specific G125 fixed; root live document untouched; G112 synthesis open |
| T1127 | Retry same native G125 test source using a distinct transport quote pair | Pass: one exact native test edit | Genuine skill guard, literal data and malformed no-op behavior pinned |
| T1128 | Copy two changed seed documents into package embedded mirrors | Pass: two explicit cp-f calls; mirrors byte-equal | No Rust generator executed |
| T1129 | Apply formatter output to owned native G125 test | Pass: one exact formatter edit | No manual formatter repair |
| T1130 | Ask FA to plan general typed calendar-day boundary repair | Fail: requested expected function inputs/results; no source read or repair plan | G112 stays open; parent supplied reviewed boundary mechanism |
| T1131 | Apply token-boundary day evidence in shared capability router twins | Pass: two computed writes; four role checks changed | Wikipedia/India suffixes no longer imply a calendar day |
| T1132 | Append JS/native calendar operand and original summary regressions | Pass: two exact test edits | Actual English/Spanish/CJK day references preserved; no title-specific branch |
| T1133 | Apply formatter output to owned native calendar regression | Pass: two exact formatter edits | All four owned Rust files format clean |
| T1134 | Create current CI/self-repair changelog through FA | Pass: plan and exact changelog written | Native compilation/full suites remain CI observations |
| T1135 | Preserve recovered rows and exact failure/retry request/transcript evidence | Pass: ten semantic Copies; all tracked bytes match original artifacts | Raw initial failures retained; no manual source write |
| T1136 | Create this complete final ledger through FA closed literal | Pass: complete initial and follow-up row payload byte-verified | Exact request and transcript retained in scratch |

Counts for all invoked IDs through T1136:230 successful authored-file mutations (95 edits,98 writes,23 copies,1 move,13 perl substitutions), with22 mkdir attempts and one failed edit attempt. Repository mutations115; isolated scratch mutations115. Six owned TypeScript twins from the initial batch were generated additionally; current TypeScript regeneration belongs to the coordinator. These counts exclude observational log/transport writes, test fixture activity and Git operations.

Manual implementation source writes and manual source self-repair:zero. Independently prepared36 initial scratch copies were operational inputs, never copied back as implementation. Parent supplied general source repairs; after FA repair planning failed, the latest authorization also allowed general regression design. All physical source/test mutations were executed by FA; no independent requirement-specific feature implementation is claimed.

Validation:28 closest routing/literal/path/workspace-listing JS checks plus5 concise checks passed; metadata audit, seed registry, global meaning uniqueness, prompt preservation, package mirror byte equality, owned Rust formatting and diff whitespace passed. Exact T1416 retry succeeded in a selected-file sandbox. Native compilation and full CLI/suites remain CI. Full300 JS family runtime probe still returns old specialized intents rather than migrated family identities, so that parity debt remains open.
