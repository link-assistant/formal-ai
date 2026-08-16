"""Tests for the broken-link Web Archive fallback helper."""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path


SCRIPT_PATH = (
    Path(__file__).resolve().parent.parent / "scripts" / "check_web_archive.py"
)
spec = importlib.util.spec_from_file_location("check_web_archive", SCRIPT_PATH)
module = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = module
spec.loader.exec_module(module)  # type: ignore[union-attr]


def test_extract_broken_urls_supports_lychee_markdown_and_deduplicates() -> None:
    """Every broken URL format emitted by lychee should be recognized once."""
    report = """
* [404] https://example.com/missing
- [ERROR] https://example.org/offline | connection refused
* Failure at <https://example.net/timeout>
* [500] https://example.com/missing
"""

    assert module.extract_broken_urls(report) == [
        "https://example.com/missing",
        "https://example.org/offline",
        "https://example.net/timeout",
    ]


def test_check_wayback_machine_returns_available_https_snapshot(monkeypatch) -> None:
    """An available snapshot should be normalized to an HTTPS archive URL."""
    payload = {
        "archived_snapshots": {
            "closest": {
                "available": True,
                "url": "http://web.archive.org/web/20240102030405/https://example.com",
                "timestamp": "20240102030405",
            }
        }
    }
    monkeypatch.setattr(module, "fetch_json", lambda _url: payload)

    result = module.check_wayback_machine("https://example.com")

    assert result.available is True
    assert result.archive_url.startswith("https://web.archive.org/")
    assert result.timestamp == "20240102030405"


def test_check_wayback_machine_treats_api_errors_as_unavailable(monkeypatch) -> None:
    """A Wayback outage must not incorrectly approve a broken documentation URL."""

    def fail(_url: str) -> dict[str, object]:
        raise OSError("temporary outage")

    monkeypatch.setattr(module, "fetch_json", fail)

    result = module.check_wayback_machine("https://example.com")

    assert result.available is False
    assert result.archive_url is None
    assert result.timestamp is None
