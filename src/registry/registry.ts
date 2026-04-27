import fs from "node:fs/promises";
import path from "node:path";

import { type IndexPolicy, type SkillHubStorageLayout } from "../config.js";
import {
  createManagedGroup,
  deleteManagedGroup,
  loadManagedGroups,
  updateManagedGroup,
} from "../groups/group-catalog.js";
import { parseSkillFile } from "../parser/skill-parser.js";
import type {
  GroupListItem,
  GroupCreateResult,
  GroupDeleteResult,
  GroupSearchResult,
  GroupSkillIndexEntry,
  GroupSkillsResult,
  GroupUpdateResult,
  ManagedGroupRecord,
  ParsedSkill,
  RegistryIssue,
  SkillMeta,
  SkillRecord,
} from "../types.js";
import { scoreSkillText, normalize, normalizedLevenshteinScore } from "./fuzzy.js";

const SKILL_FILENAME = "SKILL.md";

export class SkillRegistry {
  private managedGroups: ManagedGroupRecord[] = [];
  private readonly skillRecords = new Map<string, SkillRecord>();
  private readonly groupList = new Map<string, GroupListItem>();
  private readonly groupSkills = new Map<string, GroupSkillIndexEntry[]>();
  private readonly issues: RegistryIssue[] = [];
  private indexUpdatedAtMs: number | null = null;

  constructor(
    private readonly storage: SkillHubStorageLayout,
    private readonly indexPolicy: IndexPolicy = {
      defaultSearchResultLimit: 8,
      maxKeywordsPerSkill: 12,
      maxRelatedSkills: 5,
    },
  ) {}

  get packagesRoot(): string {
    return this.storage.packagesRoot;
  }

  async rebuild(): Promise<void> {
    this.managedGroups = await loadManagedGroups(this.storage);
    this.skillRecords.clear();
    this.groupList.clear();
    this.groupSkills.clear();
    this.issues.length = 0;

    const skillFiles = await collectSkillFiles(this.storage.packagesRoot);
    for (const skillFilePath of skillFiles) {
      try {
        const groupFromPath = deriveGroupFromPath(skillFilePath, this.storage.packagesRoot);
        const parsed = await parseSkillFile(
          skillFilePath,
          this.managedGroups,
          this.indexPolicy.maxKeywordsPerSkill,
          groupFromPath,
        );
        this.upsertInMemory(parsed);
      } catch (error) {
        this.issues.push({
          path: skillFilePath,
          message: error instanceof Error ? error.message : String(error),
        });
      }
    }

    await this.persistIndexes();
    await this.persistPackageMetadata();
    this.indexUpdatedAtMs = Date.now();
  }

  async refreshSkillByPath(skillFilePath: string): Promise<void> {
    try {
      const groupFromPath = deriveGroupFromPath(skillFilePath, this.storage.packagesRoot);
      const parsed = await parseSkillFile(
        skillFilePath,
        this.managedGroups,
        this.indexPolicy.maxKeywordsPerSkill,
        groupFromPath,
      );
      this.upsertInMemory(parsed);
    } catch (error) {
      this.removeByPath(skillFilePath);
      this.issues.push({
        path: skillFilePath,
        message: error instanceof Error ? error.message : String(error),
      });
    }

    await this.persistIndexes();
    await this.persistPackageMetadata();
    this.indexUpdatedAtMs = Date.now();
  }

  async createGroup(input: {
    group: string;
    groupDescription: string;
    keywords?: string[];
    aliases?: string[];
  }): Promise<GroupCreateResult> {
    await fs.mkdir(path.join(this.storage.packagesRoot, input.group), { recursive: true });
    const group = await createManagedGroup(this.storage, input);
    await this.rebuild();
    return {
      action: "created",
      group,
    };
  }

  async updateGroup(input: {
    group: string;
    newGroup?: string;
    groupDescription?: string;
    keywords?: string[];
    aliases?: string[];
  }): Promise<GroupUpdateResult> {
    if (input.newGroup) {
      const oldDir = path.join(this.storage.packagesRoot, input.group);
      const newDir = path.join(this.storage.packagesRoot, input.newGroup);
      await fs.rename(oldDir, newDir);
    }
    const result = await updateManagedGroup(this.storage, input);
    await this.rebuild();
    return {
      action: "updated",
      previousGroup: result.previousGroup,
      group: result.group,
    };
  }

