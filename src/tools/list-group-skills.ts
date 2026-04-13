import type { SkillRegistry } from "../registry/registry.js";

export function listGroupSkills(registry: SkillRegistry, group: string) {
  return registry.listGroupSkills(group);
}
