export type SkillGroupRecord = {
  group: string;
  groupDescription: string;
};

export type ManagedGroupRecord = SkillGroupRecord & {
  keywords: string[];
  aliases: string[];
  source: "builtin" | "custom";
};

export type GroupSkillIndexEntry = {
  skillId: string;
  skillName: string;
  description: string;
  keywords: string[];
  skillPath: string;
};

export type SkillRecord = {
  skillId: string;
  skillName: string;
  description: string;
  group: string;
  groupDescription: string;
  keywords: string[];
  skillPath: string;
  updatedAtMs: number;
  status: "ready";
};

export type SkillMeta = Pick<
  SkillRecord,
  "skillId" | "skillName" | "description" | "group" | "groupDescription" | "keywords" | "updatedAtMs" | "status"
>;

export type GroupListItem = SkillGroupRecord;

export type GroupSkillsResult = {
  group: string;
  groupDescription: string;
  skills: GroupSkillIndexEntry[];
};

export type ParsedSkill = {
  skillName: string;
  description: string;
  group: string;
  groupDescription: string;
  keywords: string[];
  skillPath: string;
  updatedAtMs: number;
};

export type RegistryIssue = {
  path: string;
  message: string;
};

export type GroupSearchResult = SkillGroupRecord & {
  skills: Array<{ skillId: string; skillName: string; description: string }>;
  directMatch: {
    skillId: string;
    skillName: string;
    description: string;
  } | null;
};

export type ImportCandidateStatus = "ready" | "review_required" | "blocked" | "repaired";

export type ImportCandidate = {
  id: string;
  sourcePath: string;
  relativePath: string;
  inferredTargetId: string;
  status: ImportCandidateStatus;
  issues: string[];
  kind: "package" | "markdown";
  name: string | null;
  description: string | null;
};

export type SkillInstallEntry = {
  sourcePath: string;
  skillId: string;
  installedPath: string;
  status: "installed";
};

export type SkillInstallFailure = {
  sourcePath: string;
  status: "failed";
  message: string;
};

export type SkillNeedsClassification = {
  skillId: string;
  sourcePath: string;
  description: string;
};

export type SkillInstallResult = {
  installed: SkillInstallEntry[];
  skipped: Array<{ sourcePath: string; status: "skipped"; reason: string }>;
  failed: SkillInstallFailure[];
  needsClassification: SkillNeedsClassification[];
  classificationHint?: string;
};

export type SkillCreateResult = {
  skillId: string;
  skillPath: string;
  group: string;
  keywords: string[];
};

export type SkillValidationIssue = {
  skillId: string | null;
  path: string;
  code:
    | "missing_skill_file"
    | "invalid_frontmatter"
    | "duplicate_skill_name"
    | "generic_group";
  severity: "review_required" | "blocked";
  message: string;
};

export type SkillValidationResult = {
  passed: Array<{ skillId: string; skillPath: string }>;
  reviewRequired: SkillValidationIssue[];
  blocked: SkillValidationIssue[];
  issues: SkillValidationIssue[];
};

export type WatcherStatus = {
  running: boolean;
  lastEventAtMs: number | null;
  lastError: string | null;
};

export type HubStatus = {
  groupsCount: number;
  skillsCount: number;
  importCount: number;
  indexUpdatedAt: number | null;
  watcherStatus: WatcherStatus;
  issueCount: number;
};

export type GroupCreateResult = {
  action: "created";
  group: ManagedGroupRecord;
};

export type GroupUpdateResult = {
  action: "updated";
  previousGroup: string;
  group: ManagedGroupRecord;
};

export type GroupDeleteResult = {
  action: "deleted";
  group: string;
};
