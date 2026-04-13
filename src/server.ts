import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod/v4";

import { ensureStorageLayout, loadConfig } from "./config.js";
import { SkillRegistry } from "./registry/registry.js";
import { createGroup } from "./tools/create-group.js";
import { createSkill } from "./tools/create-skill.js";
import { deleteGroup } from "./tools/delete-group.js";
import { getHubStatus } from "./tools/get-hub-status.js";
import { installSkills } from "./tools/install-skills.js";
import { listGroupSkills } from "./tools/list-group-skills.js";
import { listSkillGroups } from "./tools/list-groups.js";
import { readSkill } from "./tools/read-skill.js";
import { listImportSkillCandidates } from "./tools/list-staging-candidates.js";
import { readImportCandidate } from "./tools/read-staging-candidate.js";
import { searchSkillGroups } from "./tools/search-skills.js";
import { updateGroup } from "./tools/update-group.js";
import { validateSkills } from "./tools/validate-skills.js";
import { writeRepairedImportSkill } from "./tools/write-repaired-skill.js";
import { startSkillWatcher } from "./watcher/watch.js";

async function main() {
  const config = loadConfig();
  await ensureStorageLayout(config.storage);

  const registry = new SkillRegistry(config.storage, config.indexPolicy);
  await registry.rebuild();

  const watcher = config.watchPolicy.enabled
    ? await startSkillWatcher(config.storage.packagesRoot, registry, config.watchPolicy)
    : {
        close: async () => {},
        getStatus: () => ({
          running: false,
          lastEventAtMs: null,
          lastError: null,
        }),
      };

  const server = new McpServer({
    name: "skill-router-mcp",
    version: "0.1.0",
    instructions: [
      "Local skill library with progressive disclosure routing.",
      "",
      "ROUTING PROTOCOL:",
      "1. search_skills(query) — returns groups whose description or skill names match query tokens. Each result includes skillNames and optional directMatch.",
      "",
      "2. Decision:",
      "   - If directMatch exists: read that skill first, compare with user intent.",
      "     - Matches → proceed with read_skill.",
      "     - Mismatch → fall back to reading group descriptions below.",
      "   - If no directMatch or intent is vague: read group descriptions, identify the right group.",
      "",
      "3. list_group_skills(group) — see skill names in the chosen group.",
      "4. read_skill(skill) — load full skill body.",
      "",
      "Do NOT read multiple full skills in one turn — read one, evaluate, then decide.",
      "",
      "WHEN TO USE: Tasks requiring specialized workflow, formal process, or high-density domain knowledge.",
      "WHEN NOT TO USE: Ordinary coding, simple edits, general questions — handle directly.",
      "",
      "TOOL CATEGORIES:",
      "Read-only (routing & discovery): search_skills, list_skill_groups, list_group_skills, read_skill, validate_skills, list_import_candidates, read_import_candidate, get_hub_status.",
      "Write (mutation — always do read checks first): install_skills, create_skill, create_group, update_group, delete_group, write_repaired_import.",
    ].join("\n"),
  });

  server.registerTool(
    "search_skills",
    {
      description:
        "Step 1 of the skill-hub routing protocol. Search groups by query tokens against group descriptions and skill names. Returns matching groups with their skill name lists and optional directMatch hint. Prefer this before list_skill_groups.",
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        openWorldHint: false,
        idempotentHint: true,
      },
      inputSchema: {
        query: z.string().min(1).describe("Search query derived from the current user task."),
        limit: z
          .number()
          .int()
          .min(1)
          .max(12)
          .default(config.indexPolicy.defaultSearchResultLimit)
          .describe("Maximum number of candidate skills to return."),
      },
    },
    async ({ query, limit }) => {
      const results = searchSkillGroups(registry, query, limit);
      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(
              {
                query,
                returned: results.length,
                groups: results,
              },
              null,
              2,
            ),
          },
        ],
        structuredContent: {
          query,
          returned: results.length,
          groups: results,
        },
      };
    },
  );

  server.registerTool(
    "list_group_skills",
    {
      description:
        "Step 2 of the skill-hub routing protocol. After a group is selected, list the skill names, keywords, and paths inside that one group. Use this before read_skill to keep loading progressive and narrow.",
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        openWorldHint: false,
        idempotentHint: true,
      },
      inputSchema: {
        group: z.string().min(1).describe("Group name returned by search_skills or list_groups."),
      },
    },
    async ({ group }) => {
      const result = listGroupSkills(registry, group);
      if (!result) {
        throw new Error(`unknown group: ${group}`);
      }
      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(result, null, 2),
          },
        ],
        structuredContent: result,
      };
    },
  );

  server.registerTool(
    "read_skill",
    {
      description:
        "Step 3 of the skill-hub routing protocol. Read the full skill body only after the model has selected a specific skill name or skill id from list_group_skills. Avoid reading multiple full skills unless one skill is clearly insufficient.",
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        openWorldHint: false,
        idempotentHint: true,
      },
      inputSchema: {
        skill: z.string().min(1).describe("Skill name or skill id returned by list_group_skills."),
      },
    },
    async ({ skill }) => {
      const result = await readSkill(registry, skill, config.indexPolicy.maxRelatedSkills);
      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(result, null, 2),
          },
        ],
        structuredContent: result,
      };
    },
  );

  server.registerTool(
    "install_skills",
    {
      description:
        "Write tool. Install one skill package directory or a directory containing multiple skill packages into the hub packages store. This mutates the formal packages store, may overwrite existing package ids, and rebuilds hub indexes.",
      annotations: {
        readOnlyHint: false,
        destructiveHint: true,
        openWorldHint: false,
        idempotentHint: false,
      },
      inputSchema: {
        sourcePath: z.string().min(1).describe("Absolute or relative path to a skill package directory or parent directory."),
      },
    },
    async ({ sourcePath }) => {
      const result = await installSkills({
        storage: config.storage,
        registry,
        sourcePath,
        policy: config.installPolicy,
      });
      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(result, null, 2),
          },
        ],
        structuredContent: result,
      };
    },
  );

  server.registerTool(
    "create_skill",
    {
      description:
        "Write tool. Create a new skill package in the hub packages store, then rebuild indexes and metadata. Use when a new formal skill package should be added to the hub.",
      annotations: {
        readOnlyHint: false,
        destructiveHint: true,
        openWorldHint: false,
        idempotentHint: false,
      },
      inputSchema: {
        name: z.string().min(1).describe("Skill display name."),
        description: z.string().min(1).describe("Skill description used for group and keyword generation."),
        skillMarkdown: z.string().min(1).describe("Full SKILL.md markdown content. Frontmatter is optional."),
      },
    },
    async ({ name, description, skillMarkdown }) => {
      const result = await createSkill({
        storage: config.storage,
        registry,
        name,
        description,
        skillMarkdown,
      });
      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(result, null, 2),
          },
        ],
        structuredContent: result,
      };
    },
  );

  server.registerTool(
    "validate_skills",
    {
      description:
        "Read-only governance tool. Validate installed skills for missing SKILL.md, invalid frontmatter, duplicate names, and generic-group review cases. Use this before group cleanup or import promotion.",
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        openWorldHint: false,
        idempotentHint: true,
      },
      inputSchema: {
        skill: z.string().optional().describe("Optional skill id or skill name. When omitted, validate the whole hub."),
      },
    },
    async ({ skill }) => {
      const result = await validateSkills({
        registry,
        skill,
      });
      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(result, null, 2),
          },
        ],
        structuredContent: result,
      };
    },
  );

  server.registerTool(
    "list_skill_groups",
    {
      description: "List top-level skill groups currently indexed by the local skill library. Use when a full group overview is required; otherwise prefer search_skills for the first routing step.",
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        openWorldHint: false,
        idempotentHint: true,
      },
      inputSchema: {},
    },
    async () => {
      const groups = listSkillGroups(registry);
      const issues = registry.listIssues();
      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(
              {
                groups,
                loadedSkills: registry.size(),
                issueCount: issues.length,
              },
              null,
              2,
            ),
          },
        ],
        structuredContent: {
          groups,
          loadedSkills: registry.size(),
          issueCount: issues.length,
        },
      };
    },
  );

  server.registerTool(
    "create_group",
    {
      description:
        "Write tool. Create a new managed group in the hub group catalog. Use only when an existing group cannot stably represent a recurring domain. New groups become available to routing after index rebuild.",
      annotations: {
        readOnlyHint: false,
        destructiveHint: false,
        openWorldHint: false,
        idempotentHint: false,
      },
      inputSchema: {
        group: z.string().min(1).describe("Stable group id, usually kebab-case."),
        groupDescription: z.string().min(1).describe("LLM-facing description for this group."),
        keywords: z.array(z.string().min(1)).optional().describe("Optional routing keywords for the group."),
        aliases: z.array(z.string().min(1)).optional().describe("Optional alternative names for matching and future rename continuity."),
      },
    },
    async ({ group, groupDescription, keywords, aliases }) => {
      const result = await createGroup(registry, {
        group,
        groupDescription,
        keywords,
        aliases,
      });
      return {
        content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
        structuredContent: result,
      };
    },
  );

  server.registerTool(
    "update_group",
    {
      description:
        "Write tool. Rename or update a managed group. This rebuilds indexes and may reclassify many existing skills against the updated group catalog. Prefer this over changing many skills one by one when the group definition itself is wrong.",
      annotations: {
        readOnlyHint: false,
        destructiveHint: true,
        openWorldHint: false,
        idempotentHint: false,
      },
      inputSchema: {
        group: z.string().min(1).describe("Current group id."),
        newGroup: z.string().min(1).optional().describe("Optional new group id."),
        groupDescription: z.string().min(1).optional().describe("Optional replacement group description."),
        keywords: z.array(z.string().min(1)).optional().describe("Optional replacement routing keywords."),
        aliases: z.array(z.string().min(1)).optional().describe("Optional replacement aliases."),
      },
    },
    async ({ group, newGroup, groupDescription, keywords, aliases }) => {
      const result = await updateGroup(registry, {
        group,
        newGroup,
        groupDescription,
        keywords,
        aliases,
      });
      return {
        content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
        structuredContent: result,
      };
    },
  );

  server.registerTool(
    "delete_group",
    {
      description:
        "Write tool. Delete an empty custom group from the managed group catalog. Builtin groups cannot be deleted. Only use after confirming the group has no remaining skills.",
      annotations: {
        readOnlyHint: false,
        destructiveHint: true,
        openWorldHint: false,
        idempotentHint: false,
      },
      inputSchema: {
        group: z.string().min(1).describe("Group id to delete."),
      },
    },
    async ({ group }) => {
      const result = await deleteGroup(registry, { group });
      return {
        content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
        structuredContent: result,
      };
    },
  );

  server.registerTool(
    "list_import_candidates",
    {
      description:
        "Read-only import governance tool. List imported skill candidates that still need conversion, review, or repair before entering the formal packages store.",
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        openWorldHint: false,
        idempotentHint: true,
      },
      inputSchema: {
        status: z
          .enum(["ready", "review_required", "blocked", "repaired"])
          .optional()
          .describe("Optional status filter."),
        limit: z.number().int().min(1).max(100).default(30),
      },
    },
    async ({ status, limit }) => {
      const candidates = await listImportSkillCandidates({
        rawRoot: config.storage.stagingImportsRoot,
        repairedRoot: config.storage.stagingRepairedRoot,
        status,
        limit,
      });
      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(
              {
                status: status ?? null,
                returned: candidates.length,
                candidates,
              },
              null,
              2,
            ),
          },
        ],
        structuredContent: {
          status: status ?? null,
          returned: candidates.length,
          candidates,
        },
      };
    },
  );

  server.registerTool(
    "read_import_candidate",
    {
      description:
        "Read-only import review tool. Read one imported candidate, including its raw markdown, issues, and a small set of nearby skill groups for semantic review.",
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        openWorldHint: false,
        idempotentHint: true,
      },
      inputSchema: {
        id: z.string().min(1).describe("Candidate id returned by list_import_candidates."),
      },
    },
    async ({ id }) => {
      const result = await readImportCandidate({
        rawRoot: config.storage.stagingImportsRoot,
        repairedRoot: config.storage.stagingRepairedRoot,
        registry,
        id,
      });
      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(result, null, 2),
          },
        ],
        structuredContent: result,
      };
    },
  );

  server.registerTool(
    "write_repaired_import",
    {
      description:
        "Write tool. Write an LLM-repaired import candidate into staging/repaired without touching the original raw source or the formal packages directory. Use this only after semantic review of the candidate.",
      annotations: {
        readOnlyHint: false,
        destructiveHint: true,
        openWorldHint: false,
        idempotentHint: false,
      },
      inputSchema: {
        id: z.string().min(1).describe("Candidate id returned by list_import_candidates."),
        skillMarkdown: z.string().min(1).describe("Full repaired SKILL.md content."),
        targetId: z
          .string()
          .min(1)
          .optional()
          .describe("Optional final skill package id. Defaults to the candidate inferred target id."),
        notes: z
          .string()
          .optional()
          .describe("Optional repair notes or rationale saved alongside the repaired skill."),
      },
    },
    async ({ id, skillMarkdown, targetId, notes }) => {
      const result = await writeRepairedImportSkill({
        rawRoot: config.storage.stagingImportsRoot,
        repairedRoot: config.storage.stagingRepairedRoot,
        id,
        skillMarkdown,
        targetId,
        notes,
      });
      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(result, null, 2),
          },
        ],
        structuredContent: result,
      };
    },
  );

  server.registerTool(
    "get_hub_status",
    {
      description:
        "Return hub counts, index freshness, watcher status, and import pipeline health.",
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        openWorldHint: false,
        idempotentHint: true,
      },
      inputSchema: {},
    },
    async () => {
      const result = await getHubStatus({
        storage: config.storage,
        registry,
        watcherStatus: watcher.getStatus(),
      });
      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(result, null, 2),
          },
        ],
        structuredContent: result,
      };
    },
  );

  const transport = new StdioServerTransport();
  await server.connect(transport);

  console.error(
    `skill-router-mcp ready | hubRoot=${config.storage.hubRoot} | groupCatalog=${config.storage.groupCatalogPath} | packagesRoot=${config.storage.packagesRoot} | importRoot=${config.storage.stagingImportsRoot} | loadedSkills=${registry.size()} | issues=${registry.listIssues().length} | watch=${config.watchPolicy.enabled} | searchLimit=${config.indexPolicy.defaultSearchResultLimit}`,
  );
}

void main();
