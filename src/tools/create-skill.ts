import fs from "node:fs/promises";
import path from "node:path";

import type { SkillHubStorageLayout } from "../config.js";
import { SkillRegistry } from "../registry/registry.js";
import type { SkillCreateResult } from "../types.js";

export async function createSkill(params: {
  storage: SkillHubStorageLayout;
  registry: SkillRegistry;
  name: string;
  description: string;
  skillMarkdown: string;
}): Promise<SkillCreateResult> {
  const skillId = sanitizeId(params.name);
  if (!skillId) {
    throw new Error("skill id is empty after normalization");
  }

  const skillDir = path.join(params.storage.packagesRoot, skillId);
  await fs.rm(skillDir, { recursive: true, force: true });
  await fs.mkdir(skillDir, { recursive: true });

  const skillPath = path.join(skillDir, "SKILL.md");
  const markdown = ensureFrontmatter(params.name, params.description, params.skillMarkdown);
  await fs.writeFile(skillPath, markdown, "utf8");

  await params.registry.rebuild();
  const record = params.registry.getById(skillId);
  if (!record) {
    throw new Error(`created skill missing from registry: ${skillId}`);
  }

  return {
    skillId: record.skillId,
    skillPath: record.skillPath,
    group: record.group,
    keywords: record.keywords,
  };
}

function ensureFrontmatter(name: string, description: string, markdown: string): string {
  const trimmed = markdown.trimStart();
  if (trimmed.startsWith("---")) {
    return markdown;
  }
  return [
    "---",
    `name: ${name}`,
    `description: ${description}`,
    "---",
    "",
    markdown.trim(),
    "",
  ].join("\n");
}

function sanitizeId(value: string): string {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9-_]+/g, "-")
    .replace(/-+/g, "-")
    .replace(/^-|-$/g, "");
}
