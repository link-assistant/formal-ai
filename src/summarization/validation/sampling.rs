//! The seeded, stratified draw: which files a run validates, in which order.
//!
//! Split out of [`super`] to keep each file inside the repository's own
//! thousand-line ceiling; the protocol is unchanged.

use super::carries_embedded_grammar;
use super::{
    CorpusFile, DEFAULT_FILES_PER_ITERATION, DEFAULT_MAX_ITERATIONS, DEFAULT_MINIMUM_ITERATIONS,
    DEFAULT_SAMPLING_SEED, DEFAULT_STABILITY_TOLERANCE_PERCENT, DEFAULT_STABILITY_WINDOW,
};

/// The reproducible sampling protocol: which files are drawn, how many per
/// iteration, and when the loop is allowed to stop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SamplingProtocol {
    pub seed: u64,
    pub files_per_iteration: usize,
    pub max_iterations: usize,
    pub minimum_iterations: usize,
    pub stability_window: usize,
    pub stability_tolerance_percent: u32,
}

impl Default for SamplingProtocol {
    fn default() -> Self {
        Self {
            seed: DEFAULT_SAMPLING_SEED,
            files_per_iteration: DEFAULT_FILES_PER_ITERATION,
            max_iterations: DEFAULT_MAX_ITERATIONS,
            minimum_iterations: DEFAULT_MINIMUM_ITERATIONS,
            stability_window: DEFAULT_STABILITY_WINDOW,
            stability_tolerance_percent: DEFAULT_STABILITY_TOLERANCE_PERCENT,
        }
    }
}

impl SamplingProtocol {
    /// Builder helper pinning the seed.
    #[must_use]
    pub const fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Builder helper pinning the iteration bound.
    #[must_use]
    pub const fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    /// Deterministic sampling order over `paths`.
    ///
    /// The corpus is sorted first, so a caller's directory-walk order cannot
    /// change the draw, then permuted with a seeded Fisher-Yates shuffle. The
    /// result is the sampling plan: iteration `i` validates the slice
    /// `[i * files_per_iteration, (i + 1) * files_per_iteration)`.
    #[must_use]
    pub fn sampling_order<'corpus>(&self, paths: &[&'corpus str]) -> Vec<&'corpus str> {
        let mut ordered: Vec<&str> = paths.to_vec();
        ordered.sort_unstable();
        ordered.dedup();
        let mut prng = Prng::seeded(self.seed);
        // Fisher-Yates from the back, the standard unbiased shuffle.
        for index in (1..ordered.len()).rev() {
            let swap = prng.below(index + 1);
            ordered.swap(index, swap);
        }
        ordered
    }

    /// Deterministic sampling order stratified over the recursive case.
    ///
    /// [`Self::sampling_order`] is a uniform permutation, which is the right
    /// draw for "files nobody optimized for" but the wrong one for a
    /// requirement that a *particular kind* of file be exercised. Markdown
    /// files carrying fenced blocks are a small minority of this repository, so
    /// a uniform draw bounded at `max_iterations * files_per_iteration` files
    /// can miss every one of them — that is not a hypothetical, it failed a CI
    /// run at 100% measured quality (see `docs/case-studies/issue-893/`).
    ///
    /// So the draw is stratified rather than enlarged: the seeded permutation
    /// is computed exactly as before, then the first entry that carries an
    /// embedded grammar is promoted to the front. Every other file keeps its
    /// seeded position, the result is still a permutation of the same corpus,
    /// and it is still a pure function of the seed and the file set — but
    /// iteration 0 now always reaches the recursive case, on any corpus that
    /// contains one.
    #[must_use]
    pub fn stratified_sampling_order<'corpus>(
        &self,
        corpus: &'corpus [CorpusFile],
    ) -> Vec<&'corpus str> {
        let paths: Vec<&str> = corpus.iter().map(|file| file.path.as_str()).collect();
        let mut ordered = self.sampling_order(&paths);
        let promote = ordered.iter().position(|path| {
            corpus
                .iter()
                .find(|file| file.path == *path)
                .is_some_and(|file| carries_embedded_grammar(&file.path, &file.content))
        });
        if let Some(index) = promote {
            let file = ordered.remove(index);
            ordered.insert(0, file);
        }
        ordered
    }

    /// Files drawn for one iteration, or an empty slice when the corpus is
    /// exhausted.
    #[must_use]
    pub fn iteration_paths<'corpus>(
        &self,
        paths: &[&'corpus str],
        iteration: usize,
    ) -> Vec<&'corpus str> {
        let ordered = self.sampling_order(paths);
        let start = iteration.saturating_mul(self.files_per_iteration);
        ordered
            .into_iter()
            .skip(start)
            .take(self.files_per_iteration)
            .collect()
    }

    /// How many iterations this corpus can supply under the bound.
    #[must_use]
    pub const fn available_iterations(&self, corpus_size: usize) -> usize {
        if self.files_per_iteration == 0 {
            return 0;
        }
        let supplied = corpus_size / self.files_per_iteration;
        if supplied < self.max_iterations {
            supplied
        } else {
            self.max_iterations
        }
    }
}

/// Deterministic `splitmix64` stream: the sampling protocol's reproducible
/// randomness. Same seed, same permutation, on every platform and every run.
struct Prng {
    state: u64,
}

impl Prng {
    const fn seeded(seed: u64) -> Self {
        Self {
            state: seed ^ 0x9e37_79b9_7f4a_7c15,
        }
    }

    const fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn below(&mut self, bound: usize) -> usize {
        if bound == 0 {
            0
        } else {
            usize::try_from(self.next_u64() % bound as u64).unwrap_or(0)
        }
    }
}
