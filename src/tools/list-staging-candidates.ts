import type { ImportCandidateStatus } from "../types.js";
import { listImportCandidates } from "../staging/staging.js";

export async function listImportSkillCandidates(params: {
  rawRoot: string;
  repairedRoot: string;
  status?: ImportCandidateStatus;
  limit: number;
}) {
  return await listImportCandidates(
    params.rawRoot,
    params.repairedRoot,
    params.status,
    params.limit,
  );
}