  async deleteGroup(input: { group: string }): Promise<GroupDeleteResult> {
    const target = this.managedGroups.find((entry) => entry.group === input.group);
    if (!target) {
      throw new Error(`unknown group: ${input.group}`);
    }
    const skillCount = (this.groupSkills.get(target.group) ?? []).length;
    if (skillCount > 0) {
      throw new Error(`group is not empty: ${target.group} (${skillCount} skills)`);
    }

    await deleteManagedGroup(this.storage, input);
    const groupDir = path.join(this.storage.packagesRoot, target.group);
    await fs.rm(groupDir, { recursive: true, force: true });
    await this.rebuild();
    return {
      action: "deleted",
      group: target.group,
    };
  }

  removeByPath(skillFilePath: string): void {
    let match: SkillRecord | undefined;
    for (const record of this.skillRecords.values()) {
      if (record.skillPath === skillFilePath) {
        match = record;
        break;
      }
    }
    if (!match) {
      return;
    }
    const compositeKey = `${match.group}/${match.skillId}`;
    this.skillRecords.delete(compositeKey);
    this.rebuildGroupsFromSkills();
  }

  searchGroups(query: string, limit: number): GroupSearchResult[] {
    const normalized = query.trim().toLowerCase();
    const records = Array.from(this.groupList.values());
    const queryTokens = normalized.split(/[^a-z0-9\u4e00-\u9fff]+/u).filter((t) => t.length >= 2);

    if (!normalized || queryTokens.length === 0) {
      return records.slice(0, limit).map((record) => ({
        ...record,
        skills: (this.groupSkills.get(record.group) ?? []).map((s) => ({
          skillId: s.skillId,
          skillName: s.skillName,
          description: s.description,
        })),
        directMatch: null,
      }));
    }

    const results: GroupSearchResult[] = [];

    for (const record of records) {
      const skills = this.groupSkills.get(record.group) ?? [];
      const groupMeta = this.managedGroups.find((g) => g.group === record.group);
      const groupText = [
        record.group,
        record.groupDescription,
        groupMeta?.keywords.join(" ") ?? "",
        groupMeta?.aliases.join(" ") ?? "",
        ...skills.map((s) => `${s.skillName} ${s.description}`),
      ].join(" ").toLowerCase();

      // Check if any query token matches group description or any skill name
      let matched = queryTokens.some((token) => groupText.includes(token));

      // Also check skill names
      let directMatch: GroupSearchResult["directMatch"] = null;
      if (!matched) {
        for (const s of skills) {
          const nameNorm = normalize(s.skillName);
          if (nameNorm === normalized || nameNorm.includes(normalized) || queryTokens.some((t) => nameNorm.includes(t))) {
            matched = true;
            break;
          }
        }
      }

      if (matched) {
        // Check for skill name match (includes or fuzzy) → hint with description
        for (const s of skills) {
          const nameNorm = normalize(s.skillName);
          const idNorm = normalize(s.skillId);
          const isExactOrSubstring = nameNorm === normalized
            || nameNorm.includes(normalized)
            || idNorm === normalized
            || idNorm.includes(normalized);
          const isFuzzy = !isExactOrSubstring
            && (normalizedLevenshteinScore(nameNorm, normalized) <= 0.3
              || normalizedLevenshteinScore(idNorm, normalized) <= 0.3);

          if (isExactOrSubstring || isFuzzy) {
            const compositeKey = `${record.group}/${s.skillId}`;
            const skillRecord = this.skillRecords.get(compositeKey);
            directMatch = {
              skillId: s.skillId,
              skillName: s.skillName,
              description: skillRecord?.description ?? "",
            };
            break;
          }
        }

        results.push({
          ...record,
          skills: skills.map((s) => ({
            skillId: s.skillId,
            skillName: s.skillName,
            description: s.description,
          })),
          directMatch,
        });
      }

      if (results.length >= limit) break;
    }

    return results;
  }

  listGroups(): GroupListItem[] {
    return Array.from(this.groupList.values()).sort((a, b) => a.group.localeCompare(b.group));
  }

