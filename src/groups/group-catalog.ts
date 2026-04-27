import fs from "node:fs/promises";

import type { SkillShelfStorageLayout } from "../config.js";
import type { ManagedGroupRecord } from "../types.js";

const SPECIALIZED_GROUP_ID = "specialized-domain";

export const DEFAULT_MANAGED_GROUPS: ManagedGroupRecord[] = [
  seedGroup(
    "engineering",
    "软件工程、系统架构、前后端实现、数据与 AI 工程、自动化与基础设施能力。",
    ["software", "engineering", "backend", "frontend", "api", "database", "devops", "ai", "ml", "data", "code", "developer", "automation", "infra", "mobile", "编程", "开发", "架构", "工程师", "代码", "语言模型", "提示词", "协议", "服务器", "智能体", "编排", "自动化", "区块链", "索引", "mcp", "workflow", "工作流", "pipeline", "智能合约", "security", "安全", "审计", "n8n", "php", "swift", "终端", "后台", "重构", "优化", "集成"],
    ["software-engineering", "development", "tech", "编程", "开发", "架构师"],
  ),
  seedGroup(
    "design",
    "UI/UX、品牌、视觉系统、设计语言、创意素材与体验表达。",
    ["design", "ui", "ux", "brand", "visual", "creative", "figma", "token", "identity", "banner", "icon", "styling", "设计师", "体验", "交互", "设计"],
    ["visual-design", "ux-design"],
  ),
  seedGroup(
    "product",
    "产品规划、需求分析、用户反馈、优先级、路线图与产品研究。",
    ["product", "roadmap", "requirement", "feedback", "prioritization", "research", "insight", "discovery", "feature", "产品", "需求", "用户", "路线图", "功能"],
    ["product-management"],
  ),
  seedGroup(
    "project-management",
    "项目推进、任务协调、里程碑、交付节奏与跨团队执行管理。",
    ["project", "program", "delivery", "milestone", "planning", "execution", "coordination", "timeline", "stakeholder", "项目", "统筹", "协调", "资源分配", "跨部门"],
    ["pm", "program-management"],
  ),
  seedGroup(
    "marketing",
    "品牌营销、内容营销、市场传播、增长叙事与活动策略。",
    ["marketing", "brand", "content", "campaign", "growth", "messaging", "storytelling", "audience", "positioning", "营销", "广告", "推广", "品牌", "内容营销", "增长", "传播", "市场", "图书", "出版", "内容", "创作", "故事"],
    ["go-to-market", "content-marketing"],
  ),
  seedGroup(
    "paid-media",
    "广告投放、媒介优化、买量、渠道归因与预算效率管理。",
    ["paid", "media", "ads", "advertising", "acquisition", "attribution", "budget", "campaign-optimization", "performance-marketing"],
    ["paid-ads", "media-buying"],
  ),
  seedGroup(
    "sales",
    "销售、售前、线索管理、方案支持、成交推进与客户拓展。",
    ["sales", "lead", "pipeline", "prospect", "deal", "presales", "account", "crm", "closing", "销售", "客户", "商机", "管线", "售前", "成交"],
    ["business-development", "presales"],
  ),
  seedGroup(
    "finance",
    "财务、预算、核算、定价、经营分析与资金相关能力。",
    ["finance", "financial", "budget", "pricing", "accounting", "forecast", "cost", "revenue", "expense", "财务", "会计", "定价", "成本", "支付", "发票", "账单", "付款"],
    ["finops", "accounting"],
  ),
  seedGroup(
    "legal-compliance",
    "法务、合规、审计、政策、风险治理与监管要求处理。",
    ["legal", "compliance", "audit", "policy", "risk", "regulation", "governance", "contract", "privacy", "合规", "审计", "风险", "监管", "法规", "治理", "隐私", "政策", "风控", "安全"],
    ["legal", "compliance", "risk"],
  ),
  seedGroup(
    "hr-talent",
    "招聘、人才、培训、组织发展与人事支持工作。",
    ["hr", "recruitment", "talent", "hiring", "people", "training", "onboarding", "candidate", "organization", "培训", "人才", "招聘", "组织", "内训", "领导力"],
    ["human-resources", "talent-acquisition"],
  ),
  seedGroup(
    "support-operations",
    "客服、运营支持、行政协助、汇总汇报与后台支持工作。",
    ["support", "operations", "assistant", "reporting", "responder", "summary", "maintainer", "tracker", "coordination", "运营", "行政", "会议", "报告", "汇总", "分发", "助手"],
    ["ops-support", "backoffice"],
  ),
  seedGroup(
    "supply-chain",
    "供应链、采购、物流、库存、履约与上下游协同。",
    ["supply", "procurement", "logistics", "inventory", "fulfillment", "vendor", "warehouse", "operations"],
    ["sourcing", "supply-chain"],
  ),
  seedGroup(
    "academic-research",
    "学术研究、学习规划、人文社科分析、资料整理与研究方法。",
    ["academic", "research", "study", "historian", "anthropologist", "psychologist", "geographer", "scholar", "education", "留学", "高考", "志愿", "学习规划", "学术", "研究", "教育", "文化", "地理", "叙事", "文学", "人类学", "社会学", "民族志"],
    ["research", "education"],
  ),
  seedGroup(
    "testing-qa",
    "测试、质量保障、评估、验证、审查与质量控制。",
    ["testing", "qa", "quality", "review", "audit", "validation", "verification", "benchmark", "eval", "测试", "质量", "校准", "复现", "评估", "验证", "认证", "就绪"],
    ["quality-assurance", "evaluation"],
  ),
  seedGroup(
    "spatial-gaming",
    "游戏开发、空间计算、XR、3D 交互与沉浸式体验。",
    ["game", "gaming", "spatial", "xr", "vr", "ar", "3d", "immersive", "unity"],
    ["game-development", "spatial-computing"],
  ),
  seedGroup(
    SPECIALIZED_GROUP_ID,
    "无法稳定归入其他一级组的垂直行业、跨界领域或特殊专业技能。",
    ["specialized", "domain", "industry", "cross-functional", "vertical", "specialist"],
    ["specialized", "misc"],
  ),
];

