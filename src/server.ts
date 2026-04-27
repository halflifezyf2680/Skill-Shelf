import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { z } from "zod/v4";

import { ensureStorageLayout, loadConfig } from "./config.js";
import { SkillRegistry } from "./registry/registry.js";
import { getShelfStatus } from "./tools/get-shelf-status.js";
import { installSkills } from "./tools/install-skills.js";
import { readSkill } from "./tools/read-skill.js";
import { searchSkillsFlat } from "./tools/search-skills.js";
import { validateSkills } from "./tools/validate-skills.js";
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

  const server = new McpServer(
    {
      name: "skill-router-mcp",
      version: "0.1.0",
    },
    {
      instructions: [
        "Local skill library with flat search routing.",
        "",
        "ROUTING PROTOCOL:",
        "1. search_skills(query) — fuzzy search across ALL skills, returns top matches with name + description + score.",
        "2. read_skill(skill) — load full skill body for the chosen skill.",
        "",
        "Do NOT read multiple full skills in one turn — read one, evaluate, then decide.",
        "",
        "WHEN TO USE: Tasks requiring specialized workflow, formal process, or high-density domain knowledge.",
        "WHEN NOT TO USE: Ordinary coding, simple edits, general questions — handle directly.",
      ].join("\n"),
    },
  );

  server.registerTool(
    "search_skills",
    {
      description:
        "Step 1 of the skill-shelf routing protocol. Search all skills by query, returns top matches ranked by relevance. IMPORTANT: Always try the user's language first. If no relevant results found, retry with English keywords. For CJK queries, separate words with spaces (e.g. '品牌 视觉 设计', NOT '品牌设计视觉').",
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
          .max(20)
          .default(8)
          .describe("Maximum number of candidate skills to return."),
      },
    },
    async ({ query, limit }) => {
      const results = searchSkillsFlat(registry, query, limit);
      return {
        content: [
          {
            type: "text",
            text: JSON.stringify(
              {
                query,
                returned: results.length,
                skills: results,
              },
              null,
              2,
            ),
          },
        ],
        structuredContent: {
          query,
          returned: results.length,
          skills: results,
        },
      };
    },
  );

  server.registerTool(
    "read_skill",
    {
      description:
        "Read the full skill body. Only call this after search_skills has identified the target skill by name or skill id. Avoid reading multiple full skills unless one skill is clearly insufficient.",
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        openWorldHint: false,
        idempotentHint: true,
      },
      inputSchema: {
        skill: z.string().min(1).describe("Skill name or skill id returned by search_skills."),
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
        "Write tool. Install one skill package directory or a directory containing multiple skill packages into the shelf packages store. This mutates the formal packages store, may overwrite existing package ids, and rebuilds shelf indexes.",
      annotations: {
        readOnlyHint: false,
        destructiveHint: true,
        openWorldHint: false,
        idempotentHint: false,
      },
      inputSchema: {
        sourcePath: z.string().min(1).describe("Absolute or relative path to a skill package directory or parent directory."),
        group: z.string().min(1).optional().describe("Target group id. If omitted and SKILL.md has no frontmatter group, the tool returns skill descriptions and available groups for you to classify. Then call again with the chosen group."),
      },
    },
    async ({ sourcePath, group }) => {
      const result = await installSkills({
        storage: config.storage,
        registry,
        sourcePath,
        group,
        policy: config.installPolicy,
      });
      const parts: Array<{ type: "text"; text: string }> = [];
      // Summary
      const summary = [
        `Installed: ${result.installed.length}`,
        `Skipped: ${result.skipped.length}`,
        `Failed: ${result.failed.length}`,
        `Needs classification: ${result.needsClassification.length}`,
      ].join("\n");
      parts.push({ type: "text", text: summary });
      // Classification hint
      if (result.classificationHint) {
        parts.push({ type: "text", text: result.classificationHint });
      }
      // Failures
      if (result.failed.length > 0) {
        const failures = result.failed.map((f) => `- ${f.sourcePath}: ${f.message}`).join("\n");
        parts.push({ type: "text", text: `Failures:\n${failures}` });
      }
      return {
        content: parts,
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
        skill: z.string().optional().describe("Optional skill id or skill name. When omitted, validate the whole shelf."),
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
    "manage_group",
    {
      description:
        "Write tool. Manage skill groups: create a new group, update an existing group (description, keywords, aliases, rename), or delete an empty custom group. Builtin groups cannot be deleted.",
      annotations: {
        readOnlyHint: false,
        destructiveHint: true,
        openWorldHint: false,
        idempotentHint: false,
      },
      inputSchema: {
        mode: z.enum(["create", "update", "delete"]).describe("Operation mode."),
        group: z.string().min(1).describe("Group id (kebab-case). For update/delete, must be an existing group."),
        groupDescription: z.string().transform(v => v || undefined).optional().describe("Group description. Required for create, optional for update."),
        newGroup: z.string().transform(v => v || undefined).optional().describe("New group id for rename (update mode only)."),
        keywords: z.array(z.string()).transform(v => (v && v.length ? v : undefined)).optional().describe("Routing keywords (replaces existing on update)."),
        aliases: z.array(z.string()).transform(v => (v && v.length ? v : undefined)).optional().describe("Alternative names (replaces existing on update)."),
      },
    },
    async ({ mode, group, groupDescription, newGroup, keywords, aliases }) => {
      if (mode === "create") {
        if (!groupDescription) throw new Error("groupDescription is required for create mode");
        const { createGroup } = await import("./tools/create-group.js");
        const result = await createGroup(registry, { group, groupDescription, keywords, aliases });
        return { content: [{ type: "text", text: JSON.stringify(result, null, 2) }], structuredContent: result };
      }
      if (mode === "update") {
        const { updateGroup } = await import("./tools/update-group.js");
        const result = await updateGroup(registry, { group, newGroup, groupDescription, keywords, aliases });
        return { content: [{ type: "text", text: JSON.stringify(result, null, 2) }], structuredContent: result };
      }
      if (mode === "delete") {
        const { deleteGroup } = await import("./tools/delete-group.js");
        const result = await deleteGroup(registry, { group });
        return { content: [{ type: "text", text: JSON.stringify(result, null, 2) }], structuredContent: result };
      }
      throw new Error(`unknown mode: ${mode}`);
    },
  );

  server.registerTool(
    "get_shelf_status",
    {
      description:
        "Return shelf counts, index freshness, watcher status.",
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        openWorldHint: false,
        idempotentHint: true,
      },
      inputSchema: {},
    },
    async () => {
      const result = await getShelfStatus({
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
    `skill-router-mcp ready | shelfRoot=${config.storage.shelfRoot} | packagesRoot=${config.storage.packagesRoot} | loadedSkills=${registry.size()} | issues=${registry.listIssues().length} | watch=${config.watchPolicy.enabled} | searchLimit=${config.indexPolicy.defaultSearchResultLimit}`,
  );
}

void main();
