import fs from "node:fs/promises";
import path from "node:path";
import matter from "gray-matter";

import type { InstallPolicy, SkillHubStorageLayout } from "../config.js";
import { SkillRegistry } from "../registry/registry.js";
import type { SkillInstallEntry, SkillInstallFailure, SkillInstallResult } from "../types.js";

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

export async function installSkills(params: {
  storage: SkillHubStorageLayout;
  registry: SkillRegistry;
  sourcePath: string;
  policy?: InstallPolicy;
}): Promise<SkillInstallResult> {
  const sourcePath = path.resolve(params.sourcePath);
  const policy = params.policy ?? {
    acceptPackageDirectories: true,
    acceptRawMarkdown: true,
    packagePrecedence: "package-first",
    rawMarkdownRequiresFrontmatter: true,
  };
  const result: SkillInstallResult = {
    installed: [],
    skipped: [],
    failed: [],
  };

  const installables = await collectInstallables(sourcePath, policy);
  if (installables.length === 0) {
    result.failed.push({
      sourcePath,
      status: "failed",
      message: "no installable skill package or raw markdown candidate found under sourcePath",
    });
    return result;
  }

  for (const installable of installables) {
    try {
      const skillId = installable.skillId;
      if (!skillId) {
        result.failed.push({
          sourcePath: installable.sourcePath,
          status: "failed",
          message: "skill id is empty after normalization",
        });
        continue;
      }

      const destinationDir = path.join(params.storage.packagesRoot, skillId);
      await fs.rm(destinationDir, { recursive: true, force: true });
      if (installable.kind === "package") {
        await copyDirectory(installable.sourcePath, destinationDir);
      } else {
        await writeMarkdownCandidateAsPackage(installable.sourcePath, destinationDir);
      }

      const installedEntry: SkillInstallEntry = {
        sourcePath: installable.sourcePath,
        skillId,
        installedPath: destinationDir,
        status: "installed",
      };
      result.installed.push(installedEntry);
    } catch (error) {
      result.failed.push({
        sourcePath: installable.sourcePath,
        status: "failed",
        message: error instanceof Error ? error.message : String(error),
      });
    }
  }

  if (result.installed.length > 0) {
    await params.registry.rebuild();
  }

  return result;
}

type InstallableSource =
  | {
      kind: "package";
      sourcePath: string;
      skillId: string;
    }
  | {
      kind: "markdown";
      sourcePath: string;
      skillId: string;
    };

async function collectInstallables(
  sourcePath: string,
  policy: InstallPolicy,
): Promise<InstallableSource[]> {
  const stat = await fs.stat(sourcePath);
  if (stat.isFile()) {
    if (policy.acceptPackageDirectories && path.basename(sourcePath) === SKILL_FILENAME) {
      return [
        {
          kind: "package",
          sourcePath: path.dirname(sourcePath),
          skillId: sanitizeId(path.basename(path.dirname(sourcePath))),
        },
      ];
    }
    if (
      policy.acceptRawMarkdown &&
      isMarkdownCandidate(sourcePath) &&
      (policy.rawMarkdownRequiresFrontmatter ? await hasRequiredFrontmatter(sourcePath) : true)
    ) {
      return [
        {
          kind: "markdown",
          sourcePath,
          skillId: sanitizeId(path.basename(sourcePath, path.extname(sourcePath))),
        },
      ];
    }
    return [];
  }

  const directSkillPath = path.join(sourcePath, SKILL_FILENAME);
  if (policy.acceptPackageDirectories && (await exists(directSkillPath))) {
    return [
      {
        kind: "package",
        sourcePath,
        skillId: sanitizeId(path.basename(sourcePath)),
      },
    ];
  }

  const installables: InstallableSource[] = [];
  await walk(sourcePath, async (entryPath, isDirectory) => {
    if (isDirectory) {
      const skillPath = path.join(entryPath, SKILL_FILENAME);
      if (policy.acceptPackageDirectories && (await exists(skillPath))) {
        installables.push({
          kind: "package",
          sourcePath: entryPath,
          skillId: sanitizeId(path.basename(entryPath)),
        });
      }
      return;
    }

    // Skip SKILL.md files — they are part of a package directory handled above,
    // not standalone markdown candidates.
    if (path.basename(entryPath) === SKILL_FILENAME) {
      return;
    }

    if (
      !policy.acceptRawMarkdown ||
      !isMarkdownCandidate(entryPath) ||
      (policy.rawMarkdownRequiresFrontmatter && !(await hasRequiredFrontmatter(entryPath)))
    ) {
      return;
    }

    const relativeWithoutExt = path
      .relative(sourcePath, entryPath)
      .replaceAll("\\", "/")
      .replace(/\.[^.]+$/, "");
    installables.push({
      kind: "markdown",
      sourcePath: entryPath,
      skillId: sanitizeId(relativeWithoutExt),
    });
  });
  return dedupeInstallables(installables, policy);
}

