import { readFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { describe, expect, it } from 'vitest';

// ---------------------------------------------------------------------------
// The standing CI leg (design 021, Test strategy #10): `checkLedgers` over
// THIS repository is green — no dangling handle, no unhandled unit, no missing
// pointer, note or owner, `depends-on` in order. It is ADR-021's "coverage is
// a grep" made permanent, and it holds 013's 24 units, 021's ten units and the
// template's handle lines to the same rule. It is red from the pins commit
// until the conformance pass (template, 021, 013, the manifests spec) lands;
// from that merge, `npm run check` fails any red ledger before a run can start
// (A24).
//
// The import is dynamic on purpose — `src/pipeline/ledger.ts` is what the
// design adds; the accessors name the missing export as the failure.
// ---------------------------------------------------------------------------

const REPO_ROOT = resolve(fileURLToPath(new URL('../../../../', import.meta.url)));

interface LedgerFinding {
  file: string;
  line: number;
  code: string;
  detail: string;
}

interface LedgerUnit {
  id: string;
}

interface LedgerFile {
  units: LedgerUnit[];
  findings: LedgerFinding[];
}

interface LedgerSurface {
  parseLedger?: (content: string, nnn: string) => LedgerFile | undefined;
  checkLedgers?: (repoDir: string) => LedgerFinding[];
}

const LEDGER = (await import('../../src/pipeline/ledger.js').catch(
  (): LedgerSurface => ({}),
)) as LedgerSurface;

function parseLedger(): NonNullable<LedgerSurface['parseLedger']> {
  if (typeof LEDGER.parseLedger !== 'function') {
    throw new Error('ledger.js exports no parseLedger yet');
  }
  return LEDGER.parseLedger;
}

function checkLedgers(): NonNullable<LedgerSurface['checkLedgers']> {
  if (typeof LEDGER.checkLedgers !== 'function') {
    throw new Error('ledger.js exports no checkLedgers yet');
  }
  return LEDGER.checkLedgers;
}

const readAdr = (name: string): string =>
  readFileSync(join(REPO_ROOT, 'docs', 'adrs', name), 'utf8');

describe('the shipped ledger corpus (standing CI leg)', () => {
  // proves: Test strategy #10 — `checkLedgers` over this repository is green:
  // the join holds across both normative trees, every ledger-bearing ADR's
  // units, every handle, pointer, owner, note and `depends-on` in the corpus.
  it('is green: checkLedgers over this repository returns no findings', () => {
    expect(checkLedgers()(REPO_ROOT)).toEqual([]);
  });

  // proves: Test strategy #10 — the scan really sees the shipped ledgers:
  // 013's 24 units and 021's ten units parse as ledgers (A3, A2).
  it("holds 013's 24 units and 021's ten units to the same parse", () => {
    const ledger013 = parseLedger()(readAdr('013-game-manifests.md'), '013');
    expect(ledger013, '013 is ledger-bearing after the conformance pass').toBeDefined();
    expect(ledger013?.units).toHaveLength(24);

    const ledger021 = parseLedger()(readAdr('021-decision-unit-ledger.md'), '021');
    expect(ledger021, '021 is ledger-bearing after the conformance pass').toBeDefined();
    expect(ledger021?.units).toHaveLength(10);
  });

  // proves: Test strategy #10 (¶6) — the template is corpus but never a
  // ledger: it matches no `NNN-`, so `parseLedger` answers undefined for it
  // even once its frontmatter carries `ledger: yes`.
  it('never reads the template as a ledger', () => {
    expect(parseLedger()(readAdr('template.md'), 'template')).toBeUndefined();
  });
});
