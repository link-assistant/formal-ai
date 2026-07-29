#!/usr/bin/env python3
"""Unit tests for the falsifiable issue #840/#842 task-ladder judge."""

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from ladder import (
    fixture_for_query,
    judge,
    learning_proposals,
    load_documents,
    ratchet_errors,
)


class JudgeTests(unittest.TestCase):
    def test_raw_tool_results_cannot_satisfy_answer_assertions(self):
        result = judge(
            {"expect": ["verified target"], "forbid": []},
            assistant_output="I could not determine the answer.",
            tools_called=["bash"],
            bash_commands=["find . -print"],
        )
        self.assertFalse(result["pass"])
        self.assertEqual(result["missing_expect"], ["verified target"])
        self.assertTrue(result["refused"])

    def test_seeded_generic_fallbacks_fail_in_every_supported_language(self):
        for fallback in [
            "I don't know how to answer that yet.",
            "Я пока не знаю, как ответить на это.",
            "मुझे अभी इसका उत्तर देना नहीं आता।",
            "我还不知道如何回答这个问题。",
            "我无法从本地 Links Notation 记忆确定答案。",
        ]:
            result = judge(
                {"expect": [], "forbid": []},
                assistant_output=fallback,
                tools_called=[],
                bash_commands=[],
            )
            self.assertFalse(result["pass"], fallback)
            self.assertTrue(result["refused"], fallback)

    def test_judge_checks_alternatives_routes_and_shell_shape(self):
        node = {
            "expect": [],
            "expect_any": ["not found", "no matching"],
            "forbid": ["invented"],
            "expect_tool": ["bash"],
            "forbid_tool": ["websearch"],
            "command_forbid": ["-quit", "&&"],
        }
        passing = judge(
            node,
            assistant_output="No matching folder was found.",
            tools_called=["bash"],
            bash_commands=["find . -type d -print"],
        )
        self.assertTrue(passing["pass"], passing)

        failing = judge(
            node,
            assistant_output="No matching folder was found.",
            tools_called=["websearch", "bash"],
            bash_commands=["find . -type d -print -quit"],
        )
        self.assertFalse(failing["pass"])
        self.assertEqual(failing["forbidden_tools"], ["websearch"])
        self.assertEqual(failing["bad_commands"], ["-quit"])

    def test_fixture_matching_is_case_insensitive_and_conjunctive(self):
        documents = [
            {"keywords": ["alpha", "beta"], "text": "specific"},
            {"keywords": ["alpha"], "text": "broad"},
        ]
        self.assertEqual(fixture_for_query("ALPHA beta", documents), "specific")
        self.assertEqual(fixture_for_query("alpha only", documents), "broad")
        self.assertIsNone(fixture_for_query("gamma", documents))

    def test_mixed_script_english_request_selects_the_english_fixture(self):
        documents = load_documents(
            Path(__file__).with_name("web_fixtures.json").as_posix()
        )
        evidence = fixture_for_query(
            "fufloмицин фуфломицин answer in english", documents
        )
        self.assertIn("efficacy", evidence)
        self.assertNotIn("развернуть", evidence)


class RatchetTests(unittest.TestCase):
    @staticmethod
    def payload(rows):
        passed = sum(row["pass"] for row in rows)
        return {
            "summary": {
                "total": len(rows),
                "passed": passed,
                "failed": len(rows) - passed,
            },
            "results": rows,
        }

    def test_new_green_nodes_extend_the_dataset(self):
        baseline = self.payload([{"id": "old", "pass": True}])
        measured = self.payload(
            [{"id": "old", "pass": True}, {"id": "new", "pass": True}]
        )
        self.assertEqual(ratchet_errors(baseline, measured), [])

    def test_a_removed_or_regressed_node_fails_by_stable_id(self):
        baseline = self.payload(
            [{"id": "a", "pass": True}, {"id": "b", "pass": True}]
        )
        regressed = self.payload([{"id": "a", "pass": False}])
        errors = ratchet_errors(baseline, regressed)
        self.assertTrue(any("missing baseline node b" in error for error in errors))
        self.assertTrue(any("baseline node a regressed" in error for error in errors))

    def test_new_nodes_must_be_green_before_the_baseline_can_move(self):
        baseline = self.payload([{"id": "old", "pass": True}])
        measured = self.payload(
            [{"id": "old", "pass": True}, {"id": "new", "pass": False}]
        )
        errors = ratchet_errors(baseline, measured)
        self.assertTrue(any("current node new is failing" in error for error in errors))


class LearningProposalTests(unittest.TestCase):
    def test_only_observed_failures_become_review_gated_candidates(self):
        measured = {
            "results": [
                {"id": "green", "pass": True},
                {
                    "id": "new-report",
                    "seed": "900",
                    "level": 1,
                    "lang": "en",
                    "prompt": "Find the reported thing",
                    "pass": False,
                    "assistant_output": "I could not determine the answer.",
                    "tools_called": ["websearch"],
                    "bash_commands": [],
                    "missing_tools": ["bash"],
                    "forbidden_tools": ["websearch"],
                    "refused": True,
                },
            ]
        }

        report = learning_proposals(measured)

        self.assertEqual(report["record_type"], "task_ladder_learning_proposals")
        self.assertEqual(report["promotion"], "awaiting_human_review")
        self.assertEqual(len(report["candidates"]), 1)
        candidate = report["candidates"][0]
        self.assertEqual(candidate["id"], "new-report")
        self.assertEqual(candidate["evidence"]["tools_called"], ["websearch"])
        self.assertEqual(candidate["violations"]["missing_tools"], ["bash"])
        self.assertTrue(candidate["violations"]["refused"])
        self.assertIn("all task-ladder nodes pass", candidate["promotion_gate"])


if __name__ == "__main__":
    unittest.main()
