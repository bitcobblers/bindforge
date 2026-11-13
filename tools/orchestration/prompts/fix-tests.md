npm run check is red in this repository — that is the only reason this stage
runs. Reproduce it: run npm run check in the package that holds the change,
read what fails, and fix the implementation until it exits green. There are
no review reports for this round; code review has not run.

When the complaint is a formatting or lint rule rather than a failing test,
run npm run format in that package and commit the result. {{testOwnership}}
An empty fix that leaves the gate red burns a fix round.

Stay faithful to {{designDir}}/02-proposal.md.

To move or rename a file, do not touch the paths yourself —
declare it in `moves`, and the runner performs it.
