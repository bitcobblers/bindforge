# security review r1

verdict: request-changes

## Findings
- [major][new] ### 1. Components — "Parses libraryfolders.vdf to find library paths, then scans steamapps/appmanifest_*.acf for game metadata." — The design relies on values from local files (.vdf, .acf) and the Windows Registry to construct the `install_path`. Since these sources are untrusted, a malicious actor could use `..` or absolute paths to cause path traversal. Fix: Implement path canonicalization and validate that the resulting `install_path` is a child of a verified library directory.
- [major][new] ### 1. Components — "Parses libraryfolders.vdf to find library paths, then scans steamapps/appmanifest_*.acf for game metadata." — The parsing of .vdf and .acf files does not specify resource limits. Maliciously large or complex files could cause a Denial of Service (DoS) via memory exhaustion. Fix: Enforce strict maximum file size limits and use a parser that handles malformed/deeply nested structures with bounded resource usage.
