Implement the design at {{designDir}}/02-proposal.md so that the tests
already present in the affected package pass.
Record any deviation from the design in {{designDir}}/03-build-notes.md.
{{testOwnership}}
Never read anything under {{designDir}}/transcripts/ — run records are not
stage input, and the runner audits tool calls and fails the attempt on any
access to them.

To move or rename a file, do not touch the paths yourself —
declare it in `moves`, and the runner performs it.
