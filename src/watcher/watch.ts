import chokidar from "chokidar";

import type { WatchPolicy } from "../config.js";
import { SkillRegistry } from "../registry/registry.js";
import type { WatcherStatus } from "../types.js";

export type SkillWatcherHandle = {
  close: () => Promise<void>;
  getStatus: () => WatcherStatus;
};

export async function startSkillWatcher(
  packagesRoot: string,
  registry: SkillRegistry,
  policy: WatchPolicy,
): Promise<SkillWatcherHandle> {
  const status: WatcherStatus = {
    running: true,
    lastEventAtMs: null,
    lastError: null,
  };

  const watcher = chokidar.watch(packagesRoot, {
    ignoreInitial: true,
    usePolling: policy.usePolling,
    interval: policy.pollingIntervalMs,
    awaitWriteFinish: {
      stabilityThreshold: policy.awaitWriteStabilityMs,
      pollInterval: policy.awaitWritePollMs,
    },
  });

  const track = () => {
    status.lastEventAtMs = Date.now();
  };

  const refresh = async (filePath: string) => {
    if (!filePath.endsWith("SKILL.md")) return;
    track();
    try {
      await registry.refreshSkillByPath(filePath);
      status.lastError = null;
    } catch (error) {
      status.lastError = error instanceof Error ? error.message : String(error);
    }
  };

  watcher.on("add", refresh);
  watcher.on("change", refresh);
  watcher.on("unlink", (filePath) => {
    if (!filePath.endsWith("SKILL.md")) return;
    track();
    if (policy.syncDelete) {
      registry.removeByPath(filePath);
    }
  });
  watcher.on("error", (error) => {
    status.lastError = error instanceof Error ? error.message : String(error);
  });

  await new Promise<void>((resolve) => {
    watcher.on("ready", () => resolve());
  });

  return {
    close: async () => {
      status.running = false;
      await watcher.close();
    },
    getStatus: () => ({ ...status }),
  };
}
