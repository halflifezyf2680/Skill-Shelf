/**
 * reinstall-smart.mjs — 按内容智能分组安装
 *
 * 1. 从上游源提取所有 skill
 * 2. 扁平安装到 staging/
 * 3. 对每个 skill 调用 matchManagedGroup 确定组
 * 4. 移入 packages/{group}/{skill}/
 * 5. 清理 staging
 *
 * 用法: node scripts/reinstall-smart.mjs [--run]
 */

import { readdir, readFile, stat, mkdir, cp, rm, writeFile, unlink } from 'fs/promises';
import { join, basename, dirname, resolve, sep } from 'path';
import { fileURLToPath } from 'url';
import matter from 'gray-matter';

const __dirname = dirname(fileURLToPath(import.meta.url));
const SHELF_ROOT = resolve(__dirname, '..', 'data', 'hub');
const PACKAGES = join(SHELF_ROOT, 'packages');
const STAGING = join(SHELF_ROOT, 'staging-install');
const GROUPS_JSON = join(SHELF_ROOT, 'config', 'groups.json');
const DRY_RUN = !process.argv.includes('--run');

const SOURCES = [
  { name: 'agency-agents-zh', path: resolve(__dirname, '..', '..', 'All-jobs', 'agency-agents-zh'), type: 'upstream' },
  { name: 'awesome-design-md', path: resolve(__dirname, '..', '..', 'All-jobs', 'awesome-design-md', 'design-md'), type: 'package' },
  { name: 'claude-code-game-studios', path: resolve(__dirname, '..', '..', 'All-jobs', 'claude-code-game-studios', 'skills'), type: 'package', targetGroup: 'game-studios' },
];

// Skip entries in upstream dirs
const SKIP_ENTRIES = new Set([
  'agent-list', 'catalog', 'contributing', 'license', 'readme',
  'upstream', 'examples', 'scripts', 'package.json', 'quickstart',
  'executive-brief', '.github', 'nexus-strategy',
]);

// NEXUS docs to skip (no valid frontmatter)
const SKIP_NEXUS = new Set([
  'coordination-agent-activation-prompts',
  'coordination-handoff-templates',
  'nexus-strategy',
  'playbooks-phase-0-discovery',
  'playbooks-phase-1-strategy',
  'playbooks-phase-2-foundation',
  'playbooks-phase-3-build',
  'playbooks-phase-4-hardening',
  'playbooks-phase-5-launch',
  'playbooks-phase-6-operate',
  'runbooks-scenario-enterprise-feature',
  'runbooks-scenario-incident-response',
  'runbooks-scenario-marketing-campaign',
  'runbooks-scenario-startup-mvp',
  'agent-activation-prompts',
  'handoff-templates',
  'phase-0-discovery',
  'phase-1-strategy',
  'phase-2-foundation',
  'phase-3-build',
  'phase-4-hardening',
  'phase-5-launch',
  'phase-6-operate',
  'scenario-enterprise-feature',
  'scenario-incident-response',
  'scenario-marketing-campaign',
  'scenario-startup-mvp',
]);

