import { useSyncExternalStore } from 'react';
import { chooseGraphics, graphicsPreference, graphicsPresets, subscribeGraphics } from '../rendering/graphicsPreference';
import './graphics.css';

export function GraphicsSelect() {
  const preset = useSyncExternalStore(subscribeGraphics, graphicsPreference, () => 'auto' as const);
  return <label className="skyline-graphics">Graphics
    <select value={preset} onChange={event => {
      const selected = graphicsPresets.find(item => item === event.target.value);
      if (selected) chooseGraphics(selected);
    }}>
      {graphicsPresets.map(item => <option key={item} value={item}>{item[0]!.toUpperCase() + item.slice(1)}</option>)}
    </select>
  </label>;
}