async function walk(
  currentPath: string,
  visit: (entryPath: string, isDirectory: boolean) => Promise<void>,
): Promise<void> {
  const entries = await fs.readdir(currentPath, { withFileTypes: true });
  for (const entry of entries) {
    if (entry.name.startsWith(".")) {
      continue;
    }
    const entryPath = path.join(currentPath, entry.name);
    await visit(entryPath, entry.isDirectory());
    if (entry.isDirectory()) {
      await walk(entryPath, visit);
    }
  }
}

async function writeMarkdownCandidateAsPackage(sourceFile: string, destinationDir: string): Promise<void> {
  await fs.mkdir(destinationDir, { recursive: true });
  const sourceRaw = await fs.readFile(sourceFile, "utf8");
  const parsed = matter(sourceRaw);
  const name = typeof parsed.data.name === "string" ? parsed.data.name.trim() : "";
  const description = typeof parsed.data.description === "string" ? parsed.data.description.trim() : "";
  if (!name || !description) {
    throw new Error("raw markdown candidate is missing required frontmatter name/description");
  }

  const skillPath = path.join(destinationDir, SKILL_FILENAME);
  await fs.writeFile(skillPath, sourceRaw, "utf8");
  await fs.writeFile(
    path.join(destinationDir, "SOURCE.json"),
    JSON.stringify(
      {
        sourcePath: sourceFile,
        kind: "markdown",
      },
      null,
      2,
    ) + "\n",
    "utf8",
  );
}

async function copyDirectory(sourceDir: string, destinationDir: string): Promise<void> {
  await fs.mkdir(destinationDir, { recursive: true });
  const entries = await fs.readdir(sourceDir, { withFileTypes: true });
  for (const entry of entries) {
    const sourcePath = path.join(sourceDir, entry.name);
    const destinationPath = path.join(destinationDir, entry.name);
    if (entry.isDirectory()) {
      await copyDirectory(sourcePath, destinationPath);
    } else if (entry.isFile()) {
      await fs.copyFile(sourcePath, destinationPath);
    }
  }
}

async function hasRequiredFrontmatter(filePath: string): Promise<boolean> {
  try {
    const raw = await fs.readFile(filePath, "utf8");
    const parsed = matter(raw);
    return (
      typeof parsed.data.name === "string" &&
      parsed.data.name.trim().length > 0 &&
      typeof parsed.data.description === "string" &&
      parsed.data.description.trim().length > 0
    );
  } catch {
    return false;
  }
}

function isMarkdownCandidate(filePath: string): boolean {
  if (path.extname(filePath).toLowerCase() !== ".md") {
    return false;
  }
  const basename = path.basename(filePath, path.extname(filePath)).toLowerCase();
  return !EXCLUDED_MARKDOWN_BASENAMES.has(basename);
}

function dedupeInstallables(
  installables: InstallableSource[],
  policy: InstallPolicy,
): InstallableSource[] {
  const bySkillId = new Map<string, InstallableSource>();
  for (const installable of installables) {
    const existing = bySkillId.get(installable.skillId);
    if (!existing) {
      bySkillId.set(installable.skillId, installable);
      continue;
    }
    if (
      policy.packagePrecedence === "package-first" &&
      existing.kind === "markdown" &&
      installable.kind === "package"
    ) {
      bySkillId.set(installable.skillId, installable);
    }
  }
  return Array.from(bySkillId.values());
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
  return value
    .toLowerCase()
    .replace(/[\\/]+/g, "-")
    .replace(/[^a-z0-9-_]+/g, "-")
    .replace(/-+/g, "-")
    .replace(/^-|-$/g, "");
}
