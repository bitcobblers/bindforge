import { mkdtempSync, rmSync } from 'node:fs';
import { mkdtemp } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { afterAll } from 'vitest';

// ---------------------------------------------------------------------------
// Scratch repos under /tmp were created per test and never removed: the suite
// left 45k fixture dirs behind on a box whose /tmp is tmpfs, so every byte
// pinned RAM. Use this instead of raw mkdtempSync(join(tmpdir(), ...)): it
// registers the directory for removal after the importing file's suite
// finishes. A crash that skips the hook is caught by the tmpfiles age-out on
// /tmp/bindforge-* (see /etc/tmpfiles.d/bindforge-tmp.conf).
// ---------------------------------------------------------------------------

const created: string[] = [];

/** Create a scratch directory that is removed when this test file finishes. */
export function mkTmpDir(prefix: string): string {
  const dir = mkdtempSync(join(tmpdir(), prefix));
  created.push(dir);
  return dir;
}

/** Async variant of {@link mkTmpDir} for suites that build fixtures with await. */
export async function mkTmpDirAsync(prefix: string): Promise<string> {
  const dir = await mkdtemp(join(tmpdir(), prefix));
  created.push(dir);
  return dir;
}

afterAll(() => {
  for (const dir of created.splice(0)) {
    rmSync(dir, { recursive: true, force: true });
  }
});
