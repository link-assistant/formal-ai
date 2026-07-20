//! Upstream benchmark manifests (issue #698).
//!
//! Every suite records the exact upstream revision, the permissive license it
//! ships under, and how a slice of it is fetched at run time. Payloads are
//! cached under `target/formal-ai-benchmarks` and are never vendored into the
//! repository, which keeps the `docs/benchmarks.md` "no vendored datasets"
//! policy while still executing real upstream cases.

use std::fmt::Write as _;

/// Where a suite's cases come from and in which wire format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuiteSource {
    /// One JSON object per line, optionally gzip-compressed upstream.
    JsonLines {
        url: &'static str,
        cache_file: &'static str,
        gzip: bool,
    },
    /// A BIG-bench `task.json` document with an `examples` array.
    BigBenchTask {
        url: &'static str,
        cache_file: &'static str,
    },
    /// The Hugging Face datasets-server `rows` API, used for datasets that are
    /// published only as parquet (which this crate cannot decode).
    DatasetsServerRows {
        dataset: &'static str,
        config: &'static str,
        split: &'static str,
        cache_file: &'static str,
    },
    /// No payload is reachable under the permissive-only policy.
    Unavailable,
}

/// Whether a suite can be executed at all, and why not when it cannot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Availability {
    Runnable,
    /// Recorded as a `benchmark_unavailable` ledger row instead of being
    /// silently replaced by a repository-local proxy.
    Unavailable {
        reason: &'static str,
    },
}

/// How a produced answer is graded against the upstream expectation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grading {
    /// Run the upstream unit test against the produced Python code.
    PythonUnitTest,
    /// Run the upstream `assert` list against the produced Python code.
    PythonAsserts,
    /// Compare the final number in the answer with the upstream gold number.
    NumericAnswer,
    /// Compare the final `\boxed{...}` (or last line) with the gold answer.
    BoxedAnswer,
    /// Compare the produced text with the gold edited text.
    ExactText,
    /// Compare the produced unified diff with the gold patch.
    UnifiedDiff,
    /// Nothing to grade: the suite cannot run.
    NotApplicable,
}

impl Grading {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PythonUnitTest => "python_unit_test",
            Self::PythonAsserts => "python_asserts",
            Self::NumericAnswer => "numeric_answer",
            Self::BoxedAnswer => "boxed_answer",
            Self::ExactText => "exact_text",
            Self::UnifiedDiff => "unified_diff",
            Self::NotApplicable => "not_applicable",
        }
    }

    #[must_use]
    pub const fn needs_python(self) -> bool {
        matches!(self, Self::PythonUnitTest | Self::PythonAsserts)
    }
}

/// A single upstream suite this harness knows how to fetch, run, and score.
#[derive(Debug, Clone)]
pub struct SuiteManifest {
    pub id: &'static str,
    pub title: &'static str,
    pub task_family: &'static str,
    pub license: &'static str,
    pub license_url: &'static str,
    pub source_url: &'static str,
    /// The exact upstream revision (git sha or dataset sha) the slice is taken
    /// from, so a rerun fetches the same cases.
    pub source_ref: &'static str,
    pub source: SuiteSource,
    pub grading: Grading,
    pub availability: Availability,
}

impl SuiteManifest {
    #[must_use]
    pub const fn is_runnable(&self) -> bool {
        matches!(self.availability, Availability::Runnable)
    }

    #[must_use]
    pub fn download_url(&self) -> Option<String> {
        match &self.source {
            SuiteSource::JsonLines { url, .. } | SuiteSource::BigBenchTask { url, .. } => {
                Some((*url).to_string())
            }
            SuiteSource::DatasetsServerRows {
                dataset,
                config,
                split,
                ..
            } => Some(format!(
                "https://datasets-server.huggingface.co/rows?dataset={}&config={config}&split={split}",
                encode_component(dataset)
            )),
            SuiteSource::Unavailable => None,
        }
    }

