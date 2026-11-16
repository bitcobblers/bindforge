import { existsSync, readFileSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { homedir } from 'node:os';

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { RoleResolver } from '../src/role-resolver.js';

vi.mock('node:os', () => ({
  homedir: vi.fn(() => '/home/fakeuser'),
}));

vi.mock('node:fs', () => ({
  existsSync: vi.fn(),
  readFileSync: vi.fn(),
}));

vi.mock('node:fs/promises', () => ({
  readFile: vi.fn(),
}));

const mockHomedir = vi.mocked(homedir);
const mockExistsSync = vi.mocked(existsSync);
const mockReadFileSync = vi.mocked(readFileSync);
const mockReadFile = vi.mocked(readFile);

describe('RoleResolver', () => {
  let resolver: RoleResolver;

  beforeEach(() => {
    vi.clearAllMocks();
    mockHomedir.mockReturnValue('/home/fakeuser');
    resolver = new RoleResolver();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('resolveModel', () => {
    it('resolves a direct model string', async () => {
      const result = await resolver.resolveModel('anthropic/claude-3-5-sonnet');

      expect(result).toEqual({
        providerID: 'anthropic',
        id: 'claude-3-5-sonnet',
      });
    });

    it('resolves a model string with a variant', async () => {
      const result = await resolver.resolveModel('anthropic/claude-3-5-sonnet#low');

      expect(result).toEqual({
        providerID: 'anthropic',
        id: 'claude-3-5-sonnet',
        variant: 'low',
      });
    });

    it('resolves a role alias to its model', async () => {
      mockExistsSync.mockImplementation((path) => {
        const p = String(path);
        if (p.includes('.opencode/roles.json')) return true;
        return false;
      });

      mockReadFileSync.mockReturnValue(
        JSON.stringify({
          roles: {
            'security-specialist': 'anthropic/claude-3-5-sonnet',
          },
        }),
      );

      const result = await resolver.resolveModel('security-specialist');

      expect(result).toEqual({
        providerID: 'anthropic',
        id: 'claude-3-5-sonnet',
      });
    });

    it('resolves an agent ID via its role frontmatter', async () => {
      mockExistsSync.mockImplementation((path) => {
        const p = String(path);
        if (p.includes('.opencode/roles.json')) return true;
        if (p.includes('.opencode/agents/code/coder.md')) return true;
        return false;
      });

      mockReadFileSync.mockReturnValue(
        JSON.stringify({
          roles: {
            coder: 'llama/gemma-q6',
          },
        }),
      );

      mockReadFile.mockResolvedValue('---\nrole: coder\n---\nBody content');

      const result = await resolver.resolveModel('code/coder');

      expect(result).toEqual({
        providerID: 'llama',
        id: 'gemma-q6',
      });
    });

    it('falls back to global config if project config is missing', async () => {
      mockExistsSync.mockImplementation((path) => {
        const p = String(path);
        if (p.includes('.config/opencode/roles.json')) return true;
        return false;
      });

      mockReadFileSync.mockReturnValue(
        JSON.stringify({
          roles: {
            'global-role': 'openai/gpt-4o',
          },
        }),
      );

      const result = await resolver.resolveModel('global-role');

      expect(result).toEqual({
        providerID: 'openai',
        id: 'gpt-4o',
      });
    });

    it('prefers project config over global config', async () => {
      mockExistsSync.mockImplementation((path) => {
        const p = String(path);
        if (p.includes('roles.json')) return true;
        return false;
      });

      mockReadFileSync.mockImplementation((path) => {
        const p = String(path);
        if (p.includes('.opencode/roles.json')) {
          return JSON.stringify({
            roles: {
              'shared-role': 'project-model',
            },
          });
        }
        return JSON.stringify({
          roles: {
            'shared-role': 'global-model',
          },
        });
      });

      const result = await resolver.resolveModel('shared-role');

      expect(result).toEqual({
        providerID: 'project-model',
        id: '',
      });
    });

    it('returns undefined for unknown roles', async () => {
      mockExistsSync.mockImplementation((path) => {
        return String(path).includes('roles.json');
      });

      mockReadFileSync.mockReturnValue(
        JSON.stringify({
          roles: {
            'other-role': 'openai/gpt-4o',
          },
        }),
      );

      const result = await resolver.resolveModel('unknown-role');

      expect(result).toBeUndefined();
    });

    it('returns undefined if no config file exists', async () => {
      mockExistsSync.mockReturnValue(false);

      const result = await resolver.resolveModel('any-role');

      expect(result).toBeUndefined();
    });

    it('handles invalid JSON in config gracefully', async () => {
      mockExistsSync.mockReturnValue(true);
      mockReadFileSync.mockReturnValue('not-json');

      const result = await resolver.resolveModel('any-role');

      expect(result).toBeUndefined();
    });
  });
});
