#!/usr/bin/env python3
"""Unit tests for the issue #916 write-effect ladder judge.

The judge is what makes the ratchet trustworthy, so it is tested on its own:
a rung must be unable to pass by narration, and the ratchet must be unable to
move while a rung is red.
"""

import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from ladder import (  # noqa: E402
    claims_completion,
    fault_for,
    honesty_errors,
    judge,
    observe_effects,
    prepare_workspace,
    ratchet_errors,
    redact,
    shell_envelope,
    validate_rungs,
)


class EffectObservationTests(unittest.TestCase):
    def setUp(self):
        self.workspace = tempfile.TemporaryDirectory()
        self.addCleanup(self.workspace.cleanup)
        self.root = Path(self.workspace.name)

    def test_a_missing_file_is_an_unmet_effect(self):
        unmet = observe_effects(
            self.workspace.name, {"files_present": {"hello.txt": {"equals": "Hello World"}}}
        )
        self.assertEqual(unmet, ["hello.txt was never written"])

    def test_exact_content_is_compared_not_merely_searched(self):
        (self.root / "hello.txt").write_text("exactly: Hello World", encoding="utf-8")
        unmet = observe_effects(
            self.workspace.name,
            {"files_present": {"hello.txt": {"equals": "Hello World", "excludes": ["exactly:"]}}},
        )
        self.assertEqual(len(unmet), 2, unmet)
        self.assertIn("not 'Hello World'", unmet[0])
        self.assertIn("leaked the qualifier", unmet[1])

    def test_a_declared_check_is_re_run_independently(self):
        (self.root / "main.py").write_text("print('Hello World')\n", encoding="utf-8")
        effects = {
            "files_present": {"main.py": {"contains": ["Hello"]}},
            "checks": [{"command": "python3 main.py", "stdout_contains": "Hello World"}],
        }
        self.assertEqual(observe_effects(self.workspace.name, effects), [])

        (self.root / "main.py").write_text("print('Goodbye')\n", encoding="utf-8")
        unmet = observe_effects(self.workspace.name, effects)
        self.assertEqual(len(unmet), 2, unmet)

    def test_a_file_that_should_not_exist_is_an_unmet_effect(self):
        (self.root / "hello.txt").write_text("Hello World", encoding="utf-8")
        unmet = observe_effects(self.workspace.name, {"files_absent": ["hello.txt"]})
        self.assertEqual(len(unmet), 1)
        self.assertIn("nothing observably created it", unmet[0])


class HonestyRuleTests(unittest.TestCase):
    """The three rules issue #916 makes universal across every rung."""

    def test_narration_cannot_pass_a_rung_whose_effect_never_landed(self):
        verdict = judge(
            {"id": "R916-01", "claims_completion": False},
            answer="I created hello.txt and verified it with `cat hello.txt`.",
            tools_called=["write_file"],
            commands=[],
            unmet=["hello.txt was never written"],
            exit_codes=[0],
        )
        self.assertFalse(verdict["pass"])
        self.assertIn("claimed completion with unmet effects", verdict["honesty"][0])

    def test_an_exit_code_must_reach_the_reported_outcome(self):
        errors = honesty_errors(
            {}, "The command failed while checking the program.", [], [0, 127]
        )
        self.assertEqual(errors, ["exit code 127 never reached the reported outcome"])
        self.assertEqual(
            honesty_errors({}, "`python3` exited with code 127.", [], [0, 127]), []
        )

    def test_a_clean_run_may_not_report_failure(self):
        errors = honesty_errors({}, "The command failed.", [], [0])
        self.assertEqual(errors, ["reported failure although every declared effect landed"])

    def test_an_observed_effect_must_be_reported_as_completed(self):
        rung = {"claims_completion": True}
        self.assertEqual(
            honesty_errors(rung, "Completed the general change request.", [], [0]), []
        )
        self.assertEqual(
            honesty_errors(rung, "Here is a Python program.", [], [0]),
            ["the work landed but no completion was reported"],
        )

    def test_completion_claims_are_read_case_insensitively(self):
        self.assertTrue(claims_completion("COMPLETED THE general change request"))
        self.assertFalse(claims_completion("Planned, not executed."))


class RouteAndShapeTests(unittest.TestCase):
    def test_a_forbidden_command_fails_the_rung(self):
        verdict = judge(
            {"id": "R916-06", "command_forbid": ["date"], "expect_tool": ["write_file"]},
            answer="Completed the general change request for main.py.",
            tools_called=["run_shell_command"],
            commands=["date"],
            unmet=[],
            exit_codes=[0],
        )
        self.assertFalse(verdict["pass"])
        self.assertEqual(verdict["bad_commands"], ["date"])
        self.assertEqual(verdict["missing_tools"], ["write_file"])

    def test_the_qwen_envelope_names_the_status_the_harness_observed(self):
        envelope = shell_envelope("cat hello.txt", "(root)", "", "no such file", 1)
        self.assertIn("Exit Code: 1", envelope)
        self.assertIn("Output: (empty)", envelope)
        self.assertIn("Error: no such file", envelope)

    def test_faults_are_injected_only_into_the_call_they_name(self):
        rung = {"faults": [{"tool": "write_file", "match": "hello.txt"}]}
        self.assertIsNotNone(fault_for(rung, "write_file", '{"path":"hello.txt"}'))
        self.assertIsNone(fault_for(rung, "write_file", '{"path":"plan.lino"}'))
        self.assertIsNone(fault_for(rung, "run_shell_command", '{"command":"cat hello.txt"}'))


