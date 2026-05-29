import { readdir, readFile, mkdir, cp, rm, stat, rename, writeFile } from "fs/promises";
import { join, basename, dirname, resolve } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const shelfRoot = resolve(__dirname, "..", "data", "hub");
const packagesRoot = join(shelfRoot, "packages");
const skillsIndexRoot = join(shelfRoot, "index", "skills");
const groupsIndexRoot = join(shelfRoot, "index", "groups");
const groupListPath = join(shelfRoot, "index", "group-list.json");

// Dry run by default. Pass --run to execute.
const DRY_RUN = !process.argv.includes("--run");

/**
 * Migration script: flatten packages/{skillId}/ → packages/{group}/{skillId}/
 *
 * Steps:
 * 1. Read current group assignments from index/skills/*.json
 * 2. Strip redundant prefix from skillId (first segment === second segment → remove first)
 * 3. Move directories to packages/{group}/{clean-skillId}/
 * 4. Clean up old index files (rebuilt on next startup)
 */

async function main() {
  console.log(`Shelf root: ${shelfRoot}`);
  console.log(`Packages root: ${packagesRoot}`);
  console.log(`Mode: ${DRY_RUN ? "DRY RUN" : "EXECUTE"}\n`);

  // 1. Load current group assignments from skill index files
  const assignments = new Map(); // skillId → group
  const indexFiles = await readdir(skillsIndexRoot);
  for (const file of indexFiles) {
    if (!file.endsWith(".json")) continue;
    const content = JSON.parse(await readFile(join(skillsIndexRoot, file), "utf8"));
    assignments.set(content.skillId, content.group);
  }
  console.log(`Loaded ${assignments.size} group assignments from index.\n`);

  // 2. Scan actual package directories
  const packageDirs = await readdir(packagesRoot);
  const toMove = []; // { oldDir, newDir, group, oldId, newId }
  const skipped = [];
  const errors = [];

  // Track used (group, newId) pairs to detect collisions
  const usedKeys = new Set();

  for (const dir of packageDirs.sort()) {
    const dirPath = join(packagesRoot, dir);

    // Skip non-directories and hidden dirs
    const s = await stat(dirPath).catch(() => null);
    if (!s || !s.isDirectory()) {
      skipped.push({ dir, reason: "not a directory" });
      continue;
    }
    if (dir.startsWith(".")) {
      skipped.push({ dir, reason: "hidden" });
      continue;
    }

    // Skip .bak dirs
    if (dir.endsWith(".bak")) {
      skipped.push({ dir, reason: ".bak directory" });
      continue;
    }

    // Get group assignment
    const group = assignments.get(dir);
    if (!group) {
      skipped.push({ dir, reason: "no group assignment in index" });
      continue;
    }

    // Strip redundant prefix
    const segments = dir.split("-");
    let newId = dir;

    if (segments.length >= 2 && segments[0] === segments[1]) {
      // Double prefix: academic-academic-xxx → academic-xxx
      newId = segments.slice(1).join("-");
    }

    // Also handle deeper duplicates: game-development-unity-unity-architect
    // Check if any segment appears consecutively
    const newSegments = newId.split("-");
    const deduped = [];
    for (let i = 0; i < newSegments.length; i++) {
      if (i > 0 && newSegments[i] === newSegments[i - 1]) {
        continue; // skip consecutive duplicate
      }
      deduped.push(newSegments[i]);
    }
    if (deduped.length < newSegments.length) {
      newId = deduped.join("-");
    }

    // Handle collision
    const key = `${group}/${newId}`;
    if (usedKeys.has(key)) {
      // Append numeric suffix to disambiguate
      let suffix = 2;
      while (usedKeys.has(`${group}/${newId}-${suffix}`)) suffix++;
      newId = `${newId}-${suffix}`;
    }
    usedKeys.add(`${group}/${newId}`);

    const newDir = join(packagesRoot, group, newId);
    toMove.push({ oldDir: dirPath, newDir, group, oldId: dir, newId });
  }

  // 3. Report plan
  console.log(`=== Migration Plan ===`);
  console.log(`To move: ${toMove.length}`);
  console.log(`Skipped: ${skipped.length}`);
  console.log();

  // Show a sample
  const sample = toMove.slice(0, 10);
  for (const m of sample) {
    console.log(`  ${m.group}/${m.newId}  ←  ${m.oldId}`);
  }
  if (toMove.length > 10) {
    console.log(`  ... and ${toMove.length - 10} more`);
  }

  if (skipped.length > 0) {
    console.log(`\nSkipped (${skipped.length}):`);
    for (const s of skipped.slice(0, 5)) {
      console.log(`  ${s.dir} — ${s.reason}`);
    }
    if (skipped.length > 5) console.log(`  ... and ${skipped.length - 5} more`);
  }

  if (DRY_RUN) {
    console.log(`\n[DRY RUN] No changes made. Pass --run to execute.`);
    return;
  }

  // 4. Execute moves
  console.log(`\n=== Executing ===`);
  let moved = 0;
  for (const m of toMove) {
    try {
      // Create group directory if needed
      await mkdir(dirname(m.newDir), { recursive: true });

      // Skip if source === destination (unlikely but possible)
      if (resolve(m.oldDir) === resolve(m.newDir)) {
        console.log(`  SKIP (same path): ${m.oldId}`);
        continue;
      }

      // Move directory
      await cp(m.oldDir, m.newDir, { recursive: true });
      await rm(m.oldDir, { recursive: true, force: true });
      moved++;

      if (moved <= 5 || moved % 50 === 0) {
        console.log(`  ${m.group}/${m.newId}  ←  ${m.oldId}`);
      }
    } catch (err) {
      errors.push({ oldId: m.oldId, error: err.message });
      console.error(`  FAIL: ${m.oldId} → ${err.message}`);
    }
  }
  console.log(`Moved: ${moved}/${toMove.length}`);

  if (errors.length > 0) {
    console.log(`\nErrors (${errors.length}):`);
    for (const e of errors) {
      console.error(`  ${e.oldId}: ${e.error}`);
    }
  }

  // 5. Clean up old index files (will be rebuilt on next startup)
  console.log(`\nCleaning old index files...`);
  try {
    const groupFiles = await readdir(groupsIndexRoot);
    await Promise.all(groupFiles.map(f => rm(join(groupsIndexRoot, f), { force: true })));
    console.log(`  Cleared ${groupFiles.length} group index files`);
  } catch {}
  try {
    const skillFiles = await readdir(skillsIndexRoot);
    await Promise.all(skillFiles.map(f => rm(join(skillsIndexRoot, f), { force: true })));
    console.log(`  Cleared ${skillFiles.length} skill index files`);
  } catch {}
  try {
    await rm(groupListPath, { force: true });
    console.log(`  Cleared group-list.json`);
  } catch {}

  // 6. Clean up empty old directories
  console.log(`\nCleaning empty directories...`);
  const remaining = await readdir(packagesRoot);
  let cleaned = 0;
  for (const dir of remaining) {
    const dirPath = join(packagesRoot, dir);
    const s = await stat(dirPath).catch(() => null);
    if (s && s.isDirectory()) {
      // Check if it's a flat skill dir (has SKILL.md) — shouldn't happen after migration
      try {
        const entries = await readdir(dirPath);
        if (entries.length === 0) {
          await rm(dirPath, { recursive: true, force: true });
          cleaned++;
        }
      } catch {}
    }
  }
  console.log(`  Cleaned ${cleaned} empty directories`);

  console.log(`\nDone. Run the server to rebuild indexes.`);
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
