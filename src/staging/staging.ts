import type { Dirent } from "node:fs";
import fs from "node:fs/promises";
import path from "node:path";
import matter from "gray-matter";

import type { ImportCandidate, ImportCandidateStatus } from "../types.js";

const SKILL_FILENAME = "SKILL.md";
const EXCLUDED_MARKDOWN_BASENAMES = new Set([
  "readme",
  "contributing",
  "license",
  "upstream",
  "catalog",
  "agent-list",
  "quickstart",
  "executive-brief",
]);

export async function listImportCandidates(
  rawRoot: string,
  repairedRoot: string,
  statusFilter?: ImportCandidateStatus,
  limit = 50,
): Promise<ImportCandidate[]> {
  const candidates = await collectImportCandidates(rawRoot, repairedRoot);
  const filtered = statusFilter
    ? candidates.filter((candidate) => candidate.status === statusFilter)
    : candidates;
  return filtered.slice(0, limit);
}

export async function getImportCandidate(
  rawRoot: string,
  repairedRoot: string,
  id: string,
): Promise<ImportCandidate | null> {
  const candidates = await collectImportCandidates(rawRoot, repairedRoot);
  return candidates.find((candidate) => candidate.id === id) ?? null;
}

export async function readImportCandidateContents(candidate: ImportCandidate): Promise<string> {
  return await fs.readFile(candidate.sourcePath, "utf8");
}

export async function writeRepairedImport(params: {
  repairedRoot: string;
  candidate: ImportCandidate;
  skillMarkdown: string;
  targetId?: string;
  notes?: string;
}): Promise<{ targetId: string; skillPath: string; notesPath: string | null }> {
  const targetId = sanitizeId(params.targetId || params.candidate.inferredTargetId);
  if (!targetId) {
    throw new Error("targetId is empty after normalization");
  }

  const skillDir = path.join(params.repairedRoot, targetId);
  const skillPath = path.join(skillDir, SKILL_FILENAME);
  await fs.mkdir(skillDir, { recursive: true });
  await fs.writeFile(skillPath, params.skillMarkdown, "utf8");

  let notesPath: string | null = null;
  if (params.notes && params.notes.trim()) {
    notesPath = path.join(skillDir, "REPAIR_NOTES.md");
    await fs.writeFile(notesPath, params.notes.trim() + "\n", "utf8");
  }

  const provenancePath = path.join(skillDir, "SOURCE.json");
  await fs.writeFile(
    provenancePath,
    JSON.stringify(
      {
        sourcePath: params.candidate.sourcePath,
        relativePath: params.candidate.relativePath,
        candidateId: params.candidate.id,
      },
      null,
      2,
    ),
    "utf8",
  );

  return { targetId, skillPath, notesPath };
}

