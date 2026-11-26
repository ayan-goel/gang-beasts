import { useRef, useState, type FormEvent } from 'react';
import { colors, enter, errorText, winTargets } from './api';

export function RoomForm({ invite, joinByCode, onEnter }: { invite: string; joinByCode: boolean; onEnter: (code: string) => void }) {
  const [join, setJoin] = useState(!!invite || joinByCode);
  const [code, setCode] = useState(invite);
  const [name, setName] = useState('');
  const [color, setColor] = useState('');
  const [capacity, setCapacity] = useState(8);
  const [wins, setWins] = useState(3);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const requestId = useRef('');
  async function submit(e: FormEvent) {
    e.preventDefault();
    if (busy) return;
    setBusy(true); setError('');
    try {
      requestId.current ||= crypto.randomUUID().replaceAll('-', '');
      onEnter(await enter(name.trim(), color ? Number(color) : undefined, join ? code.toUpperCase() : null, capacity, requestId.current, wins));
    } catch (err) { setError(errorText(err)); setBusy(false); }
  }
  return <section className="room-panel">
    <h1>Private rooms</h1>
    <div className="room-mode" aria-label="Room action">
      <button type="button" aria-pressed={!join} disabled={busy} onClick={() => {setJoin(false); setError('');}}>Create</button>
      <button type="button" aria-pressed={join} disabled={busy} onClick={() => {setJoin(true); setError('');}}>Join</button>
    </div>
    <form onSubmit={submit}><fieldset disabled={busy}>
      <label>Player name<input autoComplete="nickname" value={name} onChange={e => setName(e.target.value)} maxLength={24} required /></label>
      {join && <label>Room code<input autoCapitalize="characters" spellCheck={false} value={code} maxLength={6} minLength={6}
        onChange={e => setCode(e.target.value.toUpperCase())} pattern="[A-HJ-NP-Z2-9]{6}" required /></label>}
      <div className="room-fields"><label>Fighter color<select value={color} onChange={e => setColor(e.target.value)}>
        <option value="">Automatic</option>{colors.map((c,i) => <option key={c} value={i}>{c}</option>)}
      </select></label>
      {!join && <label>Room capacity<select value={capacity} onChange={e => setCapacity(Number(e.target.value))}>
        {[2,3,4,5,6,7,8].map(n => <option key={n} value={n}>{n} players</option>)}
      </select></label>}</div>
      {!join && <label>Series target<select value={wins} onChange={e => setWins(Number(e.target.value))}>
        {winTargets.map(target => <option key={target} value={target}>First to {target} wins</option>)}
      </select></label>}
      {error && <p role="alert">{error}</p>}
      <button className="room-primary" disabled={!name.trim()}>{busy ? 'Connecting…' : join ? 'Join room' : 'Create room'}</button>
    </fieldset></form>
  </section>;
}
