import { useEffect, useRef, useState, type FormEvent } from 'react';
import { colors, enter, errorText, isCode, RoomRequestError, takenColors, winTargets } from './api';

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
  const target = join && isCode(code) ? code : null;
  const [availability, setAvailability] = useState<{code:string; taken:number[]} | null>(null);
  const [refresh, setRefresh] = useState(0);
  const taken = target && availability?.code === target ? availability.taken : null;
  const blocked = (index: number) => join && (!taken || taken.includes(index));
  const conflict = color !== '' && blocked(Number(color));
  useEffect(() => {
    if (!target) return;
    const controller = new AbortController();
    let timer: number;
    async function poll(roomCode: string) {
      try {
        const used = await takenColors(roomCode, controller.signal);
        if (!controller.signal.aborted) setAvailability({code:roomCode, taken:used});
      } catch {
        // Automatic admission still works when the preview request fails. The server checks every claim.
      }
      if (!controller.signal.aborted) timer = window.setTimeout(() => poll(roomCode), 6000);
    }
    void poll(target);
    return () => { controller.abort(); clearTimeout(timer); };
  }, [target, refresh]);
  async function submit(e: FormEvent) {
    e.preventDefault();
    if (busy || conflict) return;
    setBusy(true); setError('');
    try {
      requestId.current ||= crypto.randomUUID().replaceAll('-', '');
      onEnter(await enter(name.trim(), color ? Number(color) : undefined, join ? code.toUpperCase() : null, capacity, requestId.current, wins));
    } catch (err) {
      if (target && err instanceof RoomRequestError && err.code === 'color_taken') {
        setAvailability({code:target, taken:[...(taken ?? []), Number(color)]});
        setRefresh(n => n + 1);
      } else {
        setError(errorText(err));
      }
      setBusy(false);
    }
  }
  return <section className="room-panel">
    <h1>Private rooms</h1>
    <div className="room-mode" aria-label="Room action">
      <button type="button" aria-pressed={!join} disabled={busy} onClick={() => {setJoin(false); setError('');}}>Create</button>
      <button type="button" aria-pressed={join} disabled={busy} onClick={() => {setJoin(true); setColor(''); setError('');}}>Join</button>
    </div>
    <form onSubmit={submit}><fieldset disabled={busy}>
      <label>Player name<input autoComplete="nickname" value={name} onChange={e => setName(e.target.value)} maxLength={24} required /></label>
      {join && <label>Room code<input autoCapitalize="characters" spellCheck={false} value={code} maxLength={6} minLength={6}
        onChange={e => {setCode(e.target.value.toUpperCase()); setColor(''); setError('');}} pattern="[A-HJ-NP-Z2-9]{6}" required /></label>}
      <div className="room-fields"><label>Fighter color<select value={color} onChange={e => {setColor(e.target.value); setError('');}}>
        <option value="">Automatic</option>{colors.map((c,i) => <option key={c} value={i} disabled={blocked(i)}>{c}{taken?.includes(i) ? ' (taken)' : ''}</option>)}
      </select></label>
      {!join && <label>Room capacity<select value={capacity} onChange={e => setCapacity(Number(e.target.value))}>
        {[2,3,4,5,6,7,8].map(n => <option key={n} value={n}>{n} players</option>)}
      </select></label>}</div>
      {!join && <label>Series target<select value={wins} onChange={e => setWins(Number(e.target.value))}>
        {winTargets.map(target => <option key={target} value={target}>First to {target} wins</option>)}
      </select></label>}
      {(error || conflict) && <p role="alert">{error || 'That color is taken. Choose another or use Automatic.'}</p>}
      <button className="room-primary" disabled={!name.trim() || conflict}>{busy ? 'Connecting…' : join ? 'Join room' : 'Create room'}</button>
    </fieldset></form>
  </section>;
}
