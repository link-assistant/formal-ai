"""Distinguish code-generation targets from implementation roots in review evidence."""
from pathlib import Path
path=Path('docs/case-studies/issue-1582/README.md')
text=path.read_text().replace('The final 19 checks pass: twelve direct requests use actual automatic','The final 35 checks pass: twelve direct requests use actual automatic').replace('Spanish, six\nnegative cases exclude permission routing, and the generated TypeScript','Spanish, 22\nnegative cases exclude permission routing (including all 16 catalog code targets),\nand the generated TypeScript').replace('results are 19 permission checks','results are 35 permission checks').replace('**47 passed**','**63 passed**')
text+='''

The programming-language census includes code-generation targets as well as
implementation languages. All 16 catalog rows are covered by quoted-task
exclusions: Rust, Python, JavaScript, TypeScript, Go, C, C++, Java, C#, Ruby,
Scala, Kotlin, PHP, Swift, R and the Laravel framework target. The test compares
the complete fixture target list with both the maintained coding catalog and
the native `PROGRAM_LANGUAGES` declaration. JavaScript and the generated
TypeScript recognizer execute every case. This verifies permission-route
ownership; it does not compile or execute programs in those languages. The
shared native fixture includes the same requests but remains unexecuted under
the documented pre-merge policy. The initial census recipe omitted the required
agentic parser host bootstrap; its failed attempt is retained before the
corrected recipe that uses the established Node host bootstrap.
'''
path.write_text(text)
path=Path('docs/case-studies/pull-request-1583/README.md')
text=path.read_text().replace('languages and the JavaScript, TypeScript and Rust source roots.','languages, 16 code-generation targets and the JavaScript, TypeScript and Rust source roots.').replace('19 final JS/TS checks pass','35 final JS/TS checks pass').replace('passes 47 focused JS checks','passes 63 focused JS checks')
path.write_text(text)
path=Path('docs/requirements/issue-1582-client-read-access-and-context.md')
path.write_text(path.read_text().replace('including quoted-input exclusions;', 'including quoted-input exclusions for all 16 catalog programming targets;'))
path=Path('docs/requirements-traceability.md')
path.write_text(path.read_text().replace('automatic five-language permission guidance and quoted-input exclusions','automatic five-language permission guidance and quoted-input exclusions across all 16 programming targets'))
path=Path('experiments/issue_1582/continuation-plan.md')
path.write_text(path.read_text().replace('19 shared JS/TS\n  checks pass','35 shared JS/TS\n  checks pass, including all 16 catalog programming targets').replace('47 focused JS checks','63 focused JS checks'))
path=Path('/tmp/issue-1582-review/pull-request-body.md')
text=path.read_text().replace('all 19 final checks pass','all 35 final checks pass').replace('**47 focused JavaScript checks**','**63 focused JavaScript checks**').replace('five natural languages and three source roots','five natural languages, 16 catalog programming targets and three source roots')
text=text.replace('including execution of the generated TypeScript recognizer.', 'including execution of the generated TypeScript recognizer and quoted-task exclusions for every catalog programming target. These exclusions do not compile or execute generated programs.')
path.write_text(text)
