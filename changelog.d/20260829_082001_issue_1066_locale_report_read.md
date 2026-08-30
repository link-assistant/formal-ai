Fixed the incremental self-authoring harnesses reporting a failed run for a
dispatch that had solved its task. `experiments/issue_924_self_authoring/run.sh`
and `experiments/issue_933_self_authoring/run.sh` read the UTF-8 dispatch report
with Ruby's `File.read`, which decodes with the locale's default external
encoding, so on a host whose locale is `POSIX`/`C` the first non-ASCII byte
raised `Encoding::InvalidByteSequenceError`. Both harnesses now name the
report's encoding.
