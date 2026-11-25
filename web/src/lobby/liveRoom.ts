import { useEffect, useRef, useState } from 'react';

interface Player { slot: number; name: string; color: number; host: boolean }
interface LiveRoom {
  revision: number; slot: number; occupied: number; ready: number; host: number;
  capacity: number; players: Player[];
}

export function decodeLobbyState(bytes: Uint8Array, previous: LiveRoom | null): LiveRoom {
  const welcome = !previous;
  if (bytes.length < (welcome ? 10 : 9) || bytes[0] !== (welcome ? 17 : 19) || bytes[1] !== 1) {
    throw new RangeError('Invalid lobby packet');
  }
  const slot = previous?.slot ?? bytes[2]!;
  const offset = welcome ? 3 : 2;
  const occupied = bytes[offset]!, ready = bytes[offset + 1]!, host = bytes[offset + 2]!;
  if (slot > 7 || host > 7 || !(occupied & (1 << slot)) || !(occupied & (1 << host)) || (ready & ~occupied)) {
    throw new RangeError('Invalid lobby seats');
  }
  const revision = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength).getUint32(offset + 3, true);
  let cursor = offset + 7;
  if (previous?.revision === revision) {
    if (bytes.length !== cursor || occupied !== previous.occupied || host !== previous.host) throw new RangeError('Invalid roster revision');
    return {...previous, ready};
  }
  const capacity = bytes[cursor++] ?? 0;
  if (capacity < 2 || capacity > 8 || occupied >= (1 << capacity)) throw new RangeError('Invalid lobby capacity');
  const players: Player[] = [];
  const decoder = new TextDecoder('utf-8', {fatal: true});
  for (let i = 0; i < capacity; i++) {
    if (!(occupied & (1 << i))) continue;
    const color = bytes[cursor++], size = bytes[cursor++];
    if (color === undefined || color > 7 || !size || size > 96 || cursor + size > bytes.length) throw new RangeError('Invalid lobby player');
    const name = decoder.decode(bytes.subarray(cursor, cursor + size));
    players.push({slot: i, name, color, host: host === i});
    cursor += size;
  }
  if (cursor !== bytes.length) throw new RangeError('Invalid roster length');
  return {slot, occupied, ready, host, revision, capacity, players};
}

export function useLiveRoom(code: string) {
  const [connection, setConnection] = useState<LiveRoom | null>(null);
  const [disconnected, setDisconnected] = useState(false);
  const [attempt, setAttempt] = useState(0);
  const transport = useRef<WebSocket | null>(null);
  useEffect(() => {
    const url = new URL('/ws', location.href);
    url.protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
    const socket = new WebSocket(url);
    transport.current = socket;
    socket.binaryType = 'arraybuffer';
    setConnection(null);
    setDisconnected(false);
    let last: LiveRoom | null = null;
    socket.onopen = () => {
      const connect = new Uint8Array(8);
      connect.set([16, 1]);
      connect.set(new TextEncoder().encode(code), 2);
      socket.send(connect);
    };
    socket.onmessage = event => {
      try {
        last = decodeLobbyState(new Uint8Array(event.data), last);
        setConnection(last);
      } catch { socket.close(4002, 'Invalid room state'); }
    };
    socket.onclose = () => {
      setConnection(null);
      setDisconnected(true);
    };
    return () => {
      transport.current = null;
      socket.onopen = socket.onmessage = socket.onclose = null;
      socket.close();
    };
  }, [code, attempt]);
  function setReady(ready: boolean) {
    if (transport.current?.readyState === WebSocket.OPEN && connection) {
      transport.current.send(Uint8Array.of(18, 1, Number(ready)));
    }
  }
  return {connection, disconnected, setReady, reconnect: () => setAttempt(n => n + 1)};
}
