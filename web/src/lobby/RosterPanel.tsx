import { useEffect, useState } from 'react';
import { colors, errorText, leave, roster, type Roster } from './api';

export function RosterPanel({ code, onLeave }: { code: string; onLeave: () => void }) {
  const [state, setState] = useState<Roster | null>(null);
  const [error, setError] = useState('');
  const [copied, setCopied] = useState(false);
  const [leaving, setLeaving] = useState(false);
  const url = new URL('/rooms/', location.origin); url.searchParams.set('code', code);
  useEffect(() => {
    const controller = new AbortController();
    let timer: number;
    async function refresh() {
      try {
        const next = await roster(code, controller.signal);
        if (!controller.signal.aborted) { setState(next); setError(''); }
      } catch (err) { if (!controller.signal.aborted) setError(errorText(err)); }
      if (!controller.signal.aborted) timer = window.setTimeout(refresh, 6000);
    }
    void refresh();
    return () => { controller.abort(); clearTimeout(timer); };
  }, [code]);
  async function exit() {
    setLeaving(true);
    try { await leave(code); onLeave(); } catch (err) { setError(errorText(err)); setLeaving(false); }
  }
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
      <span>{p.name}{p.slot === state.you ? ' (you)' : ''}</span>{p.host && <b>Host</b>}
    </li>)}</ul> : <p role="status">Loading players…</p>}
    {error && <p role="alert">{error}</p>}
    <button className="room-secondary" onClick={exit} disabled={leaving}>{leaving ? 'Leaving…' : 'Leave room'}</button>
  </section>;
}
