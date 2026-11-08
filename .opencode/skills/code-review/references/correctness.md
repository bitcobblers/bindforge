# Code correctness checklist — faithful to the design

1. Every component and interface named in `02-proposal.md` exists, is
   wired in, and matches the proposal's names and signatures.
2. Assumption register: the code honors each row; an assumption the code
   silently changed is a blocker.
3. Deviations: any gap between proposal and code must appear in
   `03-build-notes.md`. Undeclared deviation: blocker.
4. Scope creep: behavior in the diff that the proposal never asked for is
   a finding (major), even if it looks useful.
5. `03-build-notes.md` claims are true: check each against the code.
6. Error paths from the design exist and fail closed.
7. Revision rounds: previous round's blocker/major findings actually
   addressed in this diff?
