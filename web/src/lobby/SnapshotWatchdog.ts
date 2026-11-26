const silenceLimit = 8000;

export class SnapshotWatchdog {
  private deadline = 0;
  private timer?: ReturnType<typeof setTimeout>;

  constructor(private expire: () => void) {}

  progress() {
    this.deadline = performance.now() + silenceLimit;
    // At 30 snapshots per second, keep one timer and move only the deadline.
    this.timer ??= setTimeout(this.check, silenceLimit);
  }

  dispose() {
    clearTimeout(this.timer);
    this.timer = undefined;
  }

  private check = () => {
    const remaining = this.deadline - performance.now();
    if (remaining > 0) this.timer = setTimeout(this.check, remaining);
    else {
      this.timer = undefined;
      this.expire();
    }
  };
}
