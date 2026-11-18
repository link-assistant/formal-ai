### Fixed

- Run the focused native PR check when its isolated regression tests, embedded
  fixtures, or HTTP replay inputs change. Replay the existing synthesis contracts
  in the same lane without compiling the complete unit test suite.
