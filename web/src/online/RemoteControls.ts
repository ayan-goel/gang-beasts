import { MovementInput } from '../game/MovementInput';
import { bindMovementInput } from '../game/bindMovementInput';
import { InputSampler } from '../net/InputSampler';
import { bindCameraOrbit } from '../game/bindCameraOrbit';
import type { GroupCamera } from '../rendering/GroupCamera';
import type { MatchFeed } from './MatchFeed';

export function bindRemoteControls(canvas: HTMLCanvasElement, feed: MatchFeed, camera: GroupCamera, orbitKeys: HTMLElement = canvas) {
  const unbindOrbit = bindCameraOrbit(canvas, camera, () => !document.hidden && !canvas.inert, orbitKeys);
  if (feed.own === null) {
    canvas.focus({preventScroll:true});
    return unbindOrbit;
  }
  let sampler = new InputSampler();
  let active = false;
  let lastSent = -Infinity;
  let actionSendUntil = -Infinity;
  const playing = () => feed.latest?.phase === 1 && feed.latest.fighters[feed.own!]!.consciousness !== 3;
  const button = (name: Parameters<InputSampler['setButton']>[0], down: boolean) => {
    try { sampler.setButton(name, down); } catch { sampler.neutralize(); }
  };
  const press = (name: 'jump' | 'leftHand' | 'rightHand' | 'headbutt', held: boolean) => {
    button(name,held);
    if (held && performance.now() <= actionSendUntil && !document.hidden) {
      actionSendUntil = -Infinity;
      transmit();
    }
  };
  const movement = new MovementInput({
    setInput(x,z,run) { sampler.setMovement(...camera.movement(x,z)); button('run',run); },
    setJump: held => press('jump',held),
    clearJumpInput: () => button('jump',false),
    setHand: (side,held) => press(side === 0 ? 'leftHand' : 'rightHand',held),
    clearHands: () => { button('leftHand',false); button('rightHand',false); },
    setHeadbutt: held => press('headbutt',held),
    setLift: held => button('lift',held),
  }, () => playing() && document.activeElement === canvas);
  const unbind = bindMovementInput(canvas,movement);
  const transmit = () => {
    const [x,z] = movement.command;
    sampler.setMovement(...camera.movement(x,z));
    lastSent = performance.now();
    if (feed.latest) for (const packet of sampler.sample(feed.latest.tick)) feed.transmit(packet);
  };
  const neutralize = () => {
    actionSendUntil = -Infinity;
    movement.clear();
    sampler.neutralize();
    if (active && playing()) transmit();
    active = false;
  };
  feed.onRoundChange = () => {
    actionSendUntil = -Infinity;
    movement.clear();
    sampler = new InputSampler();
    lastSent = -Infinity;
    active = false;
  };
  feed.onSnapshot = () => {
    if (!playing() || document.hidden || document.activeElement !== canvas) {
      if (active) neutralize();
      return;
    }
    active = true;
    // Jump, hand and headbutt presses share one immediate send per fresh publication. Chorded edges stay in the sampler.
    actionSendUntil = performance.now() + 100;
    // Coalesce queued publications after a stall. Fresh network frames provide the input cadence.
    if (performance.now() - lastSent >= 25) transmit();
  };
  canvas.addEventListener('blur', neutralize);
  window.addEventListener('blur', neutralize);
  canvas.focus({preventScroll:true});
  return () => {
    feed.onRoundChange = undefined;
    feed.onSnapshot = undefined;
    canvas.removeEventListener('blur',neutralize);
    window.removeEventListener('blur',neutralize);
    neutralize();
    unbindOrbit();
    unbind();
  };
}
