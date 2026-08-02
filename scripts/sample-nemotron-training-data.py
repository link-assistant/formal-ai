#!/usr/bin/env python3
"""Sample small, licensed slices of NVIDIA Nemotron training-data shards.

The script intentionally uses the Hugging Face datasets-server `rows` endpoint
with `length=1`; it never downloads parquet files or a full split. Output is a
compact provenance JSON with text digests and short excerpts that can be checked
into the repository for deterministic tests.
"""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import random
import sys
import textwrap
import time
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass
from typing import Any


API_BASE = "https://datasets-server.huggingface.co"
MAX_RETRIES_PER_CONFIG = 8
USER_AGENT = "formal-ai-issue-482-nemotron-sampler/1.0"


@dataclass(frozen=True)
class DatasetConfig:
    source_id: str
    dataset: str
    config: str
    source_ref: str
    dataset_card: str


CONFIGS = [
    DatasetConfig(
        "nemotron_legal_v1",
        "nvidia/Nemotron-Pretraining-Legal-v1",
        "Nemotron-Pretraining-Legal-California-Code-Of-Regulations",
        "3d91d58a5c0c46fe9944300ec46719f97a385b13",
        "https://huggingface.co/datasets/nvidia/Nemotron-Pretraining-Legal-v1",
    ),
    DatasetConfig(
        "nemotron_legal_v1",
        "nvidia/Nemotron-Pretraining-Legal-v1",
        "Nemotron-Pretraining-Legal-Case-Law-Summary",
        "3d91d58a5c0c46fe9944300ec46719f97a385b13",
        "https://huggingface.co/datasets/nvidia/Nemotron-Pretraining-Legal-v1",
    ),
    DatasetConfig(
        "nemotron_legal_v1",
        "nvidia/Nemotron-Pretraining-Legal-v1",
        "Nemotron-Pretraining-Legal-CaseHOLD",
        "3d91d58a5c0c46fe9944300ec46719f97a385b13",
        "https://huggingface.co/datasets/nvidia/Nemotron-Pretraining-Legal-v1",
    ),
    DatasetConfig(
        "nemotron_legal_v1",
        "nvidia/Nemotron-Pretraining-Legal-v1",
        "Nemotron-Pretraining-Legal-Definition-Classification",
        "3d91d58a5c0c46fe9944300ec46719f97a385b13",
        "https://huggingface.co/datasets/nvidia/Nemotron-Pretraining-Legal-v1",
    ),
    DatasetConfig(
        "nemotron_legal_v1",
        "nvidia/Nemotron-Pretraining-Legal-v1",
        "Nemotron-Pretraining-Legal-Diversity-Jurisdiction",
        "3d91d58a5c0c46fe9944300ec46719f97a385b13",
        "https://huggingface.co/datasets/nvidia/Nemotron-Pretraining-Legal-v1",
    ),
    DatasetConfig(
        "nemotron_legal_v1",
        "nvidia/Nemotron-Pretraining-Legal-v1",
        "Nemotron-Pretraining-Legal-Function-Of-Decision",
        "3d91d58a5c0c46fe9944300ec46719f97a385b13",
        "https://huggingface.co/datasets/nvidia/Nemotron-Pretraining-Legal-v1",
    ),
    DatasetConfig(
        "nemotron_legal_v1",
        "nvidia/Nemotron-Pretraining-Legal-v1",
        "Nemotron-Pretraining-Legal-GlobalCit",
        "3d91d58a5c0c46fe9944300ec46719f97a385b13",
        "https://huggingface.co/datasets/nvidia/Nemotron-Pretraining-Legal-v1",
    ),
    DatasetConfig(
        "nemotron_legal_v1",
        "nvidia/Nemotron-Pretraining-Legal-v1",
        "Nemotron-Pretraining-Legal-LegalBench-CUAD-v2",
        "3d91d58a5c0c46fe9944300ec46719f97a385b13",
        "https://huggingface.co/datasets/nvidia/Nemotron-Pretraining-Legal-v1",
    ),
    DatasetConfig(
        "nemotron_legal_v1",
        "nvidia/Nemotron-Pretraining-Legal-v1",
        "Nemotron-Pretraining-Legal-NYCourts-Judicial-Ethics-Opinions",
        "3d91d58a5c0c46fe9944300ec46719f97a385b13",
        "https://huggingface.co/datasets/nvidia/Nemotron-Pretraining-Legal-v1",
    ),
    DatasetConfig(
        "nemotron_legal_v1",
        "nvidia/Nemotron-Pretraining-Legal-v1",
        "Nemotron-Pretraining-Legal-ToS-Clause-Understanding",
        "3d91d58a5c0c46fe9944300ec46719f97a385b13",
        "https://huggingface.co/datasets/nvidia/Nemotron-Pretraining-Legal-v1",
    ),
    DatasetConfig(
        "nemotron_legal_v1",
        "nvidia/Nemotron-Pretraining-Legal-v1",
        "Nemotron-Pretraining-Legal-ToSDR-QA",
        "3d91d58a5c0c46fe9944300ec46719f97a385b13",
        "https://huggingface.co/datasets/nvidia/Nemotron-Pretraining-Legal-v1",
    ),
    DatasetConfig(
        "nemotron_legal_v1",
        "nvidia/Nemotron-Pretraining-Legal-v1",
        "Nemotron-Pretraining-Legal-eCFR",
        "3d91d58a5c0c46fe9944300ec46719f97a385b13",
        "https://huggingface.co/datasets/nvidia/Nemotron-Pretraining-Legal-v1",
    ),
    DatasetConfig(
        "nemotron_legal_v1",
        "nvidia/Nemotron-Pretraining-Legal-v1",
        "Nemotron-Pretraining-Legal-eCFR-QA",
        "3d91d58a5c0c46fe9944300ec46719f97a385b13",
        "https://huggingface.co/datasets/nvidia/Nemotron-Pretraining-Legal-v1",
    ),
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Fetch a deterministic random slice of Nemotron training-data rows "
            "without downloading full datasets."
        )
    )
    parser.add_argument("--count", type=int, default=10)
    parser.add_argument("--seed", default="issue-482")
    parser.add_argument("--output", default="-")
    parser.add_argument("--api-base", default=API_BASE)
    parser.add_argument("--max-text-chars", type=int, default=360)
    return parser.parse_args()