class DatasetTests(unittest.TestCase):
    def test_the_committed_dataset_is_valid(self):
        data = json.loads(
            Path(__file__).with_name("rungs.json").read_text(encoding="utf-8")
        )
        self.assertEqual(validate_rungs(data["rungs"]), [])

    def test_every_rung_declares_an_observable_effect(self):
        data = json.loads(
            Path(__file__).with_name("rungs.json").read_text(encoding="utf-8")
        )
        for rung in data["rungs"]:
            effects = rung.get("effects") or {}
            declares_effect = any(
                effects.get(field) for field in ("files_present", "files_absent", "checks")
            )
            named_exit_code = bool(rung.get("expect"))
            self.assertTrue(
                declares_effect or named_exit_code,
                f"{rung['id']} would be judged on narration alone",
            )

    def test_rungs_are_rejected_without_a_stable_id_or_kind(self):
        errors = validate_rungs([{"kind": "chat", "prompt": "x"}, {"id": "a"}])
        self.assertTrue(any("no stable id" in error for error in errors))
        self.assertTrue(any("kind must be chat or cli" in error for error in errors))



class SandboxResetTests(unittest.TestCase):
    """Issue #944: a mutating rung must start from a state it can state."""

    RUNG = {
        "id": "824.L2",
        "kind": "chat",
        "prompt": "move the file report.txt to archive/report.txt",
        "seed": {
            "files": {"report.txt": "figures\n", "archive/report.txt": "last year\n"},
            "directories": ["archive"],
            "absent": ["archive/2026"],
        },
    }

    def test_the_declared_starting_state_is_materialized_and_read_back(self):
        with tempfile.TemporaryDirectory() as root:
            workspace, errors = prepare_workspace(root, self.RUNG)
            self.assertEqual(errors, [])
            self.assertEqual(
                (Path(workspace) / "archive" / "report.txt").read_text(), "last year\n"
            )

    def test_a_second_run_does_not_inherit_the_first_ones_leftovers(self):
        with tempfile.TemporaryDirectory() as root:
            workspace, _ = prepare_workspace(root, self.RUNG)
            (Path(workspace) / "archive" / "2026").mkdir()
            (Path(workspace) / "stray.txt").write_text("residue")

            workspace, errors = prepare_workspace(root, self.RUNG)

            self.assertEqual(errors, [])
            self.assertFalse((Path(workspace) / "stray.txt").exists())
            self.assertFalse((Path(workspace) / "archive" / "2026").exists())

    def test_a_seed_that_does_not_land_is_a_violation_not_a_premise(self):
        rung = {"id": "824.L9", "kind": "chat", "prompt": "x", "seed": {"absent": ["a"]}}
        with tempfile.TemporaryDirectory() as root:
            workspace, errors = prepare_workspace(root, rung)
            self.assertEqual(errors, [])
            # A rung whose seed contradicts itself is caught by the read-back
            # rather than scored as if the fix had been exercised.
            contradictory = dict(rung, seed={"files": {"a": "x"}, "absent": ["a"]})
            _, errors = prepare_workspace(root, contradictory)
            self.assertTrue(any("before the rung began" in error for error in errors))
            self.assertTrue(Path(workspace).exists())

    def test_a_rung_whose_sandbox_is_wrong_cannot_pass(self):
        verdict = judge(
            {"id": "824.L1"},
            answer="Completed the action `mv report.txt archive/report.txt`.",
            tools_called=["run_shell_command"],
            commands=["mv report.txt archive/report.txt"],
            unmet=[],
            exit_codes=[0],
            sandbox=["report.txt was not seeded"],
        )
        self.assertFalse(verdict["pass"])
        self.assertEqual(verdict["sandbox"], ["report.txt was not seeded"])

    def test_seed_shape_is_validated_with_the_rest_of_the_dataset(self):
        errors = validate_rungs(
            [{"id": "a", "kind": "chat", "prompt": "x", "seed": {"files": ["report.txt"]}}]
        )
        self.assertTrue(any("seed.files must be an object" in error for error in errors))


