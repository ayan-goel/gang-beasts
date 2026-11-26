import { textSize } from './textSize';
import './textSize.css';

function apply() {
  const size = textSize.value();
  if (size === '100') delete document.documentElement.dataset.textSize;
  else document.documentElement.dataset.textSize = size;
}

apply();
const unsubscribe = textSize.subscribe(apply);
if (import.meta.hot) import.meta.hot.dispose(unsubscribe);