def stable_rng(seed: str) -> random.Random:
    digest = hashlib.sha256(seed.encode("utf-8")).digest()
    return random.Random(int.from_bytes(digest[:16], "big"))


def fetch_json(api_base: str, path: str, params: dict[str, Any]) -> dict[str, Any]:
    url = f"{api_base.rstrip('/')}/{path}?{urllib.parse.urlencode(params)}"
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    for attempt in range(5):
        try:
            with urllib.request.urlopen(request) as response:
                payload = response.read().decode("utf-8")
            break
        except urllib.error.HTTPError as error:
            if error.code != 429 or attempt == 4:
                raise
            retry_after = error.headers.get("Retry-After")
            delay = float(retry_after) if retry_after else float(attempt + 2)
            time.sleep(delay)
    data = json.loads(payload)
    if "error" in data:
        raise RuntimeError(f"{url} returned error: {data['error']}")
    return data


def rows_url(api_base: str, config: DatasetConfig, offset: int) -> str:
    params = {
        "dataset": config.dataset,
        "config": config.config,
        "split": "train",
        "offset": offset,
        "length": 1,
    }
    return f"{api_base.rstrip('/')}/rows?{urllib.parse.urlencode(params)}"


def fetch_row(api_base: str, config: DatasetConfig, offset: int) -> dict[str, Any]:
    return fetch_json(
        api_base,
        "rows",
        {
            "dataset": config.dataset,
            "config": config.config,
            "split": "train",
            "offset": offset,
            "length": 1,
        },
    )


def row_count(api_base: str, config: DatasetConfig) -> int:
    data = fetch_row(api_base, config, 0)
    count = data.get("num_rows_total")
    if not isinstance(count, int) or count <= 0:
        raise RuntimeError(f"{config.dataset}/{config.config} has invalid row count: {count!r}")
    return count


def normalize_excerpt(text: str, max_chars: int) -> str:
    normalized = " ".join(text.split())
    shortened = textwrap.shorten(normalized, width=max_chars, placeholder="...")
    return shortened


