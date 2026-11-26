import { loadSkyline, loadedPacket, type SkylineContent } from './content';

export interface LoadedArena {
  View: typeof import('../online/OnlineMatch').default;
  content: SkylineContent;
}

export interface LoadState { loading: boolean; error: string }

export class ContentHandshake {
  arena?: LoadedArena;
  private timer?: number;
  private attempt = 0;

  constructor(private socket: WebSocket, private publish: (state: LoadState) => void) {}

  async ready() {
    if (this.timer !== undefined) return;
    const attempt = ++this.attempt;
    this.publish({loading: true, error: ''});
    this.timer = window.setTimeout(() => this.finish('Skyline took too long to load. Try again.'), 12000);
    try {
      const [content, arena] = await Promise.all([loadSkyline(), import('../online/OnlineMatch')]);
      if (attempt !== this.attempt || this.socket.readyState !== WebSocket.OPEN) return;
      this.arena = {View: arena.default, content};
      this.socket.send(loadedPacket(content));
    } catch {
      if (attempt === this.attempt) this.finish('Could not load Skyline. Check your connection and try again.');
    }
  }

  acknowledge(bytes: Uint8Array) {
    if (bytes.length !== 3 || bytes[1] !== 1 || bytes[2]! > 2) throw new Error('Invalid map response');
    if (this.timer === undefined) return;
    if (bytes[2] === 0) {
      this.socket.send(Uint8Array.of(18, 1, 1));
      this.finish('');
    } else {
      this.finish(bytes[2] === 1 ? 'Your Skyline build differs from the server. Reload to get the current build.'
        : 'The server is waiting for Skyline to load. Try again.');
    }
  }

  dispose() {
    this.attempt++;
    clearTimeout(this.timer);
    this.timer = undefined;
  }

  private finish(error: string) {
    this.dispose();
    this.publish({loading: false, error});
  }
}
