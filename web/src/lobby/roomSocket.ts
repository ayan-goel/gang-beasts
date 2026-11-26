import { socketTicket } from './api';

export async function openRoomSocket(signal: AbortSignal): Promise<WebSocket> {
  const url = new URL('/ws?resume=1&lineups=1&compact=1&landings=1&footsteps=1&timing=1&tour=1',
    import.meta.env.VITE_BACKEND_URL || location.origin);
  const remote = url.origin !== location.origin;
  url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
  if (!remote) return new WebSocket(url);

  // The room cookie stays on the frontend origin; each direct upgrade uses a fresh ticket.
  const ticket = await socketTicket(AbortSignal.any([signal, AbortSignal.timeout(8000)]));
  signal.throwIfAborted();
  return new WebSocket(url, ['skyline.v1', 'ticket.' + ticket]);
}
