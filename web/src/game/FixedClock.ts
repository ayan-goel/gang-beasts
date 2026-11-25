const STEP_SECONDS = 1 / 60;
const MAX_CATCH_UP_STEPS = 4;

export class FixedClock {
  private accumulator = 0;
  droppedSeconds = 0;

  advance(seconds: number): number {
    if (!Number.isFinite(seconds) || seconds < 0) return 0;
    this.accumulator += seconds;
    const available = Math.floor((this.accumulator + 1e-10) / STEP_SECONDS);
    const steps = Math.min(available, MAX_CATCH_UP_STEPS);
    // The local playground drops suspended time instead of blocking the UI.
    // This is not the authoritative server's overload policy.
    this.droppedSeconds += (available - steps) * STEP_SECONDS;
    this.accumulator = Math.max(0, this.accumulator - available * STEP_SECONDS);
    return steps;
  }
}
