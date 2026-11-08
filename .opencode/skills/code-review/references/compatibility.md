# Code compatibility checklist

1. Platform code matches the proposal's Compatibility matrix: OS-specific
   code behind explicit guards, and every "supported" platform actually
   has a code path.
2. Node floor: no API above the package's supported Node version
   (`engines`, README, or the proposal's floor).
3. Paths: built with `path`/URL joins, never string concatenation; case
   sensitivity assumptions flagged.
4. Guarded platform imports: platform-only modules must not break load on
   other platforms (lazy/conditional import).
5. Locale/layout reality: non-ASCII binding names, non-QWERTY layouts —
   do file formats and IDs survive them?
6. Native dependencies introduced in the diff: packaging story and CI
   matrix (major if silent).
7. Tests named in the plan actually run on the platforms they claim —
   platform-specific tests guarded, not skipped everywhere.
