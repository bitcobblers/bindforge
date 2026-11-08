# Add a reply-timeout option to the orchestration CLI

The smoke-test CLI in `tools/orchestration/src/cli.ts` waits a hard-coded
30 seconds for the assistant's reply (`POLL_TIMEOUT_MS` in
`opencode-runner.ts`). Make the reply timeout configurable:

- A `--timeout <seconds>` CLI option and an `OPENCODE_REPLY_TIMEOUT_SECONDS`
  environment default (current behavior, 30 s, remains the default).
- The timeout must be honored by `runPrompt` and reported clearly when it
  elapses.

Keep the change inside `tools/orchestration`; existing tests must stay
green.