  listManagedGroups(): ManagedGroupRecord[] {
    return [...this.managedGroups].sort((a, b) => a.group.localeCompare(b.group));
  }

  listGroupSkills(group: string, query?: string): GroupSkillsResult | null {
    const groupRecord = this.groupList.get(group);
    if (!groupRecord) {
      return null;
    }

    let skills = [...(this.groupSkills.get(group) ?? [])];

    if (query && query.trim()) {
      const normalized = normalize(query.trim());
      const queryTokens = normalized
        .split(/[^a-z0-9\u4e00-\u9fff]+/u)
        .filter((t) => t.length >= 2);

      skills = skills.filter((s) => {
        const text = `${s.skillName} ${s.description}`.toLowerCase();
        if (text.includes(normalized)) return true;
        if (queryTokens.some((t) => text.includes(t))) return true;
        if (normalizedLevenshteinScore(normalize(s.skillName), normalized) <= 0.3) return true;
        return false;
      });
    }

    return {
      group: groupRecord.group,
      groupDescription: groupRecord.groupDescription,
      skills: skills.sort((a, b) => a.skillName.localeCompare(b.skillName)),
    };
  }

  getById(id: string): SkillRecord | null {
    // Try composite key first (group/skillId)
    const direct = this.skillRecords.get(id);
    if (direct) return direct;

    // Try bare skillId (linear scan, return first match)
    const bareId = id.includes("/") ? id.split("/").pop()! : id;
    for (const record of this.skillRecords.values()) {
      if (record.skillId === bareId) {
        return record;
      }
    }
    return null;
  }

  getByName(skillName: string): SkillRecord | null {
    const normalized = skillName.trim().toLowerCase();
    return (
      Array.from(this.skillRecords.values()).find(
        (record) => record.skillName.trim().toLowerCase() === normalized,
      ) ?? null
    );
  }

  listRelatedSkills(skillId: string, limit = 5): GroupSkillIndexEntry[] {
    // Try to find by composite key first, then bare skillId
    let record = this.skillRecords.get(skillId);
    if (!record) {
      for (const r of this.skillRecords.values()) {
        if (r.skillId === skillId) {
          record = r;
          break;
        }
      }
    }
    if (!record) {
      return [];
    }
    return (this.groupSkills.get(record.group) ?? [])
      .filter((entry) => entry.skillId !== record.skillId)
      .sort((a, b) => a.skillName.localeCompare(b.skillName))
      .slice(0, limit);
  }

  listIssues(): RegistryIssue[] {
    return [...this.issues];
  }

  size(): number {
    return this.skillRecords.size;
  }

  listSkillRecords(): SkillRecord[] {
    return Array.from(this.skillRecords.values()).sort((a, b) => {
      const cmp = a.group.localeCompare(b.group);
      return cmp !== 0 ? cmp : a.skillId.localeCompare(b.skillId);
    });
  }

  getIndexUpdatedAt(): number | null {
    return this.indexUpdatedAtMs;
  }

  async persistIndexes(): Promise<void> {
    await fs.mkdir(this.storage.indexRoot, { recursive: true });
    await fs.mkdir(this.storage.groupsRoot, { recursive: true });
    await fs.mkdir(this.storage.skillsRoot, { recursive: true });

    const groups = this.listGroups();
    await fs.writeFile(this.storage.groupListPath, JSON.stringify(groups, null, 2) + "\n", "utf8");

    await clearDirectoryJsonFiles(this.storage.groupsRoot);
    for (const groupRecord of groups) {
      const groupEntry = this.listGroupSkills(groupRecord.group);
      if (!groupEntry) {
        continue;
      }
      const filePath = path.join(this.storage.groupsRoot, `${groupRecord.group}.json`);
      await fs.writeFile(filePath, JSON.stringify(groupEntry, null, 2) + "\n", "utf8");
    }

    await clearDirectoryJsonFiles(this.storage.skillsRoot);
    for (const record of this.skillRecords.values()) {
      const indexKey = `${record.group}--${record.skillId}`;
      const filePath = path.join(this.storage.skillsRoot, `${indexKey}.json`);
      await fs.writeFile(filePath, JSON.stringify(record, null, 2) + "\n", "utf8");
    }
  }

