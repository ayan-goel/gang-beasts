import type { Page } from '@playwright/test';

export async function measureLink(page: Page, port: number) {
  await page.addInitScript(port => {
    const samples: number[] = [], sent = new Map<number, number>();
    Object.assign(window, {matchRtts:samples});
    window.WebSocket = class extends WebSocket {
      constructor(url: string | URL, protocols?: string | string[]) {
        const path = new URL(url, location.href);
        if (path.pathname === '/ws') path.port = String(port);
        super(path, protocols);
        this.addEventListener('message', event => {
          if (!(event.data instanceof ArrayBuffer) || event.data.byteLength !== 6) return;
          const bytes = new DataView(event.data);
          if (bytes.getUint8(0) !== 37) return;
          const id = bytes.getUint32(2,true), at = sent.get(id);
          if (at !== undefined) { samples.push(performance.now() - at); sent.delete(id); }
        });
      }
      override send(data: Parameters<WebSocket['send']>[0]) {
        if (data instanceof Uint8Array && data.length === 6 && data[0] === 36) {
          sent.set(new DataView(data.buffer, data.byteOffset).getUint32(2,true),performance.now());
        }
        super.send(data);
      }
    };
  }, port);
  return () => page.evaluate(() => Reflect.get(window,'matchRtts') as number[]);
}
