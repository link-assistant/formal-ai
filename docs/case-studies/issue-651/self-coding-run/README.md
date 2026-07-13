# Hive-Mind-dispatched self-coding run

Run `cargo build --release --bin formal-ai` and then
`examples/self-coding/run.sh`. The script creates a scratch Git repository,
starts `formal-ai serve` in agent mode, drives the real Agent CLI with stream
JSON output, verifies the requested file, and captures every inner-loop layer
here. `examples/self-coding/run.sh --live ISSUE_URL` invokes the outer Hive Mind
`solve` entry; the offline replay does not require GitHub access or mutate a
remote repository.
