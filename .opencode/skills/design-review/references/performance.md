# Performance checklist

1. Is there an input-path latency budget? Any per-event cost (allocation,
   table rebuild, synchronous IO) on the keystroke/event path is major.
2. Structure costs: O(n²) anywhere n is unbounded (bindings × devices ×
   platforms).
3. Memory growth: translation tables, caches — are they bounded?
4. Startup cost: file reads and schema parsing at boot.
5. Caching: is invalidation correct, not just fast?
6. Quote the research baseline; flag a gap between the design's expected
   numbers and the baseline without explanation.
