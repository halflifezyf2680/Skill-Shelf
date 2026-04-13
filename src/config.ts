import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

export type SkillHubStorageLayout = {
  hubRoot: string;
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
  storage: SkillHubStorageLayout;
  installPolicy: InstallPolicy;
  indexPolicy: IndexPolicy;
  watchPolicy: WatchPolicy;
};

export function loadConfig(): SkillRouterConfig {
  const packageRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const hubRoot =
    process.env.SKILL_HUB_ROOT ??
    path.join(packageRoot, "data", "hub");

  const storage = resolveStorageLayout(hubRoot);
  return {
    storage,
    installPolicy: {
      acceptPackageDirectories: process.env.SKILL_ROUTER_ACCEPT_PACKAGES !== "0",
      acceptRawMarkdown: process.env.SKILL_ROUTER_ACCEPT_RAW_MARKDOWN !== "0",
      packagePrecedence: "package-first",
      rawMarkdownRequiresFrontmatter: process.env.SKILL_ROUTER_RAW_REQUIRES_FRONTMATTER !== "0",
    },
    indexPolicy: {
      defaultSearchResultLimit: Number(process.env.SKILL_ROUTER_SEARCH_LIMIT ?? "8"),
      maxKeywordsPerSkill: Number(process.env.SKILL_ROUTER_MAX_KEYWORDS ?? "12"),
      maxRelatedSkills: Number(process.env.SKILL_ROUTER_MAX_RELATED_SKILLS ?? "5"),
    },
    watchPolicy: {
      enabled: process.env.SKILL_ROUTER_WATCH !== "0",
      usePolling: process.env.SKILL_ROUTER_WATCH_USE_POLLING !== "0",
      pollingIntervalMs: Number(process.env.SKILL_ROUTER_WATCH_INTERVAL_MS ?? "100"),
      awaitWriteStabilityMs: Number(process.env.SKILL_ROUTER_WATCH_STABILITY_MS ?? "300"),
      awaitWritePollMs: Number(process.env.SKILL_ROUTER_WATCH_POLL_MS ?? "50"),
      syncDelete: process.env.SKILL_ROUTER_WATCH_SYNC_DELETE !== "0",
    },
  };
}

export function resolveStorageLayout(hubRoot: string): SkillHubStorageLayout {
  const normalizedHubRoot = path.resolve(hubRoot);
  const configRoot = path.join(normalizedHubRoot, "config");
  const indexRoot = path.join(normalizedHubRoot, "index");
  const stagingRoot = path.join(normalizedHubRoot, "staging");

  return {
    hubRoot: normalizedHubRoot,
    configRoot,
    groupCatalogPath: path.join(configRoot, "groups.json"),
    packagesRoot: path.join(normalizedHubRoot, "packages"),
    indexRoot,
    groupListPath: path.join(indexRoot, "group-list.json"),
    groupsRoot: path.join(indexRoot, "groups"),
    skillsRoot: path.join(indexRoot, "skills"),
    stagingRoot,
    stagingImportsRoot: path.join(stagingRoot, "imports"),
    stagingRepairedRoot: path.join(stagingRoot, "repaired"),
    logsRoot: path.join(normalizedHubRoot, "logs"),
  };
}

export async function ensureStorageLayout(layout: SkillHubStorageLayout): Promise<void> {
  await Promise.all([
    fs.mkdir(layout.hubRoot, { recursive: true }),
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
