import { useEffect, useState } from 'react';
import { useLiveRoom } from './liveRoom';
import { colors, errorText, leave, roster, RoomRequestError, type Roster } from './api';


export function RosterPanel({ code, onLeave }: { code: string; onLeave: () => void }) {
  const [fallback, setFallback] = useState<Roster | null>(null);
  const [error, setError] = useState('');
  const [lostAccess, setLostAccess] = useState<'seat' | 'room' | null>(null);
  const {connection, disconnected, reconnecting, game, canRematch, requestRematch, start, startError, load, setReady, reconnect} = useLiveRoom(code);
  const state = connection ?? (disconnected ? fallback : null);
  const mySlot = connection?.slot ?? fallback?.you;
  const myReady = !!(connection && (connection.ready & (1 << connection.slot)));
  const [copied, setCopied] = useState(false);
  const [leaving, setLeaving] = useState(false);
  const url = new URL('/rooms/', location.origin); url.searchParams.set('code', code);
  useEffect(() => {
    if (!disconnected) { setLostAccess(null); return; }
    const controller = new AbortController();
    let timer: number;
    async function refresh() {
      try {
        const next = await roster(code, controller.signal);
        if (!controller.signal.aborted) { setFallback(next); setError(''); }
      } catch (err) {
        if (controller.signal.aborted) return;
        setError(errorText(err));
        if (err instanceof RoomRequestError && ['not_a_member', 'session_required', 'room_not_found'].includes(err.code)) {
          setFallback(null);
          setLostAccess(err.code === 'room_not_found' ? 'room' : 'seat');
          return;
        }
      }
      if (!controller.signal.aborted) timer = window.setTimeout(refresh, 6000);
    }
    void refresh();
    return () => { controller.abort(); clearTimeout(timer); };
  }, [code, disconnected]);
  async function exit() {
    setLeaving(true);
    try { await leave(code); onLeave(); } catch (err) { setError(errorText(err)); setLeaving(false); }
  }
  if (reconnecting) return <section className="room-panel">
    <h1>Reconnecting to the match</h1>
    <p role="status">Trying to recover your seat…</p>
    <button className="room-secondary" onClick={exit} disabled={leaving}>Leave room</button>
    {error && <p role="alert">{error}</p>}
  </section>;
  if (game) {
    const {View, feed, content} = game;
    return <><View key={feed.assignmentRound?.toString() ?? 'start'} feed={feed} content={content} exit={exit} rematch={canRematch ? requestRematch : undefined}/>
      {error && <p role="alert">{error}</p>}</>;
  }
  if (lostAccess && disconnected) return <section className="room-panel">
    <h1>{lostAccess === 'room' ? 'Room closed' : 'Seat expired'}</h1>
    <p role="status">{lostAccess === 'room' ? 'This room is no longer available.' : 'Your seat was released while you were disconnected.'}</p>
    <a className="room-primary" href={lostAccess === 'room' ? '/rooms/' : url.href}>{lostAccess === 'room' ? 'Find another room' : 'Rejoin room'}</a>
  </section>;
  const canStart = connection && connection.players.length >= 2 && connection.occupied === connection.ready;
  return <section className="room-panel">
    <h1>Room lobby</h1>
    <div className="room-code"><span>Room code</span><strong>{code}</strong></div>
    <label>Invite link<input readOnly value={url.href} onFocus={e => e.target.select()} /></label>
    <button className="room-primary" onClick={async () => {
      try { await navigator.clipboard.writeText(url.href); setCopied(true); }
      catch { setError('Select the invite link above and copy it.'); }
    }}>{copied ? 'Link copied' : 'Copy invite link'}</button>
    <h2>Players <small>{state ? state.players.length + ' / ' + state.capacity : ''}</small></h2>
    {state ? <ul className="room-roster" aria-label="Players">{state.players.map(p => <li key={p.slot}>
      <i className={'swatch color-' + p.color} aria-label={colors[p.color]} />
      <span>{p.name}{p.slot === mySlot ? ' (you)' : ''}</span>{p.host && <b>Host</b>}
      {connection && <small>{connection.ready & (1 << p.slot) ? 'Ready' : 'Waiting'}</small>}
    </li>)}</ul> : <p role="status">Loading players…</p>}
    {disconnected ? <div><p role="status">Ready connection closed.</p><button className="room-secondary" onClick={reconnect}>Reconnect</button></div>
      : <button className="room-primary" disabled={!connection || load.loading} onClick={() => setReady(!myReady)}>
        {load.loading ? 'Loading Skyline…' : myReady ? 'Cancel ready' : 'Ready up'}
      </button>}
    {connection?.host === mySlot && connection && <>
      <button className="room-primary" disabled={!canStart} onClick={start}>Start match</button>
      {!canStart && <small>{connection.players.length < 2 ? 'Invite at least one more player.' : 'Waiting for all players to ready up.'}</small>}
    </>}
    {(error || load.error || startError) && <p role="alert">{error || load.error || startError}</p>}
    <button className="room-secondary" onClick={exit} disabled={leaving}>{leaving ? 'Leaving…' : 'Leave room'}</button>
  </section>;
}
