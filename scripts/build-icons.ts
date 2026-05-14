import sharp from 'sharp';
import { writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import png2icons from 'png2icons';

const MASTER = resolve('src-tauri/icons/icon.png');
const OUT = resolve('src-tauri/icons');

async function pngAt(size: number, name: string): Promise<void> {
  await sharp(MASTER).resize(size, size).png().toFile(resolve(OUT, name));
}

(async () => {
  await pngAt(32, '32x32.png');
  await pngAt(128, '128x128.png');
  await pngAt(256, '128x128@2x.png');

  const master = await sharp(MASTER).png().toBuffer();
  const icns = png2icons.createICNS(master, png2icons.BICUBIC, 0);
  const ico = png2icons.createICO(master, png2icons.BICUBIC, 0, false);
  if (!icns) throw new Error('png2icons.createICNS returned null');
  if (!ico) throw new Error('png2icons.createICO returned null');
  writeFileSync(resolve(OUT, 'icon.icns'), icns);
  writeFileSync(resolve(OUT, 'icon.ico'), ico);

  console.log(`icons written to ${OUT}`);
})();
