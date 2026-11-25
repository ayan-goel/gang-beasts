import { useEffect, useRef, useState } from 'react';

import { decodeLobbyState, type LiveRoom } from './lobbyState';
import { ContentHandshake, type LoadState } from './ContentHandshake';

export function useLiveRoom(code: string) {
  const [connection, setConnection] = useState<LiveRoom | null>(null);
  const [disconnected, setDisconnected] = useState(false);
  const [attempt, setAttempt] = useState(0);
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
    socket.onopen = () => {
      const connect = new Uint8Array(8);
      connect.set([16, 1]);
      connect.set(new TextEncoder().encode(code), 2);
      socket.send(connect);
    };
    socket.onmessage = event => {
      try {
        const bytes = new Uint8Array(event.data);
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
  return {connection, disconnected, load, setReady, reconnect: () => setAttempt(n => n + 1)};
}
