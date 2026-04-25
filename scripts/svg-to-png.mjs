import { Resvg } from '@resvg/resvg-js';
import { readdir, readFile, writeFile, mkdir } from 'fs/promises';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const svgDir = join(__dirname, '..', 'docs', 'diagrams', 'svg');
const pngDir = join(__dirname, '..', 'docs', 'diagrams', 'png');
const SCALE = 2; // 2x for retina

async function convert() {
  const files = await readdir(svgDir);
  const svgFiles = files.filter(f => f.endsWith('.svg')).sort();

  for (const file of svgFiles) {
    const svgPath = join(svgDir, file);
    const pngPath = join(pngDir, file.replace('.svg', '.png'));
    const svgData = await readFile(svgPath);

    const resvg = new Resvg(svgData, {
      fitTo: {
        mode: 'zoom',
        value: SCALE,
      },
    });

    const pngData = resvg.render().asPng();
    await writeFile(pngPath, pngData);

    const { width, height } = resvg;
    console.log(`${file} → ${width}×${height}px`);
  }
}

convert().catch(err => {
  console.error(err);
  process.exit(1);
});
