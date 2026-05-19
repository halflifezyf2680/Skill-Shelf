/**
 * One-time reclassification script.
 * Moves skills from wrong group directories to correct ones based on matchManagedGroup.
 *
 * Usage: npx tsx scripts/reclassify.ts [--dry-run]
 */
import fs from "node:fs/promises";
import fsSync from "node:fs";
import path from "node:path";
import matter from "gray-matter";

import { loadConfig, ensureStorageLayout } from "../src/config.js";
import { loadManagedGroups, matchManagedGroup } from "../src/groups/group-catalog.js";
import type { ManagedGroupRecord } from "../src/types.js";

const SKILL_FILENAME = "SKILL.md";
const dryRun = process.argv.includes("--dry-run");

async function main() {
  const config = loadConfig();
  await ensureStorageLayout(config.storage);
  const packagesRoot = config.storage.packagesRoot;
  const managedGroups = await loadManagedGroups(config.storage);

  // Collect all skill files
  const skills: Array<{
    skillPath: string;
    skillDir: string;
    skillId: string;
    currentGroup: string;
    correctGroup: string;
    skillName: string;
  }> = [];

  const dirs = fsSync.readdirSync(packagesRoot, { withFileTypes: true });
  for (const groupDir of dirs) {
    if (!groupDir.isDirectory() || groupDir.name.startsWith(".")) continue;
    const groupPath = path.join(packagesRoot, groupDir.name);
    const skillDirs = fsSync.readdirSync(groupPath, { withFileTypes: true });
    for (const skillDir of skillDirs) {
      if (!skillDir.isDirectory() || skillDir.name.startsWith(".")) continue;
      const skillPath = path.join(groupPath, skillDir.name, SKILL_FILENAME);
      if (!fsSync.existsSync(skillPath)) continue;

      const raw = fsSync.readFileSync(skillPath, "utf8");
      const parsed = matter(raw);
      const name = (parsed.data.name as string)?.trim() ?? skillDir.name;
      const description = (parsed.data.description as string)?.trim() ?? "";
      const keywords = deriveKeywords(description, 12);

      const { group: correctGroup } = matchManagedGroup({
        skillName: name,
        description,
        keywords,
        groups: managedGroups,
      });

      skills.push({
        skillPath,
        skillDir: path.join(groupPath, skillDir.name),
        skillId: skillDir.name,
        currentGroup: groupDir.name,
        correctGroup,
        skillName: name,
      });
    }
  }

  // Find mismatches
  const mismatches = skills.filter((s) => s.currentGroup !== s.correctGroup);

  console.log(`Total skills: ${skills.length}`);
  console.log(`Mismatches: ${mismatches.length}`);
  console.log("");

  if (mismatches.length === 0) {
    console.log("All skills are correctly classified. Nothing to do.");
    return;
  }

  // Group by target for readability
  const byTarget = new Map<string, typeof mismatches>();
  for (const m of mismatches) {
    const key = `${m.currentGroup} → ${m.correctGroup}`;
    if (!byTarget.has(key)) byTarget.set(key, []);
    byTarget.get(key)!.push(m);
  }

  console.log("=== Reclassification Plan ===\n");
  for (const [move, items] of byTarget) {
    console.log(`${move} (${items.length} skills):`);
    for (const item of items) {
      console.log(`  - ${item.skillId}: ${item.skillName}`);
    }
    console.log("");
  }

  if (dryRun) {
    console.log("Dry run — no changes made.");
    return;
  }

  // Execute moves
  let moved = 0;
  let errors = 0;
  for (const m of mismatches) {
    const targetDir = path.join(packagesRoot, m.correctGroup, m.skillId);
    try {
      await fs.mkdir(path.join(packagesRoot, m.correctGroup), { recursive: true });
      await fs.rename(m.skillDir, targetDir);
      moved++;
    } catch (err) {
      console.error(`  ERROR moving ${m.skillId}: ${err}`);
      errors++;
    }
  }

  console.log(`\nMoved: ${moved}, Errors: ${errors}`);

  // Clean up empty group directories
  for (const groupDir of dirs) {
    if (!groupDir.isDirectory()) continue;
    const groupPath = path.join(packagesRoot, groupDir.name);
    try {
      const remaining = fsSync.readdirSync(groupPath);
      if (remaining.length === 0) {
        console.log(`Empty directory: ${groupDir.name}/ (left for managed group)`);
      }
    } catch {}
  }
}

function deriveKeywords(description: string, maxKeywords: number): string[] {
  const parts = description
    .toLowerCase()
    .split(/[\s,]+/)
    .map((part) => part.trim())
    .map((part) => part.replace(/^[^a-z0-9一-鿿-]+|[^a-z0-9一-鿿-]+$/giu, ""))
    .filter((part) => part.length >= 3);
  return Array.from(new Set(parts)).slice(0, Math.max(1, maxKeywords));
}

void main();