    #[must_use]
    pub const fn cache_file(&self) -> Option<&'static str> {
        match &self.source {
            SuiteSource::JsonLines { cache_file, .. }
            | SuiteSource::BigBenchTask { cache_file, .. }
            | SuiteSource::DatasetsServerRows { cache_file, .. } => Some(cache_file),
            SuiteSource::Unavailable => None,
        }
    }
}

/// Percent-encode the parts of a dataset id that appear in a query string.
#[must_use]
pub fn encode_component(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                let _ = write!(encoded, "%{byte:02X}");
            }
        }
    }
    encoded
}

/// Directory (relative to the repository root) that caches fetched payloads.
/// It lives under `target/`, so it is a build artifact and never committed.
pub const CACHE_DIR: &str = "target/formal-ai-benchmarks";

/// The committed ledger of honest upstream results.
pub const LEDGER_PATH: &str = "data/benchmarks/external-results.lino";

/// Licenses this harness is allowed to fetch (issue #698 requirement 5).
pub const PERMISSIVE_LICENSES: [&str; 3] = ["MIT", "Apache-2.0", "CC-BY-4.0"];

/// Every upstream suite, in the order issue #698 lists them.
pub const SUITES: &[SuiteManifest] = &[
    SuiteManifest {
        id: "humaneval",
        title: "HumanEval",
        task_family: "program_synthesis",
        license: "MIT",
        license_url: "https://raw.githubusercontent.com/openai/human-eval/6d43fb980f9fee3c892a914eda09951f772ad10d/LICENSE",
        source_url: "https://github.com/openai/human-eval",
        source_ref: "github:6d43fb980f9fee3c892a914eda09951f772ad10d",
        source: SuiteSource::JsonLines {
            url: "https://raw.githubusercontent.com/openai/human-eval/6d43fb980f9fee3c892a914eda09951f772ad10d/data/HumanEval.jsonl.gz",
            cache_file: "humaneval.jsonl",
            gzip: true,
        },
        grading: Grading::PythonUnitTest,
        availability: Availability::Runnable,
    },
    SuiteManifest {
        id: "mbpp",
        title: "Mostly Basic Python Problems (MBPP)",
        task_family: "program_synthesis",
        license: "Apache-2.0",
        license_url: "https://raw.githubusercontent.com/google-research/google-research/1fa17414f56c3703d5adb3818338b6e35e0fd550/LICENSE",
        source_url: "https://github.com/google-research/google-research/tree/master/mbpp",
        source_ref: "github:1fa17414f56c3703d5adb3818338b6e35e0fd550",
        source: SuiteSource::JsonLines {
            url: "https://raw.githubusercontent.com/google-research/google-research/1fa17414f56c3703d5adb3818338b6e35e0fd550/mbpp/mbpp.jsonl",
            cache_file: "mbpp.jsonl",
            gzip: false,
        },
        grading: Grading::PythonAsserts,
        availability: Availability::Runnable,
    },
    SuiteManifest {
        id: "gsm8k",
        title: "GSM8K",
        task_family: "math_word_problem",
        license: "MIT",
        license_url: "https://raw.githubusercontent.com/openai/grade-school-math/3101c7d5072418e28b9008a6636bde82a006892c/LICENSE",
        source_url: "https://github.com/openai/grade-school-math",
        source_ref: "github:3101c7d5072418e28b9008a6636bde82a006892c",
        source: SuiteSource::JsonLines {
            url: "https://raw.githubusercontent.com/openai/grade-school-math/3101c7d5072418e28b9008a6636bde82a006892c/grade_school_math/data/test.jsonl",
            cache_file: "gsm8k-test.jsonl",
            gzip: false,
        },
        grading: Grading::NumericAnswer,
        availability: Availability::Runnable,
    },
    SuiteManifest {
        id: "math",
        title: "MATH (500-problem split from `openai/prm800k`)",
        task_family: "competition_math",
        license: "MIT",
        license_url: "https://raw.githubusercontent.com/openai/prm800k/7ecc794703b2877f63226f2477a49b34f9b25163/LICENSE",
        source_url: "https://github.com/openai/prm800k",
        source_ref: "github:7ecc794703b2877f63226f2477a49b34f9b25163",
        source: SuiteSource::JsonLines {
            // `raw.githubusercontent.com` serves the Git LFS pointer for this
            // path; `media.githubusercontent.com` serves the payload itself.
            url: "https://media.githubusercontent.com/media/openai/prm800k/7ecc794703b2877f63226f2477a49b34f9b25163/prm800k/math_splits/test.jsonl",
            cache_file: "math-test.jsonl",
            gzip: false,
        },
        grading: Grading::BoxedAnswer,
        availability: Availability::Runnable,
    },
    SuiteManifest {
        id: "object_counting",
        title: "BIG-bench object_counting",
        task_family: "counting_reasoning",
        license: "Apache-2.0",
        license_url: "https://raw.githubusercontent.com/google/BIG-bench/092b196c1f8f14a54bbc62f24759d43bde46dd3b/LICENSE",
        source_url: "https://github.com/google/BIG-bench/tree/main/bigbench/benchmark_tasks/object_counting",
        source_ref: "github:092b196c1f8f14a54bbc62f24759d43bde46dd3b",
        source: SuiteSource::BigBenchTask {
            url: "https://raw.githubusercontent.com/google/BIG-bench/092b196c1f8f14a54bbc62f24759d43bde46dd3b/bigbench/benchmark_tasks/object_counting/task.json",
            cache_file: "object-counting-task.json",
        },
        grading: Grading::NumericAnswer,
        availability: Availability::Runnable,
    },
    SuiteManifest {
        id: "coedit",
        title: "CoEdIT",
        task_family: "instructed_text_editing",
        license: "Apache-2.0",
        license_url: "https://huggingface.co/datasets/grammarly/coedit/blob/main/README.md",
        source_url: "https://huggingface.co/datasets/grammarly/coedit",
        source_ref: "huggingface:e9a255c33ef910bc33a9d2b522653fa87521583e",
        source: SuiteSource::DatasetsServerRows {
            dataset: "grammarly/coedit",
            config: "default",
            split: "validation",
            cache_file: "coedit-validation.jsonl",
        },
        grading: Grading::ExactText,
        availability: Availability::Runnable,
    },
    SuiteManifest {
        id: "editeval",
        title: "EditEval",
        task_family: "instructed_text_editing",
        license: "CC0-1.0 (harness code only)",
        license_url: "https://raw.githubusercontent.com/facebookresearch/EditEval/main/LICENSE",
        source_url: "https://github.com/facebookresearch/EditEval",
        source_ref: "github:main",
        source: SuiteSource::Unavailable,
        grading: Grading::NotApplicable,
        availability: Availability::Unavailable {
            reason: "EditEval ships an evaluation harness with no task payload (configs/dataset_paths.json points at per-corpus download directories), and its constituent corpora fail the permissive-only policy: ASSET is CC BY-NC 4.0 and JFLEG is CC BY-NC-SA 4.0. The instructed-text-editing requirement is executed through the Apache-2.0 CoEdIT suite instead.",
        },
    },
    SuiteManifest {
        id: "swebench_lite",
        title: "SWE-bench Lite (dev split)",
        task_family: "agentic_repository_patch",
        license: "MIT",
        license_url: "https://raw.githubusercontent.com/SWE-bench/SWE-bench/main/LICENSE",
        source_url: "https://huggingface.co/datasets/princeton-nlp/SWE-bench_Lite",
        source_ref: "huggingface:6ec7bb89b9342f664a54a6e0a6ea6501d3437cc2",
        source: SuiteSource::DatasetsServerRows {
            dataset: "princeton-nlp/SWE-bench_Lite",
            config: "default",
            split: "dev",
            cache_file: "swebench-lite-dev.jsonl",
        },
        grading: Grading::UnifiedDiff,
        availability: Availability::Runnable,
    },
];

/// Look a suite up by its stable id.
#[must_use]
pub fn suite(id: &str) -> Option<&'static SuiteManifest> {
    SUITES.iter().find(|manifest| manifest.id == id)
}

/// Every suite id, in manifest order.
#[must_use]
pub fn suite_ids() -> Vec<&'static str> {
    SUITES.iter().map(|manifest| manifest.id).collect()
}
