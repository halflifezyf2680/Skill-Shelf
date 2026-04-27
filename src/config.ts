import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

export type SkillShelfStorageLayout = {
  shelfRoot: string;
  configRoot: string;
  groupCatalogPath: string;
  packagesRoot: string;
  indexRoot: string;
  groupListPath: string;
  groupsRoot: string;
  skillsRoot: string;
  stagingRoot: string;
  stagingImportsRoot: string;
  stagingRepairedRoot: string;
  logsRoot: string;
};

export type InstallPolicy = {
  acceptPackageDirectories: boolean;
  acceptRawMarkdown: boolean;
  packagePrecedence: "package-first";
  rawMarkdownRequiresFrontmatter: boolean;
};

export type IndexPolicy = {
  defaultSearchResultLimit: number;
  maxKeywordsPerSkill: number;
  maxRelatedSkills: number;
};

export type WatchPolicy = {
  enabled: boolean;
  usePolling: boolean;
  pollingIntervalMs: number;
  awaitWriteStabilityMs: number;
  awaitWritePollMs: number;
  syncDelete: boolean;
};

export type SkillRouterConfig = {
  storage: SkillShelfStorageLayout;
  installPolicy: InstallPolicy;
  indexPolicy: IndexPolicy;
  watchPolicy: WatchPolicy;
};

export function loadConfig(): SkillRouterConfig {
  const packageRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const shelfRoot =
    process.env.SKILL_SHELF_ROOT ??
    path.join(packageRoot, "data", "hub");

  const storage = resolveStorageLayout(shelfRoot);
  return {
    storage,
    installPolicy: {
      acceptPackageDirectories: process.env.SKILL_SHELF_ACCEPT_PACKAGES !== "0",
      acceptRawMarkdown: process.env.SKILL_SHELF_ACCEPT_RAW_MARKDOWN !== "0",
      packagePrecedence: "package-first",
      rawMarkdownRequiresFrontmatter: process.env.SKILL_SHELF_RAW_REQUIRES_FRONTMATTER !== "0",
    },
    indexPolicy: {
      defaultSearchResultLimit: Number(process.env.SKILL_SHELF_SEARCH_LIMIT ?? "8"),
      maxKeywordsPerSkill: Number(process.env.SKILL_SHELF_MAX_KEYWORDS ?? "12"),
      maxRelatedSkills: Number(process.env.SKILL_SHELF_MAX_RELATED_SKILLS ?? "5"),
    },
    watchPolicy: {
      enabled: process.env.SKILL_SHELF_WATCH !== "0",
      usePolling: process.env.SKILL_SHELF_WATCH_USE_POLLING !== "0",
      pollingIntervalMs: Number(process.env.SKILL_SHELF_WATCH_INTERVAL_MS ?? "100"),
      awaitWriteStabilityMs: Number(process.env.SKILL_SHELF_WATCH_STABILITY_MS ?? "300"),
      awaitWritePollMs: Number(process.env.SKILL_SHELF_WATCH_POLL_MS ?? "50"),
      syncDelete: process.env.SKILL_SHELF_WATCH_SYNC_DELETE !== "0",
    },
  };
}

export function resolveStorageLayout(shelfRoot: string): SkillShelfStorageLayout {
  const normalizedShelfRoot = path.resolve(shelfRoot);
  const configRoot = path.join(normalizedShelfRoot, "config");
  const indexRoot = path.join(normalizedShelfRoot, "index");
  const stagingRoot = path.join(normalizedShelfRoot, "staging");

  return {
    shelfRoot: normalizedShelfRoot,
    configRoot,
    groupCatalogPath: path.join(configRoot, "groups.json"),
    packagesRoot: path.join(normalizedShelfRoot, "packages"),
    indexRoot,
    groupListPath: path.join(indexRoot, "group-list.json"),
    groupsRoot: path.join(indexRoot, "groups"),
    skillsRoot: path.join(indexRoot, "skills"),
    stagingRoot,
    stagingImportsRoot: path.join(stagingRoot, "imports"),
    stagingRepairedRoot: path.join(stagingRoot, "repaired"),
    logsRoot: path.join(normalizedShelfRoot, "logs"),
  };
}

export async function ensureStorageLayout(layout: SkillShelfStorageLayout): Promise<void> {
  await Promise.all([
    fs.mkdir(layout.shelfRoot, { recursive: true }),
    fs.mkdir(layout.configRoot, { recursive: true }),
    fs.mkdir(layout.packagesRoot, { recursive: true }),
    fs.mkdir(layout.indexRoot, { recursive: true }),
    fs.mkdir(layout.groupsRoot, { recursive: true }),
    fs.mkdir(layout.skillsRoot, { recursive: true }),
    fs.mkdir(layout.stagingRoot, { recursive: true }),
    fs.mkdir(layout.stagingImportsRoot, { recursive: true }),
    fs.mkdir(layout.stagingRepairedRoot, { recursive: true }),
    fs.mkdir(layout.logsRoot, { recursive: true }),
  ]);
}
