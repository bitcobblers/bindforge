# Code security checklist

1. External input validated at the boundary: shared binding files are
   untrusted — schema-validated, size-limited, no code evaluation.
2. Path handling: no traversal (`..`), no absolute paths from untrusted
   data, no string-interpolated shell commands.
3. Injection: user-controlled strings into shell, regex (catastrophic
   backtracking), or file paths.
4. Secrets and PII: none in source, fixtures, or log lines; sharing
   exports must not leak them.
5. New dependencies in the diff: justified, pinned, and checked against
   known advisories (webfetch if needed); unfixed advisory = blocker.
6. Filesystem writes: least privilege, atomic where partial writes would
   corrupt state.
7. Failures on malformed/hostile input must reject cleanly — no partial
   application, no stack traces to end users.
