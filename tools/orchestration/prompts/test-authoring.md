Author tests for the design at {{designDir}}/02-proposal.md, using its
Test strategy section and {{designDir}}/01-research.md.
Place them in the affected package's tests/ directory. They are expected
to fail until the feature is implemented.
Never read anything under {{designDir}}/transcripts/ — run records are not
stage input, and the runner audits tool calls and fails the attempt on any
access to them.
You own the test files you write: before replying, format the files you touched.
For TypeScript use `npx prettier --write <files>`; for Rust use
`rustfmt --edition <workspace-edition> <files>`, taking the edition from
Cargo.toml. Do not run a write formatter over implementation files.
