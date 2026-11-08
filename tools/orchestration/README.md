# @bindforge/orchestration

A small CLI that drives a running [OpenCode](https://opencode.ai) server through
the `@opencode/client` SDK: it creates a session, sends a sample prompt, waits
for the assistant's completed reply, validates the reply as JSON, and prints a
structured summary. It doubles as an end-to-end connectivity and auth smoke
test for a server.

## Usage

```sh
npm start            # run from source (tsx src/cli.ts)
npm run build && node dist/cli.js   # run the compiled CLI
```

| Option       | Environment variable       | Default                                       |
| ------------ | -------------------------- | --------------------------------------------- |
| `--base-url` | `OPENCODE_BASE_URL`        | `http://127.0.0.1:4096`                       |
| `--username` | `OPENCODE_SERVER_USERNAME` | `opencode` (used when only a password is set) |
| `--password` | `OPENCODE_SERVER_PASSWORD` | —                                             |
| `--agent`    | `OPENCODE_AGENT`           | the server's default agent                    |

When a password is present (via flag or environment variable), every request is
sent with HTTP Basic auth (`Authorization: Basic base64(<username>:<password>)`).

## Behaviour

1. `session.create` makes a session titled "SDK connection test". When `--agent`
   (or `$OPENCODE_AGENT`) is set, that agent is bound to the session and
   therefore processes the prompt — in the v2 API the agent belongs to the
   session, not to the individual prompt.
2. `session.prompt` enqueues the sample prompt, which asks the model to reply
   with a one-field JSON object.
3. The CLI polls `message.list` every 250 ms (up to 30 s) until a completed
   assistant message is available.
4. The reply text is parsed as JSON and validated with a Zod schema; the full
   result (session, prompt, reply, model, tokens, cost) is printed as JSON.

## Layout

| File                     | Purpose                                                        |
| ------------------------ | -------------------------------------------------------------- |
| `src/cli.ts`             | Option parsing, environment defaults, printing the result      |
| `src/opencode-runner.ts` | Everything that talks to the OpenCode SDK (client, prompt run) |

## Development

| Script                            | Purpose                                       |
| --------------------------------- | --------------------------------------------- |
| `npm run typecheck`               | TypeScript check without emitting             |
| `npm run lint` / `lint:fix`       | ESLint                                        |
| `npm run format` / `format:check` | Prettier                                      |
| `npm test`                        | Vitest unit tests (`tests/`)                  |
| `npm run check`                   | lint + format check + tests                   |
| `npm run build`                   | Compile `src/` to `dist/` (JS + declarations) |
