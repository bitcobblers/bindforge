import { randomBytes } from 'node:crypto';
import { mkdir, readFile, rename, writeFile } from 'node:fs/promises';
import { join } from 'node:path';

import type { PipelineState } from './types.js';

export const STATE_FILE = '.pipeline.json';

export function statePath(designDir: string): string {
  return join(designDir, STATE_FILE);
}

/** Repo-relative design directory for a run, e.g. docs/designs/002-x. */
export function designDirOf(state: PipelineState): string {
  return join('docs', 'designs', `${state.nnn}-${state.slug}`);
}

/** Load a run's checkpoint, or null if none exists yet. */
export async function loadState(designDir: string): Promise<PipelineState | null> {
  let raw: string;
  try {
    raw = await readFile(statePath(designDir), 'utf8');
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
      return null;
    }
    throw error;
  }
  const state = JSON.parse(raw) as PipelineState;
  if (state.version !== 1) {
    throw new Error(`unsupported pipeline state version: ${String(state.version)}`);
  }
  return state;
}

/**
 * Atomically checkpoint the state: write to a unique temp file in the same
 * directory, then rename. A crash mid-write can never truncate the last
 * good checkpoint.
 */
export async function saveState(designDir: string, state: PipelineState): Promise<void> {
  state.updatedAt = Date.now();
  const target = statePath(designDir);
  const tmp = `${target}.${randomBytes(4).toString('hex')}.tmp`;
  await mkdir(designDir, { recursive: true });
  await writeFile(tmp, `${JSON.stringify(state, null, 2)}\n`, 'utf8');
  await rename(tmp, target);
}