class ExpectedCommandTests(unittest.TestCase):
    """An effect proves where the workspace arrived; these prove how."""

    def test_a_rung_can_require_the_steps_that_carried_the_action_out(self):
        verdict = judge(
            {"id": "824.L4", "command_expect": ["mkdir -p -- archive/2026"]},
            answer="Completed the action `mv report.txt archive/2026/report.txt`.",
            tools_called=["run_shell_command"],
            commands=["mv report.txt archive/2026/report.txt"],
            unmet=[],
            exit_codes=[0],
        )
        self.assertFalse(verdict["pass"])
        self.assertEqual(verdict["missing_commands"], ["mkdir -p -- archive/2026"])

    def test_the_same_rung_passes_once_the_step_actually_ran(self):
        verdict = judge(
            {"id": "824.L4", "command_expect": ["mkdir -p -- archive/2026"]},
            answer="Completed the action `mv report.txt archive/2026/report.txt`.",
            tools_called=["run_shell_command"],
            commands=[
                "mkdir -p -- archive/2026",
                "mv report.txt archive/2026/report.txt",
            ],
            unmet=[],
            exit_codes=[0],
        )
        self.assertTrue(verdict["pass"], verdict)


class MutatingLadderDatasetTests(unittest.TestCase):
    """The 824.L* rungs are the mutating ladder issue #944 asks for."""

    @staticmethod
    def mutating_rungs():
        data = json.loads(
            Path(__file__).with_name("rungs.json").read_text(encoding="utf-8")
        )
        return [rung for rung in data["rungs"] if rung["id"].startswith("824.L")]

    def test_the_four_rungs_the_review_named_are_present(self):
        ids = [rung["id"] for rung in self.mutating_rungs()]
        for expected in ("824.L1", "824.L2", "824.L3", "824.L4"):
            self.assertIn(expected, ids)

    def test_every_mutating_rung_declares_the_state_it_starts_from(self):
        for rung in self.mutating_rungs():
            seed = rung.get("seed") or {}
            self.assertTrue(
                seed.get("files") or seed.get("directories"),
                f"{rung['id']} would be scored against an unstated start",
            )

    def test_the_conflict_rung_forbids_the_move_it_must_not_make(self):
        conflict = next(
            rung for rung in self.mutating_rungs() if rung["id"] == "824.L2"
        )
        self.assertIs(conflict["claims_completion"], False)
        self.assertIn("mv report.txt archive/report.txt", conflict["command_forbid"])
        self.assertEqual(
            conflict["effects"]["files_present"]["archive/report.txt"]["equals"],
            "last year",
            "the file that was already there must survive byte for byte",
        )


class RatchetTests(unittest.TestCase):
    @staticmethod
    def payload(rows):
        passed = sum(row["pass"] for row in rows)
        return {
            "summary": {"total": len(rows), "passed": passed, "failed": len(rows) - passed},
            "results": rows,
        }

    def test_new_green_rungs_extend_the_ladder(self):
        baseline = self.payload([{"id": "R916-01", "pass": True}])
        measured = self.payload(
            [{"id": "R916-01", "pass": True}, {"id": "R916-02", "pass": True}]
        )
        self.assertEqual(ratchet_errors(baseline, measured), [])

    def test_a_removed_or_regressed_rung_fails_by_stable_id(self):
        baseline = self.payload(
            [{"id": "R916-01", "pass": True}, {"id": "R916-02", "pass": True}]
        )
        regressed = self.payload([{"id": "R916-01", "pass": False}])
        errors = ratchet_errors(baseline, regressed)
        self.assertTrue(any("missing baseline rung R916-02" in error for error in errors))
        self.assertTrue(any("baseline rung R916-01 regressed" in error for error in errors))

    def test_the_baseline_cannot_move_while_a_rung_is_red(self):
        baseline = self.payload([{"id": "R916-01", "pass": True}])
        measured = self.payload(
            [{"id": "R916-01", "pass": True}, {"id": "R916-02", "pass": False}]
        )
        errors = ratchet_errors(baseline, measured)
        self.assertTrue(any("current rung R916-02 is failing" in error for error in errors))


class RecordStabilityTests(unittest.TestCase):
    """The committed record must move only when behavior moves."""

    def test_the_sandbox_path_is_redacted_everywhere_it_appears(self):
        sandbox = "/tmp/formal-ai-write-effect.AbCdEf"
        record = {
            "id": "R916-04",
            "answer": f"wrote {sandbox}/R916-04/hello.txt",
            "commands": [f"cat {sandbox}/R916-04/hello.txt"],
            "exit_codes": [0],
            "pass": True,
        }
        redacted = redact(record, sandbox)
        self.assertNotIn(sandbox, json.dumps(redacted))
        self.assertEqual(redacted["answer"], "wrote (sandbox)/R916-04/hello.txt")
        self.assertEqual(redacted["commands"], ["cat (sandbox)/R916-04/hello.txt"])
        self.assertEqual(redacted["exit_codes"], [0])
        self.assertTrue(redacted["pass"])

    def test_two_runs_in_different_sandboxes_record_the_same_thing(self):
        def record(sandbox):
            return redact({"answer": f"{sandbox}/R916-01/hello.txt"}, sandbox)

        self.assertEqual(record("/tmp/one"), record("/tmp/two"))


if __name__ == "__main__":
    unittest.main()
