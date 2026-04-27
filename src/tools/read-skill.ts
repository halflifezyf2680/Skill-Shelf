import fs from "node:fs/promises";
import path from "node:path";

import type { SkillRegistry } from "../registry/registry.js";

function stripFrontmatter(body: string): string {
  const match = body.match(/^---\r?\n[\s\S]*?\r?\n---\r?\n?/);
  return match ? body.slice(match[0].length) : body;
}

export async function readSkill(
  registry: SkillRegistry,
  idOrName: string,
  maxRelatedSkills = 5,
) {
  const record = registry.getById(idOrName) ?? registry.getByName(idOrName);
  if (!record) {
    throw new Error(`unknown skill: ${idOrName}`);
  }

  const raw = await fs.readFile(record.skillPath, "utf8");
  const contents = stripFrontmatter(raw);

  const skillDir = path.dirname(record.skillPath);
  const assets = await listRelativeFiles(path.join(skillDir, "assets"));
  const references = await listRelativeFiles(path.join(skillDir, "references"));
  const relatedSkills = registry.listRelatedSkills(record.skillId, maxRelatedSkills);

  return {
    skillId: record.skillId,
    skillName: record.skillName,
    group: record.group,
    keywords: record.keywords,
    skillPath: record.skillPath,
    contents,
    assets,
    references,
    relatedSkills,
  };
}

async function listRelativeFiles(rootDir: string): Promise<string[]> {
  try {
    const stat = await fs.stat(rootDir);
    if (!stat.isDirectory()) {
      return [];
    }
  } catch {
    return [];
  }

  const files: string[] = [];
  await walk(rootDir, rootDir, files);
  return files.sort();
}

async function walk(baseDir: string, currentDir: string, files: string[]): Promise<void> {
  const entries = await fs.readdir(currentDir, { withFileTypes: true });
  for (const entry of entries) {
    if (entry.name.startsWith(".")) {
      continue;
    }
    const fullPath = path.join(currentDir, entry.name);
    if (entry.isDirectory()) {
      await walk(baseDir, fullPath, files);
      continue;
    }
    if (entry.isFile()) {
      files.push(path.relative(baseDir, fullPath).replaceAll("\\", "/"));
    }
  }
}
