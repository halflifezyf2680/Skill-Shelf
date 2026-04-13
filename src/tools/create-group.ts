import type { SkillRegistry } from "../registry/registry.js";

export async function createGroup(
  registry: SkillRegistry,
  params: {
    group: string;
    groupDescription: string;
    keywords?: string[];
    aliases?: string[];
  },
) {
  return await registry.createGroup(params);
}
