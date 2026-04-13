import type { SkillRegistry } from "../registry/registry.js";
import { getImportCandidate, readImportCandidateContents } from "../staging/staging.js";
import { searchSkillGroups } from "./search-skills.js";

export async function readImportCandidate(params: {
  rawRoot: string;
  repairedRoot: string;
  registry: SkillRegistry;
  id: string;
}) {
  const candidate = await getImportCandidate(params.rawRoot, params.repairedRoot, params.id);
  if (!candidate) {
    throw new Error(`unknown import candidate id: ${params.id}`);
  }

  const contents = await readImportCandidateContents(candidate);
  const query = [candidate.name, candidate.description, candidate.inferredTargetId]
    .filter(Boolean)
    .join(" ");
  const neighborGroups = query ? searchSkillGroups(params.registry, query, 6) : [];

  return {
    candidate,
    neighborGroups,
    contents,
  };
}
