Fixed the native planner enum wrapper and strict-warning condition identified by genuine PR CI. Source-member move and copy requests now require source-aware changes; whole-file path operations keep their existing behavior. Preserved the original memory byte roundtrip tests through a physical test module and corrected source-capture meanings to use the declared plural words field.

All 34 closest JavaScript controls and twenty source and projection checks pass. Fresh native CI and production release validation remain required.
