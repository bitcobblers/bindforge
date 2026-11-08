# Compatibility checklist

1. Compatibility matrix: every platform row has a status. `unknown` must
   appear as a finding (major), empty cell is a blocker.
2. OS input APIs: X11 vs Wayland vs Windows (Raw Input) vs macOS (HID).
   The design must name which it targets per platform, or justify the
   abstraction.
3. Node version floor: any API used above the floor is a finding.
4. Native modules: packaging story (prebuilds, node-gyp, CI matrix) or
   pure JS. Silent native dependencies: major.
5. Keyboard reality: non-QWERTY layouts, dead keys, IME, non-ASCII
   binding names.
6. File paths: separators and case handled by a library, not string
   concatenation; per-OS config dir conventions (XDG / AppData).
7. The Test strategy must name a platform test for every platform the
   matrix claims as supported.
