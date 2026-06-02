#!/usr/bin/env node
import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, chmodSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { basename, dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const packageRoot = resolve(__dirname, '..');
const args = parseArgs(process.argv.slice(2));
const packageJson = readJson(join(packageRoot, 'package.json'));

const target = requiredArg(args, 'target');
const binaryPath = resolve(packageRoot, requiredArg(args, 'binary'));
const version = (args.version ?? packageJson.version).replace(/^v/, '');
const outDir = resolve(packageRoot, args['out-dir'] ?? 'dist/release');
const archiveExt = args['archive-ext'] ?? (process.platform === 'win32' ? 'zip' : 'tar.gz');
const packageName = `skill-shelf-v${version}-${targetLabel(target)}`;
const packageDir = join(outDir, packageName);
const archivePath = join(outDir, `${packageName}.${archiveExt}`);

if (!existsSync(binaryPath)) {
  throw new Error(`Rust binary does not exist: ${binaryPath}`);
}

rmSync(packageDir, { recursive: true, force: true });
mkdirSync(join(packageDir, 'bin'), { recursive: true });

copyFile('LICENSE');
copyFile('README.md');
copyFile('package.json');
copyFile('server.json');
copyDir('data/hub/config');
copyDir('data/hub/packages');
copyFile('bin/skill-shelf.js');

const releaseBinaryName = target.includes('windows') ? 'skill-shelf.exe' : 'skill-shelf';
const releaseBinaryPath = join(packageDir, 'bin', releaseBinaryName);
cpSync(binaryPath, releaseBinaryPath);

if (!target.includes('windows')) {
  chmodSync(join(packageDir, 'bin', 'skill-shelf.js'), 0o755);
  chmodSync(releaseBinaryPath, 0o755);
}

rmSync(archivePath, { force: true });
createArchive();

process.stdout.write(`${archivePath}\n`);

function copyFile(relativePath) {
  const source = join(packageRoot, relativePath);
  const destination = join(packageDir, relativePath);
  mkdirSync(dirname(destination), { recursive: true });
  cpSync(source, destination);
}

function copyDir(relativePath) {
  const source = join(packageRoot, relativePath);
  const destination = join(packageDir, relativePath);
  cpSync(source, destination, { recursive: true });
}

function createArchive() {
  if (archiveExt === 'zip') {
    const command = [
      '-NoProfile',
      '-Command',
      `Compress-Archive -LiteralPath '${escapePowerShell(packageDir)}' -DestinationPath '${escapePowerShell(archivePath)}' -Force`,
    ];
    execFileSync('powershell', command, { stdio: 'inherit' });
    return;
  }

  if (archiveExt === 'tar.gz') {
    execFileSync('tar', ['-czf', archivePath, '-C', outDir, basename(packageDir)], {
      stdio: 'inherit',
    });
    return;
  }

  throw new Error(`Unsupported archive extension: ${archiveExt}`);
}

function parseArgs(rawArgs) {
  const parsed = {};
  for (let index = 0; index < rawArgs.length; index += 1) {
    const arg = rawArgs[index];
    if (!arg.startsWith('--')) {
      throw new Error(`Unexpected positional argument: ${arg}`);
    }

    const key = arg.slice(2);
    const value = rawArgs[index + 1];
    if (!value || value.startsWith('--')) {
      throw new Error(`Missing value for --${key}`);
    }

    parsed[key] = value;
    index += 1;
  }

  return parsed;
}

function requiredArg(parsed, key) {
  if (!parsed[key]) {
    throw new Error(`Missing required argument --${key}`);
  }

  return parsed[key];
}

function readJson(path) {
  return JSON.parse(readFileSync(path, 'utf8'));
}

function escapePowerShell(value) {
  return value.replaceAll("'", "''");
}

function targetLabel(target) {
  const labels = {
    'x86_64-pc-windows-msvc': 'windows-x64',
    'aarch64-pc-windows-msvc': 'windows-arm64',
    'x86_64-unknown-linux-gnu': 'linux-x64',
    'aarch64-unknown-linux-gnu': 'linux-arm64',
    'x86_64-apple-darwin': 'macos-x64',
    'aarch64-apple-darwin': 'macos-arm64',
  };

  return labels[target] ?? target;
}
