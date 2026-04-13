import type { SkillRegistry } from "../registry/registry.js";

export async function updateGroup(
  registry: SkillRegistry,
  params: {
    group: string;
    newGroup?: string;
    groupDescription?: string;
    keywords?: string[];
    aliases?: string[];
  },
) {
  return await registry.updateGroup(params);
}
