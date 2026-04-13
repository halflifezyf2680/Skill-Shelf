import type { SkillRegistry } from "../registry/registry.js";

export function searchSkillGroups(registry: SkillRegistry, query: string, limit: number) {
  return registry.searchGroups(query, limit);
}
