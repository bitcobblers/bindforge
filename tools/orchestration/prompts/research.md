Research the topic of {{sourcePath}} for this pipeline run.
Write your findings to {{designDir}}/01-research.md.

Your read set is: the source document at {{sourcePath}}; the directory listing
of `docs/adrs/` (that listing is the ADR index — there is no index file); the
ADRs that source cites by number; and, if this design directory already holds
them, the immediately prior run's `01-research.md` and `02-proposal.md`.
Outside that set, read at most 25 more files. The ceiling is the point: reads
beyond the sources are context the run pays for twice.
Never open {{designDir}}/reviews/ or {{designDir}}/transcripts/ — run records
are not research input, and the runner audits tool calls and fails the attempt
on any access to them.
