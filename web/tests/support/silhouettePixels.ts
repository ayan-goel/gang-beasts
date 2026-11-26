import type { Page } from '@playwright/test';

export async function silhouettePixels(page: Page, image: Buffer, color: number[]) {
  return page.evaluate(async ({png, color}) => {
    const bitmap = await createImageBitmap(await (await fetch('data:image/png;base64,' + png)).blob());
    const surface = new OffscreenCanvas(bitmap.width, bitmap.height);
    const context = surface.getContext('2d')!;
    context.drawImage(bitmap, 0, 0);
    bitmap.close();
    const {data} = context.getImageData(0, 0, surface.width, surface.height);
    let count = 0, top = surface.height, bottom = -1;
    for (let offset = 0; offset < data.length; offset += 4) {
      if (!color.every((channel, i) => Math.abs(data[offset + i]! - channel) <= 1)) continue;
      const y = Math.floor(offset / (4 * surface.width));
      count++;
      top = Math.min(top, y);
      bottom = Math.max(bottom, y);
    }
    return {count, height: count ? bottom - top + 1 : 0};
  }, {png: image.toString('base64'), color});
}
