npm run check is red in this repository — that is the only reason this stage
runs. Reproduce it: run npm run check in the package that holds the change,
read what fails, and fix the implementation until it exits green. There are
no review reports for this round; code review has not run.

When the complaint is a formatting or lint rule rather than a failing test,
run npm run format in that package and commit the result. When it names files
outside your ownership — a test file that is unformatted or a test that
asserts the wrong thing — do not touch them: set testsNeedUpdate in your reply
and the test engineer's stage fixes the files it owns. A red gate whose cause
lives in test files is unsolvable by you alone, and an empty fix that leaves
it red burns a fix round.

Stay faithful to {{designDir}}/02-proposal.md. Never edit test files; if a
test itself must change, set testsNeedUpdate in your reply.