async function main() {
  console.log(`Mode: ${DRY_RUN ? 'DRY RUN' : 'EXECUTE'}\n`);

  // Load groups for matchManagedGroup
  const groupsRaw = await readFile(GROUPS_JSON, 'utf8');
  const groups = JSON.parse(groupsRaw);

  // Ensure game-studios is in groups
  if (!groups.find(g => g.group === 'game-studios')) {
    groups.push({
      group: 'game-studios',
      groupDescription: '游戏工作室全流程：立项、原型、关卡、战斗、叙事、音频、UI、发布、复盘。',
      keywords: ['game', 'gaming', 'studio', 'playtest', 'prototype', 'level', 'combat', 'narrative', 'audio', 'polish', 'retrospective', 'sprint', '游戏', '工作室', '原型', '关卡', '战斗', '叙事', '音频', '发布'],
      aliases: ['game-studio', 'game-dev-workflow'],
      source: 'builtin',
    });
    await writeFile(GROUPS_JSON, JSON.stringify(groups, null, 2) + '\n', 'utf8');
    console.log('Added game-studios to groups.json');
  }

  // Stage 1: Collect all skills to staging
  console.log('=== Stage 1: Collecting skills ===');
  const skills = []; // { stagingPath, skillName, source }
  let skipped = 0;

  // agency-agents-zh: scan {group}/{skill}.md and {group}/{subgroup}/{skill}.md
  const agencyDir = SOURCES[0].path;
  const agencyEntries = await readdir(agencyDir, { withFileTypes: true });
  for (const entry of agencyEntries) {
    if (!entry.isDirectory() || SKIP_ENTRIES.has(entry.name.toLowerCase())) { skipped++; continue; }

    const groupDir = join(agencyDir, entry.name);
    const subEntries = await readdir(groupDir, { withFileTypes: true });

    for (const sub of subEntries) {
      if (sub.name.startsWith('.')) continue;
      const fullPath = join(groupDir, sub.name);

      if (sub.isDirectory()) {
        const skillPath = join(fullPath, 'SKILL.md');
        if (await exists(skillPath)) {
          const skillName = sub.name;
          if (SKIP_NEXUS.has(skillName)) { skipped++; continue; }
          skills.push({ stagingPath: skillPath, skillName, source: 'agency-agents-zh' });
        } else {
          // Subgroup with .md files
          const mdEntries = await readdir(fullPath, { withFileTypes: true });
          for (const md of mdEntries) {
            if (!md.isFile() || !md.name.endsWith('.md')) continue;
            const stem = md.name.replace(/\.md$/, '');
            if (SKIP_ENTRIES.has(stem.toLowerCase())) { skipped++; continue; }
            if (SKIP_NEXUS.has(stem)) { skipped++; continue; }
            const skillName = stem.startsWith(entry.name + '-') ? stem : `${entry.name}-${stem}`;
            skills.push({ stagingPath: join(fullPath, md.name), skillName, source: 'agency-agents-zh' });
          }
        }
      } else if (sub.isFile() && sub.name.endsWith('.md') && sub.name !== 'SKILL.md') {
        const stem = sub.name.replace(/\.md$/, '');
        if (SKIP_ENTRIES.has(stem.toLowerCase())) { skipped++; continue; }
        const skillName = stem.startsWith(entry.name + '-') ? stem : `${entry.name}-${stem}`;
        skills.push({ stagingPath: fullPath, skillName, source: 'agency-agents-zh' });
      }
    }
  }

  // awesome-design-md
  const designMdPath = SOURCES[1].path;
  if (await exists(designMdPath)) {
    const skillMd = join(designMdPath, 'SKILL.md');
    if (await exists(skillMd)) {
      skills.push({ stagingPath: designMdPath, skillName: 'design-md', source: 'awesome-design-md', type: 'package' });
    }
  }

  // claude-code-game-studios
  const gameStudiosPath = SOURCES[2].path;
  if (await exists(gameStudiosPath)) {
    const entries = await readdir(gameStudiosPath, { withFileTypes: true });
    for (const entry of entries) {
      if (!entry.isDirectory()) continue;
      const skillMd = join(gameStudiosPath, entry.name, 'SKILL.md');
      if (await exists(skillMd)) {
        skills.push({ stagingPath: skillMd, skillName: entry.name, source: 'claude-code-game-studios', type: 'package', forceGroup: 'game-studios' });
      }
    }
  }

  console.log(`Collected ${skills.length} skills (skipped ${skipped})\n`);

  // Stage 2: Classify each skill
  console.log('=== Stage 2: Classifying ===');
  const classified = [];
  let noFrontmatter = 0;

  for (const skill of skills) {
    let raw;
    let sourceDir;

    if (skill.type === 'package') {
      sourceDir = skill.stagingPath;
      raw = await readFile(skill.stagingPath, 'utf8');
    } else {
      sourceDir = dirname(skill.stagingPath);
      raw = await readFile(skill.stagingPath, 'utf8');
    }

    const parsed = matter(raw);
    if (!parsed.data.name || !parsed.data.description) {
      noFrontmatter++;
      continue;
    }

    const name = String(parsed.data.name).trim();
    const description = String(parsed.data.description).trim();
    const keywords = deriveKeywords(description);
    const skillName = skill.skillName;

    let group;
    if (skill.forceGroup) {
      group = skill.forceGroup;
    } else {
      const result = matchManagedGroup({ skillName: name, description, keywords, groups });
      group = result.group;
    }

    classified.push({ skillName, group, name, description, source: skill.source, sourceDir, raw, type: skill.type });
  }

  console.log(`Classified: ${classified.length}, No frontmatter: ${noFrontmatter}\n`);

  // Report distribution
  const dist = new Map();
  for (const c of classified) {
    dist.set(c.group, (dist.get(c.group) ?? 0) + 1);
  }
  console.log('=== Distribution ===');
  for (const [group, count] of [...dist.entries()].sort((a, b) => b[1] - a[1])) {
    console.log(`  ${group}: ${count}`);
  }

  if (DRY_RUN) {
    console.log('\n[DRY RUN] Pass --run to execute.');
    return;
  }

  // Stage 3: Install to packages/{group}/{skillName}/
  console.log('\n=== Stage 3: Installing ===');
  let installed = 0;
  for (const c of classified) {
    try {
      const destDir = join(PACKAGES, c.group, c.skillName);
      await mkdir(destDir, { recursive: true });
      await writeFile(join(destDir, 'SKILL.md'), c.raw, 'utf8');

      // Write SOURCE.json
      await writeFile(join(destDir, 'SOURCE.json'), JSON.stringify({
        source: c.source,
        installedAt: new Date().toISOString(),
      }, null, 2) + '\n', 'utf8');

      installed++;
    } catch (err) {
      console.error(`FAIL: ${c.group}/${c.skillName} — ${err.message}`);
    }
  }
  console.log(`Installed: ${installed}/${classified.length}`);
}

