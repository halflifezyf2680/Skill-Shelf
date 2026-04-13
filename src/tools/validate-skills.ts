import fs from "node:fs/promises";
import path from "node:path";
import matter from "gray-matter";

import { SkillRegistry } from "../registry/registry.js";
import type { SkillValidationIssue, SkillValidationResult } from "../types.js";

export async function validateSkills(params: {
  registry: SkillRegistry;
  skill?: string;
}): Promise<SkillValidationResult> {
  const records = params.skill
    ? [resolveSingleSkill(params.registry, params.skill)]
    : params.registry.listSkillRecords();

  const issues: SkillValidationIssue[] = [];
  const nameToRecords = new Map<string, typeof records>();

  for (const record of records) {
    const key = record.skillName.trim().toLowerCase();
    const list = nameToRecords.get(key) ?? [];
    list.push(record);
    nameToRecords.set(key, list);

    const skillFileExists = await exists(record.skillPath);
    if (!skillFileExists) {
      issues.push({
        skillId: record.skillId,
        path: record.skillPath,
        code: "missing_skill_file",
        severity: "blocked",
        message: "SKILL.md is missing from package path",
      });
      continue;
    }

    try {
      const raw = await fs.readFile(record.skillPath, "utf8");
      const parsed = matter(raw);
      if (typeof parsed.data.name !== "string" || typeof parsed.data.description !== "string") {
        issues.push({
          skillId: record.skillId,
          path: record.skillPath,
          code: "invalid_frontmatter",
          severity: "blocked",
          message: "frontmatter missing required name/description",
        });
      }
    } catch (error) {
      issues.push({
        skillId: record.skillId,
        path: record.skillPath,
        code: "invalid_frontmatter",
        severity: "blocked",
        message: error instanceof Error ? error.message : String(error),
      });
    }

    if (record.group === "general") {
      issues.push({
        skillId: record.skillId,
        path: record.skillPath,
        code: "generic_group",
        severity: "review_required",
        message: "skill resolved to generic group and may need manual grouping refinement",
      });
    }
  }

  for (const duplicates of nameToRecords.values()) {
    if (duplicates.length <= 1) {
      continue;
    }
    for (const record of duplicates) {
      issues.push({
        skillId: record.skillId,
        path: record.skillPath,
        code: "duplicate_skill_name",
        severity: "review_required",
        message: `duplicate skillName detected: ${record.skillName}`,
      });
    }
  }

  const blocked = issues.filter((issue) => issue.severity === "blocked");
  const reviewRequired = issues.filter((issue) => issue.severity === "review_required");
  const blockedIds = new Set(blocked.map((issue) => issue.skillId).filter(Boolean));
  const reviewIds = new Set(reviewRequired.map((issue) => issue.skillId).filter(Boolean));

  const passed = records
    .filter((record) => !blockedIds.has(record.skillId) && !reviewIds.has(record.skillId))
    .map((record) => ({
      skillId: record.skillId,
      skillPath: record.skillPath,
    }));

  return {
    passed,
    reviewRequired,
    blocked,
    issues,
  };
}

function resolveSingleSkill(registry: SkillRegistry, skill: string) {
  const record = registry.getById(skill) ?? registry.getByName(skill);
  if (!record) {
    throw new Error(`unknown skill: ${skill}`);
  }
  return record;
}

async function exists(targetPath: string): Promise<boolean> {
  try {
    await fs.access(targetPath);
    return true;
  } catch {
    return false;
  }
}
