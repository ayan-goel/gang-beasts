import type { MovementPort } from './movementInspection';

const directions: Record<string, readonly [number, number]> = {
  KeyW: [0,-1], ArrowUp: [0,-1], KeyS: [0,1], ArrowDown: [0,1],
  KeyA: [-1,0], ArrowLeft: [-1,0], KeyD: [1,0], ArrowRight: [1,0],
};

export class MovementInput {
  private keys = new Set<string>();
  private pointers = new Map<number, readonly [number, number]>();
  private running = false;
  command: readonly [number, number, boolean] = [0,0,false];
  constructor(
    private movement: Pick<MovementPort, 'setInput' | 'setJump' | 'clearJumpInput'>,
    private allowed: () => boolean,
  ) {}
  tapJump(): void {
    if (!this.allowed() || this.keys.has('Space')) return;
    this.movement.setJump(true);
    this.movement.setJump(false);
  }
  get run(): boolean { return this.running; }
  setRun(run: boolean): void { this.running = run; this.publish(); }
  key(code: string, down: boolean, repeat = false): boolean {
    if (repeat) return this.keys.has(code);
    if (!(code in directions) && !['ShiftLeft','ShiftRight','Space'].includes(code)) return false;
    if (down && !this.allowed()) return false;
    if (code === 'Space') {
      if (down !== this.keys.has(code)) this.movement.setJump(down);
      if (down) this.keys.add(code); else this.keys.delete(code);
      return true;
    }
    if (down) this.keys.add(code); else this.keys.delete(code);
    this.publish(); return true;
  }
  pointer(id: number, direction: readonly [number, number]): void {
    if (this.allowed()) { this.pointers.set(id, direction); this.publish(); }
  }
  releasePointer(id: number): void { this.pointers.delete(id); this.publish(); }
  clear(): void { this.keys.clear(); this.pointers.clear(); this.movement.clearJumpInput(); this.publish(); }
  private publish(): void {
    let x = 0, z = 0;
    // Aliased keys represent one direction, so W+Up cannot double analog input.
    const held = new Set([...this.keys].flatMap(code => directions[code] ? [directions[code]!.join(',')] : []));
    for (const value of held) { const [dx,dz] = value.split(',').map(Number); x += dx!; z += dz!; }
    for (const [dx,dz] of this.pointers.values()) { x += dx; z += dz; }
    if (!this.allowed()) { x = 0; z = 0; }
    const length = Math.hypot(x,z);
    const scale = length > 1 ? 1 / length : 1;
    const run = length > 0 && (this.running || this.keys.has('ShiftLeft') || this.keys.has('ShiftRight'));
    this.command = [x*scale,z*scale,run]; this.movement.setInput(...this.command);
  }
}
