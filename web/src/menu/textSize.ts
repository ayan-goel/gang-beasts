import '../settings/applyTextSize';
import { chooseTextSize, textSize, textSizes } from '../settings/textSize';

const label = document.querySelector<HTMLLabelElement>('#menu-text-size')!;
const select = label.querySelector('select')!;
select.replaceChildren(...textSizes.map(size => new Option(size + '%', size)));
select.value = textSize.value();
select.onchange = () => chooseTextSize(select.value);
label.hidden = false;
const unsubscribe = textSize.subscribe(() => { select.value = textSize.value(); });
if (import.meta.hot) import.meta.hot.dispose(unsubscribe);
