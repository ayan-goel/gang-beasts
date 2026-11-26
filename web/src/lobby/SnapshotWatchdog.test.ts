import { afterEach, expect, it, vi } from 'vitest';
import { SnapshotWatchdog } from './SnapshotWatchdog';

afterEach(() => vi.useRealTimers());

it('allows 5 seconds of stalled delivery and expires once after 8 seconds without forward progress', () => {
  vi.useFakeTimers();
  const expire = vi.fn();
  const watchdog = new SnapshotWatchdog(expire);
  watchdog.progress();
  vi.advanceTimersByTime(5000);
  watchdog.progress();
  vi.advanceTimersByTime(7999);
  expect(expire).not.toHaveBeenCalled();
  vi.advanceTimersByTime(1);
  expect(expire).toHaveBeenCalledOnce();
  vi.advanceTimersByTime(30000);
  expect(expire).toHaveBeenCalledOnce();
});

it('releases the pending timer when a socket is replaced', () => {
  vi.useFakeTimers();
  const expire = vi.fn();
  const watchdog = new SnapshotWatchdog(expire);
  watchdog.progress();
  watchdog.dispose();
  vi.runAllTimers();
  expect(expire).not.toHaveBeenCalled();
});