export async function ensureGroupCatalog(layout: SkillShelfStorageLayout): Promise<void> {
  try {
    await fs.access(layout.groupCatalogPath);
  } catch {
    await fs.writeFile(layout.groupCatalogPath, JSON.stringify(DEFAULT_MANAGED_GROUPS, null, 2) + "\n", "utf8");
  }
}

export async function loadManagedGroups(layout: SkillShelfStorageLayout): Promise<ManagedGroupRecord[]> {
  await ensureGroupCatalog(layout);
  const raw = await fs.readFile(layout.groupCatalogPath, "utf8");
  const parsed = JSON.parse(raw) as unknown;
  if (!Array.isArray(parsed)) {
    return [...DEFAULT_MANAGED_GROUPS];
  }

  const normalized = parsed
    .map((entry) => normalizeManagedGroup(entry))
    .filter((entry): entry is ManagedGroupRecord => Boolean(entry));

  return normalized.length > 0 ? normalized : [...DEFAULT_MANAGED_GROUPS];
}

export async function createManagedGroup(
  layout: SkillShelfStorageLayout,
  input: {
    group: string;
    groupDescription: string;
    keywords?: string[];
    aliases?: string[];
  },
): Promise<ManagedGroupRecord> {
  const groups = await loadManagedGroups(layout);
  const group = normalizeGroupId(input.group);
  if (!group) {
    throw new Error("group is empty after normalization");
  }
  if (groups.some((entry) => entry.group === group)) {
    throw new Error(`group already exists: ${group}`);
  }

  const created = seedGroup(group, input.groupDescription, input.keywords ?? [], input.aliases ?? [], "custom");
  groups.push(created);
  await saveManagedGroups(layout, groups);
  return created;
}

export async function updateManagedGroup(
  layout: SkillShelfStorageLayout,
  input: {
    group: string;
    newGroup?: string;
    groupDescription?: string;
    keywords?: string[];
    aliases?: string[];
  },
): Promise<{ previousGroup: string; group: ManagedGroupRecord }> {
  const groups = await loadManagedGroups(layout);
  const currentGroup = normalizeGroupId(input.group);
  const currentIndex = groups.findIndex((entry) => entry.group === currentGroup);
  if (currentIndex < 0) {
    throw new Error(`unknown group: ${input.group}`);
  }

  const current = groups[currentIndex];
  const renamedGroup = input.newGroup ? normalizeGroupId(input.newGroup) : current.group;
  if (!renamedGroup) {
    throw new Error("newGroup is empty after normalization");
  }
  if (renamedGroup !== current.group && groups.some((entry, index) => index !== currentIndex && entry.group === renamedGroup)) {
    throw new Error(`group already exists: ${renamedGroup}`);
  }

  const nextAliases = normalizeList([
    ...(input.aliases ?? current.aliases),
    ...(renamedGroup !== current.group ? [current.group] : []),
  ]);
  const updated: ManagedGroupRecord = {
    group: renamedGroup,
    groupDescription: normalizeDescription(input.groupDescription ?? current.groupDescription),
    keywords: normalizeList(input.keywords ?? current.keywords),
    aliases: nextAliases,
    source: current.source,
  };

  groups[currentIndex] = updated;
  await saveManagedGroups(layout, groups);
  return {
    previousGroup: current.group,
    group: updated,
  };
}

