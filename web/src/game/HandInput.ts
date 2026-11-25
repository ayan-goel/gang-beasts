export type HandIndex = 0 | 1;

export class HandInput {
  private readonly sources = [new Set<string>(), new Set<string>()] as const;

  constructor(
    private readonly send: (side: HandIndex, held: boolean) => void,
    private readonly cancel: () => void,
    private readonly allowed: () => boolean,
  ) {}

  hold(side: HandIndex, source: string, down: boolean): boolean {
    if (down && !this.allowed()) return false;
    const pressed = this.sources[side];
    const held = pressed.size > 0;
    if (down) pressed.add(source); else pressed.delete(source);
    // Releasing the mouse must not release a hand still held by the keyboard.
    if (held !== (pressed.size > 0)) this.send(side, !held);
    return true;
  }

  clear(): void {
    for (const sources of this.sources) sources.clear();
    this.cancel();
  }
}
