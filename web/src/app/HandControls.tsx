import type { InspectorProps } from './types';

export function HandControls({ act, disabled }: Pick<InspectorProps, 'act'> & {disabled: boolean}) {
  return <div className="button-row hand-controls">
    {([0, 1] as const).map(side => {
      const hold = (source: string, down: boolean) => act(p => p.movementInput?.hands.hold(side, source, down));
      return <button key={side} disabled={disabled}
        onPointerDown={event => {
          if (event.button !== 0) return;
          event.preventDefault();
          event.currentTarget.setPointerCapture(event.pointerId);
          hold(`pointer-${event.pointerId}`, true);
        }}
        onPointerUp={event => hold(`pointer-${event.pointerId}`, false)}
        onPointerCancel={event => hold(`pointer-${event.pointerId}`, false)}
        onLostPointerCapture={event => hold(`pointer-${event.pointerId}`, false)}
        onKeyDown={event => {
          if (![' ', 'Enter'].includes(event.key)) return;
          event.preventDefault();
          if (!event.repeat) hold(event.key, true);
        }}
        onKeyUp={event => { if ([' ', 'Enter'].includes(event.key)) hold(event.key, false); }}
        onBlur={() => {hold(' ', false); hold('Enter', false);}}
        onClick={event => { if (event.detail === 0) {hold('activate',true);hold('activate',false);} }}
      >{side === 0 ? 'Left hand' : 'Right hand'}</button>;
    })}
  </div>;
}
