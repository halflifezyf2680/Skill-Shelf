import { chromium } from 'file:///C:/Users/HomeAdmin/AppData/Roaming/npm/node_modules/@playwright/mcp/node_modules/playwright/index.mjs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const pngDir = resolve(__dirname, '..', 'png');
const server = 'http://127.0.0.1:8768';

const files = [
  { name: '01-overview',         w: 640, h: 1138 },
  { name: '02-routing',          w: 640, h: 1138 },
  { name: '03-storage',          w: 640, h: 1138 },
  { name: '04-classification',   w: 640, h: 1138 },
];

const browser = await chromium.launch({
  executablePath: resolve(
    process.env.LOCALAPPDATA,
    'ms-playwright',
    'chromium-1208',
    'chrome-win64',
    'chrome.exe'
  ),
});
const ctx = await browser.newContext();
const page = await ctx.newPage();

for (const f of files) {
  await page.setViewportSize({ width: f.w, height: f.h });
  await page.goto(`${server}/${f.name}.svg`, { waitUntil: 'networkidle' });
  await page.screenshot({
    path: resolve(pngDir, `${f.name}.png`),
    clip: { x: 0, y: 0, width: f.w, height: f.h },
  });
  console.log(`${f.name}.png  ${f.w}x${f.h}`);
}

await browser.close();
