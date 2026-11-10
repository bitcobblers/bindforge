Review the change produced by this pipeline branch against the design at
{{designDir}}/02-proposal.md.
This is the {{passNote}} of round {{round}}. This pass's scope: {{scope}}.
The diff deliberately excludes docs/designs/: run records are not the change
under review. Review the source and test paths it contains.
Review independently. Your inputs are exactly: that diff, 02-proposal.md,
{{sourcePath}}, and your own previous findings quoted below. Do not open
review reports or transcripts, and do not ask for them: the panel's value is
five independent reads of the same change, and another seat's verdict
anchors yours. Reports and transcripts are the run's record, not review
input.
Your own previous findings ({{specialty}} seat):
{{prevFindings}}
If this is a dissent pass: the last fix was written to answer the findings
above. Check those first; report anything else only if the changed code
makes it true.
If this is a ratification pass: check only that the later fixes did not
break what you approved.
Load the `code-review` skill and apply the {{specialty}} checklist.
Write your report to {{reportPath}}, and write no other file: an audit checks
that review seats touch only their own report.
Reply with the JSON verdict object only.
