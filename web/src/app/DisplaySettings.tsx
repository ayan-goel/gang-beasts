import { useSyncExternalStore } from 'react';
import { cameraMotion, motionChoices } from '../rendering/cameraMotion';
import { TextSizeSelect } from './TextSizeSelect';
import { GraphicsSelect } from './GraphicsSelect';

const motionLabels = {system:'Use device setting', reduced:'Reduced', full:'Full'};

export function DisplaySettings() {
  const motion = useSyncExternalStore(cameraMotion.subscribe, cameraMotion.value, () => 'system' as const);
  return <><GraphicsSelect /><label className="skyline-graphics">Camera motion
    <select value={motion} onChange={event => {
      const selected = motionChoices.find(choice => choice === event.target.value);
      if (selected) cameraMotion.choose(selected);
    }}>
      {motionChoices.map(choice => <option key={choice} value={choice}>{motionLabels[choice]}</option>)}
    </select>
  </label><TextSizeSelect className="skyline-graphics" /></>;
}