  private async persistPackageMetadata(): Promise<void> {
    for (const record of this.skillRecords.values()) {
      const metaPath = path.join(path.dirname(record.skillPath), "meta.json");
      const meta: SkillMeta = {
        skillId: record.skillId,
        skillName: record.skillName,
        description: record.description,
        group: record.group,
        groupDescription: record.groupDescription,
        keywords: record.keywords,
        updatedAtMs: record.updatedAtMs,
        status: record.status,
      };
      await fs.writeFile(metaPath, JSON.stringify(meta, null, 2) + "\n", "utf8");
    }
  }

  private upsertInMemory(parsed: ParsedSkill): void {
    const skillId = path.basename(path.dirname(parsed.skillPath));
    const group = parsed.group;
    const compositeKey = `${group}/${skillId}`;
    this.skillRecords.set(compositeKey, {
      skillId,
      skillName: parsed.skillName,
      description: parsed.description,
      group,
      groupDescription: parsed.groupDescription,
      keywords: parsed.keywords,
      skillPath: parsed.skillPath,
      updatedAtMs: parsed.updatedAtMs,
      status: "ready",
    });
    this.rebuildGroupsFromSkills();
  }

  private rebuildGroupsFromSkills(): void {
    this.groupList.clear();
    this.groupSkills.clear();

    // Seed from managedGroups (provides metadata even for empty groups)
    for (const group of this.managedGroups) {
      this.groupList.set(group.group, {
        group: group.group,
        groupDescription: group.groupDescription,
      });
      this.groupSkills.set(group.group, []);
    }

    // Populate from actual skill records (directory structure is truth)
    for (const record of this.skillRecords.values()) {
      if (!this.groupList.has(record.group)) {
        // Group exists in directory but not in groups.json — add with description from record
        this.groupList.set(record.group, {
          group: record.group,
          groupDescription: record.groupDescription,
        });
      }

      const groupSkills = this.groupSkills.get(record.group) ?? [];
      groupSkills.push({
        skillId: record.skillId,
        skillName: record.skillName,
        description: record.description,
        keywords: record.keywords,
        skillPath: record.skillPath,
      });
      this.groupSkills.set(record.group, groupSkills);
    }
  }
}

/**
 * Derive group from file path: the immediate parent directory of SKILL.md
 * under packagesRoot is the group, and the SKILL.md's own directory is the skill.
 *
 * packagesRoot/{group}/{skill}/SKILL.md → group
 * packagesRoot/{skill}/SKILL.md         → undefined (flat structure, caller falls back)
 */
function deriveGroupFromPath(skillFilePath: string, packagesRoot: string): string | undefined {
  const skillDir = path.dirname(skillFilePath);
  const parentDir = path.dirname(skillDir);
  // Check if parentDir is packagesRoot (flat structure — no group layer)
  if (path.resolve(parentDir) === path.resolve(packagesRoot)) {
    return undefined;
  }
  // parentDir is the group directory
  return path.basename(parentDir);
}

async function collectSkillFiles(root: string): Promise<string[]> {
  const files: string[] = [];
  if (!(await exists(root))) {
    return files;
  }
  await walk(root, files);
  return files;
}

async function walk(currentPath: string, files: string[]): Promise<void> {
  const entries = await fs.readdir(currentPath, { withFileTypes: true });
  for (const entry of entries) {
    if (entry.name.startsWith(".")) {
      continue;
    }
    const nextPath = path.join(currentPath, entry.name);
    if (entry.isDirectory()) {
      await walk(nextPath, files);
      continue;
    }
    if (entry.isFile() && entry.name === SKILL_FILENAME) {
      files.push(nextPath);
    }
  }
}

async function clearDirectoryJsonFiles(targetDir: string): Promise<void> {
  if (!(await exists(targetDir))) {
    return;
  }
  const entries = await fs.readdir(targetDir, { withFileTypes: true });
  await Promise.all(
    entries
      .filter((entry) => entry.isFile() && entry.name.toLowerCase().endsWith(".json"))
      .map((entry) => fs.rm(path.join(targetDir, entry.name), { force: true })),
  );
}

async function exists(targetPath: string): Promise<boolean> {
  try {
    await fs.access(targetPath);
    return true;
  } catch {
    return false;
  }
}
