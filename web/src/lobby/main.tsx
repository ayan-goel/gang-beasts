import { SkylineSound, MuteButton } from '../audio/SkylineSound';
import { createRoot } from 'react-dom/client';
import { useEffect, useState } from 'react';
import { bootstrap, errorText, isCode } from './api';
import { RoomForm } from './RoomForm';
import { GraphicsSelect } from '../app/GraphicsSelect';
import { RosterPanel } from './RosterPanel';
import arena from './skyline.webp';
import './rooms.css';

function PrivateRooms() {
  const [room, setRoom] = useState<string | null>(null);
  const [requested] = useState(() => new URLSearchParams(location.search).get('code')?.toUpperCase() ?? '');
  const [invite, setInvite] = useState(requested);
  const [ready, setReady] = useState(false);
  const [error, setError] = useState('');
  const [attempt, setAttempt] = useState(0);
  function enterRoom(code: string) {
    history.replaceState(null, '', '/rooms/?code=' + code);
    setInvite('');
    setRoom(code);
  }
  useEffect(() => {
    let active = true;
    setError('');
    void bootstrap().then(existing => {
      if (!active) return;
      if (existing) {
        if (isCode(requested) && requested !== existing) setRoom(existing);
        else enterRoom(existing);
      }
      setReady(true);
    }).catch(err => { if (active) setError(errorText(err)); });
    return () => { active = false; };
  }, [attempt, requested]);
  return <main className="rooms">
    <header><a className="brand" href="/" aria-label="Main menu"><span aria-hidden="true">▰</span> Skyline Melee</a>
      <GraphicsSelect /><MuteButton /><a href="/play/">Local match ↗</a></header>
    <div className="room-layout">
      <div className="room-console">{!ready ? <section className="room-panel"><h1>Private rooms</h1>
        {error ? <><p role="alert">{error}</p><button className="room-primary" onClick={() => setAttempt(a => a + 1)}>Retry connection</button></>
          : <p role="status">Connecting to the room service…</p>}</section>
        : room ? <>
          {invite && <p role="status">You already have a seat. Leave this room to join {invite}.</p>}
          <RosterPanel code={room} onLeave={() => {
            history.replaceState(null, '', invite ? '/rooms/?code=' + invite : '/rooms/');
            setRoom(null);
          }} />
        </> : <RoomForm invite={invite} joinByCode={new URLSearchParams(location.search).get('action') === 'join'} onEnter={enterRoom} />}</div>
      <figure><img src={arena} alt="Skyline’s rooftop arena with crates and breakaway walkways" />
        <figcaption><small>Arena</small><strong>Skyline</strong><span>Melee · First to three</span></figcaption></figure>
    </div>
  </main>;
}
createRoot(document.getElementById('app')!).render(<SkylineSound><PrivateRooms /></SkylineSound>);
