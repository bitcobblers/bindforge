# Code performance checklist

1. Event/keystroke path: no synchronous fs or network, no per-event
   table rebuilds, no avoidable allocations.
2. Complexity: flag O(n²) where n is unbounded (bindings × devices ×
   platforms); maps/sets where identity lookup was intended.
3. Unbounded growth: caches, arrays, registries — bounded or documented?
4. Startup: parsing/reading on boot only what boot needs.
5. Cache invalidation logic present and covered by a test.
6. Where the cheap check is a benchmark-like test or timing note, run it
   and quote the number in the finding.
