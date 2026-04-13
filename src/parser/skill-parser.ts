import fs from "node:fs/promises";
import matter from "gray-matter";
import { z } from "zod";

import { matchManagedGroup } from "../groups/group-catalog.js";
import type { ParsedSkill } from "../types.js";
import type { ManagedGroupRecord } from "../types.js";

const frontmatterSchema = z.object({
  name: z.string().trim().min(1),
  description: z.string().trim().min(1),
});

export async function parseSkillFile(
  skillFilePath: string,
  managedGroups: ManagedGroupRecord[],
  maxKeywords: number,
): Promise<ParsedSkill> {
  const raw = await fs.readFile(skillFilePath, "utf8");
  const parsed = matter(raw);
  const frontmatter = frontmatterSchema.parse(parsed.data);
  const stat = await fs.stat(skillFilePath);

  const normalizedDescription = normalizeDescription(frontmatter.description);
  const keywords = deriveKeywords(normalizedDescription, maxKeywords);
  const skillName = frontmatter.name.trim();
  const { group, groupDescription } = matchManagedGroup({
    skillName,
    description: normalizedDescription,
    keywords,
    groups: managedGroups,
  });

  return {
    skillName,
    description: normalizedDescription,
    group,
    groupDescription,
    keywords,
    skillPath: skillFilePath,
    updatedAtMs: stat.mtimeMs,
  };
}

function normalizeDescription(description: string): string {
  return description.replace(/\s+/g, " ").trim();
}

function deriveKeywords(description: string, maxKeywords: number): string[] {
  const parts = description
    .toLowerCase()
    .split(/[\s,]+/)
    .map((part) => part.trim())
    .map((part) => part.replace(/^[^a-z0-9\u4e00-\u9fff-]+|[^a-z0-9\u4e00-\u9fff-]+$/giu, ""))
    .filter((part) => part.length >= 3);

  return Array.from(new Set(parts)).slice(0, Math.max(1, maxKeywords));
}
