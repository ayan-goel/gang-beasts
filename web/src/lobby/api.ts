export const colors = ['Coral', 'Blue', 'Gold', 'Mint', 'Lilac', 'Rose', 'Lime', 'Slate'] as const;
export interface Seat { code: string; slot: number; host: boolean; color: number; name: string }
export interface Roster { wins: number; code: string; capacity: number; you: number; revision: number; players: Seat[] }

const messages: Record<string, string> = {
  room_not_found: 'Room code was not found. Check the six characters and try again.',
  room_full: 'This room is full.', color_taken: 'That color is taken. Choose another or use Automatic.',
  session_required: 'Your session expired. Reload to reconnect.',
  request_conflict: 'This session already has a seat. Reload to return to your room.',
  rate_limited: 'Too many requests. Wait a minute and try again.',
  unavailable: 'The room service is busy. Try again shortly.',
  invalid_input: 'Check the player name, room code and series settings.',
  not_a_member: 'Your seat is no longer in this room. Reload to join again.',
};

export class RoomRequestError extends Error {
  constructor(readonly code: string) { super(messages[code] ?? 'Room service unavailable. Try again.'); }
}

async function request(path: string, data?: object, signal?: AbortSignal): Promise<unknown> {
  const response = await fetch('/api/' + path, { method: data ? 'POST' : 'GET', signal,
    credentials: 'same-origin', headers: data ? { 'Content-Type': 'application/json' } : undefined,
    body: data ? JSON.stringify(data) : undefined });
  if (response.status === 204) return null;
  const value = await response.json().catch(() => null);
  if (!response.ok) throw new RoomRequestError(value?.error?.code ?? 'unavailable');
  return value;
}

function object(value: unknown): value is Record<string, unknown> { return value !== null && typeof value === 'object'; }
export const winTargets = [3, 5, 7] as const;
export function isWinTarget(value: unknown): value is 3 | 5 | 7 {
  return value === 3 || value === 5 || value === 7;
}
function slot(v: unknown): v is number { return typeof v === 'number' && Number.isInteger(v) && v >= 0 && v < 8; }
export function isCode(v: unknown): v is string { return typeof v === 'string' && /^[A-HJ-NP-Z2-9]{6}$/.test(v); }
function isSeat(v: unknown): v is Seat {
  return object(v) && isCode(v.code) && slot(v.slot) && slot(v.color) && typeof v.host === 'boolean' && typeof v.name === 'string';
}
export async function bootstrap(): Promise<{room:string | null; completed?:string}> {
  const data = await request('session', {});
  if (!object(data) || data.active !== true || !(data.room === null || isCode(data.room))) throw new Error('Invalid session response.');
  return {room:data.room, completed:isCode(data.completed) ? data.completed : undefined};
}
export async function enter(name: string, color: number | undefined, code: string | null, capacity: number, requestId: string, wins = 3): Promise<string> {
  const data = code ? await request('rooms/' + encodeURIComponent(code) + '/join', { name, color })
    : await request('rooms', { name, color, capacity, requestId, wins });
  if (!isSeat(data)) throw new Error('Invalid room admission response.');
  return data.code;
}
export async function roster(code: string, signal: AbortSignal): Promise<Roster> {
  const data = await request('rooms/' + code + '/lobby', undefined, signal);
  if (!object(data) || data.code !== code || !slot(data.you) || typeof data.capacity !== 'number'
    || (data.wins !== undefined && !isWinTarget(data.wins))
    || typeof data.revision !== 'number' || !Number.isInteger(data.revision) || data.revision < 0 || data.revision > 0xffffffff
    || !Number.isInteger(data.capacity) || data.capacity < 2 || data.capacity > 8 || !Array.isArray(data.players)
    || data.players.length > data.capacity || !data.players.every(p => isSeat(p) && p.code === code)
    || !data.players.some(p => p.slot === data.you)) throw new Error('Invalid room roster.');
  return { code, wins: isWinTarget(data.wins) ? data.wins : 3, capacity: data.capacity, you: data.you, revision: data.revision, players: data.players };
}
export async function leave(code: string) { await request('rooms/' + code + '/leave', {}); }
export const errorText = (e: unknown) => e instanceof TypeError ? 'Cannot reach the room service. Check your connection and try again.'
  : e instanceof Error ? e.message : 'Room request failed. Try again.';

export async function socketTicket(signal: AbortSignal): Promise<string> {
  const reply = await request('socket-ticket', {}, signal);
  if (!object(reply) || typeof reply.ticket !== 'string' || !/^[a-f0-9]{64}$/.test(reply.ticket)) {
    throw new Error('Invalid socket ticket.');
  }
  return reply.ticket;
}
