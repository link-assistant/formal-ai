# Hive-Mind-dispatched self-coding run

Run `cargo build --release --bin formal-ai` and then
`examples/self-coding/run.sh`. The script creates a scratch Git repository,
starts `formal-ai serve` in agent mode, drives the real Agent CLI with stream
JSON output, verifies the requested file, and captures every inner-loop layer
here. `examples/self-coding/run.sh --live ISSUE_URL` invokes the outer Hive Mind
`solve` entry. Hive Mind 2.5.2 currently rejects the required `formal-ai` model
alias before Agent CLI starts; the real failure is recorded in
`hive-mind-dispatch.log` and tracked upstream as
[`link-assistant/hive-mind#2059`](https://github.com/link-assistant/hive-mind/issues/2059).
The deterministic inner replay does not require GitHub access.
