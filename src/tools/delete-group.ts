import type { SkillRegistry } from "../registry/registry.js";

export async function deleteGroup(
  registry: SkillRegistry,
  params: {
    group: string;
  },
) {
  return await registry.deleteGroup(params);
}
