import { afterEach, expect, it, vi } from 'vitest';
import { RoomTransport } from './RoomTransport';

afterEach(() => vi.useRealTimers());

function setup() {
  vi.useFakeTimers();
  const attempt = vi.fn();
  const status = vi.fn();
  const transport = new RoomTransport('ABCDEF', attempt, status);
  transport.acceptTicket(Uint8Array.of(31,1,7,0,0,0,0,0,0,0));
  return {transport, attempt, status};
}

it('retries with the original ticket, caps the outage and cancels an unfinished handshake', () => {
  const {transport, attempt, status} = setup();
  expect([...transport.connectPacket()]).toEqual([16,1,65,66,67,68,69,70]);
  transport.closed(true, 1006);
  vi.advanceTimersByTime(250);
  expect(attempt).toHaveBeenCalledTimes(1);
  expect([...transport.connectPacket()]).toEqual([32,1,65,66,67,68,69,70,7,0,0,0,0,0,0,0]);
  transport.closed(false, 1006);
  vi.advanceTimersByTime(500);
  expect(attempt).toHaveBeenCalledTimes(2);
  vi.advanceTimersByTime(19250);
  expect(status).toHaveBeenLastCalledWith('disconnected');
  expect(transport.expired).toBe(true);
  vi.advanceTimersByTime(30000);
  expect(attempt).toHaveBeenCalledTimes(2);
});

it('stops retrying after a snapshot, deliberate exit, protocol error or lobby disconnect', () => {
  for (const [active, code] of [[false,1006],[true,4002]] as const) {
    const {transport, attempt, status} = setup();
    transport.closed(active, code);
    vi.runAllTimers();
    expect(attempt).not.toHaveBeenCalled();
    expect(status).toHaveBeenLastCalledWith('disconnected');
  }
  for (const action of ['restored','dispose'] as const) {
    const {transport, attempt, status} = setup();
    transport.closed(true, 1012);
    transport[action]();
    vi.runAllTimers();
    expect(attempt).not.toHaveBeenCalled();
    expect(status).not.toHaveBeenCalledWith('disconnected');
  }
});
