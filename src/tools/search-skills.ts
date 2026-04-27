import type { SkillRegistry } from "../registry/registry.js";
import { scoreSkillText } from "../registry/fuzzy.js";

export type FlatSkillSearchResult = {
  skillId: string;
  skillName: string;
  description: string;
  group: string;
  score: number;
  reasons: string[];
};

export function searchSkillsFlat(
  registry: SkillRegistry,
  query: string,
  limit: number,
): FlatSkillSearchResult[] {
  const results: FlatSkillSearchResult[] = [];

  for (const record of registry.listSkillRecords()) {
    const { score, reasons } = scoreSkillText({
      id: record.skillId,
      name: record.skillName,
      group: record.group,
      summary: record.description,
      tags: record.keywords,
      query,
    });

    if (score > 0) {
      results.push({
        skillId: record.skillId,
        skillName: record.skillName,
        description: record.description,
        group: record.group,
        score,
        reasons,
      });
    }
  }

  results.sort((a, b) => b.score - a.score);
  return results.slice(0, limit);
}
