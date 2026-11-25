import type { InspectorProps } from './types';

export function MovementControls({state, act, focus}: InspectorProps & {focus(): void}) {
  if (!state.movementInput) return null;
  const blocked = state.paused || !state.rig?.controller?.enabled;
  const directions = [['Left',-1,0,'←'],['Forward',0,-1,'↑'],['Back',0,1,'↓'],['Right',1,0,'→']] as const;
  return <div className="movement-controls" aria-label="Movement controls">
    <div className="button-row">
      <button className="primary" onClick={() => { act(p=>p.setPaused(!state.paused)); focus(); }}>{state.paused ? 'Resume' : 'Pause'}</button>
      <button disabled={!state.paused} onClick={()=>act(p=>p.singleStep())}>Single step</button>
      <button onClick={()=>act(p=>p.reset())}>Reset fixture</button>
    </div>
    <div className="movement-pad">
      {directions.map(([name,x,z,arrow], index) => <button key={name} aria-label={`Move ${name.toLowerCase()}`} disabled={blocked}
        onPointerDown={event=>{ if(event.button!==0) return; event.preventDefault(); event.currentTarget.setPointerCapture(event.pointerId); act(p=>p.movementInput?.pointer(event.pointerId,[x,z])); }}
        onPointerUp={event=>act(p=>p.movementInput?.releasePointer(event.pointerId))}
        onPointerCancel={event=>act(p=>p.movementInput?.releasePointer(event.pointerId))}
        onLostPointerCapture={event=>act(p=>p.movementInput?.releasePointer(event.pointerId))}
        onKeyDown={event=>{
          const key = [' ', 'Enter'].indexOf(event.key);
          if (key < 0) return;
          event.preventDefault();
          if (!event.repeat) act(p=>p.movementInput?.pointer(-(index*2+key+1),[x,z]));
        }}
        onKeyUp={event=>{
          const key = [' ', 'Enter'].indexOf(event.key);
          if (key >= 0) act(p=>p.movementInput?.releasePointer(-(index*2+key+1)));
        }}
        onBlur={()=>act(p=>{
          p.movementInput?.releasePointer(-(index*2+1));
          p.movementInput?.releasePointer(-(index*2+2));
        })}>{arrow}</button>)}
      <label className="check"><input type="checkbox" checked={state.movementInput.run} onChange={event=>act(p=>p.movementInput?.setRun(event.target.checked))}/>Run</label>
    </div>
    <button className="focus-movement" onClick={focus}>Focus movement controls</button>
    <p>Hold WASD / arrows · Shift to run · Moving away releases input</p>
  </div>;
}
