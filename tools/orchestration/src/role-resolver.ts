import { existsSync, readFileSync } from 'node:fs';
import { readFile } from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

import type { ModelRef } from '@opencode/client';
import { z } from 'zod';

const ConfigSchema = z.object({
  roles: z.record(z.string(), z.string()),
});

export type RoleConfig = z.infer<typeof ConfigSchema>;

export class RoleResolver {
  private config: RoleConfig | null = null;
  private projectConfigPath: string;
  private globalConfigPath: string;

  constructor() {
    this.projectConfigPath = join(process.cwd(), '.opencode', 'roles.json');
    this.globalConfigPath = join(homedir(), '.config', 'opencode', 'roles.json');
  }

  private async loadConfig(): Promise<RoleConfig> {
    if (this.config) {
      return this.config;
    }

    let configPath = this.globalConfigPath;
    if (existsSync(this.projectConfigPath)) {
      configPath = this.projectConfigPath;
    }

    if (!existsSync(configPath)) {
      throw new Error(`Role configuration not found at ${configPath}`);
    }

    try {
      const content = readFileSync(configPath, 'utf8');
      const parsed = ConfigSchema.parse(JSON.parse(content));
      this.config = parsed;
      return parsed;
    } catch (err) {
      throw new Error(
        `Failed to load role config from ${configPath}: ${err instanceof Error ? err.message : String(err)}`,
        { cause: err },
      );
    }
  }

  /**
   * Parses a model string (e.g. 'google/gemma#medium') into a ModelRef.
   */
  private parseModelString(modelStr: string): ModelRef {
    const [providerAndId, variant] = modelStr.split('#');
    const [providerID, id] = providerAndId.split('/');
    return {
      providerID: providerID ?? '',
      id: id ?? '',
      ...(variant ? { variant } : {}),
    };
  }

  /**
   * Resolves an agent ID, a role name, or a direct model string to a model Ref.
   *
   * @param input The agent ID (e.g. 'code/coder'), a role name (e.g. 'security'), or a model string.
   * @returns The model Ref or undefined if no mapping exists.
   */
  async resolveModel(input: string): Promise<ModelRef | undefined> {
    // 1. Is it an agent ID that has a role?
    // We check the file first so that agent IDs (e.g. 'code/coder') are
    // not mistakenly parsed as model strings.
    const agentFilePath = join(process.cwd(), '.opencode', 'agents', `${input}.md`);
    if (existsSync(agentFilePath)) {
      try {
        const content = await readFile(agentFilePath, 'utf8');
        // Extract YAML frontmatter only
        const fmMatch = content.match(/^---\s*\n([\s\S]*?)\n---/);
        const fm = fmMatch ? fmMatch[1] : content;
        // Simple regex for YAML frontmatter 'role: <name>'
        const match = fm.match(/^role:\s*(.+)$/m);
        if (match && match[1]) {
          const role = match[1].trim();
          try {
            const cfg = await this.loadConfig();
            if (cfg.roles[role]) {
              return this.parseModelString(cfg.roles[role]);
            }
          } catch {
            // Config load failure – fall through to other checks
          }
        }
      } catch {
        // If we can't read the file, we fall through to other checks.
      }
    }

    // 2. Is it a direct model string? (e.g. 'anthropic/claude-3-5-sonnet')
    // We check this before loading config so it works even if no config exists.
    if (input.includes('/')) {
      return this.parseModelString(input);
    }

    // 3. Is it a role name in the roles map?
    try {
      const cfg = await this.loadConfig();
      if (cfg.roles[input]) {
        return this.parseModelString(cfg.roles[input]);
      }
      return undefined;
    } catch {
      // If config is missing or invalid, we fall back to no role.
      return undefined;
    }
  }
}

export const roleResolver = new RoleResolver();
