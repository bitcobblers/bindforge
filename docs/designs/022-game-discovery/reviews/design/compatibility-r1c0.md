# compatibility review r1

verdict: request-changes

## Findings
- [major][new] Test strategy — "2. Plugin Integration Tests" — The test strategy relies on mocks for Steam Linux/Windows parsing and does not specify any platform-specific tests (running on actual Linux/Windows environments) for the platforms claimed as supported in the compatibility matrix. Fix: Add platform-specific integration tests that run on actual Linux and Windows environments (e.g., in CI) to verify the Steam parsing logic against real filesystem structures and the Windows Registry.