def text_shape(text: str) -> str:
    lowered = text.lower()
    if "question:" in lowered and "answer:" in lowered:
        return "question_answer"
    if "options:" in lowered or "choices:" in lowered:
        return "multiple_choice"
    if "summary:" in lowered:
        return "summary"
    if "yes" in lowered and "no" in lowered and "answer:" in lowered:
        return "binary_decision"
    return "free_text"


def sample_configs(rng: random.Random, count: int) -> list[DatasetConfig]:
    if count <= len(CONFIGS):
        return rng.sample(CONFIGS, count)
    configs: list[DatasetConfig] = list(CONFIGS)
    while len(configs) < count:
        configs.append(rng.choice(CONFIGS))
    rng.shuffle(configs)
    return configs


def sample_row(
    api_base: str,
    rng: random.Random,
    config: DatasetConfig,
    sample_number: int,
    max_text_chars: int,
) -> dict[str, Any]:
    total = row_count(api_base, config)
    for _attempt in range(MAX_RETRIES_PER_CONFIG):
        offset = rng.randrange(total)
        data = fetch_row(api_base, config, offset)
        rows = data.get("rows") or []
        if len(rows) != 1:
            continue
        row = rows[0].get("row") or {}
        license_value = str(row.get("license", "")).lower()
        if license_value != "cc-by-4.0":
            continue
        text = str(row.get("text", ""))
        if not text.strip():
            continue
        excerpt = normalize_excerpt(text, max_text_chars)
        metadata = row.get("metadata", {})
        metadata_keys = sorted(metadata.keys()) if isinstance(metadata, dict) else []
        return {
            "sample_id": f"issue_482_sample_{sample_number:03}",
            "source_id": config.source_id,
            "dataset": config.dataset,
            "config": config.config,
            "split": "train",
            "row_index": offset,
            "num_rows_total": total,
            "row_uuid": row.get("uuid", ""),
            "license": "CC-BY-4.0",
            "source_ref": config.source_ref,
            "dataset_card": config.dataset_card,
            "provenance_url": rows_url(api_base, config, offset),
            "row_fields": sorted(row.keys()),
            "metadata_keys": metadata_keys,
            "text_length": len(text),
            "text_sha256": hashlib.sha256(text.encode("utf-8")).hexdigest(),
            "text_excerpt": excerpt,
            "excerpt_sha256": hashlib.sha256(excerpt.encode("utf-8")).hexdigest(),
            "test_shape": text_shape(text),
            "download_mode": "datasets-server rows endpoint, length=1",
        }
    raise RuntimeError(
        f"could not find a cc-by-4.0 text row in {config.dataset}/{config.config} "
        f"after {MAX_RETRIES_PER_CONFIG} attempts"
    )


def build_payload(args: argparse.Namespace) -> dict[str, Any]:
    if args.count <= 0:
        raise ValueError("--count must be positive")
    if args.max_text_chars < 80:
        raise ValueError("--max-text-chars must be at least 80")

    rng = stable_rng(args.seed)
    configs = sample_configs(rng, args.count)
    samples = [
        sample_row(args.api_base, rng, config, index, args.max_text_chars)
        for index, config in enumerate(configs)
    ]
    return {
        "generated_at_utc": dt.datetime.now(dt.timezone.utc)
        .replace(microsecond=0)
        .isoformat(),
        "seed": args.seed,
        "count": args.count,
        "data_policy": (
            "Only individual rows are requested through datasets-server with "
            "length=1; full parquet splits are not downloaded."
        ),
        "license_policy": "Only sampled rows with explicit CC-BY-4.0 licenses are emitted.",
        "samples": samples,
    }


def main() -> int:
    args = parse_args()
    try:
        payload = build_payload(args)
    except Exception as error:  # noqa: BLE001 - CLI should print a concise failure.
        print(f"error: {error}", file=sys.stderr)
        return 1

    rendered = json.dumps(payload, indent=2, ensure_ascii=True)
    if args.output == "-":
        print(rendered)
    else:
        with open(args.output, "w", encoding="utf-8") as handle:
            handle.write(rendered)
            handle.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
