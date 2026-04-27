import fs from "node:fs/promises";

import type { SkillShelfStorageLayout } from "../config.js";
import { SkillRegistry } from "../registry/registry.js";
import type { ShelfStatus, WatcherStatus } from "../types.js";

export async function getShelfStatus(params: {
  storage: SkillShelfStorageLayout;
  registry: SkillRegistry;
  watcherStatus: WatcherStatus;
}): Promise<ShelfStatus> {
  const importCount = await countDirectories(params.storage.stagingImportsRoot);
  return {
    groupsCount: params.registry.listGroups().length,
    skillsCount: params.registry.size(),
    importCount,
    indexUpdatedAt: params.registry.getIndexUpdatedAt(),
    watcherStatus: params.watcherStatus,
    issueCount: params.registry.listIssues().length,
  };
}

async function countDirectories(root: string): Promise<number> {
  try {
    const entries = await fs.readdir(root, { withFileTypes: true });
    return entries.filter((entry) => entry.isDirectory()).length;
  } catch {
    return 0;
  }
}
