const STEP_SECONDS = 1 / 60;
const MAX_CATCH_UP_STEPS = 4;

export class FixedClock {
  private accumulator = 0;
  droppedSeconds = 0;

  clearDebt(): void {
    this.accumulator = 0;
  }

  advance(seconds: number, waitingTicks = 0): number {
    if (!Number.isFinite(seconds) || seconds < 0) return 0;
    this.accumulator += seconds;
    const available = Math.floor((this.accumulator + 1e-10) / STEP_SECONDS);
    // Countdown and results ticks have no physics solve. Only the current phase can enlarge the budget.
    const steps = Math.min(available, Math.max(waitingTicks, MAX_CATCH_UP_STEPS));
    this.droppedSeconds += (available - steps) * STEP_SECONDS;
    this.accumulator = Math.max(0, this.accumulator - available * STEP_SECONDS);
    return steps;
  }
}
