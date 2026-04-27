#!/usr/bin/env node
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const serverPath = join(__dirname, '..', 'src', 'server.ts');

try {
  execFileSync(process.execPath, ['--import', 'tsx/esm', serverPath], {
    stdio: 'inherit',
  });
} catch (e) {
  process.exit(e.status ?? 1);
}
