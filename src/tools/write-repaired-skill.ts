import { getImportCandidate, writeRepairedImport } from "../staging/staging.js";

export async function writeRepairedImportSkill(params: {
  rawRoot: string;
  repairedRoot: string;
  id: string;
  skillMarkdown: string;
  targetId?: string;
  notes?: string;
}) {
  const candidate = await getImportCandidate(params.rawRoot, params.repairedRoot, params.id);
  if (!candidate) {
    throw new Error(`unknown import candidate id: ${params.id}`);
  }

  return await writeRepairedImport({
    repairedRoot: params.repairedRoot,
    candidate,
    skillMarkdown: params.skillMarkdown,
    targetId: params.targetId,
    notes: params.notes,
  });
}
