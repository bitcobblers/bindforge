# Correctness checklist

1. Assumption register: every `verified` row — does the cited basis actually
   support it? Unverified assumptions stated as fact in the body: blocker.
2. Spot-check 3–5 research URLs with webfetch. Dead or contradicting
   sources: blocker/major. (This is the anti-hallucination gate.)
3. Internal contradictions: components, data flow, and interfaces must
   agree with each other across sections.
4. Missing edge cases: empty input, absent platform, name collisions,
   concurrent modification.
5. Interfaces: the same thing must not appear under two names or two
   shapes in different sections.
6. Does the design actually decide what the source document asked?
7. Revision rounds: are the previous round's blocker/major findings closed
   for real, not just reworded?
