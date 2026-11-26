import { useSyncExternalStore } from 'react';
import { chooseTextSize, textSize, textSizes } from '../settings/textSize';

export function TextSizeSelect({className}: {className: string}) {
  const value = useSyncExternalStore(textSize.subscribe, textSize.value, () => '100' as const);
  return <label className={className}>Text size
    <select value={value} onChange={event => chooseTextSize(event.target.value)}>
      {textSizes.map(size => <option key={size} value={size}>{size}%</option>)}
    </select>
  </label>;
}
