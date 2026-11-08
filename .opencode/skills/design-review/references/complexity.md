# Complexity checklist

1. Every new package, module, or interface: does the repo already have
   something reusable? Check `tools/` before flagging reuse.
2. Does each moving part trace to a stated Goal? If not: YAGNI, major.
3. Is there a design 30–50% smaller that still meets the Goals? Sketch it
   in the finding.
4. New dependencies: required, or is there an in-repo or stdlib
   alternative?
5. Could the second half of the design be deferred so the first merged
   slice stays thin?
6. Configuration knobs with no consumer; abstraction layers with one
   implementation.
7. Value check: state plainly whether the complexity looks proportionate to
   what the feature brings.
