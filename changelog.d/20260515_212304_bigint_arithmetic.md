### Fixed
- Large integer multiplication no longer returns an overflow error; integer-only expressions now use arbitrary-precision arithmetic so results like `123123980921093128 * 2348023048230429324 * …` are exact (issue #55).
