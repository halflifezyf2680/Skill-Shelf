import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import {fileURLToPath} from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const skillRoot = path.resolve(here, '..');

const parseArgs = () => {
  const args = {out: null};
  for (let index = 2; index < process.argv.length; index += 1) {
    const arg = process.argv[index];
    const next = process.argv[index + 1];
    if (arg === '--out') {
      args.out = next;
      index += 1;
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }
  if (!args.out) {
    throw new Error('Missing --out path');
  }
  return args;
};

const args = parseArgs();
const source = path.join(skillRoot, 'assets', 'vite-infographic');
const target = path.resolve(process.cwd(), args.out);

if (fs.existsSync(target) && fs.readdirSync(target).length > 0) {
  throw new Error(`Output directory is not empty: ${target}`);
}

fs.mkdirSync(target, {recursive: true});
fs.cpSync(source, target, {recursive: true});
console.log(`Created React infographic frontend at ${target}`);
