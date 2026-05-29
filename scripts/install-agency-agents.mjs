/**
 * install-agency-agents.mjs
 *
 * 从 agency-agents-zh 上游安装到 skill shelf
 * 结构: {group}/{skill}.md 或 {group}/{skill}/SKILL.md
 * 目标: packages/{group}/{skill}/SKILL.md
 *
 * 用法: node scripts/install-agency-agents.mjs [--run]
 */

import { readdir, readFile, stat, mkdir, cp, rm, writeFile } from 'fs/promises';
import { join, basename, dirname, resolve, parse } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const DRY_RUN = !process.argv.includes('--run');

const SOURCE = resolve(__dirname, '..', '..', 'All-jobs', 'agency-agents-zh');
const PACKAGES = resolve(__dirname, '..', 'data', 'hub', 'packages');

// 上游目录名 → skill shelf 组名映射
const GROUP_MAP = {
  'academic':            'academic-research',
  'design':              'design',
  'engineering':         'engineering',
  'finance':             'finance',
  'game-development':    'spatial-gaming',
  'hr':                  'hr-talent',
  'integrations':        'engineering',
  'legal':               'legal-compliance',
  'marketing':           'marketing',
  'paid-media':          'paid-media',
  'product':             'product',
  'project-management':  'project-management',
  'sales':               'sales',
  'spatial-computing':   'spatial-gaming',
  'specialized':         'specialized-domain',
  'strategy':            'specialized-domain',
  'supply-chain':        'supply-chain',
  'support':             'support-operations',
  'testing':             'testing-qa',
};

// 跳过的非 skill 目录/文件
const SKIP_ENTRIES = new Set([
  'agent-list', 'catalog', 'contributing', 'license', 'readme',
  'upstream', 'examples', 'scripts', 'package.json', 'quickstart',
  'executive-brief', '.github',
]);

async function main() {
  console.log(`Source: ${SOURCE}`);
  console.log(`Target: ${PACKAGES}`);
  console.log(`Mode: ${DRY_RUN ? 'DRY RUN' : 'EXECUTE'}\n`);

  const entries = await readdir(SOURCE, { withFileTypes: true });
  const plans = [];
  let skipped = 0;

  for (const entry of entries) {
    if (!entry.isDirectory()) continue;
    const groupName = entry.name;

    // Skip non-skill directories
    if (SKIP_ENTRIES.has(groupName.toLowerCase())) {
      skipped++;
      continue;
    }

    const targetGroup = GROUP_MAP[groupName] ?? groupName;
    const sourceDir = join(SOURCE, groupName);
    const groupPlans = await processGroup(sourceDir, targetGroup, groupName);
    plans.push(...groupPlans);
  }

  console.log(`=== Install plan ===`);
  console.log(`Total: ${plans.length} skills`);

  // Group by target
  const byGroup = new Map();
  for (const p of plans) {
    const list = byGroup.get(p.targetGroup) ?? [];
    list.push(p);
    byGroup.set(p.targetGroup, list);
  }
  for (const [group, list] of [...byGroup.entries()].sort()) {
    console.log(`  ${group}: ${list.length}`);
  }
  console.log(`Skipped entries: ${skipped}\n`);

  if (DRY_RUN) {
    console.log('[DRY RUN] Pass --run to execute.');
    return;
  }

  // Execute
  let installed = 0;
  let failed = 0;
  for (const p of plans) {
    try {
      await mkdir(p.destDir, { recursive: true });

      if (p.type === 'package') {
        // Directory: copy all contents
        await cp(p.sourcePath, p.destDir, { recursive: true });
      } else {
        // Raw .md: copy as SKILL.md
        const raw = await readFile(p.sourcePath, 'utf8');
        await writeFile(join(p.destDir, 'SKILL.md'), raw, 'utf8');
      }

      // Write SOURCE.json provenance
      const sourceJson = JSON.stringify({
        source: 'agency-agents-zh',
        upstreamDir: p.upstreamGroup,
        installedAt: new Date().toISOString(),
      }, null, 2) + '\n';
      await writeFile(join(p.destDir, 'SOURCE.json'), sourceJson, 'utf8');
      installed++;
    } catch (err) {
      console.error(`FAIL: ${p.targetGroup}/${p.skillName} — ${err.message}`);
      failed++;
    }
  }
  console.log(`Installed: ${installed}, Failed: ${failed}`);
}

async function processGroup(sourceDir, targetGroup, upstreamGroup) {
  const plans = [];
  const entries = await readdir(sourceDir, { withFileTypes: true });

  for (const entry of entries) {
    if (entry.name.startsWith('.')) continue;

    if (entry.isDirectory()) {
      // Sub-directory: check if it contains SKILL.md
      const skillPath = join(sourceDir, entry.name, 'SKILL.md');
      if (await exists(skillPath)) {
        plans.push({
          skillName: entry.name,
          targetGroup,
          upstreamGroup,
          sourcePath: join(sourceDir, entry.name),
          destDir: join(PACKAGES, targetGroup, entry.name),
          type: 'package',
        });
        continue;
      }

      // Sub-directory without SKILL.md: check for .md files (subgroup)
      const subEntries = await readdir(join(sourceDir, entry.name), { withFileTypes: true });
      for (const sub of subEntries) {
        if (!sub.isFile() || !sub.name.endsWith('.md')) continue;
        if (SKIP_ENTRIES.has(basename(sub.name, '.md').toLowerCase())) continue;

        // Build skill name: {parent-dir}-{file-stem}
        // e.g., godot/godot-shader-developer.md → godot-shader-developer
        const stem = sub.name.replace(/\.md$/, '');
        // Avoid double prefix: if parent dir name is already part of stem
        const parentName = entry.name.toLowerCase();
        const skillName = stem.startsWith(parentName + '-')
          ? stem
          : `${entry.name}-${stem}`;

        plans.push({
          skillName,
          targetGroup,
          upstreamGroup,
          sourcePath: join(sourceDir, entry.name, sub.name),
          destDir: join(PACKAGES, targetGroup, skillName),
          type: 'raw-md',
        });
      }
      continue;
    }

    if (!entry.isFile() || !entry.name.endsWith('.md')) continue;
    if (SKIP_ENTRIES.has(basename(entry.name, '.md').toLowerCase())) continue;

    // Standalone .md file: skill name is the file stem
    const stem = entry.name.replace(/\.md$/, '');
    plans.push({
      skillName: stem,
      targetGroup,
      upstreamGroup,
      sourcePath: join(sourceDir, entry.name),
      destDir: join(PACKAGES, targetGroup, stem),
      type: 'raw-md',
    });
  }

  return plans;
}

async function exists(path) {
  try {
    await stat(path);
    return true;
  } catch {
    return false;
  }
}

main().catch(e => { console.error(e); process.exit(1); });
