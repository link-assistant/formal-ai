# Main release merge and governed source conservation

Main advanced to release 0.356.10 at 03a636334975cd7ca5cedff119e2dd0efbc8579f.
The binary governed baseline changed on both branches. Main's complete object
change is limited to four Cargo version source hashes. The merged baseline also
includes six chat-source hash updates and four unsupported case identities that
follow the reviewed source moves. All 223 supported case objects remain intact.
All unsupported objects remain intact after those four explicit path moves.

The source manifest retains every old path, updates the two extracted parent
hashes and adds both child modules. The reviewed reader remains unchanged.
The existing regression fails before the canonical refresh and passes afterward.
This is source conservation evidence; native behavior still requires hosted CI.

[Complete observations](../../../experiments/issue-1580/evidence/checks/main-03a-governed-witness/manifest.json)
retain all three merge-stage baselines, the candidate, complete graph comparison,
keyed changes, conservation proof, procedure sources and before/after test logs.
Every archived member was read back and verified. Supplied procedures receive
zero independent authorship credit. The full issue remains incomplete.
