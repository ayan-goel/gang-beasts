import { bootstrap, enter } from '../../src/lobby/api';
import { decodeLobbyState, type LiveRoom } from '../../src/lobby/lobbyState';
import { MatchFeed } from '../../src/online/MatchFeed';
import { encodeInput } from '../../src/net/inputFrame';
import { chooseInput, verifyGrips } from './workload';

export async function admit(slot: number, code: string | null) {
  await bootstrap();
  return enter('Endurance ' + (slot+1),slot,code,8,crypto.randomUUID().replaceAll('-',''));
}

export function play(slot: number, code: string, hashes: string) {
  const stats = {frames:0, bytes:0, inputs:0, applied:0, rounds:0, matches:0, scored:0, gripFrames:0,
    damageFrames:0, gustFrames:0, resumes:0, maxGapMs:0, error:''};
  let lastAt = performance.now(), lastEpoch = -1n, lastMatch = -1n, scoredEpoch = -1n;
  let socket: WebSocket, ticket: Uint8Array | undefined;
  let reconnect = false, playing = false;

  function open(resume: boolean) {
    let lobby: LiveRoom | null = null, feed: MatchFeed | undefined;
    let returning = false, starting = false, permission = false, requested = false;
    let prevHeld = 0, lastSent = -Infinity, lastAck: number | null = null, sequence = 0;
    let awaitingResume = resume, lastKind = 0;
    const url = new URL('/ws?compact=1&landings=1&footsteps=1&resume=1',location.href);
    url.protocol = 'ws:';
    socket = new WebSocket(url);
    socket.binaryType = 'arraybuffer';
    const send = (bytes: Uint8Array) => socket.send(new Uint8Array(bytes));
    socket.onopen = () => send(Uint8Array.from([resume ? 32 : 16,1,...new TextEncoder().encode(code),...(resume ? ticket! : [])]));
    socket.onmessage = ({data}) => {
      if (stats.error) return;
      try {
        const bytes = new Uint8Array(data);
        lastKind = bytes[0]!;
        switch (lastKind) {
          case 31: ticket = bytes.slice(2); break;
          case 17: case 19:
            lobby = decodeLobbyState(bytes,lobby);
            if (bytes[0] === 17) send(Uint8Array.from([20,1,...new TextEncoder().encode(hashes)]));
            if (lobby.host === slot && lobby.ready === 255 && !starting) {
              starting = true;
              send(Uint8Array.of(22,1));
            }
            break;
          case 28: case 33: break;
          case 21:
            if (bytes[2] !== 0) throw new Error('Content mismatch');
            send(Uint8Array.of(18,1,1));
            break;
          case 26:
            feed = new MatchFeed(bytes,send);
            if (feed.fighters.length !== 8 || feed.own !== slot) throw new Error('Lost assigned seat');
            break;
          case 29: permission = bytes[2] === 1; break;
          case 27: returning = true; break;
          case 2: {
            if (!feed || !feed.receive(bytes)) throw new Error('Unexpected or out-of-order snapshot');
            const state = feed.latest!;
            const now = performance.now();
            stats.maxGapMs = Math.max(stats.maxGapMs,now-lastAt);
            lastAt = now;
            stats.frames++;
            stats.bytes += bytes.length;
            if (awaitingResume) { stats.resumes++; awaitingResume = false; }
            if (verifyGrips(state.fighters)) stats.gripFrames++;
            const match = state.round >> 32n;
            if (state.round !== lastEpoch) {
              if (state.round < lastEpoch) throw new Error('Round clock regressed');
              if (match !== lastMatch) { stats.matches++; lastMatch = match; }
              stats.rounds++;
              lastEpoch = state.round;
              lastSent = -Infinity;
              prevHeld = 0;
              lastAck = null;
            }
            if (state.applied !== null && state.applied !== lastAck) { stats.applied++; lastAck = state.applied; }
            if (state.phase >= 2 && state.round !== scoredEpoch && state.winner !== null) {
              stats.scored++;
              scoredEpoch = state.round;
            }
            if (state.phase === 3 && permission && !requested) {
              requested = true;
              setTimeout(() => { if (socket.readyState === WebSocket.OPEN) send(Uint8Array.of(30,1)); },500);
            }
            playing = state.phase === 1;
            if (!playing) break;
            if (state.arenaTick >= 10800) stats.gustFrames++;
            if (state.fighters.some(f => f.health < 100)) stats.damageFrames++;
            if (state.tick - lastSent < 6 || state.fighters[slot]!.consciousness === 3) break;
            const [x,,z] = state.bodies.get(3+16*slot)!.position;
            const {axes,held} = chooseInput(Number(match),state.number,state.arenaTick,slot,x,z);
            send(encodeInput({axes,held,sequence:++sequence,tick:state.tick,pressed:held & ~prevHeld,released:prevHeld & ~held}));
            prevHeld = held;
            lastSent = state.tick;
            stats.inputs++;
            break;
          }
          default: throw new Error('Unexpected packet kind ' + bytes[0]);
        }
      } catch (error) { stats.error = String(error); }
    };
    socket.onclose = event => {
      playing = false;
      if (reconnect) { reconnect = false; setTimeout(() => open(true),300); }
      else if (returning) open(false);
      else stats.error ||= 'Unexpected socket close ' + event.code + ' after packet ' + lastKind + ', frames ' + stats.frames;
    };
  }
  open(false);
  return {
    sample:() => ({...stats,silentMs:Math.round(performance.now()-lastAt)}),
    interrupt:() => {
      if (!playing || !ticket || socket.readyState !== WebSocket.OPEN) return false;
      reconnect = true;
      socket.close(1000);
      return true;
    }
  };
}
