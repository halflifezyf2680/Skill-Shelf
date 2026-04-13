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
        const parsed = await parseSkillFile(
          skillFilePath,
          this.managedGroups,
          this.indexPolicy.maxKeywordsPerSkill,
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
      const parsed = await parseSkillFile(
        skillFilePath,
        this.managedGroups,
        this.indexPolicy.maxKeywordsPerSkill,
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
    await this.rebuild();
    return {
      action: "deleted",
      group: target.group,
    };
  }

  removeByPath(skillFilePath: string): void {
    const match = Array.from(this.skillRecords.values()).find((record) => record.skillPath === skillFilePath);
    if (!match) {
      return;
    }
    this.skillRecords.delete(match.skillId);
    this.rebuildGroupsFromSkills();
  }

  searchGroups(query: string, limit: number): GroupSearchResult[] {
    const normalized = query.trim().toLowerCase();
    const records = Array.from(this.groupList.values());
    const queryTokens = normalized.split(/[^a-z0-9\u4e00-\u9fff]+/u).filter((t) => t.length >= 2);

    if (!normalized || queryTokens.length === 0) {
      return records.slice(0, limit).map((record) => ({
        ...record,
        skillNames: (this.groupSkills.get(record.group) ?? []).map((s) => s.skillName),
        directMatch: null,
      }));
    }

    const results: GroupSearchResult[] = [];

    for (const record of records) {
      const skills = this.groupSkills.get(record.group) ?? [];
      const groupText = `${record.group} ${record.groupDescription}`.toLowerCase();

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
            const skillRecord = this.skillRecords.get(s.skillId);
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
          skillNames: skills.map((s) => s.skillName),
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

  listGroupSkills(group: string): GroupSkillsResult | null {
    const groupRecord = this.groupList.get(group);
    if (!groupRecord) {
      return null;
    }
    return {
      group: groupRecord.group,
      groupDescription: groupRecord.groupDescription,
      skills: [...(this.groupSkills.get(group) ?? [])].sort((a, b) => a.skillName.localeCompare(b.skillName)),
    };
  }

  getById(id: string): SkillRecord | null {
    return this.skillRecords.get(id) ?? null;
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
    const record = this.skillRecords.get(skillId);
    if (!record) {
      return [];
    }
    return (this.groupSkills.get(record.group) ?? [])
      .filter((entry) => entry.skillId !== skillId)
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
    return Array.from(this.skillRecords.values()).sort((a, b) => a.skillId.localeCompare(b.skillId));
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
      const filePath = path.join(this.storage.skillsRoot, `${record.skillId}.json`);
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
    this.skillRecords.set(skillId, {
      skillId,
      skillName: parsed.skillName,
      description: parsed.description,
      group: parsed.group,
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

    for (const group of this.managedGroups) {
      this.groupList.set(group.group, {
        group: group.group,
        groupDescription: group.groupDescription,
      });
      this.groupSkills.set(group.group, []);
    }

    for (const record of this.skillRecords.values()) {
      if (!this.groupList.has(record.group)) {
        this.groupList.set(record.group, {
          group: record.group,
          groupDescription: record.groupDescription,
        });
      }

      const groupSkills = this.groupSkills.get(record.group) ?? [];
      groupSkills.push({
        skillId: record.skillId,
        skillName: record.skillName,
        keywords: record.keywords,
        skillPath: record.skillPath,
      });
      this.groupSkills.set(record.group, groupSkills);
    }
  }
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
