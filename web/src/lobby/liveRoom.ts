import { useEffect, useRef, useState } from 'react';

import { decodeLobbyState, type LiveRoom } from './lobbyState';
import { RoomTransport, type LinkState } from './RoomTransport';
import { MatchLatency } from './MatchLatency';
import { SnapshotWatchdog } from './SnapshotWatchdog';
import { MatchFeed } from '../online/MatchFeed';
import { ContentHandshake, type LoadState, type LoadedArena } from './ContentHandshake';

export function useLiveRoom(code: string) {
  const [connection, setConnection] = useState<LiveRoom | null>(null);
  const [link, setLink] = useState<LinkState>('connected');
  const [attempt, setAttempt] = useState(0);
  const [game, setGame] = useState<(LoadedArena & {feed: MatchFeed}) | null>(null);
  const [startError, setStartError] = useState('');
  const [highLatency, setHighLatency] = useState(false);
  const [canRematch, setCanRematch] = useState(false);
  const [load, setLoad] = useState<LoadState>({loading: false, error: ''});
  const handshake = useRef<ContentHandshake | null>(null);
  const transport = useRef<WebSocket | null>(null);
  useEffect(() => {
    let cleanup = () => {};
    const recovery = new RoomTransport(code, open, state => {
      setLink(state);
      if (state === 'disconnected') {
        cleanup();
        setLoad({loading: false, error: ''});
        setConnection(null);
        setGame(null);
      }
    });
    setLink('connected');
    function open() {
      cleanup();
      const url = new URL('/ws?resume=1&lineups=1&compact=1&landings=1&footsteps=1&timing=1', location.href);
      url.protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
      const socket = new WebSocket(url);
      const content = new ContentHandshake(socket, setLoad);
      handshake.current = content;
      setLoad({loading:false, error:''});
      transport.current = socket;
      socket.binaryType = 'arraybuffer';
      setConnection(null);

      let last: LiveRoom | null = null;
      let feed: MatchFeed | null = null;
      let closed = false;
      let latency: MatchLatency | null = null;
      setHighLatency(false);
      const watchdog = new SnapshotWatchdog(() => disconnected(4008));

      function disconnected(code: number) {
        if (closed) return;
        // A half-open link can swallow the close handshake. Begin recovery as soon as silence expires.
        if (socket.readyState === WebSocket.OPEN && (code === 4002 || code === 4008)) {
          socket.close(code, code === 4002 ? 'Invalid room state' : 'Match stream stalled');
        }
        cleanup();
        setLoad({loading:false, error:''});
        setConnection(null);
        setGame(null);
        recovery.closed(feed !== null, code);
      }
      setGame(null);
      setCanRematch(false);
      setStartError('');
      socket.onopen = () => {
        socket.send(recovery.connectPacket());
      };
      socket.onmessage = event => {
        try {
          const bytes = new Uint8Array(event.data);
          if (last && !feed && !latency && bytes.length === 2 && bytes[0] === 35 && bytes[1] === 1) {
            latency = new MatchLatency();
            return;
          }
          if (feed && latency && bytes[0] === 37) {
            const high = latency.reply(bytes, performance.now());
            if (high !== null) setHighLatency(high);
            return;
          }
          if (bytes[0] === 31 && !last) { recovery.acceptTicket(bytes); return; }
          if (bytes.length === 2 && bytes[0] === 33 && bytes[1] === 1 && last && !feed) {
            recovery.restored();
            return;
          }
          if (bytes.length === 3 && bytes[0] === 29 && bytes[1] === 1 && bytes[2]! <= 1 && feed) {
            setCanRematch(bytes[2] === 1);
            return;
          }
          if (bytes.length === 2 && bytes[1] === 1 && bytes[0] === 28 && last && !feed) {
            void content.ready(false);
            return;
          }
          if (bytes.length === 2 && bytes[1] === 1 && bytes[0] === 27 && feed) {
            cleanup();
            setAttempt(n => n + 1);
            return;
          }
          if (last && ((!feed && (bytes[0] === 23 || bytes[0] === 26)) || (feed && bytes[0] === 34))) {
            if (!content.arena) throw new Error('Match assigned before content loaded');
            const transmit = (packet: Uint8Array) => {
              // Disposal of the old canvas can emit a final neutral input after the new assignment.
              if (feed !== assigned || socket.readyState !== WebSocket.OPEN) return;
              if (socket.bufferedAmount > 16384) { disconnected(4008); return; }
              socket.send(new Uint8Array(packet));
            };
            const assigned: MatchFeed = feed ? feed.reassign(bytes, transmit) : new MatchFeed(bytes, transmit);
            feed = assigned;
            watchdog.progress();
            return;
          }
          if (bytes[0] === 2 && feed) {
            const entering = feed.latest === null;
            if (feed.receive(bytes) && !closed) {
              watchdog.progress();
              if (entering && content.arena) setGame({...content.arena, feed});
              const probe = latency?.probe(performance.now());
              if (probe) feed.transmit(probe);
              if (!closed) recovery.restored();
            }
            return;
          }
          if (bytes[0] === 25 && bytes.length === 3 && bytes[1] === 1) {
            setStartError(bytes[2] === 2 ? 'All match servers are busy. Try again.' : 'All players must be ready to start.');
            return;
          }
          if (bytes[0] === 21 && last) { content.acknowledge(bytes); return; }
          last = decodeLobbyState(bytes, last);
          setConnection(last);
        } catch {
          disconnected(4002);
        }
      };
      socket.onclose = event => disconnected(event.code);
      cleanup = () => {
        closed = true;
        watchdog.dispose();
        content.dispose();
        handshake.current = null;
        transport.current = null;
        socket.onopen = socket.onmessage = socket.onclose = null;
        socket.close();
      };
    }
    open();
    return () => { recovery.dispose(); cleanup(); };
  }, [code, attempt]);
  function setReady(ready: boolean) {
    if (transport.current?.readyState === WebSocket.OPEN && connection) {
      if (ready) void handshake.current?.ready();
      else {
        handshake.current?.dispose();
        transport.current.send(Uint8Array.of(18, 1, 0));
      }
    }
  }
  function start() {
    setStartError('');
    if (transport.current?.readyState === WebSocket.OPEN) transport.current.send(Uint8Array.of(22, 1));
  }
  function requestRematch() {
    if (canRematch) game?.feed.transmit(Uint8Array.of(30, 1));
  }
  return {connection, disconnected: link === 'disconnected', reconnecting: link === 'recovering', game, highLatency, canRematch, requestRematch, start, startError, load, setReady, reconnect: () => setAttempt(n => n + 1)};
}
