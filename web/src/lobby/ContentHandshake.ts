import { assetWaitMs } from '../game/assetLoading';
import { supportsPhysics, physicsUnsupported } from '../game/physicsSupport';
import { loadSkyline, loadedPacket, type SkylineContent } from './content';
import type { PreparedArena } from '../online/preparedArena';

export interface LoadedArena {
  graphics?: PreparedArena;
  View: typeof import('../online/OnlineMatch').default;
  content: SkylineContent;
}

export interface LoadState { loading: boolean; error: string }

export class ContentHandshake {
  arena?: LoadedArena;
  private timer?: number;
  private attempt = 0;
  private pending?: AbortController;

  constructor(private socket: WebSocket, private publish: (state: LoadState) => void) {}

  async ready(prepareGraphics = true) {
    if (this.timer !== undefined) return;
    if (!supportsPhysics()) { this.finish(physicsUnsupported); return; }
    const attempt = ++this.attempt;
    const pending = this.pending = new AbortController();
    this.publish({loading: true, error: ''});
    this.timer = window.setTimeout(() => this.finish('The arenas took too long to load. Try again.'), assetWaitMs);
    try {
      const [content, arena] = await Promise.all([loadSkyline(pending.signal), import('../online/OnlineMatch')]);
      if (attempt !== this.attempt || this.socket.readyState !== WebSocket.OPEN) return;
      const graphics = prepareGraphics ? new arena.PreparedArena() : undefined;
      this.arena?.graphics?.dispose();
      await graphics?.prepare(content, pending.signal);
      if (attempt !== this.attempt || this.socket.readyState !== WebSocket.OPEN) {
        graphics?.dispose();
        return;
      }
      this.arena = {View: arena.default, content, graphics};
      this.socket.send(loadedPacket(content));
    } catch {
      if (attempt === this.attempt) this.finish('Could not load the arenas. Check your connection and try again.');
    }
  }

  acknowledge(bytes: Uint8Array) {
    if (bytes.length !== 3 || bytes[1] !== 1 || bytes[2]! > 2) throw new Error('Invalid map response');
    if (this.timer === undefined) return;
    if (bytes[2] === 0) {
      this.socket.send(Uint8Array.of(18, 1, 1));
      this.finish('');
    } else {
      this.finish(bytes[2] === 1 ? 'Your arena catalog differs from the server. Reload to get the current build.'
        : 'The server is waiting for the arenas to load. Try again.');
    }
  }

  dispose() {
    this.arena?.graphics?.dispose();
    this.clearAttempt();
  }

  private clearAttempt() {
    this.attempt++;
    this.pending?.abort();
    this.pending = undefined;
    clearTimeout(this.timer);
    this.timer = undefined;
  }

  private finish(error: string) {
    if (error) this.arena?.graphics?.dispose();
    this.clearAttempt();
    this.publish({loading: false, error});
  }
}