async function collectImportCandidates(
  rawRoot: string,
  repairedRoot: string,
): Promise<ImportCandidate[]> {
  const candidates: ImportCandidate[] = [];
  if (!(await exists(rawRoot))) {
    return candidates;
  }

  const repairedIds = await collectRepairedIds(repairedRoot);
  const packageSkillDirs = await collectPackageLikeDirs(rawRoot);

  for (const skillDir of packageSkillDirs) {
    const skillPath = path.join(skillDir, SKILL_FILENAME);
    const relativePath = path.relative(rawRoot, skillPath).replaceAll("\\", "/");
    const frontmatter = await safeReadFrontmatter(skillPath);
    const inferredTargetId = sanitizeId(path.basename(skillDir));
    const status = repairedIds.has(inferredTargetId) ? "repaired" : frontmatter.ok ? "ready" : "review_required";
    candidates.push({
      id: `package:${relativePath}`,
      sourcePath: skillPath,
      relativePath,
      inferredTargetId,
      status,
      issues: frontmatter.ok ? [] : [frontmatter.message],
      kind: "package",
      name: frontmatter.name,
      description: frontmatter.description,
    });
  }

  const packageSkillPaths = new Set(packageSkillDirs.map((dir) => path.join(dir, SKILL_FILENAME)));
  const markdownFiles = await collectMarkdownFiles(rawRoot);
  for (const markdownPath of markdownFiles) {
    if (packageSkillPaths.has(markdownPath)) {
      continue;
    }
    const basename = path.basename(markdownPath, path.extname(markdownPath)).toLowerCase();
    if (EXCLUDED_MARKDOWN_BASENAMES.has(basename)) {
      continue;
    }
    const relativePath = path.relative(rawRoot, markdownPath).replaceAll("\\", "/");
    const frontmatter = await safeReadFrontmatter(markdownPath);
    const inferredTargetId = sanitizeId(path.basename(markdownPath, path.extname(markdownPath)));
    const issues = frontmatter.ok
      ? ["raw markdown candidate requires conversion into a skill package"]
      : [frontmatter.message];
    const status: ImportCandidateStatus = repairedIds.has(inferredTargetId)
      ? "repaired"
      : frontmatter.ok
        ? "review_required"
        : "blocked";
    candidates.push({
      id: `markdown:${relativePath}`,
      sourcePath: markdownPath,
      relativePath,
      inferredTargetId,
      status,
      issues,
      kind: "markdown",
      name: frontmatter.name,
      description: frontmatter.description,
    });
  }

  return candidates.sort((a, b) => a.relativePath.localeCompare(b.relativePath));
}

async function collectPackageLikeDirs(root: string): Promise<string[]> {
  const dirs: string[] = [];
  await walk(root, async (entryPath, dirent) => {
    if (!dirent.isDirectory()) {
      return;
    }
    const skillPath = path.join(entryPath, SKILL_FILENAME);
    if (await exists(skillPath)) {
      dirs.push(entryPath);
    }
  });
  return dirs;
}

async function collectMarkdownFiles(root: string): Promise<string[]> {
  const files: string[] = [];
  await walk(root, async (entryPath, dirent) => {
    if (dirent.isFile() && entryPath.toLowerCase().endsWith(".md")) {
      files.push(entryPath);
    }
  });
  return files;
}

async function collectRepairedIds(repairedRoot: string): Promise<Set<string>> {
  const ids = new Set<string>();
  if (!(await exists(repairedRoot))) {
    return ids;
  }
  const entries = await fs.readdir(repairedRoot, { withFileTypes: true });
  for (const entry of entries) {
    if (entry.isDirectory()) {
      ids.add(entry.name);
    }
  }
  return ids;
}

async function walk(
  currentPath: string,
  visit: (entryPath: string, dirent: Dirent) => Promise<void>,
): Promise<void> {
  const entries = await fs.readdir(currentPath, { withFileTypes: true });
  for (const entry of entries) {
    if (entry.name.startsWith(".")) {
      continue;
    }
    const entryPath = path.join(currentPath, entry.name);
    await visit(entryPath, entry);
    if (entry.isDirectory()) {
      await walk(entryPath, visit);
    }
  }
}

async function safeReadFrontmatter(filePath: string): Promise<{
  ok: boolean;
  message: string;
  name: string | null;
  description: string | null;
}> {
  try {
    const raw = await fs.readFile(filePath, "utf8");
    const parsed = matter(raw);
    const name = typeof parsed.data.name === "string" ? parsed.data.name.trim() : null;
    const description =
      typeof parsed.data.description === "string" ? parsed.data.description.trim() : null;
    if (!name || !description) {
      return {
        ok: false,
        message: "frontmatter is missing required name/description",
        name,
        description,
      };
    }
    return {
      ok: true,
      message: "ok",
      name,
      description,
    };
  } catch (error) {
    return {
      ok: false,
      message: error instanceof Error ? error.message : String(error),
      name: null,
      description: null,
    };
  }
}

async function exists(targetPath: string): Promise<boolean> {
  try {
    await fs.access(targetPath);
    return true;
  } catch {
    return false;
  }
}

function sanitizeId(value: string): string {
  return value.toLowerCase().replace(/[^a-z0-9-_]+/g, "-").replace(/-+/g, "-").replace(/^-|-$/g, "");
}