// --- matchManagedGroup (ported from group-catalog.ts) ---

const SPECIALIZED_GROUP_ID = 'specialized-domain';

function matchManagedGroup(input) {
  const source = [input.skillName, input.description, ...input.keywords].join(' ').toLowerCase();

  let best = null;
  for (const candidate of input.groups) {
    const score = scoreGroupCandidate(source, candidate);
    if (!best || score > best.score) {
      best = { group: candidate.group, groupDescription: candidate.groupDescription, score };
    }
  }

  if (!best || best.score <= 0) {
    const fallback = input.groups.find(g => g.group === SPECIALIZED_GROUP_ID);
    return fallback
      ? { group: fallback.group, groupDescription: fallback.groupDescription }
      : { group: SPECIALIZED_GROUP_ID, groupDescription: 'Specialized domain skills' };
  }
  return best;
}

function scoreGroupCandidate(source, candidate) {
  let score = 0;
  const terms = [
    ...candidate.keywords.map(t => ({ term: t.toLowerCase(), weight: 4 })),
    ...candidate.aliases.map(t => ({ term: t.toLowerCase(), weight: 3 })),
    ...candidate.group.split('-').map(t => ({ term: t.toLowerCase(), weight: 2 })),
    ...candidate.groupDescription.split(/[,，;；、\s]+/).filter(t => t.length >= 2).map(t => ({ term: t.toLowerCase(), weight: 1 })),
  ];

  for (const { term, weight } of terms) {
    if (term && source.includes(term)) {
      score += weight;
    }
  }
  return score;
}

function deriveKeywords(description) {
  return [...new Set(
    description
      .toLowerCase()
      .split(/[\s,;.，、；]+/)
      .map(t => t.replace(/[^a-z0-9\u4e00-\u9fff]/g, '').replace(/^[-–—]+|[-–—]+$/g, ''))
      .filter(t => t.length >= 3)
  )];
}

async function exists(p) {
  try { await stat(p); return true; } catch { return false; }
}

main().catch(e => { console.error(e); process.exit(1); });
