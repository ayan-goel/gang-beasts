import { useEffect, useRef, useState } from 'react';

import { decodeLobbyState, type LiveRoom } from './lobbyState';
import { MatchFeed } from '../online/MatchFeed';
import { ContentHandshake, type LoadState } from './ContentHandshake';

export function useLiveRoom(code: string) {
  const [connection, setConnection] = useState<LiveRoom | null>(null);
  const [disconnected, setDisconnected] = useState(false);
  const [attempt, setAttempt] = useState(0);
  const [game, setGame] = useState<MatchFeed | null>(null);
  const [startError, setStartError] = useState('');
  const [load, setLoad] = useState<LoadState>({loading: false, error: ''});
  const handshake = useRef<ContentHandshake | null>(null);
  const transport = useRef<WebSocket | null>(null);
  useEffect(() => {
    const url = new URL('/ws', location.href);
    url.protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
    const socket = new WebSocket(url);
    const content = new ContentHandshake(socket, setLoad);
    handshake.current = content;
    setLoad({loading:false, error:''});
    transport.current = socket;
    socket.binaryType = 'arraybuffer';
    setConnection(null);
    setDisconnected(false);
    let last: LiveRoom | null = null;
    let feed: MatchFeed | null = null;
    setGame(null);
    setStartError('');
    socket.onopen = () => {
      const connect = new Uint8Array(8);
      connect.set([16, 1]);
      connect.set(new TextEncoder().encode(code), 2);
      socket.send(connect);
    };
    socket.onmessage = event => {
      try {
        const bytes = new Uint8Array(event.data);
        if (bytes[0] === 23 && last && !feed) { feed = new MatchFeed(bytes); setGame(feed); return; }
        if (bytes[0] === 2 && feed) { feed.receive(bytes); return; }
        if (bytes[0] === 25 && bytes.length === 3 && bytes[1] === 1) {
          setStartError(bytes[2] === 2 ? 'All match servers are busy. Try again.' : 'Both players must be ready to start.');
          return;
        }
        if (bytes[0] === 21 && last) { content.acknowledge(bytes); return; }
        last = decodeLobbyState(bytes, last);
        setConnection(last);
      } catch { socket.close(4002, 'Invalid room state'); }
    };
    socket.onclose = () => {
      content.dispose();
      setLoad({loading:false, error:''});
      setConnection(null);
      setDisconnected(true);
      setGame(null);
    };
    return () => {
      content.dispose();
      handshake.current = null;
      transport.current = null;
      socket.onopen = socket.onmessage = socket.onclose = null;
      socket.close();
    };
  }, [code, attempt]);
  function setReady(ready: boolean) {
    if (transport.current?.readyState === WebSocket.OPEN && connection) {
      if (ready) void handshake.current?.ready();
      else transport.current.send(Uint8Array.of(18, 1, 0));
    }
  }
  function start() {
    setStartError('');
    if (transport.current?.readyState === WebSocket.OPEN) transport.current.send(Uint8Array.of(22, 1));
  }
  return {connection, disconnected, game, start, startError, load, setReady, reconnect: () => setAttempt(n => n + 1)};
}
