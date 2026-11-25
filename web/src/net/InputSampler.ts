import { buttons, encodeInput, quantizeDirection, type InputFrame } from './inputFrame';

type Intent = Pick<InputFrame, 'axes' | 'held' | 'pressed' | 'released'>;
const maximumBurst = 4;

export class InputSampler {
  private sequence: number;
  private sentHeld = 0;
  private pending: Intent[] = [];
  private intent: Intent = { axes: [0, 0], held: 0, pressed: 0, released: 0 };

  constructor(sequence = 0) {
    if (!Number.isInteger(sequence) || sequence < 0 || sequence > 0xffffffff) throw new RangeError('invalid initial sequence');
    this.sequence = sequence;
  }

  setMovement(x: number, z: number): void {
    this.intent.axes = quantizeDirection(x, z);
  }

  setButton(button: keyof typeof buttons, down: boolean): void {
    const bit = buttons[button];
    const state = this.intent;
    if (down === Boolean(state.held & bit)) return;
    if ((down ? state.pressed : state.released) & bit) {
      // Repeating an edge cannot fit in a single mask. Preserve the preceding tap
      // as a separate ordered record before changing its final held state.
      if (this.pending.length + 1 >= maximumBurst) {
        this.neutralize();
        throw new RangeError('input burst exceeds four records');
      }
      this.pending.push({ ...state });
      state.pressed = 0;
      state.released = 0;
    }
    if (down) {
      state.held |= bit;
      state.pressed |= bit;
    } else {
      state.held &= ~bit;
      state.released |= bit;
    }
  }

  neutralize(): void {
    this.pending = [];
    this.intent = { axes: [0, 0], held: 0, pressed: 0, released: this.sentHeld };
  }

  sample(tick: number): Uint8Array[] {
    const frames = [...this.pending, this.intent].map((intent, i) =>
      encodeInput({ ...intent, sequence: (this.sequence + i) >>> 0, tick }));
    this.sequence = (this.sequence + frames.length) >>> 0;
    this.sentHeld = this.intent.held;
    this.pending = [];
    this.intent.pressed = 0;
    this.intent.released = 0;
    return frames;
  }
}
