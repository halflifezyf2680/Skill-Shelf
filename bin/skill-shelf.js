#!/usr/bin/env node
import { existsSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const packageRoot = join(__dirname, '..');
const rustBinaryPath = resolveRustBinary();
const cliArgs = process.argv.slice(2);

if (rustBinaryPath) {
  runProcess(rustBinaryPath, cliArgs.length === 0 ? ['mcp'] : cliArgs);
} else {
  process.stderr.write(
    'skill-shelf: Rust binary not found for the requested command.\n' +
      'Build it with `npm run rust:build` before running skill-shelf.\n',
  );
  process.exit(1);
}

function resolveRustBinary() {
  const binaryName = process.platform === 'win32' ? 'skill-shelf.exe' : 'skill-shelf';
  const candidates = [
    join(packageRoot, 'rust', 'skill-shelf', 'target', 'release', binaryName),
    join(packageRoot, 'rust', 'skill-shelf', 'target', 'debug', binaryName),
  ];

  return candidates.find((candidate) => existsSync(candidate));
}

function runProcess(command, args) {
  const result = spawnSync(command, args, { stdio: 'inherit' });

  if (result.error) {
    process.stderr.write(`skill-shelf: failed to start ${command}: ${result.error.message}\n`);
    process.exit(1);
  }

  if (result.signal) {
    process.kill(process.pid, result.signal);
    return;
  }

  process.exit(result.status ?? 1);
}