export async function deleteManagedGroup(
  layout: SkillShelfStorageLayout,
  input: { group: string },
): Promise<ManagedGroupRecord> {
  const groups = await loadManagedGroups(layout);
  const currentGroup = normalizeGroupId(input.group);
  const currentIndex = groups.findIndex((entry) => entry.group === currentGroup);
  if (currentIndex < 0) {
    throw new Error(`unknown group: ${input.group}`);
  }
  const current = groups[currentIndex];
  if (current.source === "builtin") {
    throw new Error(`builtin group cannot be deleted: ${current.group}`);
  }
  groups.splice(currentIndex, 1);
  await saveManagedGroups(layout, groups);
  return current;
}

export function matchManagedGroup(input: {
  skillName: string;
  description: string;
  keywords: string[];
  groups: ManagedGroupRecord[];
}): { group: string; groupDescription: string } {
  const source = normalizeList([
    input.skillName,
    input.description,
    ...input.keywords,
  ]).join(" ");

  let best: { group: string; groupDescription: string; score: number } | null = null;
  for (const candidate of input.groups) {
    const score = scoreGroupCandidate(source, candidate);
    if (!best || score > best.score) {
      best = {
        group: candidate.group,
        groupDescription: candidate.groupDescription,
        score,
      };
    }
  }

  if (!best || best.score <= 0) {
    const fallback = input.groups.find((entry) => entry.group === SPECIALIZED_GROUP_ID) ?? DEFAULT_MANAGED_GROUPS.find((entry) => entry.group === SPECIALIZED_GROUP_ID);
    if (!fallback) {
      throw new Error("missing specialized-domain fallback group");
    }
    return {
      group: fallback.group,
      groupDescription: fallback.groupDescription,
    };
  }

  return {
    group: best.group,
    groupDescription: best.groupDescription,
  };
}

function scoreGroupCandidate(source: string, candidate: ManagedGroupRecord): number {
  const normalizedSource = source.toLowerCase();
  const weightedTerms = [
    ...candidate.keywords.map((term) => ({ term, weight: 4 })),
    ...candidate.aliases.map((term) => ({ term, weight: 3 })),
    ...candidate.group.split("-").map((term) => ({ term, weight: 2 })),
    ...candidate.groupDescription.split(/[\s,，、/]+/).map((term) => ({ term, weight: 1 })),
  ];

  let score = 0;
  for (const entry of weightedTerms) {
    const term = entry.term.trim().toLowerCase();
    if (term.length < 2) {
      continue;
    }
    if (normalizedSource.includes(term)) {
      score += entry.weight;
    }
  }
  return score;
}

async function saveManagedGroups(
  layout: SkillShelfStorageLayout,
  groups: ManagedGroupRecord[],
): Promise<void> {
  const normalized = groups
    .map((entry) => normalizeManagedGroup(entry))
    .filter((entry): entry is ManagedGroupRecord => Boolean(entry))
    .sort((a, b) => a.group.localeCompare(b.group));
  await fs.writeFile(layout.groupCatalogPath, JSON.stringify(normalized, null, 2) + "\n", "utf8");
}

function normalizeManagedGroup(entry: unknown): ManagedGroupRecord | null {
  if (!entry || typeof entry !== "object") {
    return null;
  }
  const value = entry as Record<string, unknown>;
  const group = normalizeGroupId(typeof value.group === "string" ? value.group : "");
  const groupDescription = normalizeDescription(typeof value.groupDescription === "string" ? value.groupDescription : "");
  if (!group || !groupDescription) {
    return null;
  }
  return {
    group,
    groupDescription,
    keywords: normalizeList(Array.isArray(value.keywords) ? value.keywords : []),
    aliases: normalizeList(Array.isArray(value.aliases) ? value.aliases : []),
    source: value.source === "custom" ? "custom" : "builtin",
  };
}

function seedGroup(
  group: string,
  groupDescription: string,
  keywords: string[],
  aliases: string[],
  source: "builtin" | "custom" = "builtin",
): ManagedGroupRecord {
  return {
    group: normalizeGroupId(group),
    groupDescription: normalizeDescription(groupDescription),
    keywords: normalizeList(keywords),
    aliases: normalizeList(aliases),
    source,
  };
}

function normalizeDescription(value: string): string {
  return value.replace(/\s+/g, " ").trim();
}

function normalizeList(values: unknown[]): string[] {
  return Array.from(
    new Set(
      values
        .map((value) => (typeof value === "string" ? value.trim().toLowerCase() : ""))
        .map((value) => value.replace(/[_/\\]+/g, "-"))
        .map((value) => value.replace(/\s+/g, "-"))
        .map((value) => value.replace(/[^a-z0-9\u4e00-\u9fff-]+/giu, ""))
        .map((value) => value.replace(/-+/g, "-").replace(/^-|-$/g, ""))
        .filter((value) => value.length >= 2),
    ),
  );
}

function normalizeGroupId(value: string): string {
  return value
    .trim()
    .toLowerCase()
    .replace(/[_/\\]+/g, "-")
    .replace(/\s+/g, "-")
    .replace(/[^a-z0-9\u4e00-\u9fff-]+/giu, "")
    .replace(/-+/g, "-")
    .replace(/^-|-$/g, "");
}
