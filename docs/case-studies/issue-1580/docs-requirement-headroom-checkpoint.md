# Documentation assertion owners retain headroom

The documentation requirement owner had 999 lines and a real scanner warning.
Its tracked-text readers and label scanner now live in a 135-line child; the
original owner has 873 lines under unchanged warning and error thresholds.
All fourteen native test names, their order and assertion bodies remain in the
original module. An exact inverse also preserves every moved helper body.

The new focused scanner case fails before extraction. All fourteen headroom
cases, both size gates, both documentation gates and the maintained issue suite
pass afterward. The first identity comparison accidentally selected no names;
that comparison receives no verification credit. A corrected, nonempty list
and exact body comparison are retained with both failed verification drafts.
No Rust compiler ran. Hosted native execution and full issue acceptance remain
unfinished. Supplied work receives zero independent authorship or reasoning
credit. The source warning inventory is reduced from 76 to 75.

Evidence: `experiments/issue-1580/evidence/checks/docs-requirement-headroom/`.
