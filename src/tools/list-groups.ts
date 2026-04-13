import type { SkillRegistry } from "../registry/registry.js";

export function listSkillGroups(registry: SkillRegistry) {
  return registry.listGroups();
}
