/**
 * fix-groups.mjs — 按 skillId 前缀批量归位到正确组
 *
 * 前缀 → 组映射规则：
 *   skillId 第一段匹配映射表，挪到对应组目录
 *   无法匹配的保持不动
 *
 * 用法: node scripts/fix-groups.mjs [--run]
 */

import { readdir, mkdir, cp, rm, stat, unlink } from 'fs/promises';
import { dirname, join, basename, resolve } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const packagesRoot = resolve(__dirname, '..', 'data', 'hub', 'packages');
const DRY_RUN = !process.argv.includes('--run');

// 前缀 → 目标组映射
const PREFIX_MAP = {
  // 直接匹配组名
  'engineering':       'engineering',
  'design':            'design',
  'marketing':         'marketing',
  'sales':             'sales',
  'finance':           'finance',
  'product':           'product',
  'testing':           'testing-qa',
  'test':              'testing-qa',

  // 多段组名
  'project':           'project-management',
  'supply':            'supply-chain',
  'support':           'support-operations',
  'paid':              'paid-media',
  'legal':             'legal-compliance',
  'hr':                'hr-talent',
  'academic':          'academic-research',

  // 空间/游戏 → spatial-gaming
  'spatial':           'spatial-gaming',
  'game':              'spatial-gaming',

  // specialized 前缀 → specialized-domain
  'specialized':       'specialized-domain',

  // frontend → engineering
  'frontend':          'engineering',

  // gh (GitHub) → engineering
  'gh':                'engineering',

  // figma → design
  'figma':             'design',

  // playwright → engineering
  'playwright':        'engineering',

  // writing → specialized-domain
  'writing':           'specialized-domain',
};

async function main() {
  console.log(`Packages root: ${packagesRoot}`);
  console.log(`Mode: ${DRY_RUN ? 'DRY RUN' : 'EXECUTE'}\n`);

  const groupDirs = await readdir(packagesRoot);
  const moves = [];
  const skipped = [];

  for (const groupDir of groupDirs.sort()) {
    const groupPath = join(packagesRoot, groupDir);
    const s = await stat(groupPath).catch(() => null);
    if (!s || !s.isDirectory()) continue;
    if (groupDir.startsWith('.')) continue;

    const skillDirs = await readdir(groupPath);
    for (const skillDir of skillDirs.sort()) {
      const skillPath = join(groupPath, skillDir);
      const ss = await stat(skillPath).catch(() => null);
      if (!ss || !ss.isDirectory()) continue;

      const prefix = skillDir.split('-')[0];
      const targetGroup = PREFIX_MAP[prefix];

      if (!targetGroup) {
        skipped.push({ skill: skillDir, group: groupDir, reason: 'no prefix match' });
        continue;
      }

      if (targetGroup === groupDir) {
        // Already in correct group
        continue;
      }

      moves.push({
        skill: skillDir,
        from: groupDir,
        to: targetGroup,
        path: skillPath,
        dest: join(packagesRoot, targetGroup, skillDir),
      });
    }
  }

  console.log(`=== Moves needed: ${moves.length} ===\n`);

  // Group by destination
  const byTarget = new Map();
  for (const m of moves) {
    const list = byTarget.get(m.to) ?? [];
    list.push(m);
    byTarget.set(m.to, list);
  }

  for (const [target, list] of [...byTarget.entries()].sort()) {
    console.log(`→ ${target} (${list.length}):`);
    for (const m of list) {
      console.log(`  ${m.skill}  ←  ${m.from}`);
    }
    console.log();
  }

  if (skipped.length > 0) {
    const noMatch = skipped.filter(s => s.reason === 'no prefix match');
    console.log(`No prefix match (${noMatch.length}):`);
    for (const s of noMatch) {
      console.log(`  ${s.group}/${s.skill}`);
    }
  }

  if (DRY_RUN) {
    console.log(`\n[DRY RUN] Pass --run to execute.`);
    return;
  }

  console.log(`\n=== Executing ===`);
  let moved = 0;
  for (const m of moves) {
    try {
      await mkdir(m.to, { recursive: true });
      await cp(m.path, m.dest, { recursive: true });
      await rm(m.path, { recursive: true, force: true });
      moved++;
    } catch (err) {
      console.error(`FAIL: ${m.skill} → ${err.message}`);
    }
  }
  console.log(`Moved: ${moved}/${moves.length}`);

  // Clean index
  const indexRoot = resolve(__dirname, '..', 'data', 'hub', 'index');
  for (const sub of ['skills', 'groups']) {
    const dir = join(indexRoot, sub);
    try {
      const files = await readdir(dir);
      await Promise.all(files.map(f => rm(join(dir, f), { force: true })));
    } catch {}
  }
  try { await unlink(join(indexRoot, 'group-list.json')); } catch {}

  console.log('Index cleaned. Restart server to rebuild.');
}

main().catch(e => { console.error(e); process.exit(1); });
